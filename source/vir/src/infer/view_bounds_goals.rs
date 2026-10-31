//! Select an ordinary checked view-bound theorem when caller goals/preconditions
//! observe that view. This enables a lemma; it never inserts an assumed fact.
use super::*;

fn strip(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Unary(UnaryOp::Trigger(_) | UnaryOp::Clip { range: IntRange::Int, .. }, inner) => {
            strip(inner)
        }
        _ => e,
    }
}
fn local(e: &Expr, n: &VarIdent) -> bool {
    match &e.x {
        ExprX::Var(v) => v == n,
        ExprX::ReadPlace(p, _) => matches!(&p.x,PlaceX::Local(v) if v==n),
        _ => false,
    }
}
pub(super) fn lemma(f: &Function) -> Option<Dt> {
    let x = &f.x;
    if x.name.path.last_segment().as_str() != "__verus_infer_view_len_bound"
        || x.mode != Mode::Proof
        || !matches!(x.kind, FunctionKind::Static)
        || !x.visibility.is_public()
        || !x.attrs.broadcast_forall
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.infer
        || x.attrs.exec_assume_termination
        || x.attrs.exec_allows_no_decreases_clause
        || x.proxy.is_some()
        || x.params.len() != 1
        || x.params[0].x.mode != Mode::Spec
        || x.params[0].x.user_mut
        || !x.require.is_empty()
        || x.ensure.0.len() != 1
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !enums::unit(&x.inner_ret.x.typ)
    {
        return None;
    }
    let TypX::Decorate(TypDecoration::Ref, None, t) = &*x.params[0].x.typ else {
        return None;
    };
    let TypX::Datatype(dt, ..) = &**t else {
        return None;
    };
    let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), a, b) = &strip(&x.ensure.0[0]).x
    else {
        return None;
    };
    if arithmetic::arch_constant(strip(b)).is_none()
        || !matches!(
            &strip(b).x,
            ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::UnsignedMax), _)
        )
    {
        return None;
    }
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, n, ..),
        args,
        post_args: None,
        body: None,
    } = &strip(a).x
    else {
        return None;
    };
    if !models::path_is(&n.path, CrateId::Vstd, &["seq", "Seq", "len"]) || args.len() != 1 {
        return None;
    }
    let ExprX::Call { target: CallTarget::Fun(_, n, ..), args, post_args: None, body: None } =
        &strip(&args[0]).x
    else {
        return None;
    };
    (models::path_is(&n.path, CrateId::Vstd, &["view", "View", "view"])
        && args.len() == 1
        && local(&args[0], &x.params[0].x.name))
    .then(|| dt.clone())
}

fn demands(f: &Function, all: &HashMap<Fun, Function>) -> std::collections::HashSet<Dt> {
    let mut queue: std::collections::VecDeque<_> =
        f.x.require.iter().chain(f.x.ensure.0.iter()).cloned().collect();
    if let Some(b) = &f.x.body {
        queue.push_back(b.clone());
    }
    let mut visited = std::collections::HashSet::new();
    let mut observed = std::collections::HashSet::new();
    let mut remaining = 4096usize;
    while let Some(e) = queue.pop_front() {
        let mut calls = vec![];
        crate::ast_visitor::expr_visitor_walk(&e, &mut |e| {
            if remaining == 0 {
                return crate::ast_visitor::VisitorControlFlow::Stop(());
            }
            remaining -= 1;
            if let ExprX::Call { target: CallTarget::Fun(k, n, ..), .. } = &e.x {
                calls.push(match k {
                    CallTargetKind::DynamicResolved { resolved, .. } => resolved.clone(),
                    _ => n.clone(),
                });
            }
            crate::ast_visitor::VisitorControlFlow::Recurse
        });
        for n in calls {
            let Some(callee) = all.get(&n) else {
                continue;
            };
            if callee.x.attrs.infer_sequence_view {
                if let FunctionKind::TraitMethodImpl { trait_typ_args, .. } = &callee.x.kind {
                    if let Some(t) = trait_typ_args.first() {
                        if let TypX::Datatype(dt, ..) = &**t {
                            observed.insert(dt.clone());
                        }
                    }
                }
            }
            if visited.insert(n) {
                if visited.len() > 32 {
                    // Keep checked facts already demanded. A larger caller
                    // must not lose useful discoveries when traversal ends.
                    return observed;
                }
                queue.extend(callee.x.require.iter().cloned());
                if callee.x.mode == Mode::Spec {
                    if let Some(b) = &callee.x.body {
                        queue.push_back(b.clone());
                    }
                }
            }
        }
        if remaining == 0 {
            return observed;
        }
    }
    observed
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Vec<Function> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    let mut catalog: Vec<_> =
        all.values().filter_map(|f| lemma(f).map(|dt| (f.x.name.clone(), dt))).collect();
    catalog.sort_by(|a, b| a.0.cmp(&b.0));
    fs.iter()
        .map(|f| {
            if catalog.is_empty()
                || f.x.name.path.krate == CrateId::Vstd
                || f.x.mode == Mode::Spec
                || f.x.attrs.infer
                || f.x.attrs.is_external_body
                || f.x.attrs.bit_vector
                || lemma(f).is_some()
            {
                return f.clone();
            }
            let Some(body) = &f.x.body else {
                return f.clone();
            };
            let ExprX::Block(stmts, tail) = &body.x else {
                return f.clone();
            };
            let observed = demands(f, &all);
            let selected: Vec<_> = catalog
                .iter()
                .filter(|(_, dt)| observed.contains(dt))
                .map(|(n, _)| n.clone())
                .collect();
            if selected.is_empty() {
                return f.clone();
            }
            let mut prefix: Vec<_> = selected
                .iter()
                .map(|n| {
                    crate::def::Spanned::new(
                        f.span.clone(),
                        StmtX::Expr(SpannedTyped::new(
                            &f.span,
                            &crate::ast_util::unit_typ(),
                            ExprX::Fuel(n.clone(), 1, true),
                        )),
                    )
                })
                .collect();
            prefix.extend(stmts.iter().cloned());
            let mut x = f.x.clone();
            x.body = Some(body.new_x(ExprX::Block(Arc::new(prefix), tail.clone())));
            report.view_bound_lemmas.push((x.name.clone(), selected));
            f.new_x(x)
        })
        .collect()
}
