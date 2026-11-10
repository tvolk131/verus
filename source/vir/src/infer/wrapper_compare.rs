//! Candidate-only scalar comparisons through the existing core/vstd contracts.
//! Field ordering is a proposal. The original comparison call must establish it
//! through its checked implementation and ordinary trait extension definition.
use super::*;

pub(super) fn request(f: &Function) -> bool {
    f.x.name.path.last_segment().starts_with("__verus_infer_derived_")
}

pub(super) fn boundary(f: &Function) -> bool {
    let x = &f.x;
    if x.mode != Mode::Exec
        || x.attrs.infer
        || x.body.is_some()
        || x.attrs.is_unsafe
        || x.attrs.is_async
        || !x.require.is_empty()
        || x.params.len() != 2
        || !matches!(&*x.inner_ret.x.typ, TypX::Bool)
        || !matches!(x.kind, FunctionKind::TraitMethodDecl { .. })
    {
        return false;
    }
    [
        ("PartialEq", "eq"),
        ("PartialEq", "ne"),
        ("PartialOrd", "lt"),
        ("PartialOrd", "le"),
        ("PartialOrd", "gt"),
        ("PartialOrd", "ge"),
    ]
    .iter()
    .any(|(t, m)| {
        models::path_is(&x.name.path, CrateId::Core, &["cmp", t, m])
            && x.proxy.as_ref().is_some_and(|p| {
                models::path_is(&p.x, CrateId::Vstd, &["std_specs", "cmp", &format!("Ex{t}"), m])
            })
    })
}

pub(super) fn source_call(
    e: &Expr,
    all: &HashMap<Fun, Function>,
    datatypes: &[Datatype],
) -> Option<Expr> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, name, types, _, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    if !all.get(name).is_some_and(boundary)
        || types.len() != 2
        || args.len() != 2
        || !crate::ast_util::types_equal(&types[0], &types[1])
        || !matches!(&*e.typ, TypX::Bool)
        || !matches!(
            kind,
            CallTargetKind::DynamicResolved { .. } | CallTargetKind::ExternalTraitDefault
        ) && !(matches!(kind, CallTargetKind::Dynamic)
            && nonzero_u64(&types[0])
            && matches!(name.path.last_segment().as_str(), "eq" | "ne"))
        || !args.iter().all(|a| {
            let mut typ = &a.typ;
            // Copied dereferences may retain multiple shared-borrow decorations
            // in VIR. The original executable borrow remains checked normally.
            while let Some(inner) = trait_getters::shared(typ) {
                typ = inner;
            }
            crate::ast_util::types_equal(typ, &types[0])
        })
    {
        return None;
    }
    // Project genuine field equality only as a candidate. The original call
    // retains the standard NonZero contract during all three checks.
    if nonzero_u64(&types[0]) && matches!(name.path.last_segment().as_str(), "eq" | "ne") {
        return Some(e.new_x(ExprX::Binary(
            if name.path.last_segment().as_str() == "eq" {
                BinaryOp::Eq(Mode::Spec)
            } else {
                BinaryOp::Ne
            },
            args[0].clone(),
            args[1].clone(),
        )));
    }
    let TypX::Datatype(dt, ts, _) = &*types[0] else {
        return None;
    };
    // Native derive requests propose ordinary field ordering. Preserve custom
    // explicit comparison semantics, including an inherited ne whose eq_spec
    // may intentionally differ from field equality.
    let marker = if matches!(name.path.last_segment().as_str(), "eq" | "ne") {
        "__verus_infer_derived_eq"
    } else {
        "__verus_infer_derived_partial_cmp"
    };
    if !all.values().any(|f| {
        f.x.name.path.last_segment().as_str() == marker
            && f.x.params.len() == 1
            && trait_getters::shared(&f.x.params[0].x.typ)
                .is_some_and(|t| crate::ast_util::types_equal(t, &types[0]))
    }) {
        return None;
    }
    let d = datatypes.iter().find(|d| d.x.name == *dt)?;
    if !ts.is_empty()
        || d.x.mode != Mode::Exec
        || d.x.proxy.is_some()
        || !matches!(d.x.transparency, DatatypeTransparency::WhenVisible(_))
        || !d.x.typ_params.is_empty()
        || !d.x.typ_bounds.is_empty()
        || d.x.user_defined_invariant_fn.is_some()
        || d.x.destructor
        || d.x.variants.len() != 1
        || d.x.variants[0].fields.len() != 1
    {
        return None;
    }
    let variant = &d.x.variants[0];
    let field = &variant.fields[0];
    if field.a.1 != Mode::Exec
        || !(matches!(&*field.a.0,TypX::Int(r) if arithmetic::exact_range(*r))
            || matches!(name.path.last_segment().as_str(), "eq" | "ne") && nonzero_u64(&field.a.0))
    {
        return None;
    }
    let project = |a: &Expr| {
        let op = FieldOpr {
            datatype: dt.clone(),
            variant: variant.name.clone(),
            field: field.name.clone(),
            get_variant: false,
            check: VariantCheck::None,
        };
        let x = match &a.x {
            ExprX::ReadPlace(p, k) => ExprX::ReadPlace(
                SpannedTyped::new(&p.span, &field.a.0, PlaceX::Field(op, p.clone())),
                k.clone(),
            ),
            _ => ExprX::UnaryOpr(UnaryOpr::Field(op), a.clone()),
        };
        SpannedTyped::new(&a.span, &field.a.0, x)
    };
    let op = match name.path.last_segment().as_str() {
        "eq" => BinaryOp::Eq(Mode::Spec),
        "ne" => BinaryOp::Ne,
        "lt" => BinaryOp::Inequality(InequalityOp::Lt),
        "le" => BinaryOp::Inequality(InequalityOp::Le),
        "gt" => BinaryOp::Inequality(InequalityOp::Gt),
        "ge" => BinaryOp::Inequality(InequalityOp::Ge),
        _ => return None,
    };
    Some(e.new_x(ExprX::Binary(op, project(&args[0]), project(&args[1]))))
}

pub(super) fn ordinary_derive(f: &Function, owner: &Typ, suffix: &str) -> bool {
    let x = &f.x;
    let trait_name = if suffix == "cmp" { "Ord" } else { "PartialOrd" };
    let FunctionKind::TraitMethodImpl {
        method,
        trait_path,
        trait_typ_args,
        inherit_body_from: None,
        ..
    } = &x.kind
    else {
        return false;
    };
    let ordering = |t: &Typ| matches!(&**t,TypX::Datatype(Dt::Path(p),ts,_) if ts.is_empty() && models::path_is(p,CrateId::Core,&["cmp","Ordering"]));
    let result = if suffix == "cmp" {
        ordering(&x.inner_ret.x.typ)
    } else {
        matches!(&*x.inner_ret.x.typ,TypX::Datatype(Dt::Path(p),ts,_) if models::path_is(p,CrateId::Core,&["option","Option"]) && ts.len()==1 && ordering(&ts[0]))
    };
    models::path_is(trait_path, CrateId::Core, &["cmp", trait_name])
        && models::path_is(&method.path, CrateId::Core, &["cmp", trait_name, suffix])
        && x.mode == Mode::Exec
        && !x.attrs.infer
        && x.body.is_some()
        && x.proxy.is_none()
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.prophecy_dependent
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.typ_params.is_empty()
        && x.typ_bounds.is_empty()
        && x.require.is_empty()
        && x.params.len() == 2
        && result
        && crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        && trait_typ_args.iter().all(|t| crate::ast_util::types_equal(t, owner))
        && trait_typ_args.len() == if suffix == "cmp" { 1 } else { 2 }
        && x.params.iter().all(|p| {
            p.x.mode == Mode::Exec
                && !p.x.user_mut
                && trait_getters::shared(&p.x.typ)
                    .is_some_and(|t| crate::ast_util::types_equal(t, owner))
        })
}

/// Closed initial domain for view-projected checked equality. This recognizes
/// the real core type, not a user type named NonZero, and supplies no law.
pub(super) fn nonzero_u64(t: &Typ) -> bool {
    matches!(&**t, TypX::Datatype(Dt::Path(p), ts, _)
        if models::path_is(p, CrateId::Core, &["num", "nonzero", "NonZero"])
        && ts.len() == 1 && matches!(&*ts[0], TypX::Int(IntRange::U(64))))
}

pub(super) fn nonzero_owner(owner: &Typ, datatypes: &[Datatype]) -> bool {
    let TypX::Datatype(dt, ts, _) = &**owner else { return false };
    ts.is_empty()
        && datatypes.iter().any(|d| {
            d.x.name == *dt
                && d.x.mode == Mode::Exec
                && d.x.proxy.is_none()
                && matches!(d.x.transparency, DatatypeTransparency::WhenVisible(_))
                && d.x.typ_params.is_empty()
                && d.x.typ_bounds.is_empty()
                && d.x.user_defined_invariant_fn.is_none()
                && !d.x.destructor
                && d.x.variants.len() == 1
                && d.x.variants[0].fields.len() == 1
                && d.x.variants[0].fields[0].a.1 == Mode::Exec
                && nonzero_u64(&d.x.variants[0].fields[0].a.0)
        })
}

/// The observation is a candidate expression only. A malformed or weak
/// observation cannot bypass soundness of the original executable body.
pub(super) fn nonzero_observer<'a>(owner: &Typ, fs: &'a [Function]) -> Option<&'a Function> {
    let mut found = fs.iter().filter(|f| {
        let x = &f.x;
        x.name.path.last_segment().as_str() == "__verus_infer_nonzero_u64_view"
            && x.mode == Mode::Spec
            && matches!(x.kind, FunctionKind::Static)
            // A certified exported closed spec keeps its symbol/signature but
            // deliberately omits its body. Its producer's original eq body
            // and metadata checks justify the summary, not this recognizer.
            && is_pure_observation(f)
            && x.proxy.is_none()
            && !x.attrs.is_external_body
            && !x.attrs.infer
            && x.typ_params.is_empty()
            && x.typ_bounds.is_empty()
            && x.params.len() == 1
            && x.params[0].x.mode == Mode::Spec
            && trait_getters::shared(&x.params[0].x.typ)
                .is_some_and(|t| crate::ast_util::types_equal(t, owner))
            && matches!(&*x.inner_ret.x.typ, TypX::Int(IntRange::U(64)))
    });
    let f = found.next()?;
    if found.next().is_some() { None } else { Some(f) }
}
