//! Composition of ordinary, certified finite native iterator observations.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Operation {
    Count,
    Filter,
    Any,
}
pub(super) fn model(f: &Function) -> Option<Operation> {
    let x = &f.x;
    let segment = x.name.path.last_segment();
    let name = segment.as_str();
    let op = match name {
        "count" => Operation::Count,
        "filter" => Operation::Filter,
        "any" => Operation::Any,
        _ => return None,
    };
    (x.mode == Mode::Exec
        && x.body.is_none()
        && !x.attrs.infer
        && !x.attrs.is_unsafe
        && models::path_is(
            &x.name.path,
            CrateId::Core,
            &["iter", "traits", "iterator", "Iterator", name],
        )
        && x.proxy.as_ref().is_some_and(|p| {
            models::path_is(&p.x, CrateId::Vstd, &["std_specs", "iter", "ExIterator", name])
        })
        && x.params.len() == if op == Operation::Count { 1 } else { 2 })
    .then_some(op)
}
pub(super) fn operation(e: &Expr, all: &HashMap<Fun, Function>) -> Option<(Operation, Exprs)> {
    let ExprX::Call { target: CallTarget::Fun(_, n, _, _, _), args, post_args: None, body: None } =
        &e.x
    else {
        return None;
    };
    let f = all.get(n)?;
    let op = model(f)?;
    (args.len() == f.x.params.len()).then_some((op, args.clone()))
}

pub(super) fn packet(name: &Fun) -> bool {
    models::path_is(
        &name.path,
        CrateId::Vstd,
        &["std_specs", "iter_adapters", "finite_native_view"],
    )
}
fn local(e: &Expr, name: &VarIdent) -> bool {
    matches!(&e.x, ExprX::Var(n) if n == name)
        || matches!(&e.x, ExprX::ReadPlace(p, _) if matches!(&p.x,PlaceX::Local(n) if n==name))
}
pub(super) fn sequence(t: &Typ) -> Option<&Typ> {
    match &**t {
        TypX::Datatype(Dt::Path(p), ts, _)
            if ts.len() == 1 && models::path_is(p, CrateId::Vstd, &["seq", "Seq"]) =>
        {
            Some(&ts[0])
        }
        _ => None,
    }
}
fn header(f: &Function, mode: Mode) -> bool {
    let x = &f.x;
    x.mode == mode
        && matches!(x.kind, FunctionKind::Static)
        && !x.attrs.infer
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.prophecy_dependent
        && x.proxy.is_none()
        && x.attrs.autospec.is_none()
        && x.visibility.is_public()
        && (1..=3).contains(&x.params.len())
        && x.typ_params.len() <= 2
        && x.params.iter().all(|p| p.x.mode == mode && !p.x.user_mut)
        && trait_getters::shared(&x.params[0].x.typ).is_some()
        && x.params.iter().skip(1).all(|p| eligibility::scalar(&p.x.typ))
        && x.require.is_empty()
        && x.ensure.1.is_empty()
        && x.returns.is_none()
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
}
pub(super) fn observation(f: &Function) -> bool {
    header(f, Mode::Spec) && f.x.ensure.0.is_empty() && sequence(&f.x.inner_ret.x.typ).is_some()
}
pub(super) fn factory(f: &Function, all: &HashMap<Fun, Function>) -> Option<Function> {
    if !header(f, Mode::Exec) {
        return None;
    }
    let mut found = None;
    for post in f.x.ensure.0.iter() {
        let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, n, _, _, _),
            args,
            post_args: None,
            body: None,
        } = &post.x
        else {
            continue;
        };
        if !packet(n) || args.len() != 2 || !local(&args[0], &f.x.inner_ret.x.name) {
            continue;
        }
        let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, n, types, impls, _),
            args,
            post_args: None,
            body: None,
        } = &args[1].x
        else {
            return None;
        };
        let spec = all.get(n)?;
        if !observation(spec)
            || spec.x.typ_params != f.x.typ_params
            || types.len() != f.x.typ_params.len()
            || !impls.is_empty()
            || !types
                .iter()
                .zip(f.x.typ_params.iter())
                .all(|(t, p)| matches!(&**t,TypX::TypParam(n) if n==p))
            || args.len() != f.x.params.len()
            || spec.x.params.len() != f.x.params.len()
            || !args.iter().zip(f.x.params.iter()).all(|(a, p)| local(a, &p.x.name))
            || !spec
                .x
                .params
                .iter()
                .zip(f.x.params.iter())
                .all(|(a, b)| crate::ast_util::types_equal(&a.x.typ, &b.x.typ))
            || found.is_some()
        {
            return None;
        }
        found = Some(spec.clone());
    }
    found
}
pub(super) fn prepare(fs: &[Function], imported: &[Function]) -> Vec<Function> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    let mut factories = std::collections::HashSet::new();
    let mut specs = std::collections::HashSet::new();
    for f in fs {
        if let Some(spec) = factory(f, &all) {
            factories.insert(f.x.name.clone());
            specs.insert(spec.x.name.clone());
        }
    }
    fs.iter()
        .map(|f| {
            let mut x = f.x.clone();
            let attrs = Arc::make_mut(&mut x.attrs);
            attrs.infer_iterator_factory = factories.contains(&x.name);
            attrs.infer_iterator_observation = specs.contains(&x.name);
            f.new_x(x)
        })
        .collect()
}
pub(super) fn validate(fs: &[Function]) -> Result<(), VirErr> {
    let all = fs.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    for f in fs {
        if f.x.attrs.infer_iterator_observation && !observation(f)
            || f.x.attrs.infer_iterator_factory
                && !factory(f, &all).is_some_and(|s| s.x.attrs.infer_iterator_observation)
        {
            return Err(unsupported(&f.span, "invalid certified finite iterator view"));
        }
    }
    Ok(())
}
pub(super) fn source_call(
    e: &Expr,
    all: &HashMap<Fun, Function>,
) -> Option<(Function, Typs, Exprs)> {
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, n, ts, impls, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    let f = all.get(n)?;
    if !f.x.attrs.infer_iterator_factory
        || !impls.is_empty()
        || ts.len() != f.x.typ_params.len()
        || args.len() != f.x.params.len()
    {
        return None;
    }
    let spec = factory(f, all)?;
    spec.x.attrs.infer_iterator_observation.then_some((spec, ts.clone(), args.clone()))
}
pub(super) fn uses(f: &Function, all: &HashMap<Fun, Function>) -> bool {
    let mut found = false;
    if let Some(body) = &f.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
            found |= source_call(e, all).is_some();
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}
