//! Closed primitive arithmetic boundaries and checked native derive requests.
use super::wrapper_traits::{Kind, kind};
use super::*;

/// Generated extension definitions are usable only when the concrete matching
/// implementation is checked (exact for scalar outputs, ordinary for ordering
/// enums). Check even crates with zero
/// inferred functions, so a missing/misnamed derive cannot publish the model.
pub(super) fn validate_derived_requests(
    fs: &[Function],
    datatypes: &[Datatype],
) -> Result<(), VirErr> {
    for marker in
        fs.iter().filter(|f| f.x.name.path.last_segment().starts_with("__verus_infer_derived_"))
    {
        let name = marker.x.name.path.last_segment().to_string();
        let suffix = name.strip_prefix("__verus_infer_derived_").unwrap();
        let bad = || {
            unsupported(
                &marker.span,
                "derived trait request requires one checked implementation for the same scalar wrapper and signature",
            )
        };
        if !matches!(
            suffix,
            "from" | "into" | "add" | "sub" | "mul" | "div" | "rem" | "eq" | "partial_cmp" | "cmp"
        ) || !is_pure_observation(marker)
            || marker.x.params.len() != 1
            || marker.x.body.is_none()
            || !matches!(&*marker.x.inner_ret.x.typ, TypX::Bool)
        {
            return Err(bad());
        }
        let TypX::Decorate(TypDecoration::Ref, None, owner) = &*marker.x.params[0].x.typ else {
            return Err(bad());
        };
        let TypX::Datatype(dt, ts, _) = &**owner else {
            return Err(bad());
        };
        if !ts.is_empty() {
            return Err(bad());
        }
        let Some(data) = datatypes.iter().find(|d| d.x.name == *dt) else {
            return Err(bad());
        };
        if data.x.variants.len() != 1 || data.x.variants[0].fields.len() != 1 {
            return Err(bad());
        }
        let field = &data.x.variants[0].fields[0].a.0;
        if !newtypes::Newtypes::new(datatypes).wrapper(owner)
            || !matches!(&**field,TypX::Int(r) if arithmetic::exact_range(*r))
        {
            return Err(bad());
        }
        let count = fs
            .iter()
            .filter(|f| {
                if matches!(suffix, "partial_cmp" | "cmp") {
                    return wrapper_compare::ordinary_derive(f, owner, suffix);
                }
                let wanted = match suffix {
                    "from" | "into" => Some(Kind::From),
                    "eq" => Some(Kind::Compare),
                    _ => Some(Kind::Binary),
                };
                if kind(f) != wanted
                    || f.x.body.is_none()
                    || f.x.name.path.last_segment().as_str() != if suffix == "into" { "from" } else { suffix }
                    || if suffix == "eq" {
                        !matches!(&*f.x.inner_ret.x.typ, TypX::Bool)
                    } else {
                        !crate::ast_util::types_equal(&f.x.inner_ret.x.typ, if suffix == "into" { field } else { owner })
                    }
                {
                    return false;
                }
                let FunctionKind::TraitMethodImpl { trait_typ_args, .. } = &f.x.kind else {
                    return false;
                };
                trait_typ_args.len() == 2
                    && crate::ast_util::types_equal(&trait_typ_args[0], if suffix == "into" { field } else { owner })
                    && crate::ast_util::types_equal(
                        &trait_typ_args[1],
                        if suffix == "from" { field } else { owner },
                    )
            })
            .count();
        if count != 1 {
            return Err(bad());
        }
    }
    Ok(())
}

pub(super) fn binary_boundary(f: &Function) -> bool {
    if f.x.mode != Mode::Exec
        || f.x.attrs.infer
        || f.x.body.is_some()
        || f.x.params.len() != 2
        || !matches!(f.x.kind, FunctionKind::TraitMethodDecl { .. })
    {
        return false;
    }
    [("Add", "add"), ("Sub", "sub"), ("Mul", "mul"), ("Div", "div"), ("Rem", "rem")].iter().any(
        |(t, m)| {
            models::path_is(&f.x.name.path, CrateId::Core, &["ops", "arith", t, m])
                && f.x.proxy.as_ref().is_some_and(|p| {
                    models::path_is(
                        &p.x,
                        CrateId::Vstd,
                        &["std_specs", "ops", &format!("Ex{t}"), m],
                    )
                })
        },
    )
}

/// Existing primitive operator trait calls (including derive-generated UFCS).
/// Only concrete, equal integer operands at the canonical vstd boundary qualify.
pub(super) fn primitive_call(e: &Expr, all: &HashMap<Fun, Function>) -> Option<Expr> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, n, ts, _, _),
        args,
        post_args: None,
        body: None,
    } = &e.x
    else {
        return None;
    };
    let f = all.get(n)?;
    if !primitive_boundary(f, ts)
        || args.len() != 2
        || !args.iter().all(|a| crate::ast_util::types_equal(&a.typ, &ts[0]))
        || !crate::ast_util::types_equal(&e.typ, &ts[0])
    {
        return None;
    }
    if let CallTargetKind::DynamicResolved { resolved, is_trait_default: false, .. } = kind {
        let p = &resolved.path;
        if p.krate != CrateId::Core
            || p.segments.len() != 4
            || p.segments[0].as_str() != "ops"
            || p.segments[1].as_str() != "arith"
            || !p.segments[2].starts_with("impl")
            || p.last_segment() != n.path.last_segment()
        {
            return None;
        }
    } else {
        return None;
    }
    let TypX::Int(range) = &*ts[0] else {
        return None;
    };
    let signed = match range {
        IntRange::I(n) => SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::Width(*n)),
        IntRange::ISize => SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::ArchWordSize),
        _ => SignedDivEdgeCaseBehavior::Allow,
    };
    let op = match n.path.last_segment().as_str() {
        "add" => ArithOp::Add(OverflowBehavior::Error(*range)),
        "sub" => ArithOp::Sub(OverflowBehavior::Error(*range)),
        "mul" => ArithOp::Mul(OverflowBehavior::Error(*range)),
        "div" if matches!(range, IntRange::I(_) | IntRange::ISize) => {
            ArithOp::TruncatingDiv(Div0Behavior::Error, signed)
        }
        "rem" if matches!(range, IntRange::I(_) | IntRange::ISize) => {
            ArithOp::TruncatingMod(Div0Behavior::Error, signed)
        }
        "div" => ArithOp::EuclideanDiv(Div0Behavior::Error),
        "rem" => ArithOp::EuclideanMod(Div0Behavior::Error),
        _ => return None,
    };
    let result = e.new_x(ExprX::Binary(BinaryOp::Arith(op), args[0].clone(), args[1].clone()));
    if matches!(op, ArithOp::TruncatingDiv(..) | ArithOp::TruncatingMod(..)) {
        Some(e.new_x(ExprX::Unary(UnaryOp::Clip { range: *range, truncate: true }, result)))
    } else {
        Some(result)
    }
}
pub(super) fn primitive_boundary(f: &Function, ts: &Typs) -> bool {
    binary_boundary(f)
        && ts.len() == 2
        && crate::ast_util::types_equal(&ts[0], &ts[1])
        && matches!(&*ts[0],TypX::Int(r) if arithmetic::exact_range(*r))
}
