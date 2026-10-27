//! Closed existing Option::as_ref model; no additional library specification.
use super::*;
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
