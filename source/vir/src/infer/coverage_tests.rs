//! Independent typed-source construction and concrete checked reference execution.
use super::*;
use crate::def::Spanned;
use crate::infer::tests::{Value, evaluate_with, function, input, span};

fn name(s: &str) -> VarIdent {
    VarIdent(Arc::new(s.into()), VarIdentDisambiguate::AirLocal)
}
fn read(s: &str, typ: &Typ) -> Expr {
    SpannedTyped::new(&span(), typ, ExprX::Var(name(s)))
}
fn number(n: u32) -> Expr {
    SpannedTyped::new(&span(), &input("x").typ, ExprX::Const(Constant::Int(n.into())))
}
fn binary(op: BinaryOp, a: Expr, b: Expr) -> Expr {
    let typ = if matches!(op, BinaryOp::Arith(_)) { input("x").typ.clone() } else { bool_typ() };
    SpannedTyped::new(&span(), &typ, ExprX::Binary(op, a, b))
}
fn binding(n: &str, typ: &Typ) -> Pattern {
    SpannedTyped::new(
        &span(),
        typ,
        PatternX::Var(PatternBinding {
            name: name(n),
            by_ref: ByRef::No,
            typ: typ.clone(),
            user_mut: Some(false),
            copy: true,
        }),
    )
}
fn pattern(typ: &Typ, variant: &str, payload: Option<Pattern>) -> Pattern {
    let TypX::Datatype(dt, ..) = &**typ else { panic!() };
    SpannedTyped::new(
        &span(),
        typ,
        PatternX::Constructor(
            dt.clone(),
            Arc::new(variant.into()),
            Arc::new(
                payload
                    .into_iter()
                    .map(|p| {
                        Arc::new(air::ast::BinderX {
                            name: crate::def::positional_field_ident(0),
                            a: p,
                        })
                    })
                    .collect(),
            ),
        ),
    )
}
fn typed(params: Vec<(&str, Typ)>, body: Expr) -> Function {
    let base = function("coverage", body.clone());
    let mut f = base.x.clone();
    f.params = Arc::new(
        params
            .into_iter()
            .map(|(n, typ)| {
                let mut p = f.params[0].x.clone();
                p.name = name(n);
                p.typ = typ;
                f.params[0].new_x(p)
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
fn let_else_array_outputs_match_checked_execution_and_regenerate_from_cache() {
    let scalar = input("x").typ.clone();
    let option = enums::ValueType::Option(Box::new(enums::ValueType::Unsigned(8))).typ().unwrap();
    let array = arrays::typ(scalar.clone(), 2).unwrap();
    let literal =
        |xs: Vec<Expr>| SpannedTyped::new(&span(), &array, ExprX::ArrayLiteral(Arc::new(xs)));
    let result = literal(vec![
        binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
            read("v", &scalar),
            number(1),
        ),
        read("y", &scalar),
    ]);
    let never = Arc::new(TypX::Decorate(TypDecoration::Never, None, crate::ast_util::unit_typ()));
    let returned = SpannedTyped::new(
        &span(),
        &never,
        ExprX::Return(Some(literal(vec![
            binary(
                BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
                read("y", &scalar),
                number(1),
            ),
            number(0),
        ]))),
    );
    let els = SpannedTyped::new(&span(), &option, ExprX::NeverToAny(returned));
    let decl = Spanned::new(
        span(),
        StmtX::Decl {
            pattern: pattern(&option, "Some", Some(binding("v", &scalar))),
            mode: Some((Mode::Exec, DeclProph::Default)),
            init: Some(SpannedTyped::new(&span(), &option, PlaceX::Local(name("x")))),
            els: Some(els),
            assert_irrefutable: false,
        },
    );
    let body = SpannedTyped::new(&span(), &array, ExprX::Block(Arc::new(vec![decl]), Some(result)));
    let f = typed(vec![("x", option), ("y", scalar)], body);
    let inferred = infer_function(&f).unwrap();
    let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else { panic!() };
    for some in [false, true] {
        for x in 0..=255u8 {
            for y in [0, 1, 254, 255u8] {
                let vars = |n: &str| match n {
                    "x" => Value::Enum(
                        if some { "Some" } else { "None" }.into(),
                        some.then(|| Box::new(Value::Int(x.into()))),
                    ),
                    "y" => Value::Int(y.into()),
                    _ => panic!(),
                };
                let expected = if some {
                    x.checked_add(1).map(|v| vec![v, y])
                } else {
                    y.checked_add(1).map(|v| vec![v, 0])
                };
                assert_eq!(
                    evaluate_with(&inferred.x.require[0], &vars),
                    Value::Bool(expected.is_some())
                );
                if let Some(xs) = expected {
                    assert_eq!(
                        evaluate_with(output, &vars),
                        Value::Array(xs.into_iter().map(|v| Value::Int(v.into())).collect())
                    );
                }
            }
        }
    }
    let mut registry = registry::Registry::new(None);
    for _ in 0..2 {
        let fs = registry.infer(&[f.clone()]).unwrap();
        assert_eq!(format_contract(&fs[0]), format_contract(&inferred));
    }
    assert_eq!((registry.hits, registry.misses), (1, 1));
}

#[test]
fn guards_and_or_bindings_match_concrete_rust_with_first_arm_priority() {
    let scalar = input("x").typ.clone();
    let result = enums::ValueType::Result(
        Box::new(enums::ValueType::Unsigned(8)),
        Box::new(enums::ValueType::Unsigned(8)),
    )
    .typ()
    .unwrap();
    let make = |or: bool| {
        let ok = pattern(&result, "Ok", Some(binding("v", &scalar)));
        let pat = if or {
            SpannedTyped::new(
                &span(),
                &result,
                PatternX::Or(ok, pattern(&result, "Err", Some(binding("v", &scalar)))),
            )
        } else {
            ok
        };
        let guard = if or {
            boolean(&span(), true).expr
        } else {
            binary(BinaryOp::Inequality(InequalityOp::Gt), read("v", &scalar), read("y", &scalar))
        };
        let add = binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
            read("v", &scalar),
            read("y", &scalar),
        );
        let arm = |pattern, guard, body| Spanned::new(span(), ArmX { pattern, guard, body });
        let arms = Arc::new(vec![
            arm(pat, guard, add),
            arm(
                SpannedTyped::new(&span(), &result, PatternX::Wildcard),
                boolean(&span(), true).expr,
                read("y", &scalar),
            ),
        ]);
        typed(
            vec![("x", result.clone()), ("y", scalar.clone())],
            SpannedTyped::new(
                &span(),
                &scalar,
                ExprX::Match(
                    SpannedTyped::new(&span(), &result, PlaceX::Local(name("x"))),
                    arms,
                    false,
                ),
            ),
        )
    };
    for or in [false, true] {
        let f = make(or);
        let inferred = infer_function(&f).unwrap();
        let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else { panic!() };
        for ok in [false, true] {
            for x in 0..=255u8 {
                for y in 0..=255u8 {
                    let vars = |n: &str| match n {
                        "x" => Value::Enum(
                            if ok { "Ok" } else { "Err" }.into(),
                            Some(Box::new(Value::Int(x.into()))),
                        ),
                        "y" => Value::Int(y.into()),
                        _ => panic!(),
                    };
                    let actual = if or || ok && x > y { x.checked_add(y) } else { Some(y) };
                    assert_eq!(
                        evaluate_with(&inferred.x.require[0], &vars),
                        Value::Bool(actual.is_some()),
                        "{or} {ok} {x} {y}"
                    );
                    if let Some(n) = actual {
                        assert_eq!(evaluate_with(output, &vars), Value::Int(n.into()));
                    }
                }
            }
        }
    }
}

#[test]
fn nested_local_views_match_concrete_range_and_index_checks_without_leaking_views() {
    let element = input("x").typ.clone();
    let slice = Arc::new(TypX::Decorate(
        TypDecoration::Ref,
        None,
        Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![element.clone()]))),
    ));
    let index = Arc::new(TypX::Int(IntRange::USize));
    let funcs = HashMap::new();
    let mut engine = Engine {
        callees: &funcs,
        locals: [
            ("x", slice.clone()),
            ("a", index.clone()),
            ("b", index.clone()),
            ("i", index.clone()),
        ]
        .into_iter()
        .map(|(n, t)| (name(n), Term { view: None, expr: read(n, &t), size: 1 }))
        .collect(),
        remaining: FORMULA_BUDGET,
        exits: None,
    };
    let expr = read("x", &slice);
    let view = engine
        .subslice(
            &expr,
            slices::RangeCall {
                helper: expr.clone(),
                base: expr.clone(),
                fields: vec![
                    ("start".into(), read("a", &index)),
                    ("end".into(), read("b", &index)),
                ],
            },
        )
        .unwrap();
    let parent_safe = view.safe.clone();
    let offset = view.value.clone().unwrap();
    assert!(offset.view.is_some());
    let length = slices::length(&span(), offset.clone()).unwrap();
    let summary = engine
        .index_summary(
            &span(),
            view,
            Summary {
                safe: boolean(&span(), true),
                value: Some(Term { view: None, expr: read("i", &index), size: 1 }),
            },
        )
        .unwrap();
    assert!(summary.value.as_ref().unwrap().view.is_none());
    for len in 0..=4 {
        let xs: Vec<_> = (0..len).map(|n| n as u8 * 63).collect();
        for a in 0..=5 {
            for b in 0..=5 {
                for i in 0..=5 {
                    let vars = |n: &str| match n {
                        "x" => Value::Array(xs.iter().map(|v| Value::Int((*v).into())).collect()),
                        "a" => Value::Int(a as i128),
                        "b" => Value::Int(b as i128),
                        "i" => Value::Int(i as i128),
                        _ => panic!(),
                    };
                    let actual = xs.get(a..b).and_then(|s| s.get(i)).copied();
                    assert_eq!(
                        evaluate_with(&summary.safe.expr, &vars),
                        Value::Bool(actual.is_some())
                    );
                    if let Some(v) = actual {
                        assert_eq!(
                            evaluate_with(&summary.value.as_ref().unwrap().expr, &vars),
                            Value::Int(v.into())
                        );
                        assert_eq!(evaluate_with(&length.expr, &vars), Value::Int((b - a) as i128));
                    }
                }
            }
        }
    }
    engine.locals.insert(name("view"), offset);
    let nested = engine
        .subslice(
            &expr,
            slices::RangeCall {
                helper: expr.clone(),
                base: read("view", &slice),
                fields: vec![("start".into(), number(1))],
            },
        )
        .unwrap();
    let nested = engine
        .index_summary(
            &span(),
            nested,
            Summary {
                safe: boolean(&span(), true),
                value: Some(Term { view: None, expr: number(0), size: 1 }),
            },
        )
        .unwrap();
    let safe = logical(&span(), LogicalOp::And, parent_safe, nested.safe).unwrap();
    let output = nested.value.unwrap();
    assert!(output.view.is_none());
    for len in 0..=4 {
        let xs: Vec<_> = (0..len).map(|n| n as u8 * 63).collect();
        for a in 0..=5 {
            for b in 0..=5 {
                let vars = |n: &str| match n {
                    "x" => Value::Array(xs.iter().map(|v| Value::Int((*v).into())).collect()),
                    "a" => Value::Int(a as i128),
                    "b" => Value::Int(b as i128),
                    _ => panic!(),
                };
                let actual = xs.get(a..b).and_then(|s| s.get(1..)).and_then(|s| s.first()).copied();
                assert_eq!(evaluate_with(&safe.expr, &vars), Value::Bool(actual.is_some()));
                if let Some(v) = actual {
                    assert_eq!(evaluate_with(&output.expr, &vars), Value::Int(v.into()));
                }
            }
        }
    }
    assert!(
        summary::binary(
            &span(),
            BinaryOp::Eq(Mode::Spec),
            engine.locals[&name("view")].clone(),
            engine.locals[&name("view")].clone()
        )
        .is_err()
    );
}

#[test]
fn subslice_model_rejects_forged_declaration_shapes() {
    let path = |krate, parts: &[&str]| {
        Arc::new(PathX {
            krate,
            segments: Arc::new(parts.iter().map(|s| Arc::new((*s).into())).collect()),
        })
    };
    let reference = |typ| Arc::new(TypX::Decorate(TypDecoration::Ref, None, typ));
    let base = function("range_model", input("x"));
    let mut f = base.x.clone();
    f.name =
        Arc::new(FunX { path: path(CrateId::Core, &["slice", "index", "impl_test", "index"]) });
    f.proxy = Some(crate::def::Spanned {
        span: span(),
        x: path(
            CrateId::Vstd,
            &["std_specs", "slice", "_verus_external_fn_specification_test_index"],
        ),
    });
    f.attrs = Arc::new(FunctionAttrsX::default());
    f.body = None;
    f.typ_params = Arc::new(vec![Arc::new("T".into()), Arc::new("I".into())]);
    let ts: Vec<_> = f.typ_params.iter().map(|s| Arc::new(TypX::TypParam(s.clone()))).collect();
    let slice = Arc::new(TypX::Primitive(Primitive::Slice, Arc::new(vec![ts[0].clone()])));
    f.typ_bounds = Arc::new(vec![
        Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![ts[0].clone()]),
        )),
        Arc::new(GenericBoundX::Trait(
            TraitId::Sizedness(Sizedness::Sized),
            Arc::new(vec![ts[1].clone()]),
        )),
        Arc::new(GenericBoundX::Trait(
            TraitId::Path(path(CrateId::Core, &["slice", "index", "SliceIndex"])),
            Arc::new(vec![ts[1].clone(), slice.clone()]),
        )),
    ]);
    f.params = Arc::new(
        f.params
            .iter()
            .zip([reference(slice.clone()), ts[1].clone()])
            .map(|(p, typ)| {
                let mut x = p.x.clone();
                x.typ = typ;
                p.new_x(x)
            })
            .collect(),
    );
    let mut ret = f.inner_ret.x.clone();
    ret.typ = reference(Arc::new(TypX::Projection {
        trait_typ_args: Arc::new(vec![ts[1].clone(), slice]),
        trait_path: path(CrateId::Core, &["slice", "index", "SliceIndex"]),
        name: Arc::new("Output".into()),
    }));
    f.inner_ret = f.inner_ret.new_x(ret);
    f.outer_ret = f.inner_ret.clone();
    let model = base.new_x(f);
    assert!(slices::range_model(&model));
    for fault in ["proxy", "bounds", "return", "mutable", "mode", "body", "inferred", "type_params"]
    {
        let mut f = model.x.clone();
        match fault {
            "proxy" => {
                f.proxy = Some(crate::def::Spanned {
                    span: span(),
                    x: path(CrateId::Vstd, &["not_slice", "index"]),
                })
            }
            "bounds" => f.typ_bounds = Arc::new(vec![]),
            "return" => {
                let mut ret = f.inner_ret.x.clone();
                ret.typ = input("x").typ.clone();
                f.inner_ret = f.inner_ret.new_x(ret);
            }
            "mutable" => {
                let p = &mut Arc::make_mut(&mut f.params)[0];
                let mut x = p.x.clone();
                x.user_mut = true;
                *p = p.new_x(x);
            }
            "mode" => f.mode = Mode::Spec,
            "body" => f.body = Some(input("x")),
            "inferred" => Arc::make_mut(&mut f.attrs).infer = true,
            _ => f.typ_params = Arc::new(vec![]),
        }
        assert!(!slices::range_model(&model.new_x(f)), "{fault}");
    }
}
