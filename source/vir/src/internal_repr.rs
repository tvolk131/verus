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

// The only additional datatype from ordinary ast_simplify is builtin unit.
fn native_unit_datatypes(k: &Krate) -> bool {
    if k.datatypes.len() == 1 {
        return true;
    }
    if k.datatypes.len() != 2 {
        return false;
    }
    let d = &k.datatypes[1].x;
    d.name == Dt::Tuple(0)
        && d.proxy.is_none()
        && d.owning_module.is_none()
        && d.visibility.is_public()
        && matches!(&d.transparency,DatatypeTransparency::WhenVisible(v) if v.is_public())
        && d.mode == Mode::Exec
        && !d.ext_equal
        && !d.destructor
        && d.user_defined_invariant_fn.is_none()
        && d.sized_constraint.is_none()
        && d.typ_params.is_empty()
        && d.typ_bounds.is_empty()
        && d.variants.len() == 1
        && d.variants[0].name == crate::def::prefix_tuple_variant(0)
        && matches!(d.variants[0].ctor_style, CtorPrintStyle::Tuple)
        && d.variants[0].fields.is_empty()
}

/// Exact-unit native method privilege. No Clone/serde/FunctionStub form exists.
/// This is a compiler access check, never a postcondition or logical premise.
/// The frontend factory must first freshly check actual native metadata,
/// signatures/regions/MIR and independently emitted Function correspondence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeMethodQuery {
    Positive,
    FalseFirst,
    FalseFuture,
}

pub struct OwnedNativeMethodAuthority<'a> {
    query: NativeMethodQuery,
    unit: &'a Krate,
    datatype: &'a Datatype,
    functions: [&'a Function; 3],
}

pub const OWNED_NATIVE_METHOD_IDENTITIES: [u64; 3] =
    [10297035969456002870, 14652106901632559644, 13506035689665434230];

impl<'a> OwnedNativeMethodAuthority<'a> {
    /// Compiler-only closed three-method factory. Not a source installer.
    /// Native identities alone are insufficient: re-read complete current
    /// descriptor and all Function/body/contract/type/place relations too.
    pub fn from_checked_native_unit(
        unit: &'a Krate,
        native_methods: [u64; 3],
    ) -> Result<Self, &'static str> {
        Self::checked_unit(unit, native_methods, NativeMethodQuery::Positive)
    }
    /// Separate full-body falsification unit, never a callable user summary.
    pub fn from_checked_native_negative_unit(
        unit: &'a Krate,
        native_methods: [u64; 3],
    ) -> Result<Self, &'static str> {
        Self::checked_unit(unit, native_methods, NativeMethodQuery::FalseFirst)
    }
    /// Only the get_mut future postcondition is negated. No callable summary.
    pub fn from_checked_native_false_future_unit(
        unit: &'a Krate,
        native_methods: [u64; 3],
    ) -> Result<Self, &'static str> {
        Self::checked_unit(unit, native_methods, NativeMethodQuery::FalseFuture)
    }
    fn checked_unit(
        unit: &'a Krate,
        native_methods: [u64; 3],
        query: NativeMethodQuery,
    ) -> Result<Self, &'static str> {
        if native_methods != OWNED_NATIVE_METHOD_IDENTITIES
            || !native_unit_datatypes(unit)
            || unit.functions.len() != 3
            || !unit.traits.is_empty()
            || !unit.trait_impls.is_empty()
            || !unit.assoc_type_impls.is_empty()
            || !unit.opaque_types.is_empty()
            || !unit.external_fns.is_empty()
            || !unit.external_types.is_empty()
            || !unit.reveal_groups.is_empty()
        {
            return Err("owned native authority: exact native proof unit required");
        }
        let a = Self {
            query,
            unit,
            datatype: &unit.datatypes[0],
            functions: [&unit.functions[0], &unit.functions[1], &unit.functions[2]],
        };
        a.validate(unit)?;
        Ok(a)
    }
    pub fn validate(&self, unit: &Krate) -> Result<(), &'static str> {
        if !std::sync::Arc::ptr_eq(self.unit, unit)
            || !native_unit_datatypes(unit)
            || unit.functions.len() != 3
            || !std::sync::Arc::ptr_eq(self.datatype, &unit.datatypes[0])
        {
            return Err("owned native authority: reconstructed/pruned unit");
        }
        let DatatypeTransparency::Internal(r) = &self.datatype.x.transparency else {
            return Err("owned native authority: missing descriptor");
        };
        if !matches!(r.family, Family::OwnedUnsafeCellU64) {
            return Err("owned native authority: wrong descriptor family");
        }
        r.validate(&self.datatype.x)?;
        for (i, f) in unit.functions.iter().enumerate() {
            if !std::sync::Arc::ptr_eq(self.functions[i], f)
                || !owned_native_function(f, self.datatype, i, self.query)
            {
                return Err("owned native authority: changed full Function/body/contract");
            }
        }
        Ok(())
    }
    pub fn authorizes(&self, unit: &Krate, function: &Function, datatype: &Datatype) -> bool {
        self.validate(unit).is_ok()
            && std::sync::Arc::ptr_eq(self.datatype, datatype)
            && self.functions.iter().any(|f| std::sync::Arc::ptr_eq(f, function))
    }
}

/// Owned by one backend context; never carried by Function/stub/cache records.
pub(crate) struct OwnedNativeBackendBinding {
    query: NativeMethodQuery,
    unit: Krate,
}
impl OwnedNativeMethodAuthority<'_> {
    pub(crate) fn backend_binding(&self) -> OwnedNativeBackendBinding {
        OwnedNativeBackendBinding { unit: self.unit.clone(), query: self.query }
    }
}
impl OwnedNativeBackendBinding {
    pub(crate) fn validate_unit(&self, unit: &Krate) -> Result<(), &'static str> {
        if !std::sync::Arc::ptr_eq(&self.unit, unit) {
            return Err("owned native backend: changed/reconstructed unit");
        }
        OwnedNativeMethodAuthority::checked_unit(unit, OWNED_NATIVE_METHOD_IDENTITIES, self.query)
            .map(|_| ())
    }
    pub(crate) fn validate_function(&self, function: &Function, datatype: &Datatype) -> bool {
        let Ok(a) = OwnedNativeMethodAuthority::checked_unit(
            &self.unit,
            OWNED_NATIVE_METHOD_IDENTITIES,
            self.query,
        ) else {
            return false;
        };
        a.authorizes(&self.unit, function, datatype)
    }
}

fn owned_native_function(
    f: &Function,
    d: &Datatype,
    operation: usize,
    query: NativeMethodQuery,
) -> bool {
    let f = &f.x;
    let Some(owner) = &f.owning_module else {
        return false;
    };
    let a = &*f.attrs;
    // Exhaustive destructuring makes future attribute additions require review.
    let FunctionAttrsX {
        infer: _,
        infer_allow_approx: _,
        infer_precision_rlimit: _,
        infer_candidate_id: _,
        infer_sequence_view: _,
        infer_trait_getter: _,
        infer_trait_observation: _,
        infer_iterator_factory: _,
        infer_iterator_observation: _,
        infer_owned: _,
        infer_clone_contract: _,
        infer_mutable_selector: _,
        uses_ghost_blocks: _,
        inline: _,
        broadcast_forall: _,
        broadcast_forall_only: _,
        no_auto_trigger: _,
        auto_ext_equal: _,
        autospec: _,
        bit_vector: _,
        atomic: _,
        integer_ring: _,
        is_decrease_by: _,
        check_recommends: _,
        nonlinear: _,
        spinoff_prover: _,
        memoize: _,
        rlimit: _,
        print_zero_args: _,
        print_as_method: _,
        prophecy_dependent: _,
        size_of_broadcast_proof: _,
        is_type_invariant_fn: _,
        is_external_body: _,
        is_unsafe: _,
        impls_cannot_extend_spec: _,
        exec_assume_termination: _,
        exec_allows_no_decreases_clause: _,
        tracked_swap: _,
        tracked_take_option: _,
        is_async: _,
        is_drop: _,
    } = a;
    if a.infer
        || a.infer_allow_approx
        || a.infer_precision_rlimit.is_some()
        || a.infer_candidate_id.is_some()
        || a.infer_sequence_view
        || a.infer_trait_getter
        || a.infer_trait_observation
        || a.infer_iterator_factory
        || a.infer_iterator_observation
        || a.infer_owned
        || a.infer_clone_contract
        || a.infer_mutable_selector
        || !a.uses_ghost_blocks
        || a.inline
        || a.broadcast_forall
        || a.broadcast_forall_only
        || a.no_auto_trigger
        || a.auto_ext_equal != FunctionAttrsX::default().auto_ext_equal
        || a.autospec.is_some()
        || a.bit_vector
        || a.atomic
        || a.integer_ring
        || a.is_decrease_by
        || a.check_recommends
        || a.nonlinear
        || a.spinoff_prover
        || a.memoize
        || a.rlimit.is_some()
        || a.print_zero_args
        || a.print_as_method
        || a.prophecy_dependent
        || a.size_of_broadcast_proof
        || a.is_type_invariant_fn
        || a.is_external_body
        || a.is_unsafe
        || a.impls_cannot_extend_spec
        || a.exec_assume_termination
        || a.exec_allows_no_decreases_clause
        || a.tracked_swap
        || a.tracked_take_option
        || a.is_async
        || a.is_drop
    {
        return false;
    }
    if !matches!(&owner.krate, CrateId::Id(..))
        || f.name.path.segments.len() != 1
        || !owner.segments.is_empty()
        || f.name.path.pop_segment() != *owner
        || f.visibility.restricted_to != Some(owner.clone())
        || !matches!(&f.body_visibility,BodyVisibility::Visibility(v) if v.restricted_to==Some(owner.clone()))
        || !matches!(f.opaqueness, Opaqueness::Opaque)
        || f.mode != Mode::Exec
        || !matches!(f.kind, FunctionKind::Static)
        || !matches!(f.item_kind, ItemKind::Function)
        || f.proxy.is_some()
        || f.params.len() != 1
        || !f.typ_params.is_empty()
        || !f.typ_bounds.is_empty()
        || !f.require.is_empty()
        || !f.ensure.1.is_empty()
        || f.returns.is_some()
        || !f.decrease.is_empty()
        || f.decrease_when.is_some()
        || f.decrease_by.is_some()
        || f.fndef_axioms.is_some()
        || f.mask_spec.is_some()
        || f.atomic_update.is_some()
        || f.unwind_spec.is_some()
        || !f.return_opaque_type_eqs.is_empty()
        || !f.extra_dependencies.is_empty()
        || !f.hidden.is_empty()
    {
        return false;
    }
    let arg = &f.params[0].x;
    let ret = &f.inner_ret.x;
    let param_ok = |p: &ParamX| p.mode == Mode::Exec && !p.user_mut && p.unwrapped_info.is_none();
    if !param_ok(arg)
        || !param_ok(ret)
        || !param_ok(&f.outer_ret.x)
        || arg.name == ret.name
        || ret.name != f.outer_ret.x.name
        || !types_equal(&ret.typ, &f.outer_ret.x.typ)
    {
        return false;
    }
    let scalar = |t: &Typ| matches!(&**t, TypX::Int(IntRange::U(64)));
    let cell = |t: &Typ| matches!(&**t,TypX::Datatype(n,ts,is) if *n==d.x.name && ts.len()==1 && scalar(&ts[0]) && is.is_empty());
    let ty = |t: &Typ, kind: usize| match kind {
        0 => scalar(t),
        1 => cell(t),
        2 => matches!(&**t,TypX::MutRef(x) if cell(x)),
        3 => matches!(&**t,TypX::MutRef(x) if scalar(x)),
        _ => false,
    };
    let (at, rt) = match operation {
        0 => (0, 1),
        1 => (1, 0),
        2 => (2, 3),
        _ => return false,
    };
    if !ty(&arg.typ, at) || !ty(&ret.typ, rt) {
        return false;
    }
    let field = |o: &FieldOpr| {
        o.datatype == d.x.name
            && o.variant.as_str() == "UnsafeCell"
            && o.field == crate::def::field_ident_from_rust("value")
            && !o.get_variant
            && matches!(o.check, VariantCheck::None)
    };
    let var = |e: &Expr, p: &ParamX| {
        types_equal(&e.typ, &p.typ) && matches!(&e.x,ExprX::Var(v) if *v==p.name)
    };
    let root = |p: &Place| {
        types_equal(&p.typ, &arg.typ) && matches!(&p.x,PlaceX::Local(v) if *v==arg.name)
    };
    let Some(body) = &f.body else {
        return false;
    };
    let operation_body = if let ExprX::EvalAndResolve(value, tail) = &body.x {
        let unit =
            matches!(&*tail.typ,TypX::Datatype(Dt::Tuple(0),ts,is) if ts.is_empty()&&is.is_empty());
        let valid = operation == 2
            && unit
            && types_equal(&value.typ, &ret.typ)
            && matches!(&tail.x,ExprX::AssertAssume {is_assume:true,expr,msg:None}
                if matches!(&*expr.typ,TypX::Bool)
                && matches!(&expr.x,ExprX::UnaryOpr(UnaryOpr::HasResolved(t),read)
                    if types_equal(t,&arg.typ)&&matches!(&*arg.typ,TypX::MutRef(_))
                    &&types_equal(&read.typ,&arg.typ)
                    &&matches!(&read.x,ExprX::ReadPlace(p,r) if root(p)&&matches!(r.preliminary_kind,ReadKind::Spec)&&r.id==u64::MAX)));
        if !valid {
            return false;
        }
        value
    } else {
        body
    };

    if !types_equal(&body.typ, &ret.typ) {
        return false;
    }
    let body_ok = match (operation, &operation_body.x) {
        (0, ExprX::Ctor(n, v, bs, None))
            if *n == d.x.name && v.as_str() == "UnsafeCell" && bs.len() == 1 =>
        {
            bs[0].name == crate::def::field_ident_from_rust("value")
                && scalar(&bs[0].a.typ)
                && matches!(&bs[0].a.x,ExprX::ReadPlace(p,r) if root(p)&&matches!(r.preliminary_kind,ReadKind::Copy)&&r.id!=0)
        }
        (1, ExprX::ReadPlace(p, r)) => {
            scalar(&p.typ)
                && matches!(r.preliminary_kind, ReadKind::Move)
                && r.id != 0
                && matches!(&p.x,PlaceX::Field(o,p) if field(o)&&root(p))
        }
        (2, ExprX::BorrowMut(p)) => {
            scalar(&p.typ)
                && matches!(&p.x,PlaceX::Field(o,p) if field(o)&&cell(&p.typ)
            && matches!(&p.x,PlaceX::DerefMut(q) if root(q)))
        }
        _ => false,
    };
    if !body_ok {
        return false;
    }
    fn operands(e: &Expr, negative: bool) -> Option<(&Expr, &Expr)> {
        let e = if negative {
            if !matches!(&*e.typ, TypX::Bool) {
                return None;
            }
            if let ExprX::Unary(UnaryOp::Not, inner) = &e.x {
                inner
            } else {
                return None;
            }
        } else {
            e
        };
        if !matches!(&*e.typ, TypX::Bool) {
            return None;
        }
        if let ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) = &e.x { Some((a, b)) } else { None }
    }
    fn spec_field<'a>(e: &'a Expr, field: impl Fn(&FieldOpr) -> bool) -> Option<&'a Place> {
        if !matches!(&*e.typ, TypX::Int(IntRange::U(64))) {
            return None;
        }
        let ExprX::ReadPlace(p, r) = &e.x else {
            return None;
        };
        if !matches!(r.preliminary_kind, ReadKind::Spec)
            || r.id != u64::MAX
            || !types_equal(&p.typ, &e.typ)
        {
            return None;
        }
        if let PlaceX::Field(o, base) = &p.x {
            if field(o) { Some(base) } else { None }
        } else {
            None
        }
    }
    let select = |e: &Expr, p: &ParamX| {
        spec_field(e, field).is_some_and(|base| {
            types_equal(&base.typ, &p.typ) && matches!(&base.x,PlaceX::Local(v) if *v==p.name)
        })
    };
    let referent = |e: &Expr, p: &ParamX, future: bool, kind: usize| {
        ty(&e.typ, kind)
            && match &e.x {
                ExprX::Unary(UnaryOp::MutRefCurrent, v) if !future => var(v, p),
                ExprX::Unary(UnaryOp::MutRefFuture(MutRefFutureSourceName::MutRefFuture), v)
                    if future =>
                {
                    var(v, p)
                }
                _ => false,
            }
    };
    let select_ref = |e: &Expr, p: &ParamX, future: bool| {
        spec_field(e, field).is_some_and(|base| {
            cell(&base.typ) && matches!(&base.x,PlaceX::Temporary(v) if referent(v,p,future,1))
        })
    };
    match operation {
        0 if f.ensure.0.len() == 1 => {
            operands(&f.ensure.0[0], query == NativeMethodQuery::FalseFirst)
                .is_some_and(|(a, b)| select(a, ret) && var(b, arg))
        }
        1 if f.ensure.0.len() == 1 => {
            operands(&f.ensure.0[0], query == NativeMethodQuery::FalseFirst)
                .is_some_and(|(a, b)| var(a, ret) && select(b, arg))
        }
        2 if f.ensure.0.len() == 2 => {
            operands(&f.ensure.0[0], query == NativeMethodQuery::FalseFirst)
                .is_some_and(|(a, b)| referent(a, ret, false, 0) && select_ref(b, arg, false))
                && operands(&f.ensure.0[1], query == NativeMethodQuery::FalseFuture)
                    .is_some_and(|(a, b)| select_ref(a, arg, true) && referent(b, ret, true, 0))
        }
        _ => false,
    }
}

pub fn has_owned_cell_descriptor(unit: &Krate) -> bool {
    unit.datatypes.iter().any(|d|matches!(&d.x.transparency,DatatypeTransparency::Internal(r) if matches!(r.family,Family::OwnedUnsafeCellU64)))
}

pub(crate) fn is_owned_cell_datatype(d: &Datatype) -> bool {
    matches!(&d.x.transparency,DatatypeTransparency::Internal(r) if matches!(r.family,Family::OwnedUnsafeCellU64))
}
