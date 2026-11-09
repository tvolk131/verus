//! Closed core numeric models. Candidates are untrusted; specs are explicit vstd
//! trust dependencies. Match identities, proxy modules, arity, types and policy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    DivEuclid,
    RemEuclid,
    Neg,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Policy {
    Checked,
    Wrapping,
    Saturating,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Model {
    pub operation: Operation,
    pub policy: Policy,
    pub range: IntRange,
}

fn opposite(range: IntRange) -> Option<IntRange> {
    Some(match range {
        IntRange::U(n) => IntRange::I(n),
        IntRange::I(n) => IntRange::U(n),
        IntRange::USize => IntRange::ISize,
        IntRange::ISize => IntRange::USize,
        _ => return None,
    })
}
pub(super) fn recognize(function: &Function) -> Option<Model> {
    let f = &function.x;
    let segments = &f.name.path.segments;
    if f.name.path.krate != CrateId::Core || segments.first()?.as_str() != "num" {
        return None;
    }
    let name = segments.last()?.as_str();
    let (policy, rest) = if let Some(s) = name.strip_prefix("checked_") {
        (Policy::Checked, s)
    } else if let Some(s) = name.strip_prefix("wrapping_") {
        (Policy::Wrapping, s)
    } else if let Some(s) = name.strip_prefix("saturating_") {
        (Policy::Saturating, s)
    } else {
        return None;
    };
    let (base, mixed) = if let Some(s) = rest.strip_suffix("_signed") {
        (s, Some(false))
    } else if let Some(s) = rest.strip_suffix("_unsigned") {
        (s, Some(true))
    } else {
        (rest, None)
    };
    let operation = match base {
        "add" => Operation::Add,
        "sub" => Operation::Sub,
        "mul" => Operation::Mul,
        "div" => Operation::Div,
        "rem" => Operation::Rem,
        "div_euclid" => Operation::DivEuclid,
        "rem_euclid" => Operation::RemEuclid,
        "neg" => Operation::Neg,
        _ => return None,
    };
    let saturating_div = policy == Policy::Saturating && operation == Operation::Div;
    if policy != Policy::Checked
        && (!matches!(operation, Operation::Add | Operation::Sub | Operation::Mul)
            && !saturating_div
            || mixed.is_some())
    {
        return None;
    }
    if mixed.is_some()
        && (!matches!(operation, Operation::Add | Operation::Sub) || policy != Policy::Checked)
    {
        return None;
    }
    let arity = if operation == Operation::Neg { 1 } else { 2 };
    if f.mode != Mode::Exec
        || !matches!(f.kind, FunctionKind::Static)
        || !f.typ_params.is_empty()
        || !f.typ_bounds.is_empty()
        || f.params.len() != arity
        || (!saturating_div && !f.require.is_empty())
        || !matches!(f.unwind_spec, Some(UnwindSpec::NoUnwind))
        || f.params.iter().any(|p| p.x.mode != Mode::Exec || p.x.user_mut)
    {
        return None;
    }
    let TypX::Int(range) = &*f.params[0].x.typ else { return None };
    if saturating_div
        && (!matches!(range, IntRange::U(8 | 16 | 32 | 64 | 128) | IntRange::USize)
            || segments.len() != 3
            || !segments[1].starts_with("impl")
            || !matches!(f.item_kind, ItemKind::Function)
            || f.attrs.infer
            || f.attrs.is_unsafe
            || f.attrs.is_async
            || f.body.is_some()
            || !nonzero_divisor_requirement(function))
    {
        return None;
    }
    let (.., signed) = bitwise::shape(&f.params[0].x.typ)?;
    if mixed.is_some_and(|required_signed| signed != required_signed) {
        return None;
    }
    if arity == 2 {
        let expected = if mixed.is_some() { opposite(*range)? } else { *range };
        if !matches!(&*f.params[1].x.typ,TypX::Int(actual) if *actual==expected) {
            return None;
        }
    }
    let result = if policy == Policy::Checked {
        enums::ValueType::Option(Box::new(enums::ValueType::from_typ(&f.params[0].x.typ)?)).typ()?
    } else {
        f.params[0].x.typ.clone()
    };
    if !crate::ast_util::types_equal(&f.inner_ret.x.typ, &result)
        || !crate::ast_util::types_equal(&f.outer_ret.x.typ, &result)
        || f.inner_ret.x.mode != Mode::Exec
        || f.outer_ret.x.mode != Mode::Exec
    {
        return None;
    }
    let module = match range {
        IntRange::U(n) => format!("u{n}_specs_tmp"),
        IntRange::I(n) => format!("i{n}_specs_tmp"),
        IntRange::USize => "usize_specs_tmp".into(),
        IntRange::ISize => "isize_specs_tmp".into(),
        _ => return None,
    };
    let proxy = &f.proxy.as_ref()?.x;
    if !models::spec_proxy(
        proxy,
        &["std_specs", "num", &module],
        if saturating_div { "_saturating__div" } else { "" },
    ) {
        return None;
    }
    Some(Model { operation, policy, range: *range })
}

fn nonzero_divisor_requirement(function: &Function) -> bool {
    let data = &function.x;
    let [required] = data.require.as_slice() else { return false };
    let (divisor, zero) = match &required.x {
        ExprX::Binary(BinaryOp::Ne, divisor, zero) => (divisor, zero),
        ExprX::Unary(UnaryOp::Not, equality) => match &equality.x {
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), divisor, zero) => (divisor, zero),
            _ => return false,
        },
        _ => return false,
    };
    let variable = match &divisor.x {
        ExprX::Var(name) => Some(name),
        ExprX::ReadPlace(place, _) => match &place.x {
            PlaceX::Local(name) => Some(name),
            _ => None,
        },
        _ => None,
    };
    variable == Some(&data.params[1].x.name)
        && matches!(&zero.x, ExprX::Const(Constant::Int(value)) if value == &0.into())
        && crate::ast_util::types_equal(&divisor.typ, &data.params[1].x.typ)
}

pub(super) fn safety(span: &Span, function: &Function, args: &[Term]) -> Result<Term, VirErr> {
    let model =
        recognize(function).ok_or_else(|| unsupported(span, "unapproved integer safety model"))?;
    if model.policy == Policy::Saturating && model.operation == Operation::Div {
        if args.len() != 2 {
            return Err(unsupported(span, "division safety argument count"));
        }
        arithmetic::division_valid(span, args[0].clone(), args[1].clone(), None)
    } else {
        Ok(boolean(span, true))
    }
}
pub(super) fn value(span: &Span, f: &Function, args: Vec<Term>) -> Result<Term, VirErr> {
    let model =
        recognize(f).ok_or_else(|| unsupported(span, "unapproved integer arithmetic model"))?;
    if args.len() != f.x.params.len()
        || args
            .iter()
            .zip(f.x.params.iter())
            .any(|(a, p)| !enums::compatible(&a.expr.typ, &p.x.typ))
    {
        return Err(unsupported(span, "integer model argument types or count"));
    }
    let zero = term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
    let (a, b) = if model.operation == Operation::Neg {
        (zero, args[0].clone())
    } else {
        (args[0].clone(), args[1].clone())
    };
    let signed = matches!(model.range, IntRange::I(_) | IntRange::ISize);
    let op = match model.operation {
        Operation::Add => ArithOp::Add(OverflowBehavior::Allow),
        Operation::Sub | Operation::Neg => ArithOp::Sub(OverflowBehavior::Allow),
        Operation::Mul => ArithOp::Mul(OverflowBehavior::Allow),
        Operation::Div if signed => {
            ArithOp::TruncatingDiv(Div0Behavior::Allow, SignedDivEdgeCaseBehavior::Allow)
        }
        Operation::Rem if signed => {
            ArithOp::TruncatingMod(Div0Behavior::Allow, SignedDivEdgeCaseBehavior::Allow)
        }
        Operation::Div | Operation::DivEuclid => ArithOp::EuclideanDiv(Div0Behavior::Allow),
        Operation::Rem | Operation::RemEuclid => ArithOp::EuclideanMod(Div0Behavior::Allow),
    };
    let math = summary::binary(span, BinaryOp::Arith(op), a.clone(), b.clone())?;
    let scalar =
        |value| summary::unary(span, UnaryOp::Clip { range: model.range, truncate: true }, value);
    match model.policy {
        Policy::Wrapping => {
            // Match the existing standard model's bounded correction for add/sub
            // and modular product for mul. Verus's generic Clip axiom alone does
            // not specify the value of an out-of-range integer in ordinary SMT.
            let min = arithmetic::minimum(span, model.range)?;
            let max = arithmetic::maximum(span, model.range)?;
            let one = term(span, &int_typ(), ExprX::Const(Constant::Int(1.into())), 1)?;
            let unsigned_range = match model.range {
                IntRange::I(n) => IntRange::U(n),
                IntRange::ISize => IntRange::USize,
                r => r,
            };
            let modulus = summary::binary(
                span,
                BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
                arithmetic::maximum(span, unsigned_range)?,
                one,
            )?;
            let wrapped = if model.operation == Operation::Mul {
                let remainder = summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::EuclideanMod(Div0Behavior::Allow)),
                    math,
                    modulus.clone(),
                )?;
                if signed {
                    let above = summary::binary(
                        span,
                        BinaryOp::Inequality(InequalityOp::Gt),
                        remainder.clone(),
                        max,
                    )?;
                    let adjusted = summary::binary(
                        span,
                        BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                        remainder.clone(),
                        modulus,
                    )?;
                    summary::conditional(span, above, adjusted, remainder)?
                } else {
                    remainder
                }
            } else {
                let below = summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Lt),
                    math.clone(),
                    min,
                )?;
                let above = summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Gt),
                    math.clone(),
                    max,
                )?;
                let high = summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                    math.clone(),
                    modulus.clone(),
                )?;
                let low = summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
                    math.clone(),
                    modulus,
                )?;
                summary::conditional(
                    span,
                    above,
                    high,
                    summary::conditional(span, below, low, math)?,
                )?
            };
            scalar(wrapped)
        }
        Policy::Saturating => {
            if model.operation == Operation::Div {
                return scalar(math);
            }
            let min = arithmetic::minimum(span, model.range)?;
            let max = arithmetic::maximum(span, model.range)?;
            let below = summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Lt),
                math.clone(),
                min.clone(),
            )?;
            let above = summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Gt),
                math.clone(),
                max.clone(),
            )?;
            scalar(summary::conditional(
                span,
                below,
                min,
                summary::conditional(span, above, max, math)?,
            )?)
        }
        Policy::Checked => {
            // Avoid repeating bounds already guaranteed by primitive operand
            // types. This also preserves existing checked-call formula sizes.
            let mixed = f.x.params.len() == 2
                && !crate::ast_util::types_equal(&f.x.params[0].x.typ, &f.x.params[1].x.typ);
            let lower = || {
                summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Ge),
                    math.clone(),
                    arithmetic::minimum(span, model.range)?,
                )
            };
            let upper = || {
                summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Le),
                    math.clone(),
                    arithmetic::maximum(span, model.range)?,
                )
            };
            let valid = match model.operation {
                Operation::Div | Operation::Rem | Operation::DivEuclid | Operation::RemEuclid => {
                    arithmetic::division_valid(span, a, b, signed.then_some(model.range))?
                }
                Operation::Neg => {
                    if signed {
                        summary::binary(
                            span,
                            BinaryOp::Ne,
                            b,
                            arithmetic::minimum(span, model.range)?,
                        )?
                    } else {
                        summary::binary(
                            span,
                            BinaryOp::Eq(Mode::Spec),
                            b,
                            arithmetic::minimum(span, model.range)?,
                        )?
                    }
                }
                Operation::Add if !signed && !mixed || signed && mixed => upper()?,
                Operation::Mul if !signed => upper()?,
                Operation::Sub if !signed && !mixed || signed && mixed => lower()?,
                _ => arithmetic::in_range(span, math.clone(), model.range)?,
            };
            summary::conditional(
                span,
                valid,
                enums::ctor(span, &f.x.inner_ret.x.typ, &Arc::new("Some".into()), Some(math))?,
                enums::ctor(span, &f.x.inner_ret.x.typ, &Arc::new("None".into()), None)?,
            )
        }
    }
}
