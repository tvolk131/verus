use super::*;
use crate::infer::tests::{Value, evaluate_integer, input, span};

#[test]
fn variable_shift_safety_matches_checked_u64_shift_for_every_u8_amount() {
    let amount = atom(input("x")).unwrap();
    let safe = bitwise::shift_safe(&span(), amount, IntegerTypeBitwidth::Width(64)).unwrap();
    for order in 0u8..=255 {
        let inferred = evaluate_integer(&safe.expr, i128::from(order), 0);
        assert_eq!(inferred, Value::Bool(1u64.checked_shl(u32::from(order)).is_some()));
    }
    assert_eq!(evaluate_integer(&safe.expr, -1, 0), Value::Bool(false));
}

#[test]
fn shift_and_multiply_requirements_match_independent_u128_boundary_oracle() {
    let amount = atom(input("x")).unwrap();
    let safe_shift =
        bitwise::shift_safe(&span(), amount.clone(), IntegerTypeBitwidth::Width(64)).unwrap();
    let before = term(
        &span(),
        &Arc::new(TypX::Int(IntRange::U(64))),
        ExprX::Const(Constant::Int(1.into())),
        1,
    )
    .unwrap();
    let shifted = summary::binary(
        &span(),
        BinaryOp::Bitwise(
            BitwiseOp::Shl(IntegerTypeBitwidth::Width(64), false),
            BitshiftBehavior::Allow,
        ),
        before,
        amount,
    )
    .unwrap();
    let multiplied = summary::binary(
        &span(),
        BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Allow)),
        shifted,
        atom(input("y")).unwrap(),
    )
    .unwrap();
    let safe_product = arithmetic::in_range(&span(), multiplied, IntRange::U(64)).unwrap();
    let requirement = logical(&span(), LogicalOp::And, safe_shift, safe_product).unwrap();
    for order in 0u8..=255 {
        for base in [0u32, 1, 2, 4096, 65535, 65536, u32::MAX - 1, u32::MAX] {
            let expected =
                order < 64 && ((1u128 << order) * u128::from(base)) <= u128::from(u64::MAX);
            let observed = evaluate_integer(&requirement.expr, i128::from(order), i128::from(base));
            assert_eq!(observed, Value::Bool(expected), "order={order}, base={base}");
        }
    }
}
