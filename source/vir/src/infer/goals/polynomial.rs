//! Separate finite one-hot shifts from small mathematical address polynomials.
//! Every proposed fact is checked unconditionally, then used verbatim. Shift
//! bounds are checked by BV; the complete conditional polynomial by NIA.
use super::*;
use crate::ast::AssertQueryMode;
use crate::def::Spanned;

pub(super) enum Proposal {
    BitVector(Exp),
    Polynomial { fact: Exp, shifts: Vec<Exp> },
}

// Count a finite bitwise term as one algebraic atom. Degree three is the
// narrow strategy boundary: existing degree-two packing/size queries keep BV.
fn degree(e: &Exp, budget: &mut usize, depth: usize) -> Option<usize> {
    *budget = budget.checked_sub(1)?;
    let next = depth.checked_sub(1)?;
    match &e.x {
        ExpX::Const(_) => Some(0),
        ExpX::Var(_) => Some(1),
        ExpX::Unary(_, a) => degree(a, budget, next),
        ExpX::Binary(sst::BinaryOp::Bitwise(_), _, _) => Some(1),
        ExpX::Binary(op, a, b) => {
            let a = degree(a, budget, next)?;
            let b = degree(b, budget, next)?;
            Some(if matches!(op, sst::BinaryOp::Arith(sst::ArithOp::Mul)) {
                a.saturating_add(b)
            } else {
                a.max(b)
            })
        }
        ExpX::If(c, a, b) => Some(
            degree(c, budget, next)?.max(degree(a, budget, next)?.max(degree(b, budget, next)?)),
        ),
        _ => None,
    }
}

pub(super) fn needed(e: &Exp, budget: &mut usize) -> Option<bool> {
    Some(degree(e, budget, DEPTH)? >= 3)
}

fn number(e: &Exp, n: u64) -> Exp {
    SpannedTyped::new(&e.span, &int_typ(), ExpX::Const(Constant::Int(n.into())))
}

fn one_hot(e: &Exp) -> Option<(&Exp, u32)> {
    let ExpX::Binary(
        sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(shift_width), false)),
        a,
        b,
    ) = &e.x
    else {
        return None;
    };
    if !matches!(&a.x, ExpX::Const(Constant::Int(n)) if *n == 1.into())
        || !matches!(&b.x, ExpX::Var(_) | ExpX::Const(Constant::Int(_)))
        || !matches!(&*b.typ, TypX::Int(IntRange::U(_)))
    {
        return None;
    }
    let TypX::Int(IntRange::U(width @ (8 | 16 | 32 | 64))) = &*e.typ else {
        return None;
    };
    if shift_width != width {
        return None;
    }
    Some((b, *width))
}

// A closed, small polynomial fragment. Division, arbitrary bitwise terms,
// branches, calls, opaque values and architecture-dependent integers decline.
fn collect(e: &Exp, shifts: &mut Vec<Exp>, budget: &mut usize, depth: usize) -> Option<()> {
    *budget = budget.checked_sub(1)?;
    let next = depth.checked_sub(1)?;
    match &e.x {
        ExpX::Const(Constant::Int(n)) if n.bits() <= 128 => Some(()),
        ExpX::Const(Constant::Bool(_)) => Some(()),
        ExpX::Var(_)
            if matches!(&*e.typ, TypX::Bool | TypX::Int(IntRange::U(8 | 16 | 32 | 64))) =>
        {
            Some(())
        }
        ExpX::Unary(UnaryOp::Not | UnaryOp::Clip { .. }, a) => collect(a, shifts, budget, next),
        ExpX::Binary(sst::BinaryOp::Bitwise(_), _, _) => {
            one_hot(e)?;
            if shifts.len() >= 4 {
                return None;
            }
            shifts.push(e.clone());
            Some(())
        }
        ExpX::Binary(
            sst::BinaryOp::Arith(sst::ArithOp::Add | sst::ArithOp::Sub | sst::ArithOp::Mul)
            | sst::BinaryOp::And
            | sst::BinaryOp::Or
            | sst::BinaryOp::Implies
            | sst::BinaryOp::Eq
            | sst::BinaryOp::Ne
            | sst::BinaryOp::Inequality(_),
            a,
            b,
        ) => {
            collect(a, shifts, budget, next)?;
            collect(b, shifts, budget, next)
        }
        _ => None,
    }
}

pub(super) fn propose(conclusion: &Exp, premises: &[Exp], budget: &mut usize) -> Option<Proposal> {
    if degree(conclusion, budget, DEPTH)? > 4 {
        return None;
    }
    let mut terms = Vec::new();
    collect(conclusion, &mut terms, budget, DEPTH)?;
    // Assumptions need not be in the polynomial fragment: omit unsupported
    // ones conservatively. Their visits still consume the shared work budget.
    let mut accepted = Vec::new();
    for premise in premises {
        let mut ignored_shifts = Vec::new();
        if degree(premise, budget, DEPTH).is_some_and(|degree| degree <= 4)
            && collect(premise, &mut ignored_shifts, budget, DEPTH).is_some()
        {
            accepted.push(premise.clone());
        }
    }
    let mut shifts = Vec::new();
    let mut fact = conclusion.clone();
    for term in terms {
        let (amount, width) = one_hot(&term)?;
        let valid = binary(
            sst::BinaryOp::And,
            binary(sst::BinaryOp::Inequality(InequalityOp::Le), number(amount, 0), amount.clone()),
            binary(
                sst::BinaryOp::Inequality(InequalityOp::Lt),
                amount.clone(),
                number(amount, width.into()),
            ),
        );
        let positive = binary(sst::BinaryOp::Inequality(InequalityOp::Lt), number(&term, 0), term);
        let bound = binary(sst::BinaryOp::Implies, valid, positive);
        fact = binary(sst::BinaryOp::Implies, bound.clone(), fact);
        shifts.push(bound);
    }
    for premise in accepted.into_iter().rev() {
        fact = binary(sst::BinaryOp::Implies, premise, fact);
    }
    if *budget == 0 {
        return None;
    }
    Some(Proposal::Polynomial { fact, shifts })
}

pub(super) fn checked(proposal: Proposal) -> Result<Stm, VirErr> {
    let Proposal::Polynomial { fact, shifts } = proposal else {
        let Proposal::BitVector(fact) = proposal else { unreachable!() };
        return super::super::bitvector_hints::checked(fact);
    };
    let mut stms = shifts
        .into_iter()
        .map(super::super::bitvector_hints::checked)
        .collect::<Result<Vec<_>, _>>()?;
    let span = fact.span.clone();
    let assertion = Spanned::new(
        span.clone(),
        StmX::Assert(
            None,
            Some(error(&span, "Verus Infer: polynomial proposal not satisfied")),
            fact.clone(),
        ),
    );
    let query = Spanned::new(
        span.clone(),
        StmX::AssertQuery {
            mode: AssertQueryMode::NonLinear,
            typ_inv_exps: Arc::new(vec![fact.clone()]),
            typ_inv_vars: Arc::new(vec![]),
            body: assertion,
        },
    );
    let used = Spanned::new(span.clone(), StmX::Assume(fact));
    // The isolated query has no assumptions or requires. Pointer identity is
    // checked here so later refactors cannot accidentally use another fact.
    let StmX::AssertQuery { body, .. } = &query.x else { unreachable!() };
    let StmX::Assert(_, _, checked) = &body.x else { unreachable!() };
    let StmX::Assume(assumed) = &used.x else { unreachable!() };
    if !Arc::ptr_eq(checked, assumed) {
        return Err(error(&span, "Verus Infer: mismatched polynomial theorem"));
    }
    stms.extend([query, used]);
    Ok(Spanned::new(span, StmX::Block(Arc::new(stms))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variable(name: &str) -> Exp {
        let input = crate::infer::tests::input(name);
        let ExprX::Var(name) = &input.x else { panic!() };
        SpannedTyped::new(
            &input.span,
            &Arc::new(TypX::Int(IntRange::U(64))),
            ExpX::Var(name.clone()),
        )
    }
    fn polynomial() -> Exp {
        let x = variable("x");
        let shift = x.new_x(ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(64), false)),
            number(&x, 1),
            variable("order"),
        ));
        let mul = |a: Exp, b: Exp| {
            SpannedTyped::new(
                &a.span,
                &int_typ(),
                ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Mul), a.clone(), b),
            )
        };
        binary(
            sst::BinaryOp::Inequality(InequalityOp::Le),
            mul(x.clone(), mul(shift, variable("size"))),
            number(&x, u64::MAX),
        )
    }
    #[test]
    fn isolates_exact_same_fact_with_no_query_assumptions() {
        let proposal = propose(&polynomial(), &[], &mut 8192).unwrap();
        let Proposal::Polynomial { fact, shifts } = &proposal else { panic!() };
        let fact = fact.clone();
        assert_eq!(shifts.len(), 1);
        let block = checked(proposal).unwrap();
        let StmX::Block(stms) = &block.x else { panic!() };
        let StmX::AssertQuery { body, mode: AssertQueryMode::NonLinear, .. } =
            &stms[stms.len() - 2].x
        else {
            panic!()
        };
        let StmX::Assert(_, _, checked) = &body.x else {
            panic!("query may only assert its theorem")
        };
        let StmX::Assume(used) = &stms.last().unwrap().x else { panic!() };
        assert!(Arc::ptr_eq(&fact, checked));
        assert!(Arc::ptr_eq(checked, used));
    }
    #[test]
    fn selector_separates_small_bv_from_address_polynomials() {
        assert_eq!(needed(&polynomial(), &mut 8192), Some(true));
        let x = variable("x");
        let small =
            x.new_x(ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Mul), x.clone(), x.clone()));
        assert_eq!(needed(&small, &mut 8192), Some(false));
    }
    #[test]
    fn unsupported_bitwise_and_large_degree_do_not_make_proposals() {
        let x = variable("x");
        let masked = x.new_x(ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::BitAnd),
            x.clone(),
            number(&x, 7),
        ));
        assert!(propose(&masked, &[], &mut 8192).is_none());
        let mut large = x.clone();
        for _ in 0..5 {
            large =
                x.new_x(ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Mul), large, x.clone()));
        }
        assert!(propose(&large, &[], &mut 8192).is_none());
        let shifted = x.new_x(ExpX::Binary(
            sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(64), false)),
            number(&x, 2),
            x.clone(),
        ));
        assert!(propose(&shifted, &[], &mut 8192).is_none());
    }
    #[test]
    fn exhausted_shared_budget_and_depth_decline() {
        let e = polynomial();
        for limit in 0..8 {
            let mut budget = limit;
            assert!(propose(&e, &[], &mut budget).is_none());
            assert!(budget <= limit);
        }
        let mut deep = e.clone();
        for _ in 0..DEPTH + 1 {
            deep = deep.new_x(ExpX::Unary(UnaryOp::Not, deep.clone()));
        }
        assert_eq!(needed(&deep, &mut 8192), None);
    }
}
