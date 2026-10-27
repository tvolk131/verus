use super::*;
use crate::infer::tests::{Value, evaluate_integer, function, input, span};

fn variable(name: &str, range: IntRange) -> Expr {
    let base = input(name);
    SpannedTyped::new(&span(), &Arc::new(TypX::Int(range)), base.x.clone())
}

fn sound_function(body: Expr, param_range: IntRange) -> Function {
    let base = function("signed_interval", body.clone());
    let mut f = base.x.clone();
    Arc::make_mut(&mut f.attrs).infer_allow_approx = true;
    f.params = Arc::new(
        f.params
            .iter()
            .map(|p| {
                let mut param = p.x.clone();
                param.typ = Arc::new(TypX::Int(param_range));
                p.new_x(param)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = body.typ.clone();
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    base.new_x(f)
}

fn result_relation(f: &Function) -> Expr {
    let result = variable(
        "x",
        match &*f.x.inner_ret.x.typ {
            TypX::Int(r) => *r,
            _ => unreachable!(),
        },
    );
    instantiate(
        &f.x.ensure.0[0],
        &HashMap::from([(
            f.x.inner_ret.x.name.clone(),
            Term { view: None, expr: result, size: 1 },
        )]),
    )
    .unwrap()
    .expr
}

#[test]
fn exhaustive_signed_interval_contracts_cover_safe_i8_execution() {
    type Reference = fn(i8, i8) -> Option<i8>;
    let range = IntRange::I(8);
    let overflow = OverflowBehavior::Error(range);
    let signed = SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::Width(8));
    let cases: [(ArithOp, Reference); 5] = [
        (ArithOp::Add(overflow), i8::checked_add),
        (ArithOp::Sub(overflow), i8::checked_sub),
        (ArithOp::Mul(overflow), i8::checked_mul),
        (ArithOp::TruncatingDiv(Div0Behavior::Error, signed), i8::checked_div),
        (ArithOp::TruncatingMod(Div0Behavior::Error, signed), i8::checked_rem),
    ];
    for (op, reference) in cases {
        let body = SpannedTyped::new(
            &span(),
            &Arc::new(TypX::Int(range)),
            ExprX::Binary(BinaryOp::Arith(op), variable("x", range), variable("y", range)),
        );
        let original = sound_function(body, range);
        let inferred = infer_function(&original).unwrap();
        assert!(Arc::ptr_eq(original.x.body.as_ref().unwrap(), inferred.x.body.as_ref().unwrap()));
        let post = result_relation(&inferred);
        for x in i8::MIN..=i8::MAX {
            for y in i8::MIN..=i8::MAX {
                let expected = reference(x, y);
                assert_eq!(
                    evaluate_integer(&inferred.x.require[0], x.into(), y.into()),
                    Value::Bool(expected.is_some()),
                    "{op:?} ({x},{y})"
                );
                if let Some(value) = expected {
                    assert_eq!(
                        evaluate_integer(&post, value.into(), 0),
                        Value::Bool(true),
                        "{op:?} ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn negative_min_literals_negation_and_casts_keep_all_signed_results() {
    let range = IntRange::I(8);
    let number = |n| {
        SpannedTyped::new(
            &span(),
            &Arc::new(TypX::Int(range)),
            ExprX::Const(Constant::Int(BigInt::from(n))),
        )
    };
    let min_literal = SpannedTyped::new(
        &span(),
        &Arc::new(TypX::Int(range)),
        ExprX::Binary(
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
            number(0),
            number(128),
        ),
    );
    let inferred = infer_function(&sound_function(min_literal, range)).unwrap();
    assert_eq!(
        range_from_summary(&inferred, 64).unwrap(),
        Range::Int((-128).into(), (-128).into())
    );
    let negate = SpannedTyped::new(
        &span(),
        &Arc::new(TypX::Int(range)),
        ExprX::Binary(
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(range))),
            number(0),
            variable("x", range),
        ),
    );
    let inferred = infer_function(&sound_function(negate, range)).unwrap();
    let post = result_relation(&inferred);
    for x in i8::MIN..=i8::MAX {
        assert_eq!(
            evaluate_integer(&inferred.x.require[0], x.into(), 0),
            Value::Bool(x.checked_neg().is_some())
        );
        if let Some(value) = x.checked_neg() {
            assert_eq!(evaluate_integer(&post, value.into(), 0), Value::Bool(true));
        }
    }
    type Reference = fn(i128) -> i128;
    let casts: [(IntRange, IntRange, i128, i128, Reference); 5] = [
        (IntRange::I(16), IntRange::I(8), i16::MIN.into(), i16::MAX.into(), |n| (n as i8).into()),
        (IntRange::I(16), IntRange::U(8), i16::MIN.into(), i16::MAX.into(), |n| (n as u8).into()),
        (IntRange::U(16), IntRange::I(8), 0, u16::MAX.into(), |n| (n as i8).into()),
        (IntRange::I(8), IntRange::I(16), i8::MIN.into(), i8::MAX.into(), |n| (n as i16).into()),
        (IntRange::I(8), IntRange::U(16), i8::MIN.into(), i8::MAX.into(), |n| (n as u16).into()),
    ];
    for (from, to, min, max, reference) in casts {
        let body = SpannedTyped::new(
            &span(),
            &Arc::new(TypX::Int(to)),
            ExprX::Unary(UnaryOp::Clip { range: to, truncate: false }, variable("x", from)),
        );
        let inferred = infer_function(&sound_function(body, from)).unwrap();
        let post = result_relation(&inferred);
        for x in min..=max {
            assert_eq!(evaluate_integer(&inferred.x.require[0], x, 0), Value::Bool(true));
            assert_eq!(
                evaluate_integer(&post, reference(x), 0),
                Value::Bool(true),
                "{from:?} -> {to:?} ({x})"
            );
        }
    }
}

fn contains(range: &Range, value: i16) -> bool {
    match range {
        Range::Int(a, b) => a <= &BigInt::from(value) && &BigInt::from(value) <= b,
        Range::IntTop => true,
        _ => false,
    }
}

#[test]
fn division_and_remainder_bounds_cover_small_intervals_and_i8_boundaries() {
    let mut intervals = Vec::new();
    for a in -8i16..=8 {
        for b in a..=8 {
            intervals.push((a, b));
        }
    }
    intervals.extend([(-128, 127), (-128, -128), (-128, -1), (0, 0), (1, 127), (127, 127)]);
    for &(a, b) in &intervals {
        for &(c, d) in &intervals {
            let args = (&BigInt::from(a), &BigInt::from(b), &BigInt::from(c), &BigInt::from(d));
            let quotients = [
                divide(args.0, args.1, args.2, args.3, true),
                divide(args.0, args.1, args.2, args.3, false),
            ];
            let remainders = [
                remainder(args.0, args.1, args.2, args.3, true),
                remainder(args.0, args.1, args.2, args.3, false),
            ];
            for x in a..=b {
                for y in c..=d {
                    if y == 0 {
                        continue;
                    }
                    assert!(
                        contains(&quotients[0], x / y),
                        "Rust division [{a},{b}]/[{c},{d}] ({x},{y})"
                    );
                    assert!(
                        contains(&quotients[1], x.div_euclid(y)),
                        "Euclidean division [{a},{b}]/[{c},{d}] ({x},{y})"
                    );
                    assert!(
                        contains(&remainders[0], x % y),
                        "Rust remainder [{a},{b}]/[{c},{d}] ({x},{y})"
                    );
                    assert!(
                        contains(&remainders[1], x.rem_euclid(y)),
                        "Euclidean remainder [{a},{b}]/[{c},{d}] ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn sound_callee_composition_keeps_full_domains_and_cache_roundtrips() {
    let range = IntRange::I(8);
    let leaf = sound_function(variable("x", range), range);
    let call = SpannedTyped::new(
        &span(),
        &leaf.x.inner_ret.x.typ,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                leaf.x.name.clone(),
                Arc::new(vec![]),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![variable("x", range), variable("y", range)]),
            post_args: None,
            body: None,
        },
    );
    let middle = sound_function(call, range);
    let mut f = middle.x.clone();
    f.name = Arc::new(FunX {
        path: Arc::new(PathX {
            krate: leaf.x.name.path.krate.clone(),
            segments: Arc::new(vec![Arc::new("signed_middle".into())]),
        }),
    });
    let middle = middle.new_x(f);
    let mut registry = registry::Registry::new(None);
    let fresh = registry.infer(&[leaf.clone(), middle.clone()]).unwrap();
    let cached = registry.infer(&[leaf, middle]).unwrap();
    assert_eq!((registry.hits, registry.misses), (2, 2));
    for (a, b) in fresh.iter().zip(&cached) {
        assert_eq!(format_contract(a), format_contract(b));
    }
    for x in i8::MIN..=i8::MAX {
        for y in i8::MIN..=i8::MAX {
            assert_eq!(
                evaluate_integer(&fresh[1].x.require[0], x.into(), y.into()),
                Value::Bool(true),
                "({x},{y})"
            );
        }
    }
}

#[test]
fn composed_safety_domains_cover_all_outputs_permitted_by_signed_and_unsigned_callees() {
    for signed in [false, true] {
        let r = if signed { IntRange::I(8) } else { IntRange::U(8) };
        let typ = Arc::new(TypX::Int(r));
        let div = if signed {
            ArithOp::TruncatingDiv(
                Div0Behavior::Error,
                SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::Width(8)),
            )
        } else {
            ArithOp::EuclideanDiv(Div0Behavior::Error)
        };
        let denominator = if signed { 10 } else { 100 };
        let constant =
            SpannedTyped::new(&span(), &typ, ExprX::Const(Constant::Int(denominator.into())));
        let leaf = sound_function(
            SpannedTyped::new(
                &span(),
                &typ,
                ExprX::Binary(BinaryOp::Arith(div), variable("x", r), constant),
            ),
            r,
        );
        let call = SpannedTyped::new(
            &span(),
            &typ,
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    leaf.x.name.clone(),
                    Arc::new(vec![]),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![variable("x", r), variable("y", r)]),
                post_args: None,
                body: None,
            },
        );
        let middle = sound_function(
            SpannedTyped::new(
                &span(),
                &typ,
                ExprX::Binary(
                    BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(r))),
                    call,
                    variable("y", r),
                ),
            ),
            r,
        );
        let mut f = middle.x.clone();
        f.name = Arc::new(FunX {
            path: Arc::new(PathX {
                krate: leaf.x.name.path.krate.clone(),
                segments: Arc::new(vec![Arc::new("offset".into())]),
            }),
        });
        let mut registry = registry::Registry::new(None);
        let candidate = infer_function(&leaf).unwrap();
        let mut exported = candidate.x.clone();
        exported.body = None;
        let inferred =
            registry.infer_with_imports(&[middle.new_x(f)], &[candidate.new_x(exported)]).unwrap();
        let post = result_relation(&inferred[0]);
        let (min, max) = if signed { (-128, 127) } else { (0, 255) };
        let (ql, qh) = if signed { (-12, 12) } else { (0, 2) };
        for x in min..=max {
            for y in min..=max {
                let expected = y + ql >= min && y + qh <= max;
                assert_eq!(
                    evaluate_integer(&inferred[0].x.require[0], x, y),
                    Value::Bool(expected),
                    "signed={signed} ({x},{y})"
                );
                if expected {
                    for q in ql..=qh {
                        assert!((min..=max).contains(&(q + y)));
                        assert_eq!(evaluate_integer(&post, q + y, 0), Value::Bool(true));
                    }
                }
            }
        }
    }
}
