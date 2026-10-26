use super::*;
use crate::infer::tests::{Value, checked_model, evaluate_on_arch, function, input, span};

fn var(name: &str, range: IntRange) -> Expr {
    SpannedTyped::new(&span(), &Arc::new(TypX::Int(range)), input(name).x.clone())
}
fn scalar_fn(body: Expr, a: IntRange, b: IntRange) -> Function {
    let original = function("integer_operations", body.clone());
    let mut f = original.x.clone();
    f.params = Arc::new(
        f.params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let mut q = p.x.clone();
                q.typ = Arc::new(TypX::Int(if i == 0 { a } else { b }));
                p.new_x(q)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = body.typ.clone();
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    original.new_x(f)
}
fn eval(e: &Expr, a: i128, b: i128) -> Value {
    evaluate_on_arch(e, &|n| Value::Int(if n == "x" { a } else { b }), 64)
}
#[test]
fn exhaustive_u8_i8_bitwise_results_and_shift_safety_match_rust() {
    for signed in [false, true] {
        let range = if signed { IntRange::I(8) } else { IntRange::U(8) };
        let values: Vec<i128> = if signed { (-128..=127).collect() } else { (0..=255).collect() };
        for op in [
            BitwiseOp::BitAnd,
            BitwiseOp::BitOr,
            BitwiseOp::BitXor,
            BitwiseOp::Shl(IntegerTypeBitwidth::Width(8), signed),
            BitwiseOp::Shr,
        ] {
            let shift = matches!(op, BitwiseOp::Shr | BitwiseOp::Shl(..));
            let b_range = if shift { IntRange::I(16) } else { range };
            let x = var("x", range);
            let y = var("y", b_range);
            let policy = if shift {
                BitshiftBehavior::Error(IntegerTypeBitwidth::Width(8))
            } else {
                BitshiftBehavior::Allow
            };
            let body = SpannedTyped::new(
                &span(),
                &x.typ,
                ExprX::Binary(BinaryOp::Bitwise(op, policy), x.clone(), y),
            );
            let f = infer_function(&scalar_fn(body, range, b_range)).unwrap();
            let ExprX::Binary(BinaryOp::Eq(_), _, output) = &f.x.ensure.0[0].x else {
                panic!("contract")
            };
            let rhs = if shift {
                vec![-32768, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 32767]
            } else {
                values.clone()
            };
            for a in &values {
                for b in &rhs {
                    let valid = !shift || (0..8).contains(b);
                    assert_eq!(eval(&f.x.require[0], *a, *b), Value::Bool(valid));
                    if valid {
                        macro_rules! execute {
                            ($t:ty) => {{
                                let (a, b) = (*a as $t, *b as $t);
                                match op {
                                    BitwiseOp::BitAnd => (a & b) as i128,
                                    BitwiseOp::BitOr => (a | b) as i128,
                                    BitwiseOp::BitXor => (a ^ b) as i128,
                                    BitwiseOp::Shl(..) => (a << b) as i128,
                                    BitwiseOp::Shr => (a >> b) as i128,
                                }
                            }};
                        }
                        let expected = if signed { execute!(i8) } else { execute!(u8) };
                        assert_eq!(
                            eval(output, *a, *b),
                            Value::Int(expected),
                            "{signed} {op:?} {a} {b}"
                        );
                    }
                }
            }
        }
        let x = var("x", range);
        let op = UnaryOp::BitNot(if signed { None } else { Some(IntegerTypeBitwidth::Width(8)) });
        let body = SpannedTyped::new(&span(), &x.typ, ExprX::Unary(op, x.clone()));
        let f = infer_function(&scalar_fn(body, range, range)).unwrap();
        let ExprX::Binary(BinaryOp::Eq(_), _, out) = &f.x.ensure.0[0].x else { panic!("contract") };
        for a in values {
            assert_eq!(
                eval(out, a, 0),
                Value::Int(if signed { (!(a as i8)) as i128 } else { (!(a as u8)) as i128 })
            );
        }
    }
}

fn model(name: &str, lhs: IntRange, rhs: Option<IntRange>, checked: bool) -> Function {
    let original = checked_model(name);
    let mut f = original.x.clone();
    f.params = Arc::new(
        f.params
            .iter()
            .take(if rhs.is_some() { 2 } else { 1 })
            .enumerate()
            .map(|(i, p)| {
                let mut q = p.x.clone();
                q.typ = Arc::new(TypX::Int(if i == 0 { lhs } else { rhs.unwrap() }));
                p.new_x(q)
            })
            .collect(),
    );
    let scalar = enums::ValueType::from_typ(&f.params[0].x.typ).unwrap();
    let ret_typ = if checked {
        enums::ValueType::Option(Box::new(scalar)).typ().unwrap()
    } else {
        scalar.typ().unwrap()
    };
    let mut ret = f.inner_ret.x.clone();
    ret.typ = ret_typ;
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    let proxy = f.proxy.as_mut().unwrap();
    Arc::make_mut(&mut Arc::make_mut(&mut proxy.x).segments)[2] = Arc::new(match lhs {
        IntRange::U(n) => format!("u{n}_specs_tmp"),
        IntRange::I(n) => format!("i{n}_specs_tmp"),
        IntRange::USize => "usize_specs_tmp".into(),
        _ => "isize_specs_tmp".into(),
    });
    original.new_x(f)
}
fn model_term(f: &Function) -> Term {
    let args =
        f.x.params
            .iter()
            .map(|p| Term {
                expr: SpannedTyped::new(&span(), &p.x.typ, ExprX::Var(p.x.name.clone())),
                size: 1,
            })
            .collect();
    integer_models::value(&span(), f, args).unwrap()
}
fn option(n: Option<i128>) -> Value {
    match n {
        Some(n) => Value::Enum("Some".into(), Some(Box::new(Value::Int(n)))),
        None => Value::Enum("None".into(), None),
    }
}
#[test]
fn exhaustive_u8_i8_wrapping_and_saturating_candidates_match_rust() {
    for signed in [false, true] {
        for wrapping in [false, true] {
            for operation in ["add", "sub", "mul"] {
                let range = if signed { IntRange::I(8) } else { IntRange::U(8) };
                let f = model(
                    &format!("{}_{}", if wrapping { "wrapping" } else { "saturating" }, operation),
                    range,
                    Some(range),
                    false,
                );
                let out = model_term(&f);
                let values = if signed { -128..=127 } else { 0..=255 };
                for a in values.clone() {
                    for b in values.clone() {
                        macro_rules! execute {
                            ($t:ty) => {{
                                let (x, y) = (a as $t, b as $t);
                                (match (operation, wrapping) {
                                    ("add", true) => x.wrapping_add(y),
                                    ("sub", true) => x.wrapping_sub(y),
                                    ("mul", true) => x.wrapping_mul(y),
                                    ("add", false) => x.saturating_add(y),
                                    ("sub", false) => x.saturating_sub(y),
                                    _ => x.saturating_mul(y),
                                }) as i128
                            }};
                        }
                        let expected = if signed { execute!(i8) } else { execute!(u8) };
                        assert_eq!(
                            eval(&out.expr, a, b),
                            Value::Int(expected),
                            "{signed} {wrapping} {operation} {a} {b}"
                        );
                    }
                }
            }
        }
    }
}
#[test]
fn exhaustive_checked_negation_and_mixed_sign_candidates_match_rust() {
    for signed in [false, true] {
        let range = if signed { IntRange::I(8) } else { IntRange::U(8) };
        let neg = model("checked_neg", range, None, true);
        let out = model_term(&neg);
        for a in if signed { -128..=127 } else { 0..=255 } {
            let expected = if signed {
                (a as i8).checked_neg().map(i128::from)
            } else {
                (a as u8).checked_neg().map(i128::from)
            };
            assert_eq!(eval(&out.expr, a, 0), option(expected));
        }
        for sub in [false, true] {
            let method = format!(
                "checked_{}_{}",
                if sub { "sub" } else { "add" },
                if signed { "unsigned" } else { "signed" }
            );
            let other = if signed { IntRange::U(8) } else { IntRange::I(8) };
            let f = model(&method, range, Some(other), true);
            let out = model_term(&f);
            for a in if signed { -128..=127 } else { 0..=255 } {
                for b in if signed { 0..=255 } else { -128..=127 } {
                    let expected = if signed {
                        let (a, b) = (a as i8, b as u8);
                        if sub { a.checked_sub_unsigned(b) } else { a.checked_add_unsigned(b) }
                            .map(i128::from)
                    } else {
                        let (a, b) = (a as u8, b as i8);
                        if sub { a.checked_sub_signed(b) } else { a.checked_add_signed(b) }
                            .map(i128::from)
                    };
                    assert_eq!(eval(&out.expr, a, b), option(expected), "{method} {a} {b}");
                }
            }
            let wrong = model(&method, range, Some(range), true);
            assert!(integer_models::recognize(&wrong).is_none());
        }
    }
}
#[test]
fn malformed_bitwise_widths_policies_and_operand_types_are_rejected() {
    let span = span();
    let a = Term { expr: var("x", IntRange::U(8)), size: 1 };
    let b = Term { expr: var("y", IntRange::U(8)), size: 1 };
    for bad in [
        BinaryOp::Bitwise(
            BitwiseOp::Shl(IntegerTypeBitwidth::Width(16), false),
            BitshiftBehavior::Allow,
        ),
        BinaryOp::Bitwise(
            BitwiseOp::Shl(IntegerTypeBitwidth::Width(8), true),
            BitshiftBehavior::Allow,
        ),
        BinaryOp::Bitwise(BitwiseOp::Shr, BitshiftBehavior::Error(IntegerTypeBitwidth::Width(8))),
    ] {
        assert!(summary::binary(&span, bad, a.clone(), b.clone()).is_err());
    }
    assert!(summary::unary(&span, UnaryOp::BitNot(None), a.clone()).is_err());
    assert!(
        summary::binary(
            &span,
            BinaryOp::Bitwise(BitwiseOp::BitAnd, BitshiftBehavior::Allow),
            a.clone(),
            boolean(&span, true)
        )
        .is_err()
    );
    assert!(
        bitwise::source(
            BinaryOp::Bitwise(BitwiseOp::Shr, BitshiftBehavior::Allow),
            &a.expr.typ,
            &b.expr.typ,
            &a.expr.typ
        )
        .is_none()
    );
}
