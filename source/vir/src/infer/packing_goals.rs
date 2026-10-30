//! Bounded, checked arithmetic interpretation of unsigned shift/or packing.
//! An ordinary caller can need a mathematical equation for a tagged inferred
//! result. Propose a universal conditional lemma, independently prove it with
//! the existing BV backend, and use that identical fact in the original query.
//! Operand bounds are premises; no callee or application property is assumed.
use super::*;
use crate::def::Spanned;
use crate::sst::{self, Exp, ExpX, FuncCheckSst, StmX};
use num_traits::ToPrimitive;
use std::collections::BTreeSet;

fn packings(f: &Function, shapes: &mut BTreeSet<(u32, u32)>) {
    let mut remaining = FORMULA_BUDGET;
    for post in f.x.ensure.0.iter() {
        crate::ast_visitor::expr_visitor_walk(post, &mut |e| {
            if remaining == 0 {
                return crate::ast_visitor::VisitorControlFlow::<()>::Return;
            }
            remaining -= 1;
            if let ExprX::Binary(
                BinaryOp::Bitwise(BitwiseOp::BitOr, BitshiftBehavior::Allow),
                left,
                _,
            ) = &e.x
            {
                if let ExprX::Binary(
                    BinaryOp::Bitwise(
                        BitwiseOp::Shl(IntegerTypeBitwidth::Width(w @ (8 | 16 | 32)), false),
                        BitshiftBehavior::Allow,
                    ),
                    _,
                    amount,
                ) = &left.x
                {
                    if let ExprX::Const(Constant::Int(k)) = &amount.x {
                        if let Some(k) = k.to_u32().filter(|k| *k > 0 && *k < *w) {
                            shapes.insert((*w, k));
                        }
                    }
                }
            }
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
    }
}

fn fact(span: &Span, width: u32, shift: u32) -> Exp {
    let typ = Arc::new(TypX::Int(IntRange::U(width)));
    let names = ["__verus_infer_pack_high", "__verus_infer_pack_low"]
        .map(|name| VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal));
    let value = |i: usize| SpannedTyped::new(span, &typ, ExpX::Var(names[i].clone()));
    let number = |n: BigInt| SpannedTyped::new(span, &int_typ(), ExpX::Const(Constant::Int(n)));
    let bin = |op, a: Exp, b: Exp| SpannedTyped::new(span, &bool_typ(), ExpX::Binary(op, a, b));
    let within = |v: Exp, max: BigInt| {
        bin(
            sst::BinaryOp::And,
            bin(sst::BinaryOp::Inequality(InequalityOp::Le), number(0.into()), v.clone()),
            bin(sst::BinaryOp::Inequality(InequalityOp::Le), v, number(max)),
        )
    };
    let factor = BigInt::one() << shift;
    let domain = bin(
        sst::BinaryOp::And,
        within(value(0), (BigInt::one() << (width - shift)) - 1),
        within(value(1), &factor - 1),
    );
    let shifted = SpannedTyped::new(
        span,
        &typ,
        ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(width), false)),
            value(0),
            number(shift.into()),
        ),
    );
    let packed = SpannedTyped::new(
        span,
        &typ,
        ExpX::Binary(sst::BinaryOp::Bitwise(BitwiseOp::BitOr), shifted, value(1)),
    );
    let product = SpannedTyped::new(
        span,
        &int_typ(),
        ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Mul), number(factor), value(0)),
    );
    let sum = SpannedTyped::new(
        span,
        &int_typ(),
        ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Add), product, value(1)),
    );
    let relation = bin(sst::BinaryOp::Implies, domain, bin(sst::BinaryOp::Eq, packed.clone(), sum));
    let binders = Arc::new(
        names.into_iter().map(|name| Arc::new(VarBinderX { name, a: typ.clone() })).collect(),
    );
    // The normal integer query instantiates precisely at the packed expression.
    // The BV backend ignores triggers and independently checks the full theorem.
    let triggers = Arc::new(vec![Arc::new(vec![packed])]);
    let binder = Spanned::new(
        span.clone(),
        sst::BndX::Quant(crate::ast_util::QUANT_FORALL, binders, triggers, None),
    );
    SpannedTyped::new(span, &bool_typ(), ExpX::Bind(binder, relation))
}

pub(super) fn decorate(
    check: &FuncCheckSst,
    callees: &HashMap<Fun, Function>,
) -> Result<FuncCheckSst, VirErr> {
    let mut shapes = BTreeSet::new();
    crate::sst_visitor::map_stm_visitor(&check.body, &mut |stm| {
        if let StmX::Call { fun: sst::CallTarget::Fun(name), .. } = &stm.x {
            if let Some(f) =
                callees.get(name).filter(|f| f.x.attrs.infer && !f.x.attrs.infer_allow_approx)
            {
                packings(f, &mut shapes);
            }
        }
        Ok(stm.clone())
    })?;
    if shapes.is_empty() {
        return Ok(check.clone());
    }
    let mut statements = shapes
        .into_iter()
        .take(4)
        .map(|(w, k)| bitvector_hints::checked(fact(&check.body.span, w, k)))
        .collect::<Result<Vec<_>, _>>()?;
    statements.push(check.body.clone());
    let mut result = check.clone();
    result.body = Spanned::new(check.body.span.clone(), StmX::Block(Arc::new(statements)));
    Ok(result)
}
