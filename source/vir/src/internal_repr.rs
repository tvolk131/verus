//! Experimental compiler-owned nominal encoding, separate from source access.
//!
//! No source attribute installs these descriptors. The closed families are the
//! pinned core Rev<I> and owned UnsafeCell<u64>; the rustc frontend must freshly
//! validate loaded metadata before requesting them. Neither grants field access.
use crate::ast::*;
use crate::ast_util::{is_visible_to, types_equal};
use vir_macros::ToDebugSNode;

/// These semantic hashes are scoped to the pinned core artifact, not a spelling
/// lookup or cross-version authority. They are not native correspondence proofs.
pub const CORE_REV_IDENTITY: (u64, u64, u64) =
    (17281971461272426270, 14147836729157185704, 5029879297925419223);

/// Not serializable/deserializable. It can only be built by compiler code through
/// the closed checked factory below, never by a verifier attribute or import.
#[derive(Clone, Debug, ToDebugSNode)]
pub struct InternalRepresentation {
    family: Family,
    datatype: Dt,
    parameter: Ident,
    owning_module: Option<Path>,
    proxy: Option<Path>,
    source_visibility: Visibility,
}

#[derive(Clone, Debug, ToDebugSNode)]
enum Family {
    Rev,
    OwnedUnsafeCellU64,
}

pub const CORE_OWNED_CELL_IDENTITY: (u64, u64, u64) =
    (17281971461272426270, 11949802078424953320, 16821345337525786618);

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
        let Dt::Path(path) = &datatype.name else {
            return Err("internal Rev: nominal path required");
        };
        let source_visibility = Visibility { restricted_to: Some(path.pop_segment()) };
        let result = Self {
            family: Family::Rev,
            datatype: datatype.name.clone(),
            parameter: parameter.0.clone(),
            owning_module: datatype.owning_module.clone(),
            proxy: datatype.proxy.as_ref().map(|p| p.x.clone()),
            source_visibility,
        };
        result.validate(datatype)?;
        Ok(result)
    }

    /// Independently re-read the actual datatype, including its complete field
    /// shape, before logical encoding. No synthesized twin or cached shape hash.
    pub fn validate(&self, datatype: &DatatypeX) -> Result<(), &'static str> {
        match self.family {
            Family::Rev => self.validate_rev(datatype),
            Family::OwnedUnsafeCellU64 => self.validate_owned_cell(datatype),
        }
    }

    fn validate_rev(&self, datatype: &DatatypeX) -> Result<(), &'static str> {
        let Dt::Path(path) = &datatype.name else {
            return Err("internal Rev: nominal path required");
        };
        if datatype.name != self.datatype
            || datatype.owning_module != self.owning_module
            || datatype.proxy.as_ref().map(|p| p.x.clone()) != self.proxy
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
            || !types_equal(
                &field.a.0,
                &std::sync::Arc::new(TypX::TypParam(self.parameter.clone())),
            )
        {
            return Err("internal Rev: private field identity/type/mode/visibility mismatch");
        }
        Ok(())
    }

    /// Closed compiler factory, never a source installer. The frontend must
    /// freshly validate loaded native metadata/signatures/MIR before requesting
    /// this descriptor. This descriptor grants ENCODING, not field authority.
    pub fn owned_unsafe_cell_u64(
        datatype: &DatatypeX,
        loaded: (u64, u64, u64),
    ) -> Result<Self, &'static str> {
        if loaded != CORE_OWNED_CELL_IDENTITY {
            return Err("internal owned cell: native identity mismatch");
        }
        let [parameter] = datatype.typ_params.as_slice() else {
            return Err("internal owned cell: one native parameter required");
        };
        let Dt::Path(path) = &datatype.name else {
            return Err("internal owned cell: nominal type required");
        };
        let result = Self {
            family: Family::OwnedUnsafeCellU64,
            datatype: datatype.name.clone(),
            parameter: parameter.0.clone(),
            owning_module: datatype.owning_module.clone(),
            proxy: datatype.proxy.as_ref().map(|p| p.x.clone()),
            source_visibility: Visibility { restricted_to: Some(path.pop_segment()) },
        };
        result.validate(datatype)?;
        Ok(result)
    }

    fn validate_owned_cell(&self, d: &DatatypeX) -> Result<(), &'static str> {
        let Dt::Path(path) = &d.name else {
            return Err("internal owned cell: nominal type required");
        };
        let t = std::sync::Arc::new(TypX::TypParam(self.parameter.clone()));
        if d.name != self.datatype
            || path.krate != CrateId::Core
            || !path.segments.iter().map(|s| s.as_str()).eq(["cell", "UnsafeCell"])
            || d.owning_module != self.owning_module
            || d.owning_module != self.source_visibility.restricted_to
            || d.proxy.is_some()
            || self.proxy.is_some()
            || d.mode != Mode::Exec
            || !d.visibility.is_public()
            || d.ext_equal
            || d.destructor
            || d.user_defined_invariant_fn.is_some()
            || !d.typ_bounds.is_empty()
            || d.typ_params.len() != 1
            || d.typ_params[0] != (self.parameter.clone(), AcceptRecursiveType::Reject)
            || !d.sized_constraint.as_ref().is_some_and(|x| types_equal(x, &t))
        {
            return Err("internal owned cell: native datatype/signature/privacy mismatch");
        }
        let [v] = d.variants.as_slice() else {
            return Err("internal owned cell: one variant required");
        };
        let [f] = v.fields.as_slice() else {
            return Err("internal owned cell: one private field required");
        };
        if v.name.as_str() != "UnsafeCell"
            || !matches!(v.ctor_style, CtorPrintStyle::Braces)
            || f.name != crate::def::field_ident_from_rust("value")
            || f.a.1 != Mode::Exec
            || f.a.2 != self.source_visibility
            || !types_equal(&f.a.0, &t)
        {
            return Err("internal owned cell: private native field mismatch");
        }
        Ok(())
    }

    pub fn validate_instantiation(&self, args: &Typs) -> Result<(), &'static str> {
        if matches!(self.family, Family::OwnedUnsafeCellU64)
            && (args.len() != 1 || !matches!(&*args[0], TypX::Int(IntRange::U(64))))
        {
            return Err("internal owned cell: only the actual u64 instantiation is supported");
        }
        Ok(())
    }

    pub fn source_visibility(&self) -> &Visibility {
        &self.source_visibility
    }

    pub fn source_visible_to(&self, module: &Path) -> bool {
        is_visible_to(&self.source_visibility, module)
    }
}

/// Re-read the actual descriptor-bearing object at WF and backend entry.
pub fn validate_datatype(datatype: &Datatype) -> Result<(), VirErr> {
    if let DatatypeTransparency::Internal(repr) = &datatype.x.transparency {
        repr.validate(&datatype.x).map_err(|m| crate::messages::error(&datatype.span, m))?;
    }
    Ok(())
}

pub fn contains_internal_representation(krate: &Krate) -> bool {
    krate.datatypes.iter().any(|d| matches!(d.x.transparency, DatatypeTransparency::Internal(_)))
}

/// WF/backend use checking is separate from descriptor shape checking. Never
/// accept a generic, nested or wrong-width native cell instance by shape alone.
pub fn validate_type_use(
    krate: &Krate,
    typ: &Typ,
    span: &crate::messages::Span,
) -> Result<(), VirErr> {
    use crate::ast_visitor::VisitorControlFlow;
    let result = crate::ast_visitor::typ_visitor_dfs(typ, &mut |t| {
        if let TypX::Datatype(name, args, _) = &**t {
            if let Some(d) = krate.datatypes.iter().find(|d| d.x.name == *name) {
                if let DatatypeTransparency::Internal(r) = &d.x.transparency {
                    if let Err(e) = r.validate_instantiation(args) {
                        return VisitorControlFlow::Stop(crate::messages::error(span, e));
                    }
                }
            }
        }
        VisitorControlFlow::Recurse
    });
    match result {
        VisitorControlFlow::Stop(e) => Err(e),
        _ => Ok(()),
    }
}

pub fn validate_krate_uses(krate: &Krate) -> Result<(), VirErr> {
    use crate::ast_visitor::{AstVisitor, WalkTypVisitorEnv};
    if !contains_internal_representation(krate) {
        return Ok(());
    }
    for d in &krate.datatypes {
        validate_datatype(d)?;
    }
    if !krate.datatypes.iter().any(|d| {
        matches!(&d.x.transparency, DatatypeTransparency::Internal(r)
            if matches!(r.family, Family::OwnedUnsafeCellU64))
    }) {
        return Ok(());
    }
    let check = |_: &mut (), t: &Typ| {
        validate_type_use(
            krate,
            t,
            // The owned descriptor above implies a nonempty datatype inventory.
            &krate.datatypes[0].span,
        )
    };
    let mut env = ();
    let mut visitor = WalkTypVisitorEnv { env: &mut env, ft: &check };
    visitor.visit_krate(krate)?;
    // The general visitor deliberately leaves opaque constructors unchanged.
    // Check both of their type-bearing fields explicitly; destructure all fields
    // so a future metadata extension cannot silently escape this boundary.
    for opaque in &krate.opaque_types {
        let OpaqueTypeX { def_fun: _, name: _, typ_params, typ_bounds } = &opaque.x;
        visitor.visit_typs(typ_params)?;
        visitor.visit_generic_bounds(typ_bounds)?;
    }
    Ok(())
}
