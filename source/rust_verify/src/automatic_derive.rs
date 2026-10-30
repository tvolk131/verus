use crate::context::Context;
use crate::verus_items::RustItem;
use rustc_span::Span;
use std::sync::Arc;
use vir::ast::{
    BinaryOp, Expr, ExprX, FunctionX, Mode, Place, PlaceX, SpannedTyped, VirErr, VirErrAs,
};
use vir::messages::WarningAllow;

/// Traits with special handling
#[derive(Clone, Copy, Debug)]
pub enum SpecialTrait {
    Clone,
    //PartialEq,
}

/// What to do for a given automatically-derived trait impl
#[derive(Debug, Clone, Copy)]
pub enum AutomaticDeriveAction {
    Special(SpecialTrait),
    VerifyAsIs,
    /// Ignore, optionally providing a warning
    Ignore,
}

pub fn get_action(rust_item: Option<RustItem>) -> AutomaticDeriveAction {
    match rust_item {
        Some(RustItem::PartialEq | RustItem::Eq) => AutomaticDeriveAction::Ignore,
        Some(RustItem::Clone) => AutomaticDeriveAction::Special(SpecialTrait::Clone),

        Some(RustItem::Copy) => AutomaticDeriveAction::VerifyAsIs,

        Some(RustItem::Hash)
        | Some(RustItem::Default)
        | Some(RustItem::Debug)
        | Some(RustItem::Ord)
        | Some(RustItem::PartialOrd) => AutomaticDeriveAction::Ignore,

        Some(_) | None => AutomaticDeriveAction::VerifyAsIs,
    }
}

pub fn is_automatically_derived(attrs: &[rustc_hir::Attribute]) -> bool {
    for attr in attrs.iter() {
        match attr {
            rustc_hir::Attribute::Unparsed(item) => match &item.path.segments[..] {
                [segment] => {
                    if segment.as_str() == "automatically_derived" {
                        return true;
                    }
                }
                _ => {}
            },
            rustc_hir::Attribute::Parsed(rustc_hir::attrs::AttributeKind::AutomaticallyDerived) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

pub fn has_derived_request<'tcx>(tcx:rustc_middle::ty::TyCtxt<'tcx>,owner:rustc_span::def_id::DefId,name:&str)->bool {
    let marker=format!("__verus_infer_derived_{name}");
    tcx.inherent_impls(owner).iter().any(|impl_id|
        tcx.associated_items(*impl_id).in_definition_order().any(|m|
            m.tag()==rustc_middle::ty::AssocTag::Fn && m.name().as_str()==marker))
}

pub fn modify_derived_item<'tcx>(
    ctxt: &Context<'tcx>,
    id: rustc_span::def_id::DefId,
    inputs: &Vec<rustc_middle::ty::Ty<'tcx>>,
    span: Span,
    action: AutomaticDeriveAction,
    function: &mut FunctionX,
) -> Result<(), VirErr> {
    // A marker only requests inference. It supplies no evidence: the original
    // derive body and the inherited trait domain must pass all exact checks.
    let name = function.name.path.last_segment().to_string();
    let reverse_from = name == "from"
        && matches!(ctxt.tcx.fn_sig(id).skip_binder().output().skip_binder().kind(),
            rustc_middle::ty::TyKind::Int(_) | rustc_middle::ty::TyKind::Uint(_));
    let target = if name == "from" {
        let output = ctxt.tcx.fn_sig(id).skip_binder().output().skip_binder();
        if reverse_from {
            inputs.first().cloned()
        } else {
            Some(output)
        }
    } else { inputs.first().cloned() };
    let target=target.map(|t|match t.kind(){rustc_middle::ty::TyKind::Ref(_,inner,_)=>*inner,_=>t});
    if let Some(rustc_middle::ty::TyKind::Adt(adt, _)) = target.map(|t| t.kind()) {
        if matches!(name.as_str(), "from" | "add" | "sub" | "mul" | "div" | "rem" | "eq") {
            if has_derived_request(ctxt.tcx,adt.did(),if reverse_from {"into"} else {&name}) {
                Arc::make_mut(&mut function.attrs).infer = true;
            }
        }
    }
    let AutomaticDeriveAction::Special(special) = action else {
        return Ok(());
    };
    match special {
        SpecialTrait::Clone => {
            if &*function.name.path.last_segment() == "clone" {
                return clone_add_post_condition(ctxt, id, inputs, span, function);
            }
        }
    }
    Ok(())
}

fn clone_add_post_condition<'tcx>(
    ctxt: &Context<'tcx>,
    mut id: rustc_span::def_id::DefId,
    inputs: &Vec<rustc_middle::ty::Ty<'tcx>>,
    span: Span,
    functionx: &mut FunctionX,
) -> Result<(), VirErr> {
    if inputs.len() >= 1 {
        use rustc_middle::ty::{AdtDef, TyKind};
        if let TyKind::Ref(_, t, _) = inputs[0].kind() {
            if let TyKind::Adt(AdtDef(adt_def_data), _) = t.kind() {
                // It's more convenient to put verifier::allow on the datatype than on the function
                id = adt_def_data.did;
            }
        }
    }
    let warn = |msg: &str| {
        crate::attributes::warning_maybe(
            ctxt.tcx,
            id,
            span,
            &WarningAllow::AutoderiveCloneWithoutSpec,
            || msg,
            |msg| ctxt.diagnostics.borrow_mut().push(VirErrAs::Warning(msg)),
        );
    };
    let warn_unexpected = || {
        warn(
            "autoderive Clone impl does not take the form Verus expects; continuing, but without adding a specification for the derived Clone impl",
        )
    };
    let warn_unsupported = || {
        warn(
            "Verus does not (yet) support autoderive Clone impl when the clone is not a copy; continuing, but without adding a specification for the derived Clone impl",
        )
    };

    let Some(body) = &functionx.body else {
        return Ok(());
    };

    // The opt-in source adapter supplies a public view relation. Adding this
    // proposed postcondition is untrusted: the actual native derive body is
    // still checked normally, including each field's library clone contract.
    if !ctxt.no_vstd && functionx.ensure.0.is_empty() {
        if let Some(post) = view_clone_post(ctxt, inputs, functionx) {
            functionx.ensure.0 = Arc::new(vec![cleanup_span_ids(ctxt, span, &post)]);
            return Ok(());
        }
    }

    let uses_copy;
    let self_var;

    match &body.x {
        ExprX::Block(_stmts, Some(last_expr)) => match &last_expr.x {
            ExprX::ReadPlace(pl, _) => match &pl.x {
                PlaceX::Local(id) if &*id.0 == "self" => {
                    uses_copy = true;
                    self_var = Some(last_expr.clone());
                }
                _ => {
                    warn_unexpected();
                    return Ok(());
                }
            },
            ExprX::Ctor { .. } => {
                uses_copy = false;
                self_var = None;
            }
            _ => {
                warn_unexpected();
                return Ok(());
            }
        },
        _ => {
            warn_unexpected();
            return Ok(());
        }
    }

    if functionx.ensure.0.len() != 0 {
        warn_unexpected();
        return Ok(());
    }

    if uses_copy {
        // Add `ensures ret == self`
        let self_var = self_var.unwrap();
        let ret_var = SpannedTyped::new(
            &self_var.span,
            &self_var.typ,
            ExprX::Var(functionx.inner_ret.x.name.clone()),
        );
        let eq_expr = SpannedTyped::new(
            &self_var.span,
            &vir::ast_util::bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), ret_var.clone(), self_var.clone()),
        );

        let eq_expr = cleanup_span_ids(ctxt, span, &eq_expr);
        functionx.ensure.0 = Arc::new(vec![eq_expr]);
    } else {
        warn_unsupported();
    }

    Ok(())
}

fn view_clone_post<'tcx>(
    ctxt: &Context<'tcx>,
    inputs: &[rustc_middle::ty::Ty<'tcx>],
    f: &FunctionX,
) -> Option<Expr> {
    use vir::ast::*;
    let rustc_middle::ty::TyKind::Ref(_, typ, _) = inputs.first()?.kind() else { return None };
    let rustc_middle::ty::TyKind::Adt(adt, _) = typ.kind() else { return None };
    let mut methods = ctxt
        .tcx
        .inherent_impls(adt.did())
        .iter()
        .flat_map(|id| ctxt.tcx.associated_items(*id).in_definition_order())
        .filter(|m| {
            m.tag() == rustc_middle::ty::AssocTag::Fn
                && m.name().as_str() == "__verus_clone_relation"
        });
    let method = methods.next()?;
    if methods.next().is_some() || f.params.len() != 1 {
        return None;
    }
    let TypX::Datatype(_, args, _) = &*f.inner_ret.x.typ else { return None };
    let sig = ctxt.tcx.fn_sig(method.def_id).skip_binder().skip_binder();
    #[allow(deprecated)]
    let mode = crate::attributes::get_mode(Mode::Exec, ctxt.tcx.get_all_attrs(method.def_id));
    let receiver_matches = sig.inputs().first().is_some_and(|input| {
        matches!(input.kind(), rustc_middle::ty::TyKind::Ref(_, target, rustc_hir::Mutability::Not)
            if ctxt.tcx.erase_and_anonymize_regions(*target) == ctxt.tcx.erase_and_anonymize_regions(*typ))
    });
    if mode != Mode::Spec
        || sig.inputs().len() != 2
        || !sig.output().is_bool()
        || !receiver_matches
        || ctxt.tcx.erase_and_anonymize_regions(sig.inputs()[1])
            != ctxt.tcx.erase_and_anonymize_regions(*typ)
        || ctxt.tcx.generics_of(method.def_id).count() != args.len()
    {
        return None;
    }

    let source = SpannedTyped::new(
        &f.params[0].span,
        &f.params[0].x.typ,
        ExprX::Var(f.params[0].x.name.clone()),
    );
    let result = SpannedTyped::new(
        &f.inner_ret.span,
        &f.inner_ret.x.typ,
        ExprX::Var(f.inner_ret.x.name.clone()),
    );
    Some(SpannedTyped::new(
        &f.inner_ret.span,
        &vir::ast_util::bool_typ(),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                Arc::new(FunX { path: ctxt.def_id_to_vir_path(method.def_id) }),
                args.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![source, result]),
            post_args: None,
            body: None,
        },
    ))
}

/// Give each node a unique ID. This inserts 'None' for every var into the local_hir_vir_ids
/// list, so it's only safe for pure expressions (e.g., in asserts or signatures).
fn cleanup_span_ids<'tcx>(ctxt: &Context<'tcx>, span: Span, expr: &Expr) -> Expr {
    vir::ast_visitor::map_expr_place_visitor(
        expr,
        &|e: &Expr| {
            let e = ctxt.spans.spanned_typed_new(span, &e.typ, e.x.clone());
            if matches!(&e.x, ExprX::Var(..) | ExprX::VarAt(..)) {
                let mut erasure_info = ctxt.erasure_info.borrow_mut();
                erasure_info.local_hir_vir_ids.push((None, e.span.id));
            }
            Ok(e)
        },
        &|p: &Place| {
            let p = ctxt.spans.spanned_typed_new(span, &p.typ, p.x.clone());
            if matches!(&p.x, PlaceX::Local(..)) {
                let mut erasure_info = ctxt.erasure_info.borrow_mut();
                erasure_info.local_hir_vir_ids.push((None, p.span.id));
            }
            Ok(p)
        },
    )
    .unwrap()
}
