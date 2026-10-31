//! Closed primitive From conversions, using the existing vstd extension specs.
use super::*;

fn clean(f: &Function) -> bool {
    f.x.mode == Mode::Exec
        && !f.x.attrs.infer
        && !f.x.attrs.is_unsafe
        && !f.x.attrs.is_async
        && f.x.body.is_none()
        && f.x.require.is_empty()
        && f.x.ensure.1.is_empty()
        && f.x.params.len() == 1
        && f.x.params[0].x.mode == Mode::Exec
        && !f.x.params[0].x.user_mut
        && crate::ast_util::types_equal(&f.x.inner_ret.x.typ, &f.x.outer_ret.x.typ)
}

pub(super) fn model(f: &Function) -> bool {
    if !clean(f) {
        return false;
    }
    let p = &f.x.name.path;
    if models::path_is(p, CrateId::Core, &["convert", "From", "from"]) {
        return matches!(f.x.kind, FunctionKind::TraitMethodDecl { .. })
            && f.x.proxy.as_ref().is_some_and(|p| {
                models::path_is(&p.x, CrateId::Vstd, &["std_specs", "convert", "ExFrom", "from"])
            })
            && f.x.typ_params.len() == 2
            && f.x.ensure.0.len() == 1;
    }
    p.krate == CrateId::Core
        && p.segments.len() == 4
        && p.segments[0].as_str() == "convert"
        && p.segments[1].as_str() == "num"
        && p.segments[2].starts_with("impl")
        && p.segments[3].as_str() == "from"
        && f.x.typ_params.is_empty()
        && f.x.typ_bounds.is_empty()
        && matches!(
            f.x.kind,
            FunctionKind::TraitMethodImpl { inherit_body_from: None, .. }
                | FunctionKind::ForeignTraitMethodImpl { .. }
        )
        && f.x.proxy.as_ref().is_some_and(|p| {
            p.x.krate == CrateId::Vstd
                && p.x.segments.len() == 3
                && p.x.segments[0].as_str() == "std_specs"
                && p.x.segments[1].as_str() == "convert"
                && p.x.segments[2].starts_with("_verus_external_fn_specification_")
                && p.x.segments[2].ends_with("_from")
        })
        && propagation::lossless_error_conversion(&f.x.params[0].x.typ, &f.x.inner_ret.x.typ)
}

pub(super) fn call<'a>(e: &'a Expr, all: &HashMap<Fun, Function>) -> Option<&'a Expr> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, name, typs, impls, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    if args.len() != 1 || typs.len() != 2 || !all.get(name).is_some_and(model) {
        return None;
    }
    // Real conversion derives call <Integer as From<Integer>>::from. Core's
    // identity impl is fixed by Rust coherence for concrete primitive integers;
    // vstd's existing FromSpecImpl<T> for T supplies its identity equation.
    // Do not apply this to a user datatype, trait, impl, or lossy conversion.
    let core_identity_impl = |path: &Path| {
        path.krate == CrateId::Core
            && path.segments.len() == 2
            && path.segments[0].as_str() == "convert"
            && path.segments[1].starts_with("impl")
    };
    let identity_dispatch = match kind {
        CallTargetKind::Dynamic if impls.len() == 1 => {
            matches!(&impls[0], ImplPath::TraitImplPath(path) if core_identity_impl(path))
        }
        CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } => {
            let path = &resolved.path;
            path.krate == CrateId::Core
                && path.segments.len() == 3
                && path.segments[0].as_str() == "convert"
                && path.segments[1].starts_with("impl")
                && path.segments[2].as_str() == "from"
        }
        _ => false,
    };
    if identity_dispatch
        && crate::ast_util::types_equal(&typs[0], &typs[1])
        && matches!(&*typs[0], TypX::Int(range) if arithmetic::exact_range(*range))
        && crate::ast_util::types_equal(&args[0].typ, &typs[0])
        && crate::ast_util::types_equal(&e.typ, &typs[0])
    {
        return Some(&args[0]);
    }
    let f = match kind {
        CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } => {
            all.get(resolved).filter(|f| model(f))?
        }
        CallTargetKind::Dynamic if impls.len() == 1 => {
            let ImplPath::TraitImplPath(path) = &impls[0] else { return None };
            let mut found = all.values().filter(|f| model(f) && matches!(&f.x.kind,
                FunctionKind::TraitMethodImpl { impl_path, .. } | FunctionKind::ForeignTraitMethodImpl { impl_path, .. } if impl_path == path));
            let f = found.next()?;
            if found.next().is_some() {
                return None;
            }
            f
        }
        _ => return None,
    };
    let mut argument = &args[0].typ;
    while let TypX::Decorate(TypDecoration::Ref, None, inner) = &**argument {
        argument = inner;
    }
    (crate::ast_util::types_equal(argument, &f.x.params[0].x.typ)
        && crate::ast_util::types_equal(&e.typ, &f.x.inner_ret.x.typ)
        && crate::ast_util::types_equal(&typs[0], &e.typ)
        && crate::ast_util::types_equal(&typs[1], argument))
    .then_some(&args[0])
}
