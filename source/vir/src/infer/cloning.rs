//! Checked ordinary native Clone contracts for a private direct-view wrapper.
//! Equality is only a candidate: the original call and every precision query
//! must prove it from the actual verified relation and any supplied clone laws.
use super::*;

/// Bind the library artifact supplying Clone's relational vocabulary and the
/// checked builtin tuple shim contracts. This does not approve arbitrary calls.
pub(super) fn model(f: &Function) -> bool {
    let p = &f.x.name.path;
    if models::path_is(p, CrateId::Vstd, &["pervasive", "cloned"]) {
        return f.x.mode == Mode::Spec
            && f.x.typ_params.len() == 1
            && f.x.params.len() == 2
            && same(&f.x.inner_ret.x.typ, &bool_typ());
    }
    p.krate == CrateId::Vstd
        && p.segments.len() == 3
        && p.segments[0].as_str() == "std_specs"
        && p.segments[1].as_str() == "clone"
        && (1..=4).any(|n| {
            p.segments[2].as_str() == format!("tuple_clone_{n}")
                && f.x.mode == Mode::Exec
                && f.x.typ_params.len() == n
                && f.x.params.len() == 1
                && f.x.ensure.0.len() == n
                && f.x.require.is_empty()
                && !f.x.attrs.is_external_body
                && matches!(&f.x.kind, FunctionKind::TraitMethodImpl { .. })
        })
}
fn same(a: &Typ, b: &Typ) -> bool {
    crate::ast_util::types_equal(a, b)
}
fn shared(t: &Typ) -> Option<&Typ> {
    match &**t {
        TypX::Decorate(TypDecoration::Ref, None, t) => Some(t),
        _ => None,
    }
}
fn clean(f: &Function) -> bool {
    let x = &f.x;
    !x.attrs.infer
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.prophecy_dependent
        && x.proxy.is_none()
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.attrs.autospec.is_none()
        && x.typ_params.len() <= 2
        && x.typ_bounds.len() <= 24
        && matches!(x.item_kind, ItemKind::Function)
        && !x.params.iter().any(|p| p.x.user_mut)
        && x.require.is_empty()
        && x.ensure.1.is_empty()
        && x.returns.is_none()
        && same(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
}
fn local(e: &Expr, n: &VarIdent) -> bool {
    matches!(&e.x,ExprX::Var(v) if v==n)
}
fn relation(f: &Function, all: &HashMap<Fun, Function>, ds: &[Datatype]) -> Option<Function> {
    let x = &f.x;
    if !clean(f)
        || x.mode != Mode::Exec
        || x.params.len() != 1
        || x.ensure.0.len() != 1
        || x.inner_ret.x.mode != Mode::Exec
        || x.params[0].x.mode != Mode::Exec
    {
        return None;
    }
    let FunctionKind::TraitMethodImpl { method, trait_path, inherit_body_from: None, .. } = &x.kind
    else {
        return None;
    };
    if !models::path_is(trait_path, CrateId::Core, &["clone", "Clone"])
        || !models::path_is(&method.path, CrateId::Core, &["clone", "Clone", "clone"])
    {
        return None;
    }
    let typ = shared(&x.params[0].x.typ)?;
    if !same(typ, &x.inner_ret.x.typ) || sequence_views::find(typ, all, ds).is_none() {
        return None;
    }
    let ExprX::Call {
        target: CallTarget::Fun(CallTargetKind::Static, name, types, impls, _),
        args,
        post_args: None,
        body: None,
    } = &x.ensure.0[0].x
    else {
        return None;
    };
    if args.len() != 2
        || !impls.is_empty()
        || types.len() != x.typ_params.len()
        || !local(&args[0], &x.params[0].x.name)
        || !local(&args[1], &x.inner_ret.x.name)
    {
        return None;
    }
    let spec = all.get(name)?;
    let s = &spec.x;
    if !clean(spec)
        || s.mode != Mode::Spec
        || s.params.len() != 2
        || s.typ_params != x.typ_params
        || !s.ensure.0.is_empty()
        || !same(&s.inner_ret.x.typ, &bool_typ())
        || s.params.iter().any(|p| p.x.mode != Mode::Spec)
        || s.inner_ret.x.mode != Mode::Spec
        || !same(&s.params[0].x.typ, &x.params[0].x.typ)
        || !same(&s.params[1].x.typ, typ)
    {
        return None;
    }
    Some(spec.clone())
}
pub(super) fn prepare(fs: &[Function], imported: &[Function], ds: &[Datatype]) -> Vec<Function> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut marked = std::collections::HashSet::new();
    for f in fs {
        if let Some(spec) = relation(f, &all, ds) {
            marked.insert(f.x.name.clone());
            marked.insert(spec.x.name.clone());
        }
    }
    fs.iter()
        .map(|f| {
            let mut x = f.x.clone();
            Arc::make_mut(&mut x.attrs).infer_clone_contract = marked.contains(&x.name);
            f.new_x(x)
        })
        .collect()
}
pub(super) fn validate(fs: &[Function], ds: &[Datatype]) -> Result<(), VirErr> {
    let all = fs.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut seen = std::collections::HashSet::new();
    for f in fs.iter().filter(|f| f.x.attrs.infer_clone_contract && f.x.mode == Mode::Exec) {
        let spec = relation(f, &all, ds)
            .filter(|s| s.x.attrs.infer_clone_contract)
            .ok_or_else(|| unsupported(&f.span, "invalid checked native Clone relation"))?;
        seen.insert(f.x.name.clone());
        seen.insert(spec.x.name.clone());
    }
    if let Some(f) = fs.iter().find(|f| f.x.attrs.infer_clone_contract && !seen.contains(&f.x.name))
    {
        return Err(unsupported(&f.span, "unpaired native Clone relation"));
    }
    Ok(())
}
pub(super) fn call(e: &Expr, all: &HashMap<Fun, Function>) -> bool {
    let ExprX::Call {
        target:
            CallTarget::Fun(
                CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. },
                name,
                ..,
            ),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return false;
    };
    models::path_is(&name.path, CrateId::Core, &["clone", "Clone", "clone"])
        && args.len() == 1
        && all
            .get(resolved)
            .is_some_and(|f| f.x.attrs.infer_clone_contract && f.x.mode == Mode::Exec)
}
