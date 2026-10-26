use super::*;
use crate::def::Spanned;
use crate::infer::tests::{Value, evaluate, function, input, span};

fn name(n: &str) -> VarIdent {
    VarIdent(Arc::new(n.into()), VarIdentDisambiguate::AirLocal)
}
fn number(n: u32) -> Expr {
    SpannedTyped::new(&span(), &input("x").typ, ExprX::Const(Constant::Int(n.into())))
}
fn declaration(n: &str, initial: Place) -> Stmt {
    let typ = input("x").typ.clone();
    Spanned::new(
        span(),
        StmtX::Decl {
            pattern: SpannedTyped::new(
                &span(),
                &typ,
                PatternX::Var(PatternBinding {
                    name: name(n),
                    by_ref: ByRef::No,
                    typ: typ.clone(),
                    user_mut: Some(true),
                    copy: true,
                }),
            ),
            mode: Some((Mode::Exec, DeclProph::Default)),
            init: Some(initial),
            els: None,
            assert_irrefutable: false,
        },
    )
}
fn increment(n: &str) -> Stmt {
    let typ = input("x").typ.clone();
    Spanned::new(
        span(),
        StmtX::Expr(SpannedTyped::new(
            &span(),
            &crate::ast_util::unit_typ(),
            ExprX::Assign {
                place: SpannedTyped::new(&span(), &typ, PlaceX::Local(name(n))),
                rhs: number(1),
                op: Some(BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8))))),
                typ,
                resolve: false,
            },
        )),
    )
}

pub(in crate::infer) fn counted_function() -> Function {
    let typ = input("x").typ.clone();
    let unit = crate::ast_util::unit_typ();
    let cond = SpannedTyped::new(
        &span(),
        &bool_typ(),
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), input("i"), input("y")),
    );
    let body = SpannedTyped::new(
        &span(),
        &unit,
        ExprX::Block(Arc::new(vec![increment("r"), increment("i")]), None),
    );
    let loop_expr = SpannedTyped::new(
        &span(),
        &unit,
        ExprX::Loop {
            loop_isolation: true,
            allow_complex_invariants: false,
            is_for_loop: false,
            assume_termination: false,
            label: Label { id: 0, name: None },
            cond: Some(cond),
            body,
            invs: Arc::new(vec![]),
            decrease: Arc::new(vec![]),
            atomic_call: false,
        },
    );
    let body = SpannedTyped::new(
        &span(),
        &typ,
        ExprX::Block(
            Arc::new(vec![
                declaration("r", SpannedTyped::new(&span(), &typ, PlaceX::Local(name("x")))),
                declaration("i", SpannedTyped::new(&span(), &typ, PlaceX::Temporary(number(0)))),
                Spanned::new(span(), StmtX::Expr(loop_expr)),
            ]),
            Some(input("r")),
        ),
    );
    function("counted", body)
}

#[test]
fn all_u8_counted_executions_match_the_exact_domain_and_result() {
    let original = counted_function();
    let inferred = infer_function(&original).unwrap();
    let ExprX::Binary(BinaryOp::Eq(_), _, result) = &inferred.x.ensure.0[0].x else { panic!() };
    for x in 0..=255u8 {
        for n in 0..=255u8 {
            let actual = (0..n).try_fold(x, |value, _| value.checked_add(1));
            assert_eq!(evaluate(&inferred.x.require[0], x, n), Value::Bool(actual.is_some()));
            if let Some(actual) = actual {
                assert_eq!(evaluate(result, x, n), Value::Int(actual.into()));
            }
        }
    }
    let plan = recognize(&original).unwrap().unwrap();
    validate_decoration(&original, inferred.x.body.as_ref().unwrap(), &plan).unwrap();
}

#[test]
fn counted_loop_cache_reconstructs_proofs_and_never_rewrites_executable_steps() {
    let original = counted_function();
    let mut registry = registry::Registry::new(None);
    let fresh = registry.infer(&[original.clone()]).unwrap();
    let cached = registry.infer(&[original.clone()]).unwrap();
    assert_eq!((registry.hits, registry.misses), (1, 1));
    assert_eq!(format_contract(&fresh[0]), format_contract(&cached[0]));
    let plan = recognize(&original).unwrap().unwrap();
    for f in [fresh[0].clone(), cached[0].clone()] {
        validate_decoration(&original, f.x.body.as_ref().unwrap(), &plan).unwrap();
    }
    let changed = crate::ast_visitor::map_expr_visitor(cached[0].x.body.as_ref().unwrap(), &|e| {
        if let ExprX::Assign { place, rhs: _, op, typ, resolve } = &e.x {
            Ok(e.new_x(ExprX::Assign {
                place: place.clone(),
                rhs: number(2),
                op: *op,
                typ: typ.clone(),
                resolve: *resolve,
            }))
        } else {
            Ok(e.clone())
        }
    })
    .unwrap();
    assert!(validate_decoration(&original, &changed, &plan).is_err());
}

#[test]
fn annotations_cannot_smuggle_assumptions_and_source_options_cannot_skip_termination() {
    let original = counted_function();
    let annotated = annotate(&original).unwrap().unwrap();
    let plan = recognize(&original).unwrap().unwrap();
    let changed = crate::ast_visitor::map_expr_visitor(&annotated, &|e| {
        if let ExprX::Loop {
            loop_isolation,
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            invs,
            decrease,
            atomic_call,
        } = &e.x
        {
            let mut invs = invs.as_ref().clone();
            invs[0].inv = SpannedTyped::new(
                &span(),
                &bool_typ(),
                ExprX::AssertAssume {
                    is_assume: true,
                    expr: boolean(&span(), false).expr,
                    msg: None,
                },
            );
            Ok(e.new_x(ExprX::Loop {
                loop_isolation: *loop_isolation,
                allow_complex_invariants: *allow_complex_invariants,
                is_for_loop: *is_for_loop,
                assume_termination: *assume_termination,
                label: label.clone(),
                cond: cond.clone(),
                body: body.clone(),
                invs: Arc::new(invs),
                decrease: decrease.clone(),
                atomic_call: *atomic_call,
            }))
        } else {
            Ok(e.clone())
        }
    })
    .unwrap();
    assert!(validate_decoration(&original, &changed, &plan).is_err());
    for assume in [true, false] {
        let mut f = original.x.clone();
        let attrs = Arc::make_mut(&mut f.attrs);
        if assume {
            attrs.exec_assume_termination = true;
        } else {
            attrs.exec_allows_no_decreases_clause = true;
        }
        assert!(infer_function(&original.new_x(f)).is_err());
    }
    let mut f = original.x.clone();
    f.body = Some(annotated);
    assert!(
        infer_function(&original.new_x(f)).is_err(),
        "source loop annotations must be rejected"
    );
}

#[test]
fn unsupported_update_cannot_enter_generation_or_reuse_a_cached_recipe() {
    let original = counted_function();
    let changed = crate::ast_visitor::map_expr_visitor(original.x.body.as_ref().unwrap(), &|e| {
        if let ExprX::Assign { place, rhs: _, op, typ, resolve } = &e.x {
            Ok(e.new_x(ExprX::Assign {
                place: place.clone(),
                rhs: number(2),
                op: *op,
                typ: typ.clone(),
                resolve: *resolve,
            }))
        } else {
            Ok(e.clone())
        }
    })
    .unwrap();
    let mut f = original.x.clone();
    f.body = Some(changed);
    let changed = original.new_x(f);
    assert!(recognize(&changed).is_err());
    let mut registry = registry::Registry::new(None);
    registry.infer(&[original]).unwrap();
    assert!(registry.infer(&[changed]).is_err());
    assert_eq!(registry.hits, 0);
}

/// Build typed source independently of the production recognizer and formulas.
fn family_function(subtract: bool, countdown: bool, step: Option<u8>) -> Function {
    let base = counted_function();
    let typ = input("x").typ.clone();
    let mut f = base.x.clone();
    if step.is_none() {
        let mut p = f.params[0].x.clone();
        p.name = name("step");
        let param = f.params[0].new_x(p);
        Arc::make_mut(&mut f.params).push(param);
    }
    let step_expr = step.map(|n| number(n.into())).unwrap_or_else(|| input("step"));
    let mut counter_init = SpannedTyped::new(&span(), &typ, PlaceX::Temporary(number(0)));
    if countdown {
        counter_init = SpannedTyped::new(&span(), &typ, PlaceX::Local(name("y")));
    }
    let original = f.body.as_ref().unwrap();
    f.body = Some(
        crate::ast_visitor::map_expr_visitor(original, &|e| {
            let changed = match &e.x {
                ExprX::Assign { place, rhs: _, op: _, typ: assigned, resolve } => {
                    let is_counter = local(place) == Some(&name("i"));
                    let sub = if is_counter { countdown } else { subtract };
                    ExprX::Assign {
                        place: place.clone(),
                        rhs: if is_counter { number(1) } else { step_expr.clone() },
                        op: Some(BinaryOp::Arith(if sub {
                            ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8)))
                        } else {
                            ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))
                        })),
                        typ: assigned.clone(),
                        resolve: *resolve,
                    }
                }
                ExprX::Loop {
                    loop_isolation,
                    allow_complex_invariants,
                    is_for_loop,
                    assume_termination,
                    label,
                    cond,
                    body,
                    invs,
                    decrease,
                    atomic_call,
                } => ExprX::Loop {
                    loop_isolation: *loop_isolation,
                    allow_complex_invariants: *allow_complex_invariants,
                    is_for_loop: *is_for_loop,
                    assume_termination: *assume_termination,
                    label: label.clone(),
                    cond: if countdown {
                        Some(SpannedTyped::new(
                            &span(),
                            &bool_typ(),
                            ExprX::Binary(
                                BinaryOp::Inequality(InequalityOp::Gt),
                                input("i"),
                                number(0),
                            ),
                        ))
                    } else {
                        cond.clone()
                    },
                    body: body.clone(),
                    invs: invs.clone(),
                    decrease: decrease.clone(),
                    atomic_call: *atomic_call,
                },
                ExprX::Block(stmts, tail) if stmts.len() == 3 => {
                    let mut stmts = (**stmts).clone();
                    stmts[1] = declaration("i", counter_init.clone());
                    ExprX::Block(Arc::new(stmts), tail.clone())
                }
                _ => return Ok(e.clone()),
            };
            Ok(e.new_x(changed))
        })
        .unwrap(),
    );
    base.new_x(f)
}

#[test]
fn unsigned_step_families_match_concrete_checked_execution() {
    use crate::infer::tests::evaluate_with;
    for subtract in [false, true] {
        for countdown in [false, true] {
            for source_step in [None, Some(0), Some(2), Some(255)] {
                let original = family_function(subtract, countdown, source_step);
                let inferred = infer_function(&original).unwrap();
                assert_eq!(inferred.x.attrs.nonlinear, source_step.is_none());
                let ExprX::Binary(BinaryOp::Eq(_), _, output) = &inferred.x.ensure.0[0].x else {
                    panic!("contract")
                };
                let steps = source_step
                    .map(|s| vec![s])
                    .unwrap_or_else(|| vec![0, 1, 2, 3, 7, 127, 128, 255]);
                for step in steps {
                    for x in 0..=255u8 {
                        for n in 0..=255u8 {
                            let mut actual = Some(x);
                            let mut counter = if countdown { n } else { 0 };
                            while if countdown { counter > 0 } else { counter < n } {
                                actual = actual.and_then(|v| {
                                    if subtract { v.checked_sub(step) } else { v.checked_add(step) }
                                });
                                if actual.is_none() {
                                    break;
                                }
                                counter = if countdown {
                                    counter.checked_sub(1).unwrap()
                                } else {
                                    counter.checked_add(1).unwrap()
                                };
                            }
                            let vars = |v: &str| {
                                Value::Int(match v {
                                    "x" => x.into(),
                                    "y" => n.into(),
                                    "step" => step.into(),
                                    _ => panic!("variable"),
                                })
                            };
                            assert_eq!(
                                evaluate_with(&inferred.x.require[0], &vars),
                                Value::Bool(actual.is_some()),
                                "{subtract} {countdown} {x} {n} {step}"
                            );
                            if let Some(actual) = actual {
                                assert_eq!(evaluate_with(output, &vars), Value::Int(actual.into()));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn step_family_cache_regenerates_checked_annotations_and_prover_choice() {
    for subtract in [false, true] {
        for countdown in [false, true] {
            let original = family_function(subtract, countdown, None);
            let mut registry = registry::Registry::new(None);
            for hit in [false, true] {
                let inferred = registry.infer(&[original.clone()]).unwrap();
                let f = &inferred[0];
                let plan = recognize(&original).unwrap().unwrap();
                validate_decoration(&original, f.x.body.as_ref().unwrap(), &plan).unwrap();
                assert!(f.x.attrs.nonlinear);
                assert_eq!(registry.hits, if hit { 1 } else { 0 });
                let report = format_loop_proofs(f);
                assert_eq!(report[0]["arithmetic_prover"], "nonlinear");
                assert_eq!(report[0]["decreases"][0], if countdown { "i" } else { "(y - i)" });
            }
        }
    }
}

#[test]
fn step_family_direction_and_operand_guards_apply_before_cache_reuse() {
    for countdown in [false, true] {
        for fault in ["direction", "counter_step", "dynamic_step"] {
            let original = family_function(false, countdown, None);
            let mut f = original.x.clone();
            f.body = Some(
                crate::ast_visitor::map_expr_visitor(f.body.as_ref().unwrap(), &|e| {
                    let ExprX::Assign { place, rhs, op, typ, resolve } = &e.x else {
                        return Ok(e.clone());
                    };
                    let is_counter = local(place) == Some(&name("i"));
                    let (rhs, op) = match fault {
                        "direction" if is_counter => (
                            rhs.clone(),
                            Some(BinaryOp::Arith(if countdown {
                                ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))
                            } else {
                                ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8)))
                            })),
                        ),
                        "counter_step" if is_counter => (number(2), *op),
                        "dynamic_step" if !is_counter => (input("i"), *op),
                        _ => return Ok(e.clone()),
                    };
                    Ok(e.new_x(ExprX::Assign {
                        place: place.clone(),
                        rhs,
                        op,
                        typ: typ.clone(),
                        resolve: *resolve,
                    }))
                })
                .unwrap(),
            );
            let changed = original.new_x(f);
            assert!(recognize(&changed).is_err(), "{countdown} {fault}");
            let mut registry = registry::Registry::new(None);
            registry.infer(&[original]).unwrap();
            assert!(registry.infer(&[changed]).is_err());
            assert_eq!(registry.hits, 0);
        }
    }
}

#[test]
fn step_family_installation_changes_only_contract_and_documented_proof_fields() {
    for subtract in [false, true] {
        for countdown in [false, true] {
            let original = family_function(subtract, countdown, None);
            let installed = infer_function(&original).unwrap();
            let mut attrs = (*installed.x.attrs).clone();
            attrs.spinoff_prover = original.x.attrs.spinoff_prover;
            attrs.infer_candidate_id = original.x.attrs.infer_candidate_id.clone();
            attrs.nonlinear = original.x.attrs.nonlinear;
            assert_eq!(format!("{attrs:?}"), format!("{:?}", original.x.attrs));
            let mut restored = installed.x.clone();
            restored.attrs = original.x.attrs.clone();
            restored.require = original.x.require.clone();
            restored.ensure = original.x.ensure.clone();
            restored.body = original.x.body.clone();
            assert_eq!(format!("{restored:?}"), format!("{:?}", original.x));
            let plan = recognize(&original).unwrap().unwrap();
            let changed =
                crate::ast_visitor::map_expr_visitor(installed.x.body.as_ref().unwrap(), &|e| {
                    if read(e) == Some(&name("step")) { Ok(number(2)) } else { Ok(e.clone()) }
                })
                .unwrap();
            assert!(validate_decoration(&original, &changed, &plan).is_err());
        }
    }
}
