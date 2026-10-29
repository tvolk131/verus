//! Candidate-only generic equality through vstd's existing PartialEq contract.
//! Logical identity is proposed, never assumed. The original eq/ne call and all
//! precision queries must establish it from an explicitly checked equality law.
use super::*;

pub(super) fn model(f: &Function) -> bool {
    let x = &f.x;
    ["eq", "ne"].iter().any(|name| {
        models::path_is(&x.name.path, CrateId::Core, &["cmp", "PartialEq", name])
            && x.proxy.as_ref().is_some_and(|p| {
                models::path_is(&p.x, CrateId::Vstd, &["std_specs", "cmp", "ExPartialEq", name])
            })
    }) && matches!(x.kind, FunctionKind::TraitMethodDecl { .. })
        && x.mode == Mode::Exec
        && !x.attrs.infer
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && x.typ_params.len() == 2
        && x.params.len() == 2
        && x.require.is_empty()
        && x.ensure.0.len() == 1
        && x.ensure.1.len() == usize::from(x.name.path.last_segment().as_str() == "ne")
        && x.params.iter().all(|p| {
            p.x.mode == Mode::Exec && !p.x.user_mut && trait_getters::shared(&p.x.typ).is_some()
        })
        && matches!(&*x.inner_ret.x.typ, TypX::Bool)
}

pub(super) fn source_call(e: &Expr, all: &HashMap<Fun, Function>) -> Option<(bool, Exprs)> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, n, types, impls, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    if !all.get(n).is_some_and(model)
        || types.len() != 2
        || args.len() != 2
        || !crate::ast_util::types_equal(&types[0], &types[1])
        || !(matches!(kind,CallTargetKind::Dynamic) && impls.is_empty()
                && matches!(&*types[0], TypX::TypParam(_) | TypX::Projection { .. })
            || n.path.last_segment().as_str()=="ne"
                && matches!(kind,CallTargetKind::ExternalTraitDefault
                    |CallTargetKind::DynamicResolved{is_trait_default:true,..})
                && matches!(&*types[0],TypX::Datatype(Dt::Path(p),..)
                    if matches!(p.krate,CrateId::Id(..))))
        || !args.iter().all(|a| {
            trait_getters::shared(&a.typ)
                .is_some_and(|t| crate::ast_util::types_equal(t, &types[0]))
        })
    {
        return None;
    }
    Some((n.path.last_segment().as_str() == "ne", args.clone()))
}

pub(super) fn uses(f: &Function, all: &HashMap<Fun, Function>,datatypes:&[Datatype]) -> bool {
    let mut found = false;
    if let Some(b) = &f.x.body {
        crate::ast_visitor::expr_visitor_walk(b, &mut |e| {
            // Scalar wrappers use the projection pipeline. Do not route an
            // inherited ne call to owned analysis merely because it also fits
            // the broader logical-identity proposal.
            found |= source_call(e, all).is_some() && wrapper_compare::source_call(e,all,datatypes).is_none();
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
    found
}
