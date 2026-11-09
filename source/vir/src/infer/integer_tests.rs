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
                view: None,
                expr: SpannedTyped::new(&span(), &p.x.typ, ExprX::Var(p.x.name.clone())),
                size: 1,
            })
            .collect();
    integer_models::value(&span(), f, args).unwrap()
}

fn saturating_div_model(range: IntRange) -> Function {
    let original = model("saturating_div", range, Some(range), false);
    let mut data = original.x.clone();
    Arc::make_mut(&mut Arc::make_mut(&mut data.proxy.as_mut().unwrap().x).segments)[3] =
        Arc::new("_verus_external_fn_specification_test_saturating__div".into());
    let divisor = SpannedTyped::new(
        &span(),
        &data.params[1].x.typ,
        ExprX::Var(data.params[1].x.name.clone()),
    );
    let required = summary::binary(
        &span(),
        BinaryOp::Ne,
        Term { expr: divisor, size: 1, view: None },
        term(&span(), &int_typ(), ExprX::Const(Constant::Int(0.into())), 1).unwrap(),
    )
    .unwrap();
    data.require = Arc::new(vec![required.expr]);
    original.new_x(data)
}

#[test]
fn unsigned_saturating_div_exhaustive_u8_and_zero_safety() {
    let function = saturating_div_model(IntRange::U(8));
    assert!(integer_models::recognize(&function).is_some());
    let output = model_term(&function);
    let args = function
        .x
        .params
        .iter()
        .map(|param| Term {
            expr: SpannedTyped::new(&span(), &param.x.typ, ExprX::Var(param.x.name.clone())),
            size: 1,
            view: None,
        })
        .collect::<Vec<_>>();
    let safe = integer_models::safety(&span(), &function, &args).unwrap();
    for left in 0u8..=255 {
        for right in 0u8..=255 {
            assert_eq!(eval(&safe.expr, left.into(), right.into()), Value::Bool(right != 0));
            if right != 0 {
                assert_eq!(
                    eval(&output.expr, left.into(), right.into()),
                    Value::Int(left.saturating_div(right).into())
                );
            }
        }
    }
}

#[test]
fn unsigned_saturating_div_identity_type_and_requirement_near_misses() {
    for range in [IntRange::I(8), IntRange::I(64), IntRange::ISize, IntRange::Int] {
        if range != IntRange::Int {
            assert!(integer_models::recognize(&saturating_div_model(range)).is_none());
        }
    }
    let original = saturating_div_model(IntRange::U(64));
    for mutation in 0..13 {
        let mut data = original.x.clone();
        match mutation {
            0 => data.require = Arc::new(vec![]),
            1 => data.require = Arc::new(vec![boolean(&span(), true).expr]),
            2 => data.require = Arc::new(vec![data.require[0].clone(), data.require[0].clone()]),
            3 => {
                let ExprX::Binary(_, left, right) = &data.require[0].x else { panic!() };
                data.require = Arc::new(vec![SpannedTyped::new(
                    &span(),
                    &bool_typ(),
                    ExprX::Binary(BinaryOp::Eq(Mode::Spec), left.clone(), right.clone()),
                )]);
            }
            4 => Arc::make_mut(&mut Arc::make_mut(&mut data.name).path).krate = CrateId::Vstd,
            5 => {
                Arc::make_mut(
                    &mut Arc::make_mut(&mut Arc::make_mut(&mut data.name).path).segments,
                )[1] = Arc::new("other".into())
            }
            6 => {
                Arc::make_mut(&mut Arc::make_mut(&mut data.proxy.as_mut().unwrap().x).segments)[2] =
                    Arc::new("i64_specs_tmp".into())
            }
            7 => Arc::make_mut(&mut data.attrs).infer = true,
            8 => data.body = Some(input("x")),
            9 => data.unwind_spec = None,
            10 => {
                let mut parameter = data.params[1].x.clone();
                parameter.typ = Arc::new(TypX::Int(IntRange::U(32)));
                let mut parameters = data.params.as_ref().clone();
                parameters[1] = data.params[1].new_x(parameter);
                data.params = Arc::new(parameters);
            }
            11 => {
                let ExprX::Binary(op, _, right) = &data.require[0].x else { panic!() };
                data.require = Arc::new(vec![SpannedTyped::new(
                    &span(),
                    &bool_typ(),
                    ExprX::Binary(*op, input("x"), right.clone()),
                )]);
            }
            _ => {
                Arc::make_mut(&mut Arc::make_mut(&mut data.proxy.as_mut().unwrap().x).segments)[0] =
                    Arc::new("application".into())
            }
        }
        assert!(integer_models::recognize(&original.new_x(data)).is_none(), "{mutation}");
    }
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
    let a = Term { view: None, expr: var("x", IntRange::U(8)), size: 1 };
    let b = Term { view: None, expr: var("y", IntRange::U(8)), size: 1 };
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
