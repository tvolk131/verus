//! Experimental compiler-owned nominal encoding, separate from source access.
//!
//! No source attribute installs this descriptor. The sole closed family is the
//! pinned core Rev<I>; the rustc frontend must additionally validate the loaded
//! ADT/field metadata before requesting it. Stage A has no production installer.
use crate::ast::*;
use crate::ast_util::{is_visible_to, types_equal};
use vir_macros::ToDebugSNode;

/// These semantic hashes are scoped to the pinned core artifact, not a spelling
/// lookup or cross-version authority. They are not native correspondence proofs.
pub const CORE_REV_IDENTITY: (u64, u64, u64) = (
    17281971461272426270,
    14147836729157185704,
    5029879297925419223,
);

/// Not serializable/deserializable. It can only be built by compiler code through
/// the closed checked factory below, never by a verifier attribute or import.
#[derive(Clone, Debug, ToDebugSNode)]
pub struct InternalRepresentation {
    datatype: Dt,
    parameter: Ident,
    source_visibility: Visibility,
}

impl InternalRepresentation {
    pub fn core_rev(
        datatype: &DatatypeX,
        loaded_identity: (u64, u64, u64),
    ) -> Result<Self, &'static str> {
        if loaded_identity != CORE_REV_IDENTITY {
            return Err("internal Rev: loaded core/ADT/field identity mismatch");
        }
        let [parameter] = datatype.typ_params.as_slice() else {
            return Err("internal Rev: expected one type parameter");
        };
        let Dt::Path(path) = &datatype.name else { return Err("internal Rev: nominal path required") };
        let source_visibility = Visibility { restricted_to: Some(path.pop_segment()) };
        let result = Self {
            datatype: datatype.name.clone(),
            parameter: parameter.0.clone(),
            source_visibility,
        };
        result.validate(datatype)?;
        Ok(result)
    }

    /// Independently re-read the actual datatype, including its complete field
    /// shape, before logical encoding. No synthesized twin or cached shape hash.
    pub fn validate(&self, datatype: &DatatypeX) -> Result<(), &'static str> {
        let Dt::Path(path) = &datatype.name else { return Err("internal Rev: nominal path required") };
        if datatype.name != self.datatype
            || path.krate != CrateId::Core
            || !path.segments.iter().map(|s| s.as_str()).eq(["iter", "adapters", "rev", "Rev"])
            || datatype.mode != Mode::Exec
            || !datatype.visibility.is_public()
            || datatype.ext_equal
            || datatype.destructor
            || datatype.user_defined_invariant_fn.is_some()
            || datatype.sized_constraint.is_some()
            || !datatype.typ_bounds.is_empty()
            || datatype.typ_params.len() != 1
            || datatype.typ_params[0].0 != self.parameter
            || datatype.typ_params[0].1 != AcceptRecursiveType::Reject
        {
            return Err("internal Rev: datatype identity/signature mismatch");
        }
        let [variant] = datatype.variants.as_slice() else {
            return Err("internal Rev: expected one variant");
        };
        let [field] = variant.fields.as_slice() else {
            return Err("internal Rev: expected one field");
        };
        if variant.name.as_str() != "Rev"
            || !matches!(variant.ctor_style, CtorPrintStyle::Braces)
            || field.name != crate::def::field_ident_from_rust("iter")
            || field.a.1 != Mode::Exec
            || field.a.2 != self.source_visibility
            || !types_equal(&field.a.0, &std::sync::Arc::new(TypX::TypParam(self.parameter.clone())))
        {
            return Err("internal Rev: private field identity/type/mode/visibility mismatch");
        }
        Ok(())
    }

    pub fn source_visibility(&self) -> &Visibility { &self.source_visibility }

    pub fn source_visible_to(&self, module: &Path) -> bool {
        is_visible_to(&self.source_visibility, module)
    }
}
