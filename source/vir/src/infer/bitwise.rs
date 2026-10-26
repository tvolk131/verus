//! Finite integer bit operations using Verus's existing executable semantics.
//! Normal shifts check the amount; left shifts discard high bits as in Rust.
use super::*;

pub(super) fn shape(typ: &Typ) -> Option<(IntegerTypeBitwidth, bool)> {
    match &**typ {
        TypX::Int(IntRange::U(n @ (8 | 16 | 32 | 64 | 128))) => {
            Some((IntegerTypeBitwidth::Width(*n), false))
        }
        TypX::Int(IntRange::I(n @ (8 | 16 | 32 | 64 | 128))) => {
            Some((IntegerTypeBitwidth::Width(*n), true))
        }
        TypX::Int(IntRange::USize) => Some((IntegerTypeBitwidth::ArchWordSize, false)),
        TypX::Int(IntRange::ISize) => Some((IntegerTypeBitwidth::ArchWordSize, true)),
        _ => None,
    }
}
pub(super) fn coerce(span: &Span, typ: &Typ, value: Term) -> Result<Term, VirErr> {
    if shape(typ).is_none() || !eligibility::integer(&value.expr.typ) {
        return Err(unsupported(span, "invalid finite integer operand"));
    }
    if crate::ast_util::types_equal(typ, &value.expr.typ) {
        return Ok(value);
    }
    let TypX::Int(range) = &**typ else { unreachable!() };
    summary::unary(span, UnaryOp::Clip { range: *range, truncate: true }, value)
}
pub(super) fn unary_valid(op: UnaryOp, typ: &Typ) -> bool {
    let Some((width, signed)) = shape(typ) else { return false };
    matches!((op, signed), (UnaryOp::BitNot(None), true))
        || op == UnaryOp::BitNot(Some(width)) && !signed
}
pub(super) fn pure_valid(op: BitwiseOp, a: &Typ, b: &Typ) -> bool {
    let Some((width, signed)) = shape(a) else { return false };
    match op {
        BitwiseOp::BitAnd | BitwiseOp::BitOr | BitwiseOp::BitXor => {
            crate::ast_util::types_equal(a, b)
        }
        BitwiseOp::Shr => eligibility::integer(b),
        BitwiseOp::Shl(w, s) => w == width && s == signed && eligibility::integer(b),
    }
}
pub(super) fn source(
    op: BinaryOp,
    a: &Typ,
    b: &Typ,
    result: &Typ,
) -> Option<(BinaryOp, Option<IntegerTypeBitwidth>)> {
    let BinaryOp::Bitwise(bit, policy) = op else { return None };
    let (width, _) = shape(a)?;
    if !crate::ast_util::types_equal(a, result) || !pure_valid(bit, a, b) {
        return None;
    }
    let shift = match (bit, policy) {
        (BitwiseOp::BitAnd | BitwiseOp::BitOr | BitwiseOp::BitXor, BitshiftBehavior::Allow) => None,
        (BitwiseOp::Shl(..) | BitwiseOp::Shr, BitshiftBehavior::Error(w)) if w == width => {
            Some(width)
        }
        _ => return None,
    };
    Some((BinaryOp::Bitwise(bit, BitshiftBehavior::Allow), shift))
}
pub(super) fn width(span: &Span, width: IntegerTypeBitwidth) -> Result<Term, VirErr> {
    match width {
        IntegerTypeBitwidth::ArchWordSize => Ok(arithmetic::arch_bits(span)),
        IntegerTypeBitwidth::Width(n @ (8 | 16 | 32 | 64 | 128)) => {
            term(span, &int_typ(), ExprX::Const(Constant::Int(n.into())), 1)
        }
        _ => Err(unsupported(span, "unsupported bit width")),
    }
}
pub(super) fn shift_safe(
    span: &Span,
    amount: Term,
    bits: IntegerTypeBitwidth,
) -> Result<Term, VirErr> {
    let zero = term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
    logical(
        span,
        LogicalOp::And,
        summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), zero, amount.clone())?,
        summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), amount, width(span, bits)?)?,
    )
}
