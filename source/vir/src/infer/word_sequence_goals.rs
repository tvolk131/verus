//! Bounded ordinary checked physical-bit frames for word-sequence updates.
//! No contract is changed; every generated annotation is checked as user proof.
use super::*;
#[cfg(test)]
use crate::ast_visitor::map_stmt_visitor_env;
use crate::ast_visitor::{
    VisitorControlFlow, VisitorScopeMap, expr_visitor_walk, map_expr_visitor, map_expr_visitor_env,
};
use std::collections::HashSet;
struct Fresh {
    ids: assistance::FreshIds,
}
impl Fresh {
    fn new(fs: &[Function], imported: &[Function]) -> Self {
        Self { ids: assistance::FreshIds::new(fs, imported) }
    }
    #[cfg(test)]
    fn id(&mut self) -> u64 {
        self.ids.id()
    }
    fn binding(&mut self) -> VarIdent {
        self.ids.named_binding("__word_sequence_")
    }
    fn stmt(&mut self, s: &Stmt) -> Result<Stmt, VirErr> {
        self.ids.spec_stmt(s)
    }
}

fn expr(s: &Span, t: &Typ, x: ExprX) -> Expr {
    SpannedTyped::new(s, t, x)
}
fn call(s: &Span, f: &Function, ts: Vec<Typ>, args: Vec<Expr>, t: &Typ) -> Expr {
    expr(
        s,
        t,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                f.x.name.clone(),
                Arc::new(ts),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(args),
            post_args: None,
            body: None,
        },
    )
}
fn snapshot(s: &Span, n: VarIdent, value: Expr) -> Stmt {
    crate::def::Spanned::new(
        s.clone(),
        StmtX::Decl {
            pattern: SpannedTyped::new(
                s,
                &value.typ,
                PatternX::Var(PatternBinding {
                    name: n,
                    by_ref: ByRef::No,
                    typ: value.typ.clone(),
                    user_mut: Some(false),
                    copy: false,
                }),
            ),
            mode: Some((Mode::Spec, DeclProph::DelayedInfer)),
            init: Some(SpannedTyped::new(s, &value.typ, PlaceX::Temporary(value.clone()))),
            els: None,
            assert_irrefutable: false,
        },
    )
}

// Follow the frontend's ghost-initialization scaffold: declaration in the
// surrounding scope, then a checked ghost assignment before any mutable borrow.
fn snapshot_pair(s: &Span, n: VarIdent, value: Expr) -> Vec<Stmt> {
    let decl = snapshot(s, n.clone(), value.clone());
    let StmtX::Decl { pattern, mode, els, assert_irrefutable, .. } = &decl.x else {
        unreachable!()
    };
    let decl = crate::def::Spanned::new(
        s.clone(),
        StmtX::Decl {
            pattern: pattern.clone(),
            mode: *mode,
            init: None,
            els: els.clone(),
            assert_irrefutable: *assert_irrefutable,
        },
    );
    let assign = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Assign {
            place: SpannedTyped::new(s, &value.typ, PlaceX::Local(n)),
            rhs: value.clone(),
            op: None,
            typ: value.typ.clone(),
            resolve: false,
        },
    );
    let ghost = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: assign },
    );
    vec![decl, crate::def::Spanned::new(s.clone(), StmtX::Expr(ghost))]
}

fn var(s: &Span, n: &VarIdent, t: &Typ) -> Expr {
    expr(s, t, ExprX::Var(n.clone()))
}
fn bytes<T: serde::Serialize + ?Sized>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).expect("VIR serialization")
}
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(t)) if ss.is_empty() => plain(t),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, t) => plain(t),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Temporary(t) => plain(t),
            _ => e,
        },
        _ => e,
    }
}
fn call_parts(e: &Expr) -> Option<(&Fun, &Typs, &Exprs)> {
    match &plain(e).x {
        ExprX::Call {
            target: CallTarget::Fun(k, n, ts, ..),
            args,
            post_args: None,
            body: None,
        } => Some(match k {
            CallTargetKind::DynamicResolved { resolved, typs, .. } => (resolved, typs, args),
            _ => (n, ts, args),
        }),
        _ => None,
    }
}
fn substitute(e: &Expr, f: &Function, ts: &Typs, args: &Exprs) -> Result<Expr, VirErr> {
    if ts.len() != f.x.typ_params.len() || args.len() != f.x.params.len() {
        return Err(unsupported(&e.span, "word-sequence substitution arity"));
    }
    // Reject capture rather than renaming any original source variable.
    let args_json = serde_json::to_value(args).expect("VIR arguments");
    let mut captures = false;
    expr_visitor_walk(e, &mut |e| {
        match &e.x {
            ExprX::Quant(_, bs, _) | ExprX::Closure(bs, _) => {
                for b in bs.iter() {
                    captures |= contains(&args_json, &serde_json::to_value(&b.name).unwrap());
                }
            }
            ExprX::Block(ss, _) => captures |= !ss.is_empty(),
            _ => (),
        }
        VisitorControlFlow::<()>::Recurse
    });
    if captures {
        return Err(unsupported(&e.span, "word expansion binder capture or local definition"));
    }
    let names: HashMap<_, _> =
        f.x.params.iter().zip(args.iter()).map(|(p, a)| (p.x.name.clone(), a.clone())).collect();
    let types = f.x.typ_params.iter().cloned().zip(ts.iter().cloned()).collect();
    map_expr_visitor_env(
        e,
        &mut VisitorScopeMap::new(),
        &mut (),
        &|_, scope, e| {
            Ok(match &e.x {
                ExprX::Var(n) if scope.get(n).is_none() && names.contains_key(n) => {
                    names[n].clone()
                }
                _ => e.clone(),
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(crate::sst_util::subst_typ(&types, t)),
        &|_, scope, p| {
            Ok(match &p.x {
                PlaceX::Local(n) if scope.get(n).is_none() && names.contains_key(n) => {
                    p.new_x(PlaceX::Temporary(names[n].clone()))
                }
                _ => p.clone(),
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
}
// Expand only supplied visible local spec bodies and the two checked word spec
// definitions. No imported private body, native representation or text matching.
fn expand(
    e: &Expr,
    all: &HashMap<Fun, Function>,
    budget: &mut usize,
    stack: &mut Vec<Fun>,
) -> Result<Expr, VirErr> {
    let state = std::cell::RefCell::new((budget, stack));
    map_expr_visitor(e, &|e| {
        let mut state = state.borrow_mut();
        let (budget, stack) = &mut *state;
        **budget =
            budget.checked_sub(1).ok_or_else(|| unsupported(&e.span, "word expansion budget"))?;
        let Some((n, ts, args)) = call_parts(e) else { return Ok(e.clone()) };
        let Some(f) = all.get(n) else { return Ok(e.clone()) };
        if f.x.mode != Mode::Spec
            || (n.path.krate == CrateId::Vstd
                && !["seq_word_bit", "seq_word_edit", "member"]
                    .iter()
                    .any(|s| models::path_is(&n.path, CrateId::Vstd, &["word", s])))
        {
            return Ok(e.clone());
        }
        let Some(b) = &f.x.body else { return Ok(e.clone()) };
        if f.x.attrs.is_external_body || stack.contains(n) || stack.len() >= 16 {
            return Err(unsupported(&e.span, "word recursive or opaque expansion"));
        }
        stack.push(n.clone());
        let out = expand(&substitute(b, f, ts, args)?, all, budget, stack);
        stack.pop();
        out
    })
}
// Typed structural normalization ignores diagnostic IDs only. Preserve roots,
// phases, field/impl paths, types, call identities, indices and all operators.
fn key(e: &Expr) -> Result<Vec<u8>, VirErr> {
    let e = map_expr_visitor(e, &|e| {
        let e = plain(e);
        let mut s = e.span.clone();
        s.id = 0;
        s.as_string = String::new();
        Ok(SpannedTyped::new(
            &s,
            &e.typ,
            match &e.x {
                ExprX::ReadPlace(p, _) => {
                    fn value(p: &Place) -> Expr {
                        match &p.x {
                            PlaceX::Temporary(e) => e.clone(),
                            PlaceX::DerefMut(p0) => expr(
                                &p.span,
                                &p.typ,
                                ExprX::Unary(UnaryOp::MutRefCurrent, value(p0)),
                            ),
                            PlaceX::Local(n) => expr(&p.span, &p.typ, ExprX::Var(n.clone())),
                            PlaceX::Field(op, p0) => expr(
                                &p.span,
                                &p.typ,
                                ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), value(p0)),
                            ),
                            _ => expr(
                                &p.span,
                                &p.typ,
                                ExprX::ReadPlace(
                                    p.clone(),
                                    UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: 0 },
                                ),
                            ),
                        }
                    }
                    return key_expr(&value(p));
                }
                _ => e.x.clone(),
            },
        ))
    })?;
    fn scrub(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                if m.contains_key("as_string") && m.contains_key("data") {
                    *v = serde_json::Value::Null;
                    return;
                }
                for v in m.values_mut() {
                    scrub(v)
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    scrub(v)
                }
            }
            _ => (),
        }
    }
    let mut j = serde_json::to_value(e).unwrap();
    scrub(&mut j);
    Ok(bytes(&j))
}
fn as_spec(e: &Expr) -> Result<Expr, VirErr> {
    map_expr_visitor(e, &|e| {
        Ok(match &e.x {
            ExprX::ReadPlace(p, _) => e.new_x(ExprX::ReadPlace(
                p.clone(),
                UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: 0 },
            )),
            _ => e.clone(),
        })
    })
}
fn key_expr(e: &Expr) -> Result<Expr, VirErr> {
    // Apply the same place normalization recursively without changing types.
    map_expr_visitor(e, &|e| Ok(plain(e).clone()))
}
fn same(a: &Expr, b: &Expr) -> bool {
    matches!((key(a),key(b)),(Ok(a),Ok(b)) if a==b)
}
fn root_path(e: &Expr) -> Option<(VarIdent, Vec<FieldOpr>)> {
    match &plain(e).x {
        ExprX::Var(n) => Some((n.clone(), vec![])),
        ExprX::ReadPlace(p, _) | ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => place_path(p),
        ExprX::Old(e)
        | ExprX::Unary(
            UnaryOp::MutRefCurrent | UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_),
            e,
        ) => root_path(e),
        ExprX::UnaryOpr(UnaryOpr::Field(op), e) => {
            let (n, mut p) = root_path(e)?;
            p.push(op.clone());
            Some((n, p))
        }
        ExprX::Call { args, .. } if args.len() == 1 => root_path(&args[0]),
        _ => None,
    }
}
fn place_path(p: &Place) -> Option<(VarIdent, Vec<FieldOpr>)> {
    match &p.x {
        PlaceX::Local(n) => Some((n.clone(), vec![])),
        PlaceX::Temporary(e) => root_path(e),
        PlaceX::DerefMut(p) => place_path(p),
        PlaceX::Field(op, p) => {
            let (n, mut p) = place_path(p)?;
            p.push(op.clone());
            Some((n, p))
        }
        _ => None,
    }
}
fn mutation_path(p: &Place, all: &HashMap<Fun, Function>) -> Option<(VarIdent, Vec<FieldOpr>)> {
    match &p.x {
        PlaceX::Temporary(e) => {
            if let Some(args) = model(e, collections::Model::IndexMut, all) {
                root_path(&args[0])
            } else {
                root_path(e)
            }
        }
        PlaceX::DerefMut(p) => mutation_path(p, all),
        PlaceX::Field(op, p) => {
            let (n, mut ops) = mutation_path(p, all)?;
            ops.push(op.clone());
            Some((n, ops))
        }
        _ => place_path(p),
    }
}
fn word_seq(t: &Typ) -> bool {
    iterators::sequence(t).is_some_and(|t| matches!(&**t, TypX::Int(IntRange::U(64))))
}
fn model<'a>(
    e: &'a Expr,
    k: collections::Model,
    all: &HashMap<Fun, Function>,
) -> Option<&'a Exprs> {
    let (n, _, args) = call_parts(e)?;
    (all.get(n).and_then(collections::model) == Some(k)).then_some(args)
}
fn literal(e: &Expr, n: u64) -> bool {
    matches!(&plain(e).x,ExprX::Const(Constant::Int(i)) if i==&BigInt::from(n))
}
fn division(e: &Expr) -> Option<(&Expr, &Expr)> {
    match &plain(e).x {
        ExprX::Binary(BinaryOp::Arith(ArithOp::EuclideanDiv(_)), a, b) => Some((a, b)),
        _ => None,
    }
}

fn library(all: &HashMap<Fun, Function>) -> Option<Function> {
    let mut fs = all.values().filter(|f| {
        models::path_is(&f.x.name.path, CrateId::Vstd, &["word", "checked_seq_bit_update_at"])
    });
    let f = fs.next()?.clone();
    if fs.next().is_some() {
        return None;
    }
    let x = &f.x;
    (x.mode == Mode::Proof
        && matches!(x.kind, FunctionKind::Static)
        && matches!(x.item_kind, ItemKind::Function)
        && x.proxy.is_none()
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.infer
        && !x.attrs.is_async
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.typ_params.is_empty()
        && x.typ_bounds.is_empty()
        && x.params.len() == 4
        && x.params.iter().all(|p| p.x.mode == Mode::Spec && !p.x.user_mut)
        && word_seq(&x.params[0].x.typ)
        && matches!(&*x.params[1].x.typ, TypX::Int(IntRange::Int))
        && matches!(&*x.params[2].x.typ, TypX::Bool)
        && matches!(&*x.params[3].x.typ, TypX::Int(IntRange::Int))
        && enums::unit(&x.inner_ret.x.typ)
        && x.ensure.0.len() == 1
        && x.ensure.1.is_empty())
    .then_some(f)
}
#[derive(Clone)]
struct Plan {
    entry: Expr,
    frame: Expr,
    bit: Expr,
    value: Expr,
    binder: VarIdent,
    library: Function,
}
fn header(f: &Function) -> bool {
    let x = &f.x;
    x.mode == Mode::Exec
        && x.name.path.krate != CrateId::Vstd
        && !x.attrs.infer
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.bit_vector
        && !x.attrs.prophecy_dependent
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.params.iter().all(|p| p.x.mode == Mode::Exec)
        && x.params.len() == 2
        && !x.params[0].x.user_mut
        && !x.params[1].x.user_mut
        && matches!(&*x.params[0].x.typ, TypX::MutRef(_))
        && matches!(&*x.params[1].x.typ, TypX::Int(_))
        && x.typ_params.is_empty()
        && x.typ_bounds.is_empty()
}
fn visible_catalog(f: &Function, all: &HashMap<Fun, Function>) -> HashMap<Fun, Function> {
    all.iter()
        .map(|(n, g)| {
            let visible = match (&g.x.body_visibility, &f.x.owning_module) {
                (BodyVisibility::Visibility(v), Some(m)) => crate::ast_util::is_visible_to(v, m),
                (BodyVisibility::Visibility(v), None) => v.restricted_to.is_none(),
                _ => false,
            };
            if visible {
                (n.clone(), g.clone())
            } else {
                let mut x = g.x.clone();
                x.body = None;
                (n.clone(), g.new_x(x))
            }
        })
        .collect()
}
// An exit annotation may precede only a readonly boolean result. In particular
// never propose it before a tail call/assignment which still mutates the state.
fn exit_ready(body: &Expr, all: &HashMap<Fun, Function>) -> bool {
    let ExprX::Block(_, tail) = &body.x else { return false };
    let Some(tail) = tail else { return true };
    if matches!(&tail.x, ExprX::Block(..)) {
        return exit_ready(tail, all);
    }
    if !matches!(&*tail.typ, TypX::Bool) {
        return false;
    }
    fn value(e: &Expr) -> &Expr {
        match &e.x {
            ExprX::EvalAndResolve(e, _) => value(e),
            _ => e,
        }
    }
    as_spec(value(tail)).is_ok_and(|e| assistance::pure(&e, all).is_ok())
}
fn plan(f: &Function, all: &HashMap<Fun, Function>) -> Option<Plan> {
    if !header(f) {
        return None;
    }
    let body = f.x.body.as_ref()?;
    let ExprX::Block(_, _) = &body.x else { return None };
    if !exit_ready(body, all) {
        return None;
    }
    let mut ineligible = false;
    let mut nodes = 0;
    expr_visitor_walk(body, &mut |e| {
        nodes += 1;
        if let ExprX::Block(ss, _) = &e.x {
            for s in ss.iter() {
                if let StmtX::Decl { pattern, .. } = &s.x {
                    if let PatternX::Var(b) = &pattern.x {
                        ineligible |= matches!(&*b.typ, TypX::MutRef(_))
                            && !matches!(b.name.1, VarIdentDisambiguate::ResInfTemp(_));
                    }
                }
            }
        }

        if let ExprX::If(_, branch, None) = &e.x {
            let mut panic = false;
            let mut writes = false;
            expr_visitor_walk(branch, &mut |e| {
                writes |= matches!(
                    &e.x,
                    ExprX::Assign { .. } | ExprX::BorrowMut(_) | ExprX::TwoPhaseBorrowMut(_)
                );
                panic |= call_parts(e).is_some_and(|(n, _, _)| {
                    models::path_is(&n.path, CrateId::Core, &["panicking", "panic"])
                        || models::path_is(&n.path, CrateId::Core, &["panicking", "panic_fmt"])
                });
                VisitorControlFlow::<()>::Recurse
            });
            ineligible |= !panic || writes;
        } else {
            ineligible |= matches!(
                &e.x,
                ExprX::Loop { .. }
                    | ExprX::Return(..)
                    | ExprX::Closure(..)
                    | ExprX::NonSpecClosure { .. }
                    | ExprX::If(..)
                    | ExprX::Match(..)
            );
        }
        if nodes > 4096 { VisitorControlFlow::Stop(()) } else { VisitorControlFlow::Recurse }
    });
    if ineligible || nodes > 4096 {
        return None;
    }
    let lib = library(all)?;
    let mut posts = vec![];
    let mut budget = 8192;
    for e in f.x.ensure.0.iter() {
        // Expand only conjunction-level local definitions, retaining the exact
        // requested quantifier as the proof goal (not a replacement property).
        fn clauses(
            e: &Expr,
            all: &HashMap<Fun, Function>,
            out: &mut Vec<Expr>,
            b: &mut usize,
            depth: usize,
        ) -> Result<(), VirErr> {
            if depth > 16 || *b == 0 {
                return Err(unsupported(&e.span, "word clauses budget"));
            }
            *b -= 1;
            match &plain(e).x {
                ExprX::Logical(LogicalOp::And, a, c) => {
                    clauses(a, all, out, b, depth + 1)?;
                    clauses(c, all, out, b, depth + 1)
                }
                _ => {
                    if let Some((n, ts, args)) = call_parts(e) {
                        if let Some(f) = all.get(n) {
                            if n.path.krate != CrateId::Vstd
                                && f.x.mode == Mode::Spec
                                && !f.x.attrs.is_external_body
                            {
                                if let Some(body) = &f.x.body {
                                    return clauses(
                                        &substitute(body, f, ts, args)?,
                                        all,
                                        out,
                                        b,
                                        depth + 1,
                                    );
                                }
                            }
                        }
                    }
                    out.push(plain(e).clone());
                    Ok(())
                }
            }
        }
        clauses(e, all, &mut posts, &mut budget, 0).ok()?;
    }
    let mut edits = vec![];
    for post in &posts {
        let ExprX::Binary(BinaryOp::Eq(_), cur, edit) = &plain(post).x else { continue };
        if !word_seq(&cur.typ) {
            continue;
        }
        let Some(args) = model(edit, collections::Model::SeqUpdate, all) else { continue };
        if args.len() != 3 {
            continue;
        }
        let Some((i, den)) = division(&args[1]) else { continue };
        if !literal(den, 64) {
            continue;
        }
        // The exact library edit definition must match the requested update.
        let Some(edit_fn) = all
            .values()
            .find(|f| models::path_is(&f.x.name.path, CrateId::Vstd, &["word", "seq_word_edit"]))
        else {
            continue;
        };
        for value in [true, false] {
            let v = expr(&post.span, &bool_typ(), ExprX::Const(Constant::Bool(value)));
            let expected = substitute(
                edit_fn.x.body.as_ref()?,
                edit_fn,
                &Arc::new(vec![]),
                &Arc::new(vec![args[0].clone(), i.clone(), v.clone()]),
            )
            .ok()?;
            if !same(edit, &expected) {
                continue;
            }
            edits.push((cur.clone(), args[0].clone(), i.clone(), v));
        }
    }
    if edits.len() != 1 {
        return None;
    }
    let (cur, entry, bit, value) = edits.pop()?;
    // Old/current/final roots are exact: compare entry->current with final->current.
    let current = assistance::current(&cur).ok()?;
    let entry_now = map_expr_visitor(&entry, &|e| {
        Ok(match &e.x {
            ExprX::Old(a) => a.clone(),
            _ => e.clone(),
        })
    })
    .ok()?;
    if !same(&current, &entry_now) || !assistance::future(&cur) {
        return None;
    }
    let bit_param = &f.x.params[1];
    let bit_var = expr(&bit.span, &bit_param.x.typ, ExprX::Var(bit_param.x.name.clone()));
    if !same(plain(&bit), &bit_var) {
        return None;
    }
    let mut frames = vec![];
    for frame in posts {
        let ExprX::Quant(q, bs, formula) = &plain(&frame).x else { continue };
        if q.quant != air::ast::Quant::Forall
            || bs.len() != 1
            || !matches!(&*bs[0].a, TypX::Int(IntRange::Int))
        {
            continue;
        }
        let ExprX::Logical(LogicalOp::Implies, guard, ensure) = &plain(formula).x else { continue };
        let j = var(&frame.span, &bs[0].name, &int_typ());
        let args = Arc::new(vec![entry.clone(), bit.clone(), value.clone(), j.clone()]);
        let expected = substitute(&lib.x.ensure.0[0], &lib, &Arc::new(vec![]), &args).ok()?;
        let replace_edit = map_expr_visitor(&expected, &|e| {
            Ok(
                if call_parts(e).is_some_and(|(n, _, _)| {
                    models::path_is(&n.path, CrateId::Vstd, &["word", "seq_word_edit"])
                }) {
                    cur.clone()
                } else {
                    e.clone()
                },
            )
        })
        .ok()?;
        let expand_one = |e: &Expr| expand(e, all, &mut 8192, &mut vec![]).ok();
        if !same(&expand_one(ensure)?, &expand_one(&replace_edit)?) {
            continue;
        }
        // Guard exactly equals the pointwise helper's j-bound, not a narrower domain.
        let req = substitute(lib.x.require.last()?, &lib, &Arc::new(vec![]), &args).ok()?;
        // W0 writes i/j bounds as separate requires expressions or a conjunction.
        if !same(guard, &req) {
            continue;
        }
        frames.push((frame.clone(), bs[0].name.clone()));
    }
    if frames.len() != 1 {
        return None;
    }
    let (frame, binder) = frames.pop()?;
    // Every observation is pure; only the receiver root and immutable scalar
    // input may be free. Further alias/ownership checks remain ordinary gates.
    let entry = map_expr_visitor(&entry, &|e| {
        Ok(match &e.x {
            ExprX::Old(a) => a.clone(),
            _ => e.clone(),
        })
    })
    .ok()?;
    let entry = as_spec(&entry).ok()?;
    assistance::pure(&entry, all).ok()?;
    let mut valid = true;
    let mut receiver = false;
    expr_visitor_walk(&entry, &mut |e| {
        match &e.x {
            ExprX::Var(n) => {
                valid &= n == &f.x.params[0].x.name;
                receiver |= n == &f.x.params[0].x.name;
            }
            ExprX::ReadPlace(p, _) => {
                fn local(p: &Place) -> Option<&VarIdent> {
                    match &p.x {
                        PlaceX::Local(n) => Some(n),
                        PlaceX::Field(_, p) | PlaceX::DerefMut(p) => local(p),
                        _ => None,
                    }
                }
                if let Some(n) = local(p) {
                    valid &= n == &f.x.params[0].x.name;
                    receiver |= n == &f.x.params[0].x.name;
                }
            }
            ExprX::Call { .. } => {
                valid &= model(e, collections::Model::View, all).is_some();
            }
            ExprX::Old(_) | ExprX::VarAt(..) | ExprX::Quant(..) | ExprX::Closure(..) => {
                valid = false
            }
            _ => (),
        }
        VisitorControlFlow::<()>::Recurse
    });
    if !valid || !receiver {
        return None;
    }
    let (root, path) = root_path(&entry)?;
    if root != f.x.params[0].x.name {
        return None;
    }
    let mut unambiguous = true;
    expr_visitor_walk(body, &mut |e| {
        let p = match &e.x {
            ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => Some(p),
            ExprX::Assign { place, .. } => Some(place),
            _ => None,
        };
        if let Some(p) = p {
            if let Some((n, ops)) = mutation_path(p, all) {
                if n == root {
                    unambiguous &= bytes(&ops) == bytes(&path)
                } else if !matches!(n.1, VarIdentDisambiguate::ResInfTemp(_))
                    && matches!(&*p.typ, TypX::MutRef(_))
                {
                    unambiguous = false
                }
            } else {
                unambiguous = false
            }
        }
        VisitorControlFlow::<()>::Recurse
    });
    if !unambiguous {
        return None;
    }
    Some(Plan { entry, frame, bit, value, binder, library: lib })
}

#[derive(Clone)]
struct Decoration {
    snapshot: VarIdent,
    quantified: VarIdent,
    prefix: Vec<Stmt>,
    suffix: Stmt,
}
fn template(
    f: &Function,
    p: &Plan,
    snapshot: &VarIdent,
    jname: &VarIdent,
    all: &HashMap<Fun, Function>,
) -> Result<(Vec<Stmt>, Stmt), VirErr> {
    let s = &f.span;
    let receiver = &f.x.params[0];
    let TypX::MutRef(payload) = &*receiver.x.typ else {
        return Err(unsupported(s, "word snapshot receiver"));
    };
    let input = var(s, &receiver.x.name, &receiver.x.typ);
    let current_root = expr(s, payload, ExprX::Unary(UnaryOp::MutRefCurrent, input.clone()));
    let old_root = expr(
        s,
        payload,
        ExprX::Unary(UnaryOp::MutRefCurrent, input.new_x(ExprX::Old(input.clone()))),
    );
    let entry_var = var(s, snapshot, payload);
    let sequence = map_expr_visitor(&p.entry, &|e| {
        Ok(if same(e, &current_root) { entry_var.clone() } else { e.clone() })
    })?;
    // Keep the exact requested visible observation calls, as in W0, so the
    // ordinary trigger inference sees the unchanged physical-bit observation.
    let frame = p.frame.clone();
    let frame = map_expr_visitor_env(
        &frame,
        &mut VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(match &e.x {
                ExprX::Var(n) if n == &p.binder => var(&e.span, jname, &e.typ),
                _ => e.clone(),
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p0| {
            Ok(match &p0.x {
                PlaceX::Local(n) if n == &p.binder => p0.new_x(PlaceX::Local(jname.clone())),
                _ => p0.clone(),
            })
        },
        &|_, _, p| Ok(p.clone()),
    )?;
    let mapped = map_expr_visitor(&frame, &|e| {
        // The entry occurrence is recognized BEFORE phase conversion; derive
        // its exact original old form from the unchanged requested postcondition.
        if same(e, &old_root) {
            return Ok(entry_var.clone());
        }
        Ok(match &e.x {
            ExprX::Var(n) if n == &p.binder => var(&e.span, jname, &int_typ()),
            ExprX::Unary(UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_), a) => {
                e.new_x(ExprX::Unary(UnaryOp::MutRefCurrent, a.clone()))
            }
            _ => e.clone(),
        })
    })?;
    // Quantifier binder replacement is independent of occurrence substitution.
    let ExprX::Quant(_, _, formula) = &plain(&mapped).x else {
        return Err(unsupported(s, "word frame quantifier"));
    };
    let ExprX::Logical(LogicalOp::Implies, require, ensure) = &plain(formula).x else {
        return Err(unsupported(s, "word frame guard"));
    };
    assistance::pure(&as_spec(require)?, all)?;
    assistance::pure(&as_spec(ensure)?, all)?;
    let library_call = call(
        s,
        &p.library,
        vec![],
        vec![as_spec(&sequence)?, as_spec(&p.bit)?, p.value.clone(), var(s, jname, &int_typ())],
        &crate::ast_util::unit_typ(),
    );
    let proof = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Block(
            Arc::new(vec![crate::def::Spanned::new(s.clone(), StmtX::Expr(library_call))]),
            None,
        ),
    );
    let assertion = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::AssertBy {
            vars: Arc::new(vec![Arc::new(VarBinderX { name: jname.clone(), a: int_typ() })]),
            require: as_spec(require)?,
            ensure: as_spec(ensure)?,
            proof,
        },
    );
    let ghost = expr(
        s,
        &crate::ast_util::unit_typ(),
        ExprX::Ghost { alloc_wrapper: false, tracked: false, expr: assertion },
    );
    Ok((
        snapshot_pair(s, snapshot.clone(), current_root),
        crate::def::Spanned::new(s.clone(), StmtX::Expr(ghost)),
    ))
}
fn normalized<T: serde::Serialize + ?Sized>(v: &T) -> Vec<u8> {
    fn walk(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                if m.contains_key("as_string") && m.contains_key("data") {
                    *v = serde_json::Value::Null;
                    return;
                }
                if m.contains_key("preliminary_kind") {
                    m.insert("id".into(), 0.into());
                }
                for v in m.values_mut() {
                    walk(v)
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    walk(v)
                }
            }
            _ => (),
        }
    }
    let mut v = serde_json::to_value(v).unwrap();
    walk(&mut v);
    bytes(&v)
}
fn contains(v: &serde_json::Value, n: &serde_json::Value) -> bool {
    v == n
        || match v {
            serde_json::Value::Object(m) => m.values().any(|v| contains(v, n)),
            serde_json::Value::Array(a) => a.iter().any(|v| contains(v, n)),
            _ => false,
        }
}
fn ids<T: serde::Serialize + ?Sized>(v: &T) -> Vec<u64> {
    fn walk(v: &serde_json::Value, o: &mut Vec<u64>) {
        match v {
            serde_json::Value::Object(m) => {
                if (m.contains_key("as_string") && m.contains_key("data"))
                    || m.contains_key("preliminary_kind")
                {
                    if let Some(n) = m.get("id").and_then(|n| n.as_u64()) {
                        o.push(n)
                    }
                }
                for v in m.values() {
                    walk(v, o)
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    walk(v, o)
                }
            }
            _ => (),
        }
    }
    let mut out = vec![];
    walk(&serde_json::to_value(v).unwrap(), &mut out);
    out
}
fn exit_block(body: &Expr, suffix: &Stmt, remove: bool) -> Result<Expr, VirErr> {
    let bad = || unsupported(&body.span, "word exit placement");
    let ExprX::Block(ss, tail) = &body.x else { return Err(bad()) };
    if let Some(t) = tail {
        if matches!(&t.x, ExprX::Block(..)) {
            return Ok(body.new_x(ExprX::Block(ss.clone(), Some(exit_block(t, suffix, remove)?))));
        }
    }
    let mut ss = ss.as_ref().clone();
    if remove {
        if ss.last().is_none_or(|s| bytes(s) != bytes(suffix)) {
            return Err(bad());
        }
        ss.pop();
    } else {
        ss.push(suffix.clone());
    }
    Ok(body.new_x(ExprX::Block(Arc::new(ss), tail.clone())))
}
// Diagnostic raw spans are intentionally skipped by VIR serialization. Retain
// their identity too: all original statements/leaf tails are the original Arcs;
// only the enclosing tail-block shells may be rebuilt during projection.
fn original_body(a: &Expr, b: &Expr) -> bool {
    if !Arc::ptr_eq(&a.span.raw_span, &b.span.raw_span) {
        return false;
    }
    match (&a.x, &b.x) {
        (ExprX::Block(sa, ta), ExprX::Block(sb, tb)) => {
            sa.len() == sb.len()
                && sa.iter().zip(sb.iter()).all(|(a, b)| Arc::ptr_eq(a, b))
                && match (ta, tb) {
                    (Some(a), Some(b)) => original_body(a, b),
                    (None, None) => true,
                    _ => false,
                }
        }
        _ => Arc::ptr_eq(a, b),
    }
}
fn untouched_metadata(f: &Function, g: &Function) -> bool {
    Arc::ptr_eq(&f.span.raw_span, &g.span.raw_span)
        && Arc::ptr_eq(&f.x.params, &g.x.params)
        && Arc::ptr_eq(&f.x.inner_ret, &g.x.inner_ret)
        && Arc::ptr_eq(&f.x.outer_ret, &g.x.outer_ret)
        && Arc::ptr_eq(&f.x.require, &g.x.require)
        && Arc::ptr_eq(&f.x.ensure.0, &g.x.ensure.0)
        && Arc::ptr_eq(&f.x.ensure.1, &g.x.ensure.1)
        && Arc::ptr_eq(&f.x.decrease, &g.x.decrease)
        && Arc::ptr_eq(&f.x.attrs, &g.x.attrs)
}
fn validate(
    f: &Function,
    g: &Function,
    d: &Decoration,
    all: &HashMap<Fun, Function>,
    prior: &HashSet<u64>,
) -> Result<(), VirErr> {
    let bad = || unsupported(&f.span, "word-sequence full Function projection");
    if d.snapshot == d.quantified {
        return Err(bad());
    }
    let orig = serde_json::to_value(f).unwrap();
    for n in [&d.snapshot, &d.quantified] {
        let name = serde_json::Value::String(n.0.to_string());
        if contains(&orig, &name)
            || all.values().any(|f| contains(&serde_json::to_value(f).unwrap(), &name))
        {
            return Err(bad());
        }
    }
    // Independently reselect from the original contracts/catalog. Producer
    // receipts cannot authorize a different init, goal, binding or library call.
    let p = plan(f, all).ok_or_else(bad)?;
    let (prefix, suffix) = template(f, &p, &d.snapshot, &d.quantified, all)?;
    if normalized(&prefix) != normalized(&d.prefix) || normalized(&suffix) != normalized(&d.suffix)
    {
        return Err(bad());
    }
    let mut seen = prior.clone();
    for n in ids(&d.prefix).into_iter().chain(ids(&d.suffix)) {
        if !seen.insert(n) {
            return Err(bad());
        }
    }
    let b = g.x.body.as_ref().ok_or_else(bad)?;
    let ExprX::Block(stmts, tail) = &b.x else { return Err(bad()) };
    if stmts.len() < 2 || bytes(&stmts[..2]) != bytes(&d.prefix) {
        return Err(bad());
    }
    let mut x = g.x.clone();
    x.body = Some(exit_block(
        &b.new_x(ExprX::Block(Arc::new(stmts[2..].to_vec()), tail.clone())),
        &d.suffix,
        true,
    )?);
    if !untouched_metadata(f, g)
        || !original_body(f.x.body.as_ref().unwrap(), x.body.as_ref().unwrap())
        || bytes(f) != bytes(&g.new_x(x))
    {
        return Err(bad());
    }
    Ok(())
}
pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all_functions = super::pipeline::function_map(fs, imported);
    let mut fresh: Option<Fresh> = None;
    let mut used: Option<HashSet<u64>> = None;
    let mut output = vec![];
    for f in fs {
        if !header(f) {
            output.push(f.clone());
            continue;
        }
        let all = visible_catalog(f, &all_functions);
        let Some(p) = plan(f, &all) else {
            output.push(f.clone());
            continue;
        };
        let fresh = fresh.get_or_insert_with(|| Fresh::new(fs, imported));
        let used = used.get_or_insert_with(|| fs.iter().chain(imported).flat_map(ids).collect());
        let snapshot = fresh.binding();
        let quantified = fresh.binding();
        let Ok((prefix, suffix)) = template(f, &p, &snapshot, &quantified, &all) else {
            output.push(f.clone());
            continue;
        };
        let prefix = prefix.iter().map(|s| fresh.stmt(s)).collect::<Result<Vec<_>, _>>()?;
        let suffix = fresh.stmt(&suffix)?;
        let d = Decoration { snapshot, quantified, prefix, suffix };
        let body = f.x.body.as_ref().unwrap();
        let ExprX::Block(stmts, tail) = &body.x else { unreachable!() };
        let mut ss = d.prefix.clone();
        ss.extend(stmts.iter().cloned());
        let mut x = f.x.clone();
        x.body = Some(exit_block(
            &body.new_x(ExprX::Block(Arc::new(ss), tail.clone())),
            &d.suffix,
            false,
        )?);
        let g = f.new_x(x);
        validate(f, &g, &d, &all, used)?;
        used.extend(ids(&d.prefix));
        used.extend(ids(&d.suffix));
        report.body_rechecks.insert(f.x.name.clone());
        report.word_sequence_frames.push((f.x.name.clone(), p.library.x.name.clone()));
        output.push(g);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{function, span};
    fn name(s: &str) -> VarIdent {
        VarIdent(Arc::new(s.into()), VarIdentDisambiguate::AirLocal)
    }
    fn truth() -> Expr {
        expr(&span(), &bool_typ(), ExprX::Const(Constant::Bool(true)))
    }
    fn integer(n: u64) -> Expr {
        expr(&span(), &int_typ(), ExprX::Const(Constant::Int(n.into())))
    }
    fn seq(t: Typ) -> Typ {
        Arc::new(TypX::Datatype(
            Dt::Path(Arc::new(PathX {
                krate: CrateId::Vstd,
                segments: Arc::new(vec![Arc::new("seq".into()), Arc::new("Seq".into())]),
            })),
            Arc::new(vec![t]),
            Arc::new(vec![]),
        ))
    }
    fn fun(
        path: &[&str],
        mode: Mode,
        params: Vec<(&str, Typ)>,
        ret: Typ,
        body: Option<Expr>,
    ) -> Function {
        let f = function("catalog", truth());
        let mut x = f.x.clone();
        x.name = Arc::new(FunX {
            path: Arc::new(PathX {
                krate: CrateId::Vstd,
                segments: Arc::new(path.iter().map(|n| Arc::new((*n).into())).collect()),
            }),
        });
        x.attrs = Arc::new(FunctionAttrsX::default());
        x.mode = mode;
        x.body = body;
        x.body_visibility = BodyVisibility::Visibility(Visibility { restricted_to: None });
        x.params = Arc::new(
            params
                .into_iter()
                .map(|(n, typ)| {
                    crate::def::Spanned::new(
                        span(),
                        ParamX {
                            name: name(n),
                            typ,
                            mode: Mode::Spec,
                            user_mut: false,
                            unwrapped_info: None,
                        },
                    )
                })
                .collect(),
        );
        let mut r = x.inner_ret.x.clone();
        r.typ = ret;
        x.inner_ret = x.inner_ret.new_x(r);
        x.outer_ret = x.inner_ret.clone();
        f.new_x(x)
    }
    // A tiny synthetic typed catalog isolates proposal/projection plumbing. Its
    // bodies are not evidence for the ordinary checked production word laws.
    fn fixture() -> (Function, HashMap<Fun, Function>) {
        let word = Arc::new(TypX::Int(IntRange::U(64)));
        let sequence = seq(word.clone());
        let g = Arc::new(TypX::TypParam(Arc::new("T".into())));
        let u = fun(
            &["seq", "Seq", "update"],
            Mode::Spec,
            vec![("s", seq(g.clone())), ("i", int_typ()), ("v", g.clone())],
            seq(g.clone()),
            None,
        );
        let mut ux = u.x.clone();
        ux.typ_params = Arc::new(vec![Arc::new("T".into())]);
        ux.typ_bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![g]),
        ))]);
        let u = u.new_x(ux);
        let s = var(&span(), &name("s"), &sequence);
        let i = var(&span(), &name("i"), &int_typ());
        let value = var(&span(), &name("v"), &bool_typ());
        let j = var(&span(), &name("j"), &int_typ());
        let div = expr(
            &span(),
            &int_typ(),
            ExprX::Binary(
                BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Allow)),
                i.clone(),
                integer(64),
            ),
        );
        let newword = expr(
            &span(),
            &word,
            ExprX::If(
                value.clone(),
                expr(&span(), &word, ExprX::Const(Constant::Int(0.into()))),
                Some(expr(&span(), &word, ExprX::Const(Constant::Int(1.into())))),
            ),
        );
        let edit = call(&span(), &u, vec![word], vec![s.clone(), div, newword], &sequence);
        let ed = fun(
            &["word", "seq_word_edit"],
            Mode::Spec,
            vec![("s", sequence.clone()), ("i", int_typ()), ("v", bool_typ())],
            sequence.clone(),
            Some(edit),
        );
        // An uninterpreted bit observer makes selection sensitive to exact
        // roots/indices without duplicating production bit-vector reasoning.
        let bit = fun(
            &["word", "seq_word_bit"],
            Mode::Spec,
            vec![("s", sequence.clone()), ("j", int_typ())],
            bool_typ(),
            None,
        );
        let edited =
            call(&span(), &ed, vec![], vec![s.clone(), i.clone(), value.clone()], &sequence);
        let eq = expr(
            &span(),
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), j.clone(), i.clone()),
        );
        let before = call(&span(), &bit, vec![], vec![s.clone(), j.clone()], &bool_typ());
        let expected = expr(&span(), &bool_typ(), ExprX::If(eq, value.clone(), Some(before)));
        let ensure = expr(
            &span(),
            &bool_typ(),
            ExprX::Binary(
                BinaryOp::Eq(Mode::Spec),
                call(&span(), &bit, vec![], vec![edited, j.clone()], &bool_typ()),
                expected,
            ),
        );
        let lib = fun(
            &["word", "checked_seq_bit_update_at"],
            Mode::Proof,
            vec![("s", sequence.clone()), ("i", int_typ()), ("v", bool_typ()), ("j", int_typ())],
            crate::ast_util::unit_typ(),
            None,
        );
        let mut lx = lib.x.clone();
        lx.ensure.0 = Arc::new(vec![ensure.clone()]);
        lx.require = Arc::new(vec![truth()]);
        let lib = lib.new_x(lx);
        let body =
            expr(&span(), &crate::ast_util::unit_typ(), ExprX::Block(Arc::new(vec![]), None));
        let f = function("writer", body);
        let mut x = f.x.clone();
        x.attrs = Arc::new(FunctionAttrsX::default());
        let mut ps = x.params.as_ref().clone();
        let mut p = ps[0].x.clone();
        p.name = name("receiver");
        p.typ = Arc::new(TypX::MutRef(sequence.clone()));
        ps[0] = ps[0].new_x(p);
        let mut p = ps[1].x.clone();
        p.name = name("index");
        p.typ = int_typ();
        ps[1] = ps[1].new_x(p);
        x.params = Arc::new(ps);
        let receiver = var(&span(), &name("receiver"), &x.params[0].x.typ);
        let old = expr(
            &span(),
            &sequence,
            ExprX::Unary(UnaryOp::MutRefCurrent, receiver.new_x(ExprX::Old(receiver.clone()))),
        );
        let cur = expr(
            &span(),
            &sequence,
            ExprX::Unary(UnaryOp::MutRefFuture(MutRefFutureSourceName::Final), receiver),
        );
        let index = var(&span(), &name("index"), &int_typ());
        let args = Arc::new(vec![old, index, truth(), j]);
        let edited = substitute(
            ed.x.body.as_ref().unwrap(),
            &ed,
            &Arc::new(vec![]),
            &Arc::new(args[..3].to_vec()),
        )
        .unwrap();
        let update = expr(
            &span(),
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), cur.clone(), edited),
        );
        let expected = substitute(&ensure, &lib, &Arc::new(vec![]), &args).unwrap();
        let expected = map_expr_visitor(&expected, &|e| {
            Ok(if call_parts(e).is_some_and(|(n, _, _)| n == &ed.x.name) {
                cur.clone()
            } else {
                e.clone()
            })
        })
        .unwrap();
        let formula =
            expr(&span(), &bool_typ(), ExprX::Logical(LogicalOp::Implies, truth(), expected));
        let frame = expr(
            &span(),
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: name("j"), a: int_typ() })]),
                formula,
            ),
        );
        x.ensure.0 = Arc::new(vec![update, frame]);
        let f = f.new_x(x);
        (f, [u, ed, bit, lib].into_iter().map(|f| (f.x.name.clone(), f)).collect())
    }
    fn decorate(
        f: &Function,
        all: &HashMap<Fun, Function>,
    ) -> (Function, Decoration, HashSet<u64>) {
        let p = plan(f, all).expect("selected synthetic fixture");
        let mut fresh = Fresh::new(&[f.clone()], &all.values().cloned().collect::<Vec<_>>());
        let sn = fresh.binding();
        let j = fresh.binding();
        let (prefix, suffix) = template(f, &p, &sn, &j, all).unwrap();
        let d = Decoration {
            snapshot: sn,
            quantified: j,
            prefix: prefix.iter().map(|s| fresh.stmt(s).unwrap()).collect(),
            suffix: fresh.stmt(&suffix).unwrap(),
        };
        let b = f.x.body.as_ref().unwrap();
        let ExprX::Block(ss, t) = &b.x else { unreachable!() };
        let mut ns = d.prefix.clone();
        ns.extend(ss.iter().cloned());
        let mut x = f.x.clone();
        x.body = Some(
            exit_block(&b.new_x(ExprX::Block(Arc::new(ns), t.clone())), &d.suffix, false).unwrap(),
        );
        let used = [f.clone()].iter().chain(all.values()).flat_map(ids).collect();
        (f.new_x(x), d, used)
    }
    fn scaffold(f: &Function, d: &Decoration) -> Function {
        let b = f.x.body.as_ref().unwrap();
        let ExprX::Block(ss, t) = &b.x else { unreachable!() };
        let mut ns = d.prefix.clone();
        ns.extend(ss.iter().cloned());
        let mut x = f.x.clone();
        x.body = Some(
            exit_block(&b.new_x(ExprX::Block(Arc::new(ns), t.clone())), &d.suffix, false).unwrap(),
        );
        f.new_x(x)
    }
    #[test]
    fn independent_projection_rejects_metadata_contract_exec_and_receipt_mutations() {
        let (f, all) = fixture();
        let (g, d, used) = decorate(&f, &all);
        validate(&f, &g, &d, &all, &used).unwrap();
        for kind in 0..6 {
            let mut x = g.x.clone();
            match kind {
                0 => x.require = Arc::new(vec![truth()]),
                1 => x.decrease = Arc::new(vec![integer(3)]),
                2 => x.hidden = Arc::new(vec![x.name.clone()]),
                3 => {
                    x.attrs =
                        Arc::new(FunctionAttrsX { is_unsafe: true, ..x.attrs.as_ref().clone() })
                }
                4 => x.ensure.0 = Arc::new(vec![truth()]),
                _ => x.body = Some(expr(&span(), &bool_typ(), ExprX::Const(Constant::Bool(false)))),
            }
            assert!(validate(&f, &g.new_x(x), &d, &all, &used).is_err());
        }
        let mut bad = d.clone();
        bad.snapshot = f.x.params[0].x.name.clone();
        assert!(validate(&f, &g, &bad, &all, &used).is_err());
        let mut bad = d.clone();
        bad.prefix[1] = bad.prefix[0].clone();
        assert!(validate(&f, &g, &bad, &all, &used).is_err());
        let mut used = used.clone();
        used.extend(ids(&d.suffix));
        assert!(validate(&f, &g, &d, &all, &used).is_err());
    }
    #[test]
    fn catalog_domain_index_value_and_visibility_fail_closed() {
        let (f, all) = fixture();
        assert!(plan(&f, &all).is_some());
        for kind in 0..5 {
            let mut a = all.clone();
            let lib = library(&a).unwrap();
            let mut x = lib.x.clone();
            match kind {
                0 => x.mode = Mode::Exec,
                1 => {
                    x.attrs = Arc::new(FunctionAttrsX {
                        is_external_body: true,
                        ..x.attrs.as_ref().clone()
                    })
                }
                2 => x.typ_params = Arc::new(vec![Arc::new("T".into())]),
                3 => x.ensure.0 = Arc::new(vec![]),
                _ => {
                    let mut ps = x.params.as_ref().clone();
                    let mut p = ps[0].x.clone();
                    p.typ = int_typ();
                    ps[0] = ps[0].new_x(p);
                    x.params = Arc::new(ps);
                }
            }
            a.insert(x.name.clone(), lib.new_x(x));
            assert!(plan(&f, &a).is_none());
        }
        let mut x = f.x.clone();
        x.params = Arc::new(vec![]);
        assert!(plan(&f.new_x(x), &all).is_none());
        let mut x = f.x.clone();
        x.ensure.0 = Arc::new(vec![
            f.x.ensure.0[0].clone(),
            f.x.ensure.0[0].clone(),
            f.x.ensure.0[1].clone(),
        ]);
        assert!(plan(&f.new_x(x), &all).is_none());
        let visible = visible_catalog(&f, &all);
        assert!(plan(&f, &visible).is_some());
        let mut a = all.clone();
        let ed = a
            .values()
            .find(|f| f.x.name.path.segments.last().unwrap().as_str() == "seq_word_edit")
            .unwrap()
            .clone();
        let mut x = ed.x.clone();
        x.body_visibility = BodyVisibility::Uninterpreted;
        a.insert(x.name.clone(), ed.new_x(x));
        assert!(plan(&f, &visible_catalog(&f, &a)).is_none());
    }
    #[test]
    fn allocator_is_global_across_preceding_decorations() {
        let (f, all) = fixture();
        let (g, d, _) = decorate(&f, &all);
        let mut a = Fresh::new(&[f.clone(), g.clone()], &all.values().cloned().collect::<Vec<_>>());
        let n = a.id();
        assert!(!ids(&g).contains(&n));
        let s = a.stmt(&d.suffix).unwrap();
        let mut used: HashSet<_> = ids(&g).into_iter().collect();
        for n in ids(&s) {
            assert!(used.insert(n));
        }
    }
    fn replace<T: serde::Serialize + serde::de::DeserializeOwned, A: serde::Serialize>(
        v: &T,
        old: &A,
        new: &A,
    ) -> T {
        fn walk(v: &mut serde_json::Value, a: &serde_json::Value, b: &serde_json::Value) {
            if v == a {
                *v = b.clone();
                return;
            }
            match v {
                serde_json::Value::Object(m) => {
                    for v in m.values_mut() {
                        walk(v, a, b)
                    }
                }
                serde_json::Value::Array(v) => {
                    for v in v {
                        walk(v, a, b)
                    }
                }
                _ => (),
            }
        }
        let mut v = serde_json::to_value(v).unwrap();
        walk(&mut v, &serde_json::to_value(old).unwrap(), &serde_json::to_value(new).unwrap());
        serde_json::from_value(v).unwrap()
    }
    #[test]
    fn consistent_scaffold_forgery_and_duplicate_calls_are_rejected() {
        let (f, all) = fixture();
        let (g, d, used) = decorate(&f, &all);
        // Mutate both the decorated Function and its receipt. Equality against
        // producer data alone would accept these; independent regeneration must not.
        for k in 0..5 {
            let suffix = map_stmt_visitor_env(
                &d.suffix,
                &mut VisitorScopeMap::new(),
                &mut (),
                &|_, _, e| {
                    Ok(match &e.x {
                        ExprX::Call { target, args, post_args, body }
                            if call_parts(e)
                                .is_some_and(|(n, _, _)| n == &library(&all).unwrap().x.name) =>
                        {
                            let mut args = args.as_ref().clone();
                            match k {
                                0 => {
                                    args[0] = expr(
                                        &e.span,
                                        &args[0].typ,
                                        ExprX::Var(f.x.params[0].x.name.clone()),
                                    )
                                }
                                1 => args[1] = integer(63),
                                2 => {
                                    args[2] = expr(
                                        &e.span,
                                        &bool_typ(),
                                        ExprX::Const(Constant::Bool(false)),
                                    )
                                }
                                3 => args[3] = integer(0),
                                _ => (),
                            }
                            let call = e.new_x(ExprX::Call {
                                target: target.clone(),
                                args: Arc::new(args),
                                post_args: post_args.clone(),
                                body: body.clone(),
                            });
                            if k == 4 {
                                e.new_x(ExprX::Block(
                                    Arc::new(vec![crate::def::Spanned::new(
                                        e.span.clone(),
                                        StmtX::Expr(call.clone()),
                                    )]),
                                    Some(call),
                                ))
                            } else {
                                call
                            }
                        }
                        _ => e.clone(),
                    })
                },
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| Ok(t.clone()),
                &|_, _, p| Ok(p.clone()),
                &|_, _, p| Ok(p.clone()),
            )
            .unwrap()
            .pop()
            .unwrap();
            let mut corrupt = d.clone();
            corrupt.suffix = suffix.clone();
            let forged = scaffold(&f, &corrupt);
            assert!(validate(&f, &forged, &corrupt, &all, &used).is_err());
        }
        for n in [f.x.params[0].x.name.clone(), f.x.params[1].x.name.clone(), name("j")] {
            let mut corrupt = d.clone();
            corrupt.snapshot = n.clone();
            corrupt.prefix = replace(&d.prefix, &d.snapshot, &n);
            corrupt.suffix = replace(&d.suffix, &d.snapshot, &n);
            let forged = scaffold(&f, &corrupt);
            assert!(validate(&f, &forged, &corrupt, &all, &used).is_err());
        }
        // Relocate the exact suffix before the original executable tail chain.
        let mut x = g.x.clone();
        let b = x.body.as_ref().unwrap();
        let stripped = exit_block(b, &d.suffix, true).unwrap();
        let ExprX::Block(ss, t) = &stripped.x else { unreachable!() };
        let mut ss = ss.as_ref().clone();
        ss.insert(0, d.suffix.clone());
        x.body = Some(stripped.new_x(ExprX::Block(Arc::new(ss), t.clone())));
        assert!(validate(&f, &g.new_x(x), &d, &all, &used).is_err());
    }

    #[test]
    fn authored_alias_and_conditional_forms_skip_unchanged() {
        let (f, all) = fixture();
        let mut x = f.x.clone();
        let b = x.body.as_ref().unwrap().clone();
        let alias = name("alias");
        let mut ss =
            snapshot_pair(&span(), alias, var(&span(), &x.params[0].x.name, &x.params[0].x.typ));
        let ExprX::Block(original, tail) = &b.x else { unreachable!() };
        ss.extend(original.iter().cloned());
        x.body = Some(b.new_x(ExprX::Block(Arc::new(ss), tail.clone())));
        assert!(plan(&f.new_x(x), &all).is_none());
        let mut x = f.x.clone();
        x.body = Some(expr(
            &span(),
            &crate::ast_util::unit_typ(),
            ExprX::Block(
                Arc::new(vec![crate::def::Spanned::new(
                    span(),
                    StmtX::Expr(expr(
                        &span(),
                        &crate::ast_util::unit_typ(),
                        ExprX::If(truth(), b.clone(), Some(b.clone())),
                    )),
                )]),
                None,
            ),
        ));
        assert!(plan(&f.new_x(x), &all).is_none());
    }

    #[test]
    fn translated_place_reads_preserve_snapshot_and_quantifier_binding() {
        fn places(f: &Function) -> Function {
            fn transform(e: &Expr) -> Expr {
                map_expr_visitor(e, &|e| {
                    Ok(match &e.x {
                        ExprX::Var(n) => e.new_x(ExprX::ReadPlace(
                            SpannedTyped::new(&e.span, &e.typ, PlaceX::Local(n.clone())),
                            UnfinalizedReadKind { preliminary_kind: ReadKind::Copy, id: 0 },
                        )),
                        _ => e.clone(),
                    })
                })
                .unwrap()
            }
            let mut x = f.x.clone();
            x.body = x.body.as_ref().map(transform);
            x.ensure.0 = Arc::new(x.ensure.0.iter().map(transform).collect());
            x.require = Arc::new(x.require.iter().map(transform).collect());
            f.new_x(x)
        }
        let (f, all) = fixture();
        let f = places(&f);
        let all = all.into_iter().map(|(n, f)| (n, places(&f))).collect();
        let (g, d, used) = decorate(&f, &all);
        validate(&f, &g, &d, &all, &used).unwrap();
        let j = serde_json::to_value(name("j")).unwrap();
        assert!(!contains(&serde_json::to_value(&d.suffix).unwrap(), &j));
    }
    #[test]
    fn visible_definition_substitution_rejects_capture() {
        let s = span();
        let n = name("bound");
        let f = fun(
            &["test", "definition"],
            Mode::Spec,
            vec![("argument", int_typ())],
            bool_typ(),
            None,
        );
        let quant = expr(
            &s,
            &bool_typ(),
            ExprX::Quant(
                Quant { quant: air::ast::Quant::Forall },
                Arc::new(vec![Arc::new(VarBinderX { name: n.clone(), a: int_typ() })]),
                truth(),
            ),
        );
        assert!(
            substitute(&quant, &f, &Arc::new(vec![]), &Arc::new(vec![var(&s, &n, &int_typ())]))
                .is_err()
        );
    }
    #[test]
    fn effectful_or_nonboolean_tails_skip_without_moving_original_code() {
        let (f, all) = fixture();
        let body = f.x.body.as_ref().unwrap();
        for tail in [integer(0), call(&span(), &f, vec![], vec![], &bool_typ())] {
            let mut x = f.x.clone();
            x.body = Some(body.new_x(ExprX::Block(Arc::new(vec![]), Some(tail))));
            assert!(plan(&f.new_x(x), &all).is_none());
        }
        let mut x = f.x.clone();
        x.body = Some(body.new_x(ExprX::Block(Arc::new(vec![]), Some(truth()))));
        assert!(plan(&f.new_x(x), &all).is_some());
    }
}
