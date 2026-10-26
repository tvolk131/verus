use super::*;
use num_traits::ToPrimitive;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Value {
    Int(i128),
    Bool(bool),
    Enum(String, Option<Box<Value>>),
    Array(Vec<Value>),
}
impl Value {
    fn int(self) -> i128 {
        if let Self::Int(n) = self { n } else { panic!("expected int") }
    }
    fn boolean(self) -> bool {
        if let Self::Bool(b) = self { b } else { panic!("expected bool") }
    }
}

// An independent concrete evaluator for the mathematical formulas, used only on
// the small u8 domain. The reference execution uses Rust's checked operations.
pub(super) fn evaluate(e: &Expr, x: u8, y: u8) -> Value {
    evaluate_integer(e, x.into(), y.into())
}

pub(super) fn evaluate_integer(e: &Expr, x: i128, y: i128) -> Value {
    evaluate_with(e, &|name| {
        Value::Int(match name {
            "x" => x,
            "y" => y,
            _ => panic!("unknown variable"),
        })
    })
}

pub(super) fn evaluate_with(e: &Expr, variable: &dyn Fn(&str) -> Value) -> Value {
    evaluate_on_arch(e, variable, 64)
}

pub(super) fn evaluate_on_arch(
    e: &Expr,
    variable: &dyn Fn(&str) -> Value,
    word_bits: u32,
) -> Value {
    match &e.x {
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), _) => {
            Value::Int(word_bits.into())
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::UnsignedMax), a) => {
            Value::Int((1i128 << evaluate_on_arch(a, variable, word_bits).int()) - 1)
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMin), a) => {
            Value::Int(-(1i128 << (evaluate_on_arch(a, variable, word_bits).int() - 1)))
        }
        ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::SignedMax), a) => {
            Value::Int((1i128 << (evaluate_on_arch(a, variable, word_bits).int() - 1)) - 1)
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::ISize, .. }, a) => {
            let modulus = 1i128 << word_bits;
            let n = evaluate_on_arch(a, variable, word_bits).int().rem_euclid(modulus);
            Value::Int(if n >= modulus / 2 { n - modulus } else { n })
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::USize, .. }, a) => Value::Int(
            evaluate_on_arch(a, variable, word_bits).int().rem_euclid(1i128 << word_bits),
        ),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), a) => {
            let Value::Array(values) = evaluate_on_arch(a, variable, word_bits) else {
                panic!("expected slice")
            };
            Value::Int(values.len() as i128)
        }
        ExprX::ArrayLiteral(elements) => Value::Array(
            elements.iter().map(|e| evaluate_on_arch(e, variable, word_bits)).collect(),
        ),
        ExprX::Binary(
            BinaryOp::Index(ArrayKind::Array | ArrayKind::Slice, BoundsCheck::Allow),
            a,
            i,
        ) => {
            let Value::Array(values) = evaluate_on_arch(a, variable, word_bits) else {
                panic!("expected array")
            };
            let index = evaluate_on_arch(i, variable, word_bits).int() as usize;
            values.into_iter().nth(index).expect("safe array index")
        }
        ExprX::Ctor(_, variant, fields, None) => Value::Enum(
            variant.to_string(),
            fields.first().map(|f| Box::new(evaluate_on_arch(&f.a, variable, word_bits))),
        ),
        ExprX::UnaryOpr(UnaryOpr::IsVariant { variant, .. }, arg) => {
            let Value::Enum(actual, _) = evaluate_on_arch(arg, variable, word_bits) else {
                panic!("expected enum");
            };
            Value::Bool(actual == variant.as_str())
        }
        ExprX::UnaryOpr(UnaryOpr::Field(op), arg) => {
            let Value::Enum(actual, field) = evaluate_on_arch(arg, variable, word_bits) else {
                panic!("expected enum");
            };
            assert_eq!(actual, op.variant.as_str());
            *field.expect("expected payload")
        }
        ExprX::Const(Constant::Bool(b)) => Value::Bool(*b),
        ExprX::Unary(UnaryOp::Not, a) => {
            Value::Bool(!evaluate_on_arch(a, variable, word_bits).boolean())
        }
        ExprX::Const(Constant::Int(n)) => Value::Int(n.to_i128().unwrap()),
        ExprX::Var(v) => variable(v.0.as_str()),
        ExprX::Logical(op, a, b) => {
            let a = evaluate_on_arch(a, variable, word_bits).boolean();
            Value::Bool(match op {
                LogicalOp::And => a && evaluate_on_arch(b, variable, word_bits).boolean(),
                LogicalOp::Or => a || evaluate_on_arch(b, variable, word_bits).boolean(),
                LogicalOp::Implies => !a || evaluate_on_arch(b, variable, word_bits).boolean(),
            })
        }
        ExprX::Binary(op, a, b) => {
            let a = evaluate_on_arch(a, variable, word_bits);
            let b = evaluate_on_arch(b, variable, word_bits);
            match op {
                BinaryOp::Eq(_) => Value::Bool(a == b),
                BinaryOp::Ne => Value::Bool(a != b),
                BinaryOp::Inequality(op) => {
                    let (a, b) = (a.int(), b.int());
                    Value::Bool(match op {
                        InequalityOp::Le => a <= b,
                        InequalityOp::Lt => a < b,
                        InequalityOp::Ge => a >= b,
                        InequalityOp::Gt => a > b,
                    })
                }
                BinaryOp::Arith(op) => {
                    let (a, b) = (a.int(), b.int());
                    Value::Int(match op {
                        ArithOp::Add(_) => a + b,
                        ArithOp::Sub(_) => a - b,
                        ArithOp::Mul(_) => a * b,
                        ArithOp::EuclideanDiv(_) => a.div_euclid(b),
                        ArithOp::EuclideanMod(_) => a.rem_euclid(b),
                        ArithOp::TruncatingDiv(..) => a / b,
                        ArithOp::TruncatingMod(..) => a % b,
                    })
                }
                _ => panic!("unexpected operator"),
            }
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::U(bits), .. }, a) => {
            Value::Int(evaluate_on_arch(a, variable, word_bits).int().rem_euclid(1i128 << bits))
        }
        ExprX::Unary(UnaryOp::Clip { range: IntRange::I(bits), .. }, a) => {
            let modulus = 1i128 << bits;
            let wrapped = evaluate_on_arch(a, variable, word_bits).int().rem_euclid(modulus);
            Value::Int(if wrapped >= modulus / 2 { wrapped - modulus } else { wrapped })
        }
        ExprX::If(c, a, Some(b)) => evaluate_on_arch(
            if evaluate_on_arch(c, variable, word_bits).boolean() { a } else { b },
            variable,
            word_bits,
        ),
        _ => panic!("unexpected formula: {e:?}"),
    }
}

pub(super) fn span() -> Span {
    Span {
        raw_span: Arc::new(()),
        id: 0,
        data: vec![],
        as_string: "exhaustive inference test".into(),
    }
}

pub(crate) fn input(name: &str) -> Expr {
    SpannedTyped::new(
        &span(),
        &Arc::new(TypX::Int(IntRange::U(8))),
        ExprX::Var(VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal)),
    )
}

fn binary(op: BinaryOp, a: Expr, b: Expr) -> Expr {
    let typ = if matches!(op, BinaryOp::Arith(_)) {
        Arc::new(TypX::Int(IntRange::U(8)))
    } else {
        bool_typ()
    };
    SpannedTyped::new(&span(), &typ, ExprX::Binary(op, a, b))
}

fn summarize(body: &Expr) -> Summary {
    let mut locals = HashMap::new();
    for name in ["x", "y"] {
        let expr = input(name);
        let ExprX::Var(v) = &expr.x else { unreachable!() };
        locals.insert(v.clone(), Term { expr: expr.clone(), size: 1 });
    }
    Engine { callees: &HashMap::new(), locals, remaining: FORMULA_BUDGET, exits: None }
        .expr(body)
        .unwrap()
}

#[test]
fn exhaustive_u8_arithmetic_matches_concrete_execution() {
    type Reference = fn(u8, u8) -> Option<u8>;
    let checked = OverflowBehavior::Error(IntRange::U(8));
    let cases: [(ArithOp, Reference); 5] = [
        (ArithOp::Add(checked), u8::checked_add),
        (ArithOp::Sub(checked), u8::checked_sub),
        (ArithOp::Mul(checked), u8::checked_mul),
        (ArithOp::EuclideanDiv(Div0Behavior::Error), u8::checked_div),
        (ArithOp::EuclideanMod(Div0Behavior::Error), u8::checked_rem),
    ];
    for (op, reference) in cases {
        let summary = summarize(&binary(BinaryOp::Arith(op), input("x"), input("y")));
        for x in 0..=u8::MAX {
            for y in 0..=u8::MAX {
                let expected = reference(x, y);
                assert_eq!(
                    evaluate(&summary.safe.expr, x, y).boolean(),
                    expected.is_some(),
                    "{op:?}({x}, {y})"
                );
                if let Some(result) = expected {
                    assert_eq!(
                        evaluate(&summary.value.as_ref().unwrap().expr, x, y),
                        Value::Int(result.into()),
                        "{op:?}({x}, {y})"
                    );
                }
            }
        }
    }
}

#[test]
fn exhaustive_u8_branching_matches_concrete_execution() {
    let cmp = binary(BinaryOp::Inequality(InequalityOp::Ge), input("x"), input("y"));
    let sub = BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8))));
    let body = SpannedTyped::new(
        &span(),
        &input("x").typ,
        ExprX::If(
            cmp,
            binary(sub, input("x"), input("y")),
            Some(binary(sub, input("y"), input("x"))),
        ),
    );
    let summary = summarize(&body);
    for x in 0..=u8::MAX {
        for y in 0..=u8::MAX {
            assert!(evaluate(&summary.safe.expr, x, y).boolean());
            assert_eq!(
                evaluate(&summary.value.as_ref().unwrap().expr, x, y),
                Value::Int(x.abs_diff(y).into())
            );
        }
    }
}

#[test]
fn exhaustive_u8_short_circuit_matches_concrete_execution() {
    let constant = |n| SpannedTyped::new(&span(), &input("x").typ, ExprX::Const(Constant::Int(n)));
    let nonzero = binary(BinaryOp::Ne, input("y"), constant(0.into()));
    let div =
        binary(BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Error)), input("x"), input("y"));
    let cmp = binary(BinaryOp::Inequality(InequalityOp::Gt), div, constant(2.into()));
    let body =
        SpannedTyped::new(&span(), &bool_typ(), ExprX::Logical(LogicalOp::And, nonzero, cmp));
    let summary = summarize(&body);
    for x in 0..=u8::MAX {
        for y in 0..=u8::MAX {
            assert!(evaluate(&summary.safe.expr, x, y).boolean());
            assert_eq!(
                evaluate(&summary.value.as_ref().unwrap().expr, x, y),
                Value::Bool(y != 0 && x / y > 2)
            );
        }
    }
}

pub(crate) fn function(name: &str, body: Expr) -> Function {
    let span = span();
    let ident = |s: &str| VarIdent(Arc::new(s.into()), VarIdentDisambiguate::AirLocal);
    let param = |s: &str| {
        crate::def::Spanned::new(
            span.clone(),
            ParamX {
                name: ident(s),
                typ: input("x").typ.clone(),
                mode: Mode::Exec,
                user_mut: false,
                unwrapped_info: None,
            },
        )
    };
    let ret = param("result");
    let name = Arc::new(FunX {
        path: Arc::new(PathX {
            krate: CrateId::Id(Arc::new("test".into()), 1),
            segments: Arc::new(vec![Arc::new(name.into())]),
        }),
    });
    crate::def::Spanned::new(
        span.clone(),
        FunctionX {
            name,
            proxy: None,
            kind: FunctionKind::Static,
            visibility: Visibility { restricted_to: None },
            body_visibility: BodyVisibility::Uninterpreted,
            opaqueness: Opaqueness::Opaque,
            owning_module: None,
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(vec![param("x"), param("y")]),
            outer_ret: ret.clone(),
            inner_ret: ret,
            require: Arc::new(vec![]),
            ensure: (Arc::new(vec![]), Arc::new(vec![])),
            returns: None,
            decrease: Arc::new(vec![]),
            decrease_when: None,
            decrease_by: None,
            fndef_axioms: None,
            mask_spec: None,
            atomic_update: None,
            unwind_spec: None,
            item_kind: ItemKind::Function,
            attrs: Arc::new(FunctionAttrsX { infer: true, ..FunctionAttrsX::default() }),
            body: Some(body),
            return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![],
            hidden: Arc::new(vec![]),
        },
    )
}

fn call(f: &Function, x: Expr) -> Expr {
    SpannedTyped::new(
        &span(),
        &f.x.inner_ret.x.typ,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                f.x.name.clone(),
                Arc::new(vec![]),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![x, input("y")]),
            post_args: None,
            body: None,
        },
    )
}

#[test]
fn registry_reuses_candidates_and_invalidates_transitive_dependents() {
    let leaf = function("leaf", input("x"));
    let middle = function("middle", call(&leaf, input("x")));
    let top = function("top", call(&middle, input("x")));
    let unrelated = function("unrelated", input("y"));
    let mut registry = registry::Registry::new(None);
    let functions = vec![top.clone(), unrelated.clone(), middle.clone(), leaf.clone()];
    registry.infer(&functions).unwrap();
    assert_eq!((registry.hits, registry.misses), (0, 4));
    registry.infer(&functions).unwrap();
    assert_eq!((registry.hits, registry.misses), (4, 4));
    let changed = function(
        "leaf",
        binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
            input("x"),
            input("y"),
        ),
    );
    registry.infer(&[top, unrelated, middle, changed]).unwrap();
    assert_eq!((registry.hits, registry.misses), (5, 7));
}

#[test]
fn modular_summary_matches_concrete_execution_exhaustively() {
    let leaf = function(
        "leaf",
        binary(
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8)))),
            input("x"),
            input("y"),
        ),
    );
    let top = function("top", call(&leaf, call(&leaf, input("x"))));
    let inferred = registry::Registry::new(None).infer(&[top, leaf]).unwrap();
    let ExprX::Binary(_, _, output) = &inferred[0].x.ensure.0[0].x else { unreachable!() };
    for x in 0..=255u8 {
        for y in 0..=255u8 {
            let expected = x.checked_sub(y).and_then(|n| n.checked_sub(y));
            assert_eq!(evaluate(&inferred[0].x.require[0], x, y).boolean(), expected.is_some());
            if let Some(expected) = expected {
                assert_eq!(evaluate(output, x, y).int(), expected as i128);
            }
        }
    }
}

#[test]
fn imported_summary_changes_invalidate_local_transitive_candidates() {
    let exported = |body| {
        let inferred = infer_function(&function("library", body)).unwrap();
        let mut f = inferred.x.clone();
        f.body = None;
        inferred.new_x(f)
    };
    let original = exported(input("x"));
    let middle = function("middle", call(&original, input("x")));
    let top = function("top", call(&middle, input("x")));
    let functions = [top, middle];
    let mut registry = registry::Registry::new(None);
    registry.infer_with_imports(&functions, &[original.clone()]).unwrap();
    assert_eq!((registry.hits, registry.misses), (0, 2));
    registry.infer_with_imports(&functions, &[original]).unwrap();
    assert_eq!((registry.hits, registry.misses), (2, 2));
    let changed = exported(binary(
        BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
        input("x"),
        input("y"),
    ));
    registry.infer_with_imports(&functions, &[changed]).unwrap();
    assert_eq!((registry.hits, registry.misses), (2, 4));
}

#[test]
fn interval_contracts_cover_every_safe_u8_result() {
    type Reference = fn(u8, u8) -> Option<u8>;
    let checked = OverflowBehavior::Error(IntRange::U(8));
    let cases: [(ArithOp, Reference); 5] = [
        (ArithOp::Add(checked), u8::checked_add),
        (ArithOp::Sub(checked), u8::checked_sub),
        (ArithOp::Mul(checked), u8::checked_mul),
        (ArithOp::EuclideanDiv(Div0Behavior::Error), u8::checked_div),
        (ArithOp::EuclideanMod(Div0Behavior::Error), u8::checked_rem),
    ];
    for (op, concrete) in cases {
        let f = function("interval", binary(BinaryOp::Arith(op), input("x"), input("y")));
        let mut attrs = (*f.x.attrs).clone();
        attrs.infer_allow_approx = true;
        let mut x = f.x.clone();
        x.attrs = Arc::new(attrs);
        let inferred = infer_function(&f.new_x(x)).unwrap();
        let replace_result = HashMap::from([(
            inferred.x.inner_ret.x.name.clone(),
            Term { expr: input("x"), size: 1 },
        )]);
        let post = instantiate(&inferred.x.ensure.0[0], &replace_result).unwrap().expr;
        for x in 0..=255u8 {
            for y in 0..=255u8 {
                let expected = concrete(x, y);
                assert_eq!(evaluate(&inferred.x.require[0], x, y).boolean(), expected.is_some());
                if let Some(result) = expected {
                    assert!(evaluate(&post, result, 0).boolean());
                }
            }
        }
    }
}

#[test]
fn cached_approximations_cannot_be_consumed_as_exact_summaries() {
    let mut leaf = function("interval_leaf", input("x"));
    let mut x = leaf.x.clone();
    Arc::make_mut(&mut x.attrs).infer_allow_approx = true;
    leaf = leaf.new_x(x);
    let mut middle = function("interval_middle", call(&leaf, input("x")));
    let mut x = middle.x.clone();
    Arc::make_mut(&mut x.attrs).infer_allow_approx = true;
    middle = middle.new_x(x);
    let mut registry = registry::Registry::new(None);
    registry.infer(&[middle.clone(), leaf.clone()]).unwrap();
    registry.infer(&[middle.clone(), leaf.clone()]).unwrap();
    assert_eq!((registry.hits, registry.misses), (2, 2));
    let mut x = middle.x.clone();
    Arc::make_mut(&mut x.attrs).infer_allow_approx = false;
    assert!(registry.infer(&[middle.new_x(x), leaf]).is_err());
}

pub(super) fn checked_model(name: &str) -> Function {
    let f = function("model", input("x"));
    let mut x = f.x.clone();
    x.name = Arc::new(FunX {
        path: Arc::new(PathX {
            krate: CrateId::Core,
            segments: Arc::new(vec![
                Arc::new("num".into()),
                Arc::new("impl_test".into()),
                Arc::new(name.into()),
            ]),
        }),
    });
    x.proxy = Some(crate::def::Spanned {
        span: span(),
        x: Arc::new(PathX {
            krate: CrateId::Vstd,
            segments: Arc::new(vec![
                Arc::new("std_specs".into()),
                Arc::new("num".into()),
                Arc::new("u8_specs_tmp".into()),
                Arc::new("_verus_external_fn_specification_test".into()),
            ]),
        }),
    });
    let ret_typ = enums::ValueType::Option(Box::new(enums::ValueType::Unsigned(8))).typ().unwrap();
    let mut ret = x.inner_ret.x.clone();
    ret.typ = ret_typ;
    x.inner_ret = x.inner_ret.new_x(ret);
    x.outer_ret = x.inner_ret.clone();
    Arc::make_mut(&mut x.attrs).infer = false;
    Arc::make_mut(&mut x.attrs).is_external_body = true;
    x.unwind_spec = Some(UnwindSpec::NoUnwind);
    x.body = None;
    f.new_x(x)
}

#[test]
fn exhaustive_checked_u8_enum_summaries_match_rust() {
    type Reference = fn(u8, u8) -> Option<u8>;
    let cases: [(&str, Reference); 5] = [
        ("checked_add", u8::checked_add),
        ("checked_sub", u8::checked_sub),
        ("checked_mul", u8::checked_mul),
        ("checked_div", u8::checked_div),
        ("checked_rem", u8::checked_rem),
    ];
    for (name, concrete) in cases {
        let model = checked_model(name);
        let output = enums::checked_value(
            &span(),
            &model,
            Term { expr: input("x"), size: 1 },
            Term { expr: input("y"), size: 1 },
        )
        .unwrap();
        for x in 0..=255u8 {
            for y in 0..=255u8 {
                let expected = match concrete(x, y) {
                    Some(n) => Value::Enum("Some".into(), Some(Box::new(Value::Int(n.into())))),
                    None => Value::Enum("None".into(), None),
                };
                assert_eq!(evaluate(&output.expr, x, y), expected, "{name}({x}, {y})");
            }
        }
    }
}

#[test]
fn enum_cache_roundtrip_and_invalid_payload_types_are_misses() {
    let model = checked_model("checked_sub");
    let mut f = function("debit", call(&model, input("x")));
    let mut x = f.x.clone();
    x.inner_ret = model.x.inner_ret.clone();
    x.outer_ret = model.x.outer_ret.clone();
    f = f.new_x(x);
    let mut registry = registry::Registry::new(None);
    let original = registry.infer_with_imports(&[f.clone()], &[model.clone()]).unwrap();
    let cached = registry.infer_with_imports(&[f], &[model]).unwrap();
    assert_eq!((registry.hits, registry.misses), (1, 1));
    assert_eq!(format_contract(&original[0]), format_contract(&cached[0]));
    let typ = enums::ValueType::Option(Box::new(enums::ValueType::Unsigned(8))).typ().unwrap();
    assert!(
        enums::ctor(&span(), &typ, &Arc::new("Some".into()), Some(boolean(&span(), true))).is_err()
    );
    assert!(
        enums::ValueType::Option(Box::new(enums::ValueType::Option(Box::new(
            enums::ValueType::Unsigned(8)
        ))))
        .typ()
        .is_none()
    );
}

#[test]
fn imported_summaries_reject_unsupported_operators_and_inconsistent_types() {
    let inferred = infer_function(&function("identity", input("x"))).unwrap();
    let mut x = inferred.x.clone();
    x.body = None;
    let imported = inferred.new_x(x);
    validate_imported_summary(&imported).unwrap();
    let u8_typ = input("x").typ.clone();
    let malformed = [
        ExprX::Unary(UnaryOp::BitNot(None), input("x")),
        ExprX::Unary(UnaryOp::Not, input("x")),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::U(256), truncate: true }, input("x")),
        ExprX::Binary(BinaryOp::BoolAndNoSC, input("x"), input("y")),
        ExprX::Binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
            input("x"),
            input("y"),
        ),
        ExprX::Logical(LogicalOp::And, input("x"), input("y")),
        ExprX::If(input("x"), input("x"), Some(input("y"))),
        ExprX::Const(Constant::Bool(true)),
    ];
    for output in malformed {
        let mut x = imported.x.clone();
        let ExprX::Binary(BinaryOp::Eq(_), ret, _) = &x.ensure.0[0].x else { unreachable!() };
        let output = SpannedTyped::new(&span(), &u8_typ, output);
        x.ensure.0 = Arc::new(vec![SpannedTyped::new(
            &span(),
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), ret.clone(), output),
        )]);
        assert!(validate_imported_summary(&imported.new_x(x)).is_err());
    }
    let mut x = imported.x.clone();
    x.require = Arc::new(vec![SpannedTyped::new(
        &span(),
        &bool_typ(),
        ExprX::Unary(UnaryOp::Not, input("x")),
    )]);
    assert!(validate_imported_summary(&imported.new_x(x)).is_err());
}

#[test]
fn summary_traversal_is_bounded_even_when_every_branch_simplifies() {
    let mut expr = input("x");
    let ExprX::Var(name) = &expr.x else { unreachable!() };
    let vars = HashMap::from([(name.clone(), Term { expr: expr.clone(), size: 1 })]);
    for _ in 0..13 {
        expr = SpannedTyped::new(
            &span(),
            &expr.typ,
            ExprX::If(boolean(&span(), true).expr, expr.clone(), Some(expr.clone())),
        );
    }
    assert!(instantiate(&expr, &vars).is_err());
}

pub(super) fn array_function() -> Function {
    let typ = arrays::typ(input("x").typ.clone(), 2).unwrap();
    let index_type = Arc::new(TypX::Int(IntRange::USize));
    let base = SpannedTyped::new(
        &span(),
        &typ,
        PlaceX::Local(VarIdent(Arc::new("x".into()), VarIdentDisambiguate::AirLocal)),
    );
    let index = SpannedTyped::new(&span(), &index_type, input("y").x.clone());
    let place = SpannedTyped::new(
        &span(),
        &input("x").typ,
        PlaceX::Index(base, index, ArrayKind::Array, BoundsCheck::Error),
    );
    let body = SpannedTyped::new(
        &span(),
        &input("x").typ,
        ExprX::ReadPlace(place, UnfinalizedReadKind { preliminary_kind: ReadKind::Copy, id: 0 }),
    );
    let f = function("array_at", body);
    let mut x = f.x.clone();
    let mut params = (*x.params).clone();
    let mut p = params[0].x.clone();
    p.typ = typ;
    params[0] = params[0].new_x(p);
    let mut p = params[1].x.clone();
    p.typ = index_type;
    params[1] = params[1].new_x(p);
    x.params = Arc::new(params);
    f.new_x(x)
}

#[test]
fn exhaustive_array_bounds_and_overflow_match_concrete_rust() {
    let f = array_function();
    let typ = f.x.params[0].x.typ.clone();
    let literal = arrays::literal(
        &span(),
        &typ,
        vec![Term { expr: input("x"), size: 1 }, Term { expr: input("y"), size: 1 }],
    )
    .unwrap();
    let body = binary(
        BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
        f.x.body.as_ref().unwrap().clone(),
        input("x"),
    );
    for index in 0..4usize {
        let index_value = Term {
            expr: SpannedTyped::new(&span(), &int_typ(), ExprX::Const(Constant::Int(index.into()))),
            size: 1,
        };
        let mut locals = HashMap::from([
            (f.x.params[0].x.name.clone(), literal.clone()),
            (f.x.params[1].x.name.clone(), index_value),
        ]);
        // The first element is also used as a scalar addend; give that read a
        // distinct local name so the array parameter remains an array.
        let addend_name = VarIdent(Arc::new("addend".into()), VarIdentDisambiguate::AirLocal);
        locals.insert(addend_name.clone(), Term { expr: input("x"), size: 1 });
        let ExprX::Binary(op, a, _) = &body.x else { unreachable!() };
        let body = binary(
            *op,
            a.clone(),
            SpannedTyped::new(&span(), &input("x").typ, ExprX::Var(addend_name)),
        );
        let summary =
            Engine { callees: &HashMap::new(), locals, remaining: FORMULA_BUDGET, exits: None }
                .expr(&body)
                .unwrap();
        for x in 0..=255u8 {
            for y in 0..=255u8 {
                let concrete = [x, y].get(index).and_then(|value| value.checked_add(x));
                assert_eq!(evaluate(&summary.safe.expr, x, y), Value::Bool(concrete.is_some()));
                if let Some(expected) = concrete {
                    assert_eq!(
                        evaluate(&summary.value.as_ref().unwrap().expr, x, y),
                        Value::Int(expected.into())
                    );
                }
            }
        }
    }
}

#[test]
fn array_summary_cache_roundtrip_and_literal_type_validation() {
    let f = array_function();
    let mut registry = registry::Registry::new(None);
    let fresh = registry.infer(&[f.clone()]).unwrap();
    let cached = registry.infer(&[f]).unwrap();
    assert_eq!((registry.hits, registry.misses), (1, 1));
    assert_eq!(format_contract(&fresh[0]), format_contract(&cached[0]));
    let typ = arrays::typ(input("x").typ.clone(), 2).unwrap();
    assert!(arrays::literal(&span(), &typ, vec![]).is_err());
    assert!(
        arrays::literal(&span(), &typ, vec![boolean(&span(), true), boolean(&span(), false)])
            .is_err()
    );
    assert!(arrays::typ(input("x").typ.clone(), 65).is_none());
    assert!(arrays::typ(typ, 2).is_none());
}
