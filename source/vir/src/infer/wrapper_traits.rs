//! Concrete wrapper implementations, checked through the original trait boundary.
//!
//! No arithmetic/conversion meaning is assigned to a user implementation. Its
//! candidate comes from its body; maximality also proves that the trait's
//! inherited requires equals the proposed domain.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    From,
    AddAssign,
    Compare,
}

/// Existing vstd specifications for core's blanket Into-through-From bridge.
pub(super) fn into_model(f: &Function) -> bool {
    if f.x.mode != Mode::Exec
        || f.x.attrs.infer
        || f.x.body.is_some()
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || !f.x.require.is_empty()
        || f.x.params.len() != 1
        || f.x.params[0].x.mode != Mode::Exec
        || f.x.params[0].x.user_mut
        || f.x.ensure.0.len() != 1
        || !f.x.ensure.1.is_empty()
        || f.x.typ_params.len() != 2
        || !matches!(&*f.x.params[0].x.typ, TypX::TypParam(n) if n == &f.x.typ_params[0])
        || !matches!(&*f.x.inner_ret.x.typ, TypX::TypParam(n) if n == &f.x.typ_params[1])
        || !crate::ast_util::types_equal(&f.x.inner_ret.x.typ, &f.x.outer_ret.x.typ)
    {
        return false;
    }
    if models::path_is(&f.x.name.path, CrateId::Core, &["convert", "Into", "into"]) {
        matches!(f.x.kind, FunctionKind::TraitMethodDecl { .. })
            && f.x.proxy.as_ref().is_some_and(|p| {
                models::path_is(&p.x, CrateId::Vstd, &["std_specs", "convert", "ExInto", "into"])
            })
    } else {
        let p = &f.x.name.path;
        p.krate == CrateId::Core
            && p.segments.len() == 3
            && p.segments[0].as_str() == "convert"
            && p.segments[1].starts_with("impl")
            && p.segments[2].as_str() == "into"
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
                    && p.x.segments[2].ends_with("_into")
            })
    }
}

pub(super) fn kind(f: &Function) -> Option<Kind> {
    let FunctionKind::TraitMethodImpl { method, trait_path, inherit_body_from: None, .. } =
        &f.x.kind
    else {
        return None;
    };
    if !f.x.attrs.infer
        || f.x.attrs.infer_allow_approx
        || !f.x.typ_params.is_empty()
        || !f.x.typ_bounds.is_empty()
        || f.x.proxy.is_some()
        || f.x.attrs.is_external_body
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || f.x.attrs.prophecy_dependent
        || f.x.attrs.exec_assume_termination
        || f.x.attrs.exec_allows_no_decreases_clause
    {
        return None;
    }
    if models::path_is(trait_path, CrateId::Core, &["convert", "From"])
        && models::path_is(&method.path, CrateId::Core, &["convert", "From", "from"])
        && f.x.params.len() == 1
    {
        Some(Kind::From)
    } else if models::path_is(trait_path, CrateId::Core, &["ops", "arith", "AddAssign"])
        && models::path_is(
            &method.path,
            CrateId::Core,
            &["ops", "arith", "AddAssign", "add_assign"],
        )
        && f.x.params.len() == 2
        && records::mutable(f)
        && enums::unit(&f.x.inner_ret.x.typ)
    {
        Some(Kind::AddAssign)
    } else if f.x.params.len() == 2
        && matches!(&*f.x.inner_ret.x.typ, TypX::Bool)
        && f.x.params.iter().all(|p| matches!(&*p.x.typ, TypX::Decorate(TypDecoration::Ref,None,_)))
        && crate::ast_util::types_equal(&f.x.params[0].x.typ,&f.x.params[1].x.typ)
        && ((models::path_is(trait_path,CrateId::Core,&["cmp","PartialEq"])
            && ["eq","ne"].iter().any(|n| models::path_is(&method.path,CrateId::Core,&["cmp","PartialEq",n])))
            || (models::path_is(trait_path,CrateId::Core,&["cmp","PartialOrd"])
            && ["lt","le","gt","ge"].iter().any(|n| models::path_is(&method.path,CrateId::Core,&["cmp","PartialOrd",n]))))
    {
        Some(Kind::Compare)
    } else {
        None
    }
}

/// Resolve only the exact concrete implementation selected by Rust. Rewriting
/// is candidate-only; executable trait dispatch remains unchanged.
pub(super) fn source_call(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Expr> {
    let ExprX::Call {
        target: CallTarget::Fun(call_kind, method, types, impls, attrs),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    let blanket=match call_kind {
        CallTargetKind::DynamicResolved{resolved,is_trait_default:false,..}=>all.get(resolved).is_some_and(into_model),
        CallTargetKind::Dynamic=>impls.first().is_some_and(|p|matches!(p,ImplPath::TraitImplPath(path)
            if all.values().any(|f|into_model(f) && matches!(&f.x.kind,FunctionKind::TraitMethodImpl{impl_path,..}|FunctionKind::ForeignTraitMethodImpl{impl_path,..} if impl_path==path)))),
        _=>false,
    };
    if all.get(method).is_some_and(into_model)
        && types.len() == 2
        && args.len() == 1
        && blanket
        && crate::ast_util::types_equal(&types[0], &args[0].typ)
        && crate::ast_util::types_equal(&types[1], &e.typ)
    {
        let mut matches = all.values().filter(|f| {
            kind(f) == Some(Kind::From)
                && crate::ast_util::types_equal(&f.x.params[0].x.typ, &args[0].typ)
                && crate::ast_util::types_equal(&f.x.inner_ret.x.typ, &e.typ)
        });
        let f = matches.next()?;
        let FunctionKind::TraitMethodImpl { impl_path, .. } = &f.x.kind else { return None };
        if matches.next().is_some()
            || !impls.iter().any(|p| matches!(p,ImplPath::TraitImplPath(path) if path==impl_path))
        {
            return None;
        }
        return Some(e.new_x(ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                f.x.name.clone(),
                Arc::new(vec![]),
                Arc::new(vec![]),
                attrs.clone(),
            ),
            args: args.clone(),
            post_args: None,
            body: None,
        }));
    }
    let f = match call_kind {
        CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } => {
            all.get(resolved)?
        }
        CallTargetKind::Dynamic if impls.len() == 1 => {
            let ImplPath::TraitImplPath(path) = &impls[0] else { return None };
            let mut matches=all.values().filter(|f|matches!(&f.x.kind,FunctionKind::TraitMethodImpl{impl_path,method:m,..} if impl_path==path && m==method));
            let f = matches.next()?;
            if matches.next().is_some() {
                return None;
            }
            f
        }
        _ => return None,
    };
    kind(f)?;
    let FunctionKind::TraitMethodImpl { method: expected, trait_typ_args, .. } = &f.x.kind else {
        return None;
    };
    if expected != method
        || types.len() != trait_typ_args.len()
        || !types.iter().zip(trait_typ_args.iter()).all(|(a, b)| crate::ast_util::types_equal(a, b))
        || args.len() != f.x.params.len()
        || !args
            .iter()
            .zip(f.x.params.iter())
            .all(|(a, p)| crate::ast_util::types_equal(&a.typ, &p.x.typ))
        || !crate::ast_util::types_equal(&e.typ, &f.x.inner_ret.x.typ)
    {
        return None;
    }
    Some(e.new_x(ExprX::Call {
        target: CallTarget::Fun(
            CallTargetKind::Static,
            f.x.name.clone(),
            Arc::new(vec![]),
            Arc::new(vec![]),
            attrs.clone(),
        ),
        args: args.clone(),
        post_args: None,
        body: None,
    }))
}

pub(super) fn rewrite(f: &Function, all: &HashMap<Fun, Function>) -> Result<Function, VirErr> {
    if !f.x.attrs.infer {
        return Ok(f.clone());
    }
    let mut x = f.x.clone();
    x.body = x
        .body
        .as_ref()
        .map(|body| {
            crate::ast_visitor::map_expr_visitor_env(
                body,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut (),
                &|_, _, e| Ok(source_call(e, all).unwrap_or_else(|| e.clone())),
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| Ok(t.clone()),
                &|_, _, p| Ok(p.clone()),
                &|_, _, p| Ok(p.clone()),
            )
        })
        .transpose()?;
    Ok(f.new_x(x))
}
