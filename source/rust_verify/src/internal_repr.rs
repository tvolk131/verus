//! Inactive Stage A entry: validate loaded pinned core metadata before allowing
//! its closed logical datatype descriptor. No source attribute or installer.
use rustc_middle::ty::{self, AdtDef, TyCtxt};
use vir::internal_repr::{CORE_REV_IDENTITY, InternalRepresentation};

pub fn validate_core_rev_metadata<'tcx>(
    tcx: TyCtxt<'tcx>,
    adt: AdtDef<'tcx>,
) -> Result<(u64, u64, u64), &'static str> {
    let (core, rev, field_hash) = CORE_REV_IDENTITY;
    if tcx.stable_crate_id(adt.did().krate).as_u64() != core
        || tcx.def_path_hash(adt.did()).local_hash().as_u64() != rev
        || !adt.is_struct()
        || adt.variants().len() != 1
        || adt.destructor(tcx).is_some()
        || !tcx.visibility(adt.did()).is_public()
    {
        return Err("internal Rev: actual loaded ADT identity/shape mismatch");
    }
    let generics = tcx.generics_of(adt.did());
    if generics.parent.is_some()
        || generics.own_params.len() != 1
        || generics.count() != 1
        || !matches!(
            generics.own_params[0].kind,
            ty::GenericParamDefKind::Type { has_default: false, .. }
        )
    {
        return Err("internal Rev: loaded generic declaration mismatch");
    }
    let variant = adt.non_enum_variant();
    if variant.fields.len() != 1 || variant.ctor.is_some() {
        return Err("internal Rev: loaded variant mismatch");
    }
    let field = variant.fields.iter().next().unwrap();
    let field_type = tcx.type_of(field.did).instantiate_identity().skip_norm_wip();
    if tcx.def_path_hash(field.did).stable_crate_id().as_u64() != core
        || tcx.def_path_hash(field.did).local_hash().as_u64() != field_hash
        || !matches!(field.vis, ty::Visibility::Restricted(module) if module.to_def_id()==tcx.parent(adt.did()))
        || !matches!(field_type.kind(),ty::Param(p) if p.index==0 && p.name==generics.own_params[0].name)
    {
        return Err("internal Rev: loaded private field identity/type/visibility mismatch");
    }
    Ok(CORE_REV_IDENTITY)
}

pub fn core_rev_descriptor<'tcx>(
    tcx: TyCtxt<'tcx>,
    adt: AdtDef<'tcx>,
    datatype: &vir::ast::DatatypeX,
) -> Result<InternalRepresentation, &'static str> {
    let identity = validate_core_rev_metadata(tcx, adt)?;
    InternalRepresentation::core_rev(datatype, identity)
}
