use super::*;
use crate::infer::tests::{Value, checked_model, evaluate_on_arch, function, input, span};

fn variable(name: &str, range: IntRange) -> Expr {
    SpannedTyped::new(&span(), &Arc::new(TypX::Int(range)), input(name).x.clone())
}
fn scalar_function(body: Expr, range: IntRange, sound: bool) -> Function {
    let original = function("platform", body.clone());
    let mut f = original.x.clone();
    f.params = Arc::new(
        f.params
            .iter()
            .map(|p| {
                let mut x = p.x.clone();
                x.typ = Arc::new(TypX::Int(range));
                p.new_x(x)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = body.typ.clone();
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    Arc::make_mut(&mut f.attrs).infer_allow_approx = sound;
    original.new_x(f)
}
fn eval(e: &Expr, x: i128, y: i128, bits: u32) -> Value {
    evaluate_on_arch(e, &|n| Value::Int(if n == "x" { x } else { y }), bits)
}

#[test]
fn isize_boundary_arithmetic_matches_checked_32_and_64_bit_execution() {
    for bits in [32, 64] {
        let min = -(1i128 << (bits - 1));
        let max = -min - 1;
        for operation in 0..5 {
            let overflow = OverflowBehavior::Error(IntRange::ISize);
            let signed = SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::ArchWordSize);
            let op = match operation {
                0 => ArithOp::Add(overflow),
                1 => ArithOp::Sub(overflow),
                2 => ArithOp::Mul(overflow),
                3 => ArithOp::TruncatingDiv(Div0Behavior::Error, signed),
                _ => ArithOp::TruncatingMod(Div0Behavior::Error, signed),
            };
            let x = variable("x", IntRange::ISize);
            let y = variable("y", IntRange::ISize);
            let body = SpannedTyped::new(
                &span(),
                &x.typ,
                ExprX::Binary(BinaryOp::Arith(op), x.clone(), y),
            );
            let f = infer_function(&scalar_function(body, IntRange::ISize, false)).unwrap();
            let ExprX::Binary(BinaryOp::Eq(_), _, output) = &f.x.ensure.0[0].x else {
                panic!("contract")
            };
            for x in [min, min + 1, -10, -1, 0, 1, 10, max - 1, max] {
                for y in [min, -10, -1, 0, 1, 10, max] {
                    let expected = if bits == 32 {
                        let (x, y) = (x as i32, y as i32);
                        match operation {
                            0 => x.checked_add(y),
                            1 => x.checked_sub(y),
                            2 => x.checked_mul(y),
                            3 => x.checked_div(y),
                            _ => x.checked_rem(y),
                        }
                        .map(i128::from)
                    } else {
                        let (x, y) = (x as i64, y as i64);
                        match operation {
                            0 => x.checked_add(y),
                            1 => x.checked_sub(y),
                            2 => x.checked_mul(y),
                            3 => x.checked_div(y),
                            _ => x.checked_rem(y),
                        }
                        .map(i128::from)
                    };
                    assert_eq!(
                        eval(&f.x.require[0], x, y, bits),
                        Value::Bool(expected.is_some()),
                        "{bits} {operation} {x} {y}"
                    );
                    if let Some(n) = expected {
                        assert_eq!(eval(output, x, y, bits), Value::Int(n));
                    }
                }
            }
        }
    }
}

fn platform_model(name: &str, signed: bool) -> Function {
    let original = checked_model(name);
    let mut f = original.x.clone();
    let value = if signed { enums::ValueType::ISize } else { enums::ValueType::USize };
    let typ = value.typ().unwrap();
    f.params = Arc::new(
        f.params
            .iter()
            .map(|p| {
                let mut x = p.x.clone();
                x.typ = typ.clone();
                p.new_x(x)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = enums::ValueType::Option(Box::new(value)).typ().unwrap();
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    let proxy = f.proxy.as_mut().unwrap();
    Arc::make_mut(&mut Arc::make_mut(&mut proxy.x).segments)[2] =
        Arc::new(if signed { "isize_specs_tmp" } else { "usize_specs_tmp" }.into());
    original.new_x(f)
}

#[test]
fn checked_platform_models_match_rust_for_all_seven_operations_at_both_widths() {
    let names = [
        "checked_add",
        "checked_sub",
        "checked_mul",
        "checked_div",
        "checked_rem",
        "checked_div_euclid",
        "checked_rem_euclid",
    ];
    for signed in [false, true] {
        for (operation, name) in names.iter().enumerate() {
            let f = platform_model(name, signed);
            let arg = |index: usize| Term {
                view: None,
                expr: SpannedTyped::new(
                    &span(),
                    &f.x.params[index].x.typ,
                    ExprX::Var(f.x.params[index].x.name.clone()),
                ),
                size: 1,
            };
            let output = enums::checked_value(&span(), &f, arg(0), arg(1)).unwrap();
            for bits in [32, 64] {
                let max = if signed { (1i128 << (bits - 1)) - 1 } else { (1i128 << bits) - 1 };
                let min = if signed { -max - 1 } else { 0 };
                for x in [min, min + 1, 0, 1, max / 2, max - 1, max] {
                    for y in
                        if signed { vec![min, -10, -1, 0, 1, 10, max] } else { vec![0, 1, 2, 10] }
                    {
                        macro_rules! execute {
                            ($t:ty) => {{
                                let (a, b) = (x as $t, y as $t);
                                match operation {
                                    0 => a.checked_add(b),
                                    1 => a.checked_sub(b),
                                    2 => a.checked_mul(b),
                                    3 => a.checked_div(b),
                                    4 => a.checked_rem(b),
                                    5 => a.checked_div_euclid(b),
                                    _ => a.checked_rem_euclid(b),
                                }
                                .map(i128::from)
                            }};
                        }
                        let expected = match (signed, bits) {
                            (true, 32) => execute!(i32),
                            (true, _) => execute!(i64),
                            (false, 32) => execute!(u32),
                            (false, _) => execute!(u64),
                        };
                        let expected = match expected {
                            Some(n) => Value::Enum("Some".into(), Some(Box::new(Value::Int(n)))),
                            None => Value::Enum("None".into(), None),
                        };
                        assert_eq!(
                            eval(&output.expr, x, y, bits),
                            expected,
                            "{signed} {bits} {name} {x} {y}"
                        );
                    }
                }
            }
            let mut wrong = f.x.clone();
            let proxy = wrong.proxy.as_mut().unwrap();
            Arc::make_mut(&mut Arc::make_mut(&mut proxy.x).segments)[2] =
                Arc::new("u64_specs_tmp".into());
            assert!(enums::checked(&f.new_x(wrong)).is_none());
        }
    }
}

#[test]
fn platform_interval_contracts_cover_boundaries_without_widening_32_bit_results() {
    for range in [IntRange::USize, IntRange::ISize] {
        let x = variable("x", range);
        let ten = SpannedTyped::new(&span(), &x.typ, ExprX::Const(Constant::Int(10.into())));
        let div = if range == IntRange::ISize {
            ArithOp::TruncatingDiv(
                Div0Behavior::Error,
                SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::ArchWordSize),
            )
        } else {
            ArithOp::EuclideanDiv(Div0Behavior::Error)
        };
        let body =
            SpannedTyped::new(&span(), &x.typ, ExprX::Binary(BinaryOp::Arith(div), x.clone(), ten));
        let f = infer_function(&scalar_function(body, range, true)).unwrap();
        for bits in [32, 64] {
            let (min, max) = arithmetic::bounds_at(range, bits).unwrap();
            use num_traits::ToPrimitive;
            let (min, max) = (min.to_i128().unwrap(), max.to_i128().unwrap());
            for n in [min, min + 1, (-1i128).max(min), 0, 1, max - 1, max] {
                assert_eq!(eval(&f.x.require[0], n, 0, bits), Value::Bool(true));
                assert_eq!(
                    evaluate_on_arch(&f.x.ensure.0[0], &|_| Value::Int(n / 10), bits),
                    Value::Bool(true)
                );
            }
            assert_eq!(
                evaluate_on_arch(&f.x.ensure.0[0], &|_| Value::Int(max / 10 + 1), bits),
                Value::Bool(false)
            );
        }
        let mut imported = f.x.clone();
        imported.body = None;
        abstract_infer::validate_summary(&f.new_x(imported)).unwrap();
    }
}

#[test]
fn imported_interval_selectors_and_both_architecture_branches_are_validated() {
    let x = variable("x", IntRange::USize);
    let ten = SpannedTyped::new(&span(), &x.typ, ExprX::Const(Constant::Int(10.into())));
    let body = SpannedTyped::new(
        &span(),
        &x.typ,
        ExprX::Binary(BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Error)), x.clone(), ten),
    );
    let f = infer_function(&scalar_function(body, IntRange::USize, true)).unwrap();
    let mut exported = f.x.clone();
    exported.body = None;
    let ExprX::If(c, a, Some(b)) = &exported.ensure.0[0].x else { panic!("width split") };
    for bad in [
        ExprX::If(boolean(&span(), true).expr, a.clone(), Some(b.clone())),
        ExprX::If(c.clone(), boolean(&span(), true).expr, Some(b.clone())),
        ExprX::If(c.clone(), a.clone(), Some(boolean(&span(), true).expr)),
    ] {
        let mut poisoned = exported.clone();
        poisoned.ensure.0 = Arc::new(vec![exported.ensure.0[0].new_x(bad)]);
        assert!(abstract_infer::validate_summary(&f.new_x(poisoned)).is_err());
    }
}

#[test]
fn platform_interval_casts_cover_concrete_results_on_both_architectures() {
    for source in [IntRange::USize, IntRange::ISize] {
        for destination in [IntRange::USize, IntRange::ISize] {
            let x = variable("x", source);
            let body = SpannedTyped::new(
                &span(),
                &Arc::new(TypX::Int(destination)),
                ExprX::Unary(UnaryOp::Clip { range: destination, truncate: true }, x),
            );
            let f = infer_function(&scalar_function(body, source, true)).unwrap();
            for bits in [32, 64] {
                use num_traits::ToPrimitive;
                let (min, max) = arithmetic::bounds_at(source, bits).unwrap();
                let (min, max) = (min.to_i128().unwrap(), max.to_i128().unwrap());
                for n in [min, min + 1, (-1i128).max(min), 0, 1, max - 1, max] {
                    let expected = match (destination, bits) {
                        (IntRange::USize, 32) => (n as u32) as i128,
                        (IntRange::USize, _) => (n as u64) as i128,
                        (IntRange::ISize, 32) => (n as i32) as i128,
                        (IntRange::ISize, _) => (n as i64) as i128,
                        _ => unreachable!(),
                    };
                    assert_eq!(eval(&f.x.require[0], n, 0, bits), Value::Bool(true));
                    assert_eq!(
                        evaluate_on_arch(&f.x.ensure.0[0], &|_| Value::Int(expected), bits),
                        Value::Bool(true),
                        "{source:?} {destination:?} {bits} {n}"
                    );
                }
            }
        }
    }
}
