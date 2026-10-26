//! Integer arithmetic facts for untrusted synthesis. The original Verus
//! executable supplies all independent overflow and division checks.
use super::*;

pub(super) fn division_dependency(e: &Expr) -> Option<Fun> {
    match e.x {
        ExprX::Binary(BinaryOp::Arith(ArithOp::TruncatingDiv(..)), ..) => {
            Some(crate::def::fn_truncating_div())
        }
        ExprX::Binary(BinaryOp::Arith(ArithOp::TruncatingMod(..)), ..) => {
            Some(crate::def::fn_truncating_mod())
        }
        _ => None,
    }
}

/// Existing inline spec definitions used implicitly by Verus's signed / and %.
/// They are dependencies, never executable callees admitted by inference.
pub(super) fn division_model(f: &Function) -> bool {
    (f.x.name == crate::def::fn_truncating_div() || f.x.name == crate::def::fn_truncating_mod())
        && f.x.mode == Mode::Spec
        && matches!(f.x.kind, FunctionKind::Static)
        && f.x.typ_params.is_empty()
        && f.x.typ_bounds.is_empty()
        && f.x.proxy.is_none()
        && f.x.body.is_some()
        && f.x.attrs.inline
        && !f.x.attrs.infer
        && !f.x.attrs.is_external_body
        && f.x.params.len() == 2
        && f.x.params.iter().all(|p| {
            p.x.mode == Mode::Spec && !p.x.user_mut && matches!(&*p.x.typ, TypX::Int(IntRange::Int))
        })
        && f.x.inner_ret.x.mode == Mode::Spec
        && matches!(&*f.x.inner_ret.x.typ, TypX::Int(IntRange::Int))
        && f.x.outer_ret.x.mode == Mode::Spec
        && matches!(&*f.x.outer_ret.x.typ, TypX::Int(IntRange::Int))
}

pub(super) fn bounds(range: IntRange) -> Option<(BigInt, BigInt)> {
    match range {
        IntRange::U(bits @ (8 | 16 | 32 | 64 | 128)) => {
            Some((0.into(), (BigInt::one() << bits) - 1))
        }
        IntRange::I(bits @ (8 | 16 | 32 | 64 | 128)) => {
            let half = BigInt::one() << (bits - 1);
            Some((-half.clone(), half - 1))
        }
        _ => None,
    }
}

/// Architecture-dependent terms stay symbolic through synthesis, cache decoding,
/// and imports. Ordinary Verus supplies the existing 32/64-bit architecture model.
pub(super) fn exact_range(range: IntRange) -> bool {
    matches!(range, IntRange::USize | IntRange::ISize) || bounds(range).is_some()
}

pub(super) fn arch_bits(span: &Span) -> Term {
    let zero = number(span, 0.into());
    Term {
        expr: SpannedTyped::new(
            span,
            &int_typ(),
            ExprX::UnaryOpr(
                UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits),
                zero.expr,
            ),
        ),
        size: 2,
    }
}

pub(super) fn arch_bound(span: &Span, kind: IntegerTypeBoundKind) -> Term {
    let bits = arch_bits(span);
    Term {
        expr: SpannedTyped::new(
            span,
            &int_typ(),
            ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(kind), bits.expr),
        ),
        size: 3,
    }
}
pub(super) fn usize_max(span: &Span) -> Term {
    arch_bound(span, IntegerTypeBoundKind::UnsignedMax)
}

/// Numeric bounds for one verification architecture, never the host architecture.
pub(super) fn bounds_at(range: IntRange, word_bits: u32) -> Option<(BigInt, BigInt)> {
    if !matches!(word_bits, 32 | 64) {
        return None;
    }
    bounds(match range {
        IntRange::USize => IntRange::U(word_bits),
        IntRange::ISize => IntRange::I(word_bits),
        other => other,
    })
}

pub(super) fn signed_range(width: IntegerTypeBitwidth) -> IntRange {
    match width {
        IntegerTypeBitwidth::Width(bits) => IntRange::I(bits),
        IntegerTypeBitwidth::ArchWordSize => IntRange::ISize,
    }
}

/// Admit only these closed constants, never arbitrary integer-bound operators.
pub(super) fn arch_constant(e: &Expr) -> Option<Term> {
    if !eligibility::integer(&e.typ) {
        return None;
    }
    match &e.x {
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), a)
            if eligibility::integer(&a.typ)
                && matches!(&a.x, ExprX::Const(Constant::Int(n)) if n == &BigInt::from(0)) =>
        {
            Some(arch_bits(&e.span))
        }
        ExprX::UnaryOpr(
            UnaryOpr::IntegerTypeBound(
                kind @ (IntegerTypeBoundKind::UnsignedMax
                | IntegerTypeBoundKind::SignedMin
                | IntegerTypeBoundKind::SignedMax),
            ),
            a,
        ) if matches!(
            &a.x,
            ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), _)
        ) && arch_constant(a).is_some() =>
        {
            Some(arch_bound(&e.span, kind.clone()))
        }
        _ => None,
    }
}

/// Evaluate only closed architecture constants for an explicitly chosen model.
pub(super) fn arch_value(e: &Expr, word_bits: u32) -> Option<BigInt> {
    arch_constant(e)?;
    let ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(kind), _) = &e.x else {
        return None;
    };
    match kind {
        IntegerTypeBoundKind::ArchWordBits => Some(word_bits.into()),
        IntegerTypeBoundKind::UnsignedMax => bounds_at(IntRange::USize, word_bits).map(|(_, n)| n),
        IntegerTypeBoundKind::SignedMin => bounds_at(IntRange::ISize, word_bits).map(|(n, _)| n),
        IntegerTypeBoundKind::SignedMax => bounds_at(IntRange::ISize, word_bits).map(|(_, n)| n),
    }
}

pub(super) fn maximum(span: &Span, range: IntRange) -> Result<Term, VirErr> {
    match range {
        IntRange::USize => Ok(usize_max(span)),
        IntRange::ISize => Ok(arch_bound(span, IntegerTypeBoundKind::SignedMax)),
        _ => bounds(range)
            .map(|(_, max)| number(span, max))
            .ok_or_else(|| unsupported(span, "unsupported arithmetic range")),
    }
}
pub(super) fn minimum(span: &Span, range: IntRange) -> Result<Term, VirErr> {
    match range {
        IntRange::USize => Ok(number(span, 0.into())),
        IntRange::ISize => Ok(arch_bound(span, IntegerTypeBoundKind::SignedMin)),
        _ => bounds(range)
            .map(|(min, _)| number(span, min))
            .ok_or_else(|| unsupported(span, "unsupported arithmetic range")),
    }
}

fn number(span: &Span, n: BigInt) -> Term {
    Term { expr: SpannedTyped::new(span, &int_typ(), ExprX::Const(Constant::Int(n))), size: 1 }
}

pub(super) fn in_range(span: &Span, value: Term, range: IntRange) -> Result<Term, VirErr> {
    let min = minimum(span, range)?;
    let max = maximum(span, range)?;
    let lower = summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), min, value.clone())?;
    let upper = summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), value, max)?;
    logical(span, LogicalOp::And, lower, upper)
}

pub(super) fn division_valid(
    span: &Span,
    a: Term,
    b: Term,
    signed: Option<IntRange>,
) -> Result<Term, VirErr> {
    let nonzero = summary::binary(span, BinaryOp::Ne, b.clone(), number(span, 0.into()))?;
    let Some(range) = signed else { return Ok(nonzero) };
    let min = minimum(span, range)?;
    let not_min = summary::binary(span, BinaryOp::Ne, a, min)?;
    let not_minus_one = summary::binary(span, BinaryOp::Ne, b, number(span, (-1).into()))?;
    let no_overflow = logical(span, LogicalOp::Or, not_min, not_minus_one)?;
    logical(span, LogicalOp::And, nonzero, no_overflow)
}

/// rust_to_vir lowers a negative literal as 0 - magnitude with overflow Allow,
/// including MIN's otherwise out-of-range positive magnitude. Recognize only
/// this constant shape, rather than allowing unchecked executable arithmetic.
pub(super) fn negative_literal(e: &Expr) -> Option<BigInt> {
    let TypX::Int(range @ (IntRange::I(_) | IntRange::ISize)) = &*e.typ else { return None };
    let ExprX::Binary(BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)), a, b) = &e.x else {
        return None;
    };
    if !crate::ast_util::types_equal(&a.typ, &e.typ)
        || !crate::ast_util::types_equal(&b.typ, &e.typ)
    {
        return None;
    }
    let (ExprX::Const(Constant::Int(zero)), ExprX::Const(Constant::Int(magnitude))) = (&a.x, &b.x)
    else {
        return None;
    };
    let (min, _) = bounds_at(*range, 64)?;
    if zero != &BigInt::from(0) || magnitude < &BigInt::from(0) || -magnitude < min {
        return None;
    }
    Some(-magnitude)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_integer, function, input, span};

    fn signed_input(name: &str) -> Expr {
        let e = input(name);
        SpannedTyped::new(&span(), &Arc::new(TypX::Int(IntRange::I(8))), e.x.clone())
    }

    fn signed_function(name: &str, body: Expr) -> Function {
        let base = function(name, body.clone());
        let mut f = base.x.clone();
        f.params = Arc::new(
            f.params
                .iter()
                .map(|p| {
                    let mut param = p.x.clone();
                    param.typ = signed_input("x").typ.clone();
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

    #[test]
    fn exhaustive_i8_arithmetic_and_negation_match_rust() {
        type Reference = fn(i8, i8) -> Option<i8>;
        let overflow = OverflowBehavior::Error(IntRange::I(8));
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
                &signed_input("x").typ,
                ExprX::Binary(BinaryOp::Arith(op), signed_input("x"), signed_input("y")),
            );
            let original = signed_function("arithmetic", body);
            let inferred = infer_function(&original).unwrap();
            assert!(Arc::ptr_eq(
                original.x.body.as_ref().unwrap(),
                inferred.x.body.as_ref().unwrap()
            ));
            let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else {
                panic!("contract")
            };
            for x in i8::MIN..=i8::MAX {
                for y in i8::MIN..=i8::MAX {
                    let expected = reference(x, y);
                    assert_eq!(
                        evaluate_integer(&inferred.x.require[0], x.into(), y.into()),
                        Value::Bool(expected.is_some()),
                        "{op:?} ({x},{y})"
                    );
                    if let Some(expected) = expected {
                        assert_eq!(
                            evaluate_integer(output, x.into(), y.into()),
                            Value::Int(expected.into()),
                            "{op:?} ({x},{y})"
                        );
                    }
                }
            }
        }
        let zero = SpannedTyped::new(
            &span(),
            &signed_input("x").typ,
            ExprX::Const(Constant::Int(0.into())),
        );
        let body = SpannedTyped::new(
            &span(),
            &zero.typ,
            ExprX::Binary(BinaryOp::Arith(ArithOp::Sub(overflow)), zero.clone(), signed_input("x")),
        );
        let inferred = infer_function(&signed_function("negation", body)).unwrap();
        let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else {
            panic!("contract")
        };
        for x in i8::MIN..=i8::MAX {
            let expected = x.checked_neg();
            assert_eq!(
                evaluate_integer(&inferred.x.require[0], x.into(), 0),
                Value::Bool(expected.is_some())
            );
            if let Some(expected) = expected {
                assert_eq!(evaluate_integer(output, x.into(), 0), Value::Int(expected.into()));
            }
        }
        // Only the closed constant negation lowering may bypass an overflow check.
        let magnitude =
            SpannedTyped::new(&span(), &zero.typ, ExprX::Const(Constant::Int(128.into())));
        let literal = SpannedTyped::new(
            &span(),
            &zero.typ,
            ExprX::Binary(
                BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                zero.clone(),
                magnitude,
            ),
        );
        assert_eq!(negative_literal(&literal), Some((-128).into()));
        let unchecked = literal.new_x(ExprX::Binary(
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
            zero,
            signed_input("x"),
        ));
        assert!(infer_function(&signed_function("unchecked", unchecked)).is_err());
    }

    #[test]
    fn exhaustive_signed_casts_match_rust_and_cache_roundtrip() {
        type Reference = fn(i128) -> i128;
        let cases: [(IntRange, IntRange, i128, i128, Reference); 5] = [
            (IntRange::I(16), IntRange::I(8), i16::MIN.into(), i16::MAX.into(), |n| {
                (n as i8).into()
            }),
            (IntRange::I(16), IntRange::U(8), i16::MIN.into(), i16::MAX.into(), |n| {
                (n as u8).into()
            }),
            (IntRange::U(16), IntRange::I(8), 0, u16::MAX.into(), |n| (n as i8).into()),
            (IntRange::I(8), IntRange::I(16), i8::MIN.into(), i8::MAX.into(), |n| {
                (n as i16).into()
            }),
            (IntRange::I(8), IntRange::U(16), i8::MIN.into(), i8::MAX.into(), |n| {
                (n as u16).into()
            }),
        ];
        for (from, to, min, max, reference) in cases {
            let x =
                SpannedTyped::new(&span(), &Arc::new(TypX::Int(from)), signed_input("x").x.clone());
            let body = SpannedTyped::new(
                &span(),
                &Arc::new(TypX::Int(to)),
                ExprX::Unary(UnaryOp::Clip { range: to, truncate: false }, x),
            );
            let base = signed_function("cast", body);
            let mut f = base.x.clone();
            let mut p = f.params[0].x.clone();
            p.typ = Arc::new(TypX::Int(from));
            Arc::make_mut(&mut f.params)[0] = f.params[0].new_x(p);
            let original = base.new_x(f);
            let mut registry = registry::Registry::new(None);
            let fresh = registry.infer(&[original.clone()]).unwrap();
            let cached = registry.infer(&[original]).unwrap();
            assert_eq!((registry.hits, registry.misses), (1, 1));
            assert_eq!(format_contract(&fresh[0]), format_contract(&cached[0]));
            let ExprX::Binary(BinaryOp::Eq(_), _, output) = &fresh[0].x.ensure.0[0].x else {
                panic!("contract")
            };
            for n in min..=max {
                assert_eq!(evaluate_integer(&fresh[0].x.require[0], n, 0), Value::Bool(true));
                assert_eq!(
                    evaluate_integer(output, n, 0),
                    Value::Int(reference(n)),
                    "{from:?} -> {to:?} ({n})"
                );
            }
        }
    }

    #[test]
    fn fixed_signed_ranges_and_summary_operator_policy_are_closed() {
        for bits in [8, 16, 32, 64, 128] {
            let (min, max) = bounds(IntRange::I(bits)).unwrap();
            assert_eq!(max.clone() + 1, -min.clone());
            assert_eq!(max - min + 1, BigInt::one() << bits);
            assert!(enums::ValueType::Signed(bits).typ().is_some());
        }
        for range in [IntRange::I(0), IntRange::I(7), IntRange::I(256), IntRange::ISize] {
            assert!(bounds(range).is_none());
        }
        let term = |e: Expr| Term { expr: e, size: 1 };
        let invalid = [
            ArithOp::TruncatingDiv(Div0Behavior::Error, SignedDivEdgeCaseBehavior::Allow),
            ArithOp::TruncatingMod(
                Div0Behavior::Allow,
                SignedDivEdgeCaseBehavior::Error(IntegerTypeBitwidth::Width(8)),
            ),
        ];
        for op in invalid {
            assert!(
                summary::binary(
                    &span(),
                    BinaryOp::Arith(op),
                    term(signed_input("x")),
                    term(signed_input("y"))
                )
                .is_err()
            );
        }
        assert!(
            summary::unary(
                &span(),
                UnaryOp::Clip { range: IntRange::I(256), truncate: true },
                term(signed_input("x"))
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod checked_tests {
    use super::*;
    use crate::infer::tests::{Value, checked_model, evaluate_integer, span};

    #[test]
    fn exhaustive_i8_checked_methods_match_rust_including_euclidean_variants() {
        type Reference = fn(i8, i8) -> Option<i8>;
        let cases: [(&str, Reference); 7] = [
            ("checked_add", i8::checked_add),
            ("checked_sub", i8::checked_sub),
            ("checked_mul", i8::checked_mul),
            ("checked_div", i8::checked_div),
            ("checked_rem", i8::checked_rem),
            ("checked_div_euclid", i8::checked_div_euclid),
            ("checked_rem_euclid", i8::checked_rem_euclid),
        ];
        for (name, reference) in cases {
            let model = checked_model(name);
            let mut f = model.x.clone();
            f.params = Arc::new(
                f.params
                    .iter()
                    .map(|p| {
                        let mut param = p.x.clone();
                        param.typ = Arc::new(TypX::Int(IntRange::I(8)));
                        p.new_x(param)
                    })
                    .collect(),
            );
            let mut ret = f.inner_ret.x.clone();
            ret.typ =
                enums::ValueType::Option(Box::new(enums::ValueType::Signed(8))).typ().unwrap();
            f.inner_ret = f.inner_ret.new_x(ret);
            f.outer_ret = f.inner_ret.clone();
            let mut proxy = f.proxy.clone().unwrap();
            Arc::make_mut(&mut Arc::make_mut(&mut proxy.x).segments)[2] =
                Arc::new("i8_specs_tmp".into());
            f.proxy = Some(proxy);
            let signed = model.new_x(f);
            assert!(enums::checked(&signed).is_some());
            let arg = |index: usize| Term {
                expr: SpannedTyped::new(
                    &span(),
                    &signed.x.params[index].x.typ,
                    ExprX::Var(signed.x.params[index].x.name.clone()),
                ),
                size: 1,
            };
            let out = enums::checked_value(&span(), &signed, arg(0), arg(1)).unwrap();
            for x in i8::MIN..=i8::MAX {
                for y in i8::MIN..=i8::MAX {
                    let expected = match reference(x, y) {
                        Some(n) => Value::Enum("Some".into(), Some(Box::new(Value::Int(n.into())))),
                        None => Value::Enum("None".into(), None),
                    };
                    assert_eq!(
                        evaluate_integer(&out.expr, x.into(), y.into()),
                        expected,
                        "{name} ({x},{y})"
                    );
                }
            }
            let mut invalid = signed.x.clone();
            invalid.unwind_spec = None;
            assert!(enums::checked(&signed.new_x(invalid)).is_none());
            let mut invalid = signed.x.clone();
            let mut proxy = invalid.proxy.clone().unwrap();
            Arc::make_mut(&mut Arc::make_mut(&mut proxy.x).segments)[2] =
                Arc::new("u8_specs_tmp".into());
            invalid.proxy = Some(proxy);
            assert!(enums::checked(&signed.new_x(invalid)).is_none());
        }
    }
}

#[cfg(test)]
mod usize_tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_on_arch, function, input, span};

    fn usize_input(name: &str) -> Expr {
        SpannedTyped::new(&span(), &Arc::new(TypX::Int(IntRange::USize)), input(name).x.clone())
    }
    fn usize_function(body: Expr) -> Function {
        let base = function("usize_arithmetic", body.clone());
        let mut f = base.x.clone();
        f.params = Arc::new(
            f.params
                .iter()
                .map(|p| {
                    let mut param = p.x.clone();
                    param.typ = usize_input("x").typ.clone();
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

    #[test]
    fn usize_arithmetic_matches_checked_execution_on_both_architectures() {
        for bits in [32, 64] {
            let max = (1u128 << bits) - 1;
            for op in [
                ArithOp::Add(OverflowBehavior::Error(IntRange::USize)),
                ArithOp::Sub(OverflowBehavior::Error(IntRange::USize)),
                ArithOp::Mul(OverflowBehavior::Error(IntRange::USize)),
                ArithOp::EuclideanDiv(Div0Behavior::Error),
                ArithOp::EuclideanMod(Div0Behavior::Error),
            ] {
                let body = SpannedTyped::new(
                    &span(),
                    &usize_input("x").typ,
                    ExprX::Binary(BinaryOp::Arith(op), usize_input("x"), usize_input("y")),
                );
                let inferred = infer_function(&usize_function(body)).unwrap();
                let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else {
                    panic!("contract")
                };
                for x in [0, 1, 2, 65536, max / 2, max - 1, max] {
                    for y in [0, 1, 2, 3, 65536] {
                        let expected = match op {
                            ArithOp::Add(_) => x.checked_add(y).filter(|n| *n <= max),
                            ArithOp::Sub(_) => x.checked_sub(y),
                            ArithOp::Mul(_) => x.checked_mul(y).filter(|n| *n <= max),
                            ArithOp::EuclideanDiv(_) => x.checked_div(y),
                            ArithOp::EuclideanMod(_) => x.checked_rem(y),
                            _ => unreachable!(),
                        };
                        let vars = |name: &str| {
                            Value::Int(if name == "x" { x as i128 } else { y as i128 })
                        };
                        assert_eq!(
                            evaluate_on_arch(&inferred.x.require[0], &vars, bits),
                            Value::Bool(expected.is_some()),
                            "{bits} {op:?} {x} {y}"
                        );
                        if let Some(expected) = expected {
                            assert_eq!(
                                evaluate_on_arch(output, &vars, bits),
                                Value::Int(expected as i128)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn usize_casts_and_closed_architecture_constants_remain_symbolic() {
        let source = Arc::new(TypX::Int(IntRange::I(64)));
        let x = SpannedTyped::new(&span(), &source, input("x").x.clone());
        let body = SpannedTyped::new(
            &span(),
            &usize_input("x").typ,
            ExprX::Unary(UnaryOp::Clip { range: IntRange::USize, truncate: true }, x),
        );
        let f = usize_function(body);
        // Use the matching signed source parameter type.
        let mut fx = f.x.clone();
        let mut px = fx.params[0].x.clone();
        px.typ = source;
        let mut ps = (*fx.params).clone();
        ps[0] = fx.params[0].new_x(px);
        fx.params = Arc::new(ps);
        let inferred = infer_function(&f.new_x(fx)).unwrap();
        let ExprX::Binary(BinaryOp::Eq(_), _, out) = &inferred.x.ensure.0[0].x else {
            panic!("contract")
        };
        for bits in [32, 64] {
            for x in [i64::MIN as i128, -1, 0, 1, 1i128 << 32, i64::MAX as i128] {
                let expected = if bits == 32 { (x as u32) as i128 } else { (x as u64) as i128 };
                assert_eq!(evaluate_on_arch(out, &|_| Value::Int(x), bits), Value::Int(expected));
            }
            assert_eq!(
                evaluate_on_arch(&usize_max(&span()).expr, &|_| unreachable!(), bits),
                Value::Int((1i128 << bits) - 1)
            );
        }
        let bad = usize_max(&span()).expr.new_x(ExprX::UnaryOpr(
            UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::UnsignedMax),
            usize_input("x"),
        ));
        assert!(arch_constant(&bad).is_none());
        assert!(summary::instantiate(&bad, &HashMap::new()).is_err());
    }
}
