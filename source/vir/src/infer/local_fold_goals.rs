//! Reuse checked fold annotations inside ordinary functions with a larger result.
//! A temporary analysis projection ends at the local accumulator. No projected
//! function or contract is installed: only validated proof decorations are moved
//! back onto the complete original body and checked by ordinary verification.
use super::*;

fn prepare_one(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
) -> Result<Function, VirErr> {
    let x = &f.x;
    if x.attrs.infer
        || x.mode != Mode::Exec
        || x.attrs.is_external_body
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || x.attrs.prophecy_dependent
        || x.attrs.exec_assume_termination
        || x.attrs.exec_allows_no_decreases_clause
        || x.attrs.bit_vector
        || x.attrs.integer_ring
        || x.proxy.is_some()
        || !matches!(x.kind, FunctionKind::Static)
        || x.params
            .iter()
            .any(|p| p.x.mode != Mode::Exec || p.x.user_mut || matches!(&*p.x.typ, TypX::MutRef(_)))
        || !x.decrease.is_empty()
        || x.decrease_by.is_some()
        || x.decrease_when.is_some()
    {
        return Ok(f.clone());
    }
    let Some(body) = &x.body else {
        return Ok(f.clone());
    };
    let ExprX::Block(stmts, Some(_)) = &body.x else {
        return Ok(f.clone());
    };
    let sources = owned::native_for::sources(f);
    if sources.len() != 1 {
        return Ok(f.clone());
    }
    let source = &sources[0];
    let mut accumulator = None;
    let mut ambiguous = false;
    crate::ast_visitor::expr_visitor_walk(&source.body, &mut |e| {
        if let ExprX::Call { args, .. } = &e.x {
            for a in args.iter() {
                if let ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) = &a.x {
                    if let PlaceX::Local(n) = &p.x {
                        if accumulator.as_ref().is_some_and(|(old, _)| old != n) {
                            ambiguous = true;
                        } else {
                            accumulator = Some((n.clone(), p.typ.clone()));
                        }
                    }
                }
            }
        }
        crate::ast_visitor::VisitorControlFlow::<()>::Recurse
    });
    let Some((name, typ)) = accumulator.filter(|_| !ambiguous) else {
        return Ok(f.clone());
    };
    let end = stmts.iter().position(|s| {
        let mut found = false;
        if let StmtX::Expr(e) = &s.x {
            crate::ast_visitor::expr_visitor_walk(e, &mut |e| {
                found |= e.span.id == source.loop_expr.span.id;
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
        }
        found
    });
    let Some(end) = end else {
        return Ok(f.clone());
    };
    let mut projected = x.clone();
    let mut ret = projected.inner_ret.x.clone();
    ret.typ = typ.clone();
    projected.inner_ret = projected.inner_ret.new_x(ret.clone());
    projected.outer_ret = projected.outer_ret.new_x(ret);
    projected.require = Arc::new(vec![]);
    projected.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    projected.returns = None;
    let attrs = Arc::make_mut(&mut projected.attrs);
    attrs.infer = true;
    attrs.infer_owned = true;
    attrs.infer_allow_approx = false;
    let tail = SpannedTyped::new(
        &body.span,
        &typ,
        ExprX::ReadPlace(
            SpannedTyped::new(&body.span, &typ, PlaceX::Local(name)),
            UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
        ),
    );
    projected.body = Some(body.new_x(ExprX::Block(Arc::new(stmts[..=end].to_vec()), Some(tail))));
    // Unsupported analysis never substitutes a guessed contract. Leave the
    // original function for its normal verifier diagnostics.
    let Ok((_, decorated)) = owned::infer(&f.new_x(projected), all, ds) else {
        return Ok(f.clone());
    };
    let mut out = x.clone();
    out.body = Some(owned::restore_body(f, &decorated)?);
    Ok(f.new_x(out))
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    ds: &[Datatype],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    fs.iter()
        .map(|f| {
            let result = prepare_one(f, &all, ds)?;
            if !Arc::ptr_eq(f, &result) {
                report.local_folds.push((
                    f.x.name.clone(),
                    owned::native_for::sources(f)[0].loop_expr.span.clone(),
                ));
            }
            Ok(result)
        })
        .collect()
}
