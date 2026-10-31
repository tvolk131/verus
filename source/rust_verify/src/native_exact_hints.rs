//! Reject first-party runtime overrides hidden outside the checked capability.
//! The core trait is safe Rust: its name cannot certify ignored implementations.
use crate::context::Context;
use rustc_hir::def::DefKind;
use rustc_middle::ty::TyKind;
use std::collections::HashMap;
use vir::ast::{Function, Mode, VirErr};

pub(crate) fn validate<'tcx>(ctxt: &Context<'tcx>, functions: &[Function]) -> Result<(), VirErr> {
    let tcx = ctxt.tcx;
    let mut opted = HashMap::new();
    for f in functions.iter().filter(|f| vir::infer::exact_size_hint_opt_in(f)) {
        let did = ctxt.name_def_id_map.borrow().get(&f.x.name.path).copied().ok_or_else(|| {
            vir::messages::error(
                &f.span,
                "checked hint capability has no native implementation identity",
            )
        })?;
        let implementation = tcx.impl_of_assoc(did).ok_or_else(|| {
            vir::messages::error(
                &f.span,
                "checked hint capability is not a native trait implementation",
            )
        })?;
        let self_ty = tcx.impl_trait_ref(implementation).skip_binder().self_ty().peel_refs();
        let TyKind::Adt(d, _) = self_ty.kind() else {
            return Err(vir::messages::error(
                &f.span,
                "unsupported checked hint capability self type",
            ));
        };
        opted.insert(d.did(), f);
    }
    if opted.is_empty() {
        return Ok(());
    }
    for item in crate::util::iter_crate_free_items(tcx) {
        let rustc_hir::ItemKind::Impl(implementation) = &item.kind else { continue };
        if implementation.of_trait.is_none() {
            continue;
        }
        let did = item.owner_id.to_def_id();
        let trait_ref = tcx.impl_trait_ref(did);
        let trait_path = ctxt.def_id_to_vir_path(trait_ref.skip_binder().def_id);
        if trait_path.krate != vir::ast::CrateId::Core {
            continue;
        }
        let name = trait_path.last_segment();
        if !["Iterator", "ExactSizeIterator", "Drop"].contains(&name.as_str()) {
            continue;
        }
        let self_ty = trait_ref.skip_binder().self_ty().peel_refs();
        let TyKind::Adt(d, _) = self_ty.kind() else { continue };
        let Some(law) = opted.get(&d.did()) else { continue };
        if name.as_str() == "Drop" {
            return Err(vir::messages::error(
                &law.span,
                "unsupported checked hint capability: custom Drop is outside the bounded iterator/resource profile",
            ));
        }
        for method in tcx.associated_items(did).in_definition_order() {
            if tcx.def_kind(method.def_id) != DefKind::AssocFn {
                continue;
            }
            let path = ctxt.def_id_to_vir_path(method.def_id);
            let supported = if name.as_str() == "Iterator" {
                ["next", "nth", "size_hint"].contains(&path.last_segment().as_str())
            } else {
                path.last_segment().as_str() == "len"
            };
            if !supported {
                return Err(vir::messages::error(
                    &law.span,
                    format!(
                        "unsupported checked hint capability: custom forward adapter override outside the bounded law profile: {path:?}"
                    ),
                ));
            }
            if !functions.iter().any(|f| {
                f.x.name.path == path
                    && f.x.mode == Mode::Exec
                    && f.x.body.is_some()
                    && !f.x.attrs.is_external_body
            }) {
                return Err(vir::messages::error(
                    &law.span,
                    format!(
                        "checked exact-size-hint capability requires the actual selected native override body: {path:?}"
                    ),
                ));
            }
        }
    }
    Ok(())
}
