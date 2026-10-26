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
