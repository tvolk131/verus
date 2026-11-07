//! Closed Option models using existing specifications and checked generic bridges.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NativeOption {
    ThenSome,
    MapOr,
    IsSome,
}

pub(super) fn uses_native_option(function: &Function, all: &HashMap<Fun, Function>) -> bool {
    let mut found = false;
    if let Some(body) = &function.x.body {
        crate::ast_visitor::expr_visitor_walk(body, &mut |expression| {
            if let ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                post_args: None,
                body: None,
                ..
            } = &expression.x
            {
                found |= all.get(name).and_then(native_option).is_some();
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}

pub(super) fn native_option(function: &Function) -> Option<NativeOption> {
    let definition = &function.x;
    let path = &definition.name.path;
    if definition.mode != Mode::Exec
        || !matches!(definition.kind, FunctionKind::Static)
        || !matches!(definition.item_kind, ItemKind::Function)
        || definition.inner_ret.x.mode != Mode::Exec
        || definition.outer_ret.x.mode != Mode::Exec
        || definition.attrs.infer
        || definition.attrs.is_unsafe
        || definition.attrs.is_async
        || definition
            .params
            .iter()
            .any(|parameter| parameter.x.mode != Mode::Exec || parameter.x.user_mut)
        || !crate::ast_util::types_equal(&definition.inner_ret.x.typ, &definition.outer_ret.x.typ)
        || !definition.ensure.1.is_empty()
    {
        return None;
    }
    let parameters: Vec<_> = definition
        .typ_params
        .iter()
        .map(|parameter| Arc::new(TypX::TypParam(parameter.clone())))
        .collect();
    let same = crate::ast_util::types_equal;
    let option = |typ: &Typ, element: &Typ| {
        matches!(&**typ,
        TypX::Datatype(Dt::Path(path), types, _) if models::path_is(path, CrateId::Core, &["option", "Option"])
            && types.len() == 1 && same(&types[0], element))
    };
    if models::path_is(path, CrateId::Vstd, &["native_option", "then_some"])
        && definition.proxy.is_none()
        && !definition.attrs.is_external_body
        && parameters.len() == 1
        && definition.params.len() == 2
        && definition.require.is_empty()
        && definition.ensure.0.len() == 1
        && same(&definition.params[0].x.typ, &bool_typ())
        && same(&definition.params[1].x.typ, &parameters[0])
        && option(&definition.inner_ret.x.typ, &parameters[0])
    {
        return Some(NativeOption::ThenSome);
    }
    if models::path_is(path, CrateId::Vstd, &["native_option", "map_or"])
        && definition.proxy.is_none()
        && !definition.attrs.is_external_body
        && parameters.len() == 3
        && definition.params.len() == 3
        && definition.require.len() == 1
        && definition.ensure.0.len() == 2
        && option(&definition.params[0].x.typ, &parameters[0])
        && same(&definition.params[1].x.typ, &parameters[1])
        && same(&definition.params[2].x.typ, &parameters[2])
        && same(&definition.inner_ret.x.typ, &parameters[1])
    {
        return Some(NativeOption::MapOr);
    }
    if path.krate == CrateId::Core
        && path.segments.len() == 3
        && path.segments[0].as_str() == "option"
        && path.segments[1].starts_with("impl")
        && path.segments[2].as_str() == "is_some"
        && definition.proxy.as_ref().is_some_and(|proxy| {
            models::spec_proxy(&proxy.x, &["std_specs", "option"], "_is__some")
        })
        && definition.body.is_none()
        && parameters.len() == 1
        && definition.params.len() == 1
        && definition.require.is_empty()
        && definition.ensure.0.len() == 1
        && matches!(definition.unwind_spec, Some(UnwindSpec::NoUnwind))
        && same(&definition.inner_ret.x.typ, &bool_typ())
        && matches!(&*definition.params[0].x.typ, TypX::Decorate(TypDecoration::Ref, None, inner) if option(inner, &parameters[0]))
    {
        return Some(NativeOption::IsSome);
    }
    None
}
pub(super) fn as_ref(f: &Function) -> bool {
    let x = &f.x;
    let p = &x.name.path;
    if p.krate != CrateId::Core
        || p.segments.len() != 3
        || p.segments[0].as_str() != "option"
        || !p.segments[1].starts_with("impl")
        || p.segments[2].as_str() != "as_ref"
        || !x
            .proxy
            .as_ref()
            .is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "option"], "_as__ref"))
        || x.mode != Mode::Exec
        || !matches!(x.kind, FunctionKind::Static)
        || !matches!(x.item_kind, ItemKind::Function)
        || x.inner_ret.x.mode != Mode::Exec
        || x.outer_ret.x.mode != Mode::Exec
        || x.body.is_some()
        || x.attrs.infer
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || !x.require.is_empty()
        || x.ensure.0.len() != 2
        || !x.ensure.1.is_empty()
        || x.params.len() != 1
        || x.typ_params.len() != 1
        || !matches!(x.unwind_spec, Some(UnwindSpec::NoUnwind))
        || x.params[0].x.mode != Mode::Exec
        || x.params[0].x.user_mut
        || !crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
    {
        return false;
    }
    let t = Arc::new(TypX::TypParam(x.typ_params[0].clone()));
    let TypX::Decorate(TypDecoration::Ref, None, input) = &*x.params[0].x.typ else { return false };
    let option = |typ: &Typ, elt: &Typ| matches!(&**typ,TypX::Datatype(Dt::Path(p),ts,_) if models::path_is(p,CrateId::Core,&["option","Option"]) && ts.len()==1 && crate::ast_util::types_equal(&ts[0],elt));
    option(input, &t)
        && option(&x.inner_ret.x.typ, &models::reference(t.clone()))
        && x.typ_bounds.len() == 1
        && matches!(&*x.typ_bounds[0],GenericBoundX::Trait(TraitId::Sizedness(Sizedness::Sized),ts) if ts.len()==1 && crate::ast_util::types_equal(&ts[0],&t))
}

#[cfg(test)]
mod tests;
