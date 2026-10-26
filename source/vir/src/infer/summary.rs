//! Typed builders for the binder-free summary language. Cache decoding and
//! imported-summary substitution deliberately use the same operator rules.
use super::*;

use eligibility::integer;

pub(super) fn unary(span: &Span, op: UnaryOp, a: Term) -> Result<Term, VirErr> {
    if op == UnaryOp::Length(ArrayKind::Slice) {
        return slices::length(span, a);
    }
    let typ = match op {
        UnaryOp::BitNot(_) if bitwise::unary_valid(op, &a.expr.typ) => a.expr.typ.clone(),
        UnaryOp::Not if matches!(&*a.expr.typ, TypX::Bool) => bool_typ(),
        UnaryOp::Clip {
            range:
                range @ (IntRange::USize
                | IntRange::ISize
                | IntRange::U(8 | 16 | 32 | 64 | 128)
                | IntRange::I(8 | 16 | 32 | 64 | 128)),
            ..
        } if integer(&a.expr.typ) => Arc::new(TypX::Int(range)),
        _ => return Err(unsupported(span, "invalid summary unary operation")),
    };
    term(span, &typ, ExprX::Unary(op, a.expr), 1 + a.size)
}

pub(super) fn binary(span: &Span, op: BinaryOp, a: Term, b: Term) -> Result<Term, VirErr> {
    if matches!(op, BinaryOp::Index(ArrayKind::Slice, BoundsCheck::Allow)) {
        return slices::index(span, a, b);
    }
    if matches!(op, BinaryOp::Index(ArrayKind::Array, BoundsCheck::Allow)) {
        return arrays::index(span, a, b);
    }
    let typ = match op {
        BinaryOp::Bitwise(bit, BitshiftBehavior::Allow)
            if bitwise::pure_valid(bit, &a.expr.typ, &b.expr.typ) =>
        {
            a.expr.typ.clone()
        }
        BinaryOp::Arith(
            ArithOp::Add(OverflowBehavior::Allow)
            | ArithOp::Sub(OverflowBehavior::Allow)
            | ArithOp::Mul(OverflowBehavior::Allow)
            | ArithOp::EuclideanDiv(Div0Behavior::Allow)
            | ArithOp::EuclideanMod(Div0Behavior::Allow)
            | ArithOp::TruncatingDiv(Div0Behavior::Allow, SignedDivEdgeCaseBehavior::Allow)
            | ArithOp::TruncatingMod(Div0Behavior::Allow, SignedDivEdgeCaseBehavior::Allow),
        ) if integer(&a.expr.typ) && integer(&b.expr.typ) => int_typ(),
        BinaryOp::Inequality(_) if integer(&a.expr.typ) && integer(&b.expr.typ) => bool_typ(),
        BinaryOp::Eq(Mode::Spec | Mode::Exec) | BinaryOp::Ne
            if enums::compatible(&a.expr.typ, &b.expr.typ) =>
        {
            bool_typ()
        }
        BinaryOp::BoolAndNoSC | BinaryOp::BoolOrNoSC | BinaryOp::BoolXor
            if matches!(&*a.expr.typ, TypX::Bool) && matches!(&*b.expr.typ, TypX::Bool) =>
        {
            bool_typ()
        }
        _ => return Err(unsupported(span, "invalid summary binary operation")),
    };
    term(span, &typ, ExprX::Binary(op, a.expr, b.expr), 1 + a.size + b.size)
}

pub(super) fn logical(span: &Span, op: LogicalOp, a: Term, b: Term) -> Result<Term, VirErr> {
    if !matches!(&*a.expr.typ, TypX::Bool) || !matches!(&*b.expr.typ, TypX::Bool) {
        return Err(unsupported(span, "nonboolean summary connective"));
    }
    super::logical(span, op, a, b)
}

pub(super) fn conditional(span: &Span, c: Term, a: Term, b: Term) -> Result<Term, VirErr> {
    if !matches!(&*c.expr.typ, TypX::Bool) || !enums::compatible(&a.expr.typ, &b.expr.typ) {
        return Err(unsupported(span, "invalid summary conditional"));
    }
    super::conditional(span, c, a, b)
}

// Count input nodes before descending, in addition to the expanded sizes
// maintained by Term. This also bounds inputs that simplify to tiny formulas.
pub(super) fn instantiate(expr: &Expr, vars: &HashMap<VarIdent, Term>) -> Result<Term, VirErr> {
    fn go(
        expr: &Expr,
        vars: &HashMap<VarIdent, Term>,
        remaining: &mut usize,
    ) -> Result<Term, VirErr> {
        if *remaining == 0 {
            return Err(error(
                &expr.span,
                "Verus Infer incomplete: summary traversal budget exceeded",
            ));
        }
        *remaining -= 1;
        let invalid = || unsupported(&expr.span, "invalid summary expression");
        let span = &expr.span;
        let result = match &expr.x {
            _ if arithmetic::arch_constant(expr).is_some() => {
                arithmetic::arch_constant(expr).unwrap()
            }
            ExprX::Var(v) => vars.get(v).cloned().ok_or_else(invalid)?,
            ExprX::Const(Constant::Bool(b)) => boolean(span, *b),
            ExprX::Const(Constant::Int(n)) if n.bits() <= 4096 => {
                term(span, &int_typ(), ExprX::Const(Constant::Int(n.clone())), 1)?
            }
            ExprX::ArrayLiteral(elements) => arrays::literal(
                span,
                &expr.typ,
                elements.iter().map(|e| go(e, vars, remaining)).collect::<Result<Vec<_>, _>>()?,
            )?,
            ExprX::Ctor(dt, variant, fields, None)
                if matches!(&*expr.typ, TypX::Datatype(actual, ..) if actual == dt)
                    && fields.len() <= 1
                    && fields
                        .first()
                        .is_none_or(|f| f.name == crate::def::positional_field_ident(0)) =>
            {
                enums::ctor(
                    span,
                    &expr.typ,
                    variant,
                    fields.first().map(|f| go(&f.a, vars, remaining)).transpose()?,
                )?
            }
            ExprX::UnaryOpr(UnaryOpr::IsVariant { datatype, variant }, a) if matches!(&*a.typ, TypX::Datatype(actual, ..) if actual == datatype) => {
                enums::is_variant(span, go(a, vars, remaining)?, variant)?
            }
            ExprX::UnaryOpr(UnaryOpr::Field(op), a) if enums::valid_field(op, &a.typ) => {
                enums::field(span, go(a, vars, remaining)?, &op.variant)?
            }
            ExprX::Unary(op @ UnaryOp::BitNot(_), a) => {
                unary(span, *op, bitwise::coerce(span, &a.typ, go(a, vars, remaining)?)?)?
            }
            ExprX::Unary(op, a) => unary(span, *op, go(a, vars, remaining)?)?,
            ExprX::Binary(op @ BinaryOp::Bitwise(bit, _), a, b) => {
                let left = bitwise::coerce(span, &a.typ, go(a, vars, remaining)?)?;
                let mut right = go(b, vars, remaining)?;
                if matches!(bit, BitwiseOp::BitAnd | BitwiseOp::BitOr | BitwiseOp::BitXor) {
                    right = bitwise::coerce(span, &b.typ, right)?;
                }
                binary(span, *op, left, right)?
            }
            ExprX::Binary(op, a, b) => {
                binary(span, *op, go(a, vars, remaining)?, go(b, vars, remaining)?)?
            }
            ExprX::Logical(op, a, b) => {
                logical(span, *op, go(a, vars, remaining)?, go(b, vars, remaining)?)?
            }
            ExprX::If(c, a, Some(b)) => conditional(
                span,
                go(c, vars, remaining)?,
                go(a, vars, remaining)?,
                go(b, vars, remaining)?,
            )?,
            _ => return Err(invalid()),
        };
        if !enums::compatible(&result.expr.typ, &expr.typ)
            && !slices::substitutes(&result.expr.typ, &expr.typ)
        {
            return Err(invalid());
        }
        Ok(result)
    }
    let mut remaining = FORMULA_BUDGET;
    go(expr, vars, &mut remaining)
}
