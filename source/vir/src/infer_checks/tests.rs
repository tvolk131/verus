//! Fault injection at the candidate boundary, using the production SST-to-AIR
//! translator and Z3. None of these candidates passes through the synthesizer.
use super::*;
use crate::ast::*;
use crate::context::{Ctx, FunctionCtx, GlobalCtx, WarningCtx};
use crate::messages::{VirMessageInterface, error};
use air::context::{SmtSolver, ValidityResult};
use air::messages::{ArcDynMessage, Diagnostics, MessageLevel};
use std::collections::HashSet;
use std::sync::Mutex;

struct Silent;
impl Diagnostics for Silent {
    fn report(&self, _: &ArcDynMessage) {}
    fn report_now(&self, _: &ArcDynMessage) {}
    fn report_as(&self, _: &ArcDynMessage, _: MessageLevel) {}
    fn report_as_now(&self, _: &ArcDynMessage, _: MessageLevel) {}
}

fn span() -> Span {
    Span {
        raw_span: Arc::new(()),
        id: 0,
        data: vec![],
        as_string: "inference fault-injection test".into(),
    }
}

fn var(name: &str) -> VarIdent {
    VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal)
}

fn integer(n: u32) -> Exp {
    SpannedTyped::new(&span(), &crate::ast_util::int_typ(), ExpX::Const(Constant::Int(n.into())))
}

fn binary(op: crate::sst::BinaryOp, a: Exp, b: Exp) -> Exp {
    let typ = if matches!(op, crate::sst::BinaryOp::Arith(_)) {
        crate::ast_util::int_typ()
    } else {
        bool_typ()
    };
    SpannedTyped::new(&span(), &typ, ExpX::Binary(op, a, b))
}

fn fixture() -> (Ctx, Pars, FuncCheckSst) {
    let span = span();
    let krate_id = CrateId::Id(Arc::new("infer_test".into()), 1);
    let module_path = Arc::new(PathX { krate: krate_id.clone(), segments: Arc::new(vec![]) });
    let module = Spanned::new(span.clone(), ModuleX { path: module_path.clone(), reveals: None });
    let krate = Arc::new(KrateX {
        functions: vec![],
        reveal_groups: vec![],
        datatypes: vec![],
        traits: vec![],
        trait_impls: vec![],
        assoc_type_impls: vec![],
        modules: vec![module.clone()],
        external_fns: vec![],
        external_types: vec![],
        path_as_rust_names: vec![],
        opaque_types: vec![],
        arch: Arch { word_bits: ArchWordBits::Exactly(64) },
    });
    let global = GlobalCtx::new(
        &krate,
        krate_id.clone(),
        span.clone(),
        10.0,
        Arc::new(Mutex::new(None)),
        Arc::new(Mutex::new(None)),
        Arc::new(WarningCtx { fun_warn_configs: HashMap::new() }),
        SmtSolver::Z3,
        true,
        false,
        false,
        false,
        false,
    )
    .unwrap();
    let mut ctx = Ctx::new(
        &krate,
        global,
        module,
        vec![],
        vec![],
        HashSet::new(),
        crate::prune::UsedBuiltins {
            uses_array: false,
            uses_bytestr: false,
            uses_pointee_metadata: false,
            uses_ieee_float: false,
        },
        vec![],
        vec![],
    )
    .unwrap();
    ctx.fun = Some(FunctionCtx {
        checking_spec_preconditions: false,
        checking_spec_preconditions_for_non_spec: false,
        checking_spec_decreases: false,
        module_for_chosen_triggers: Some(module_path),
        current_fun: Arc::new(FunX {
            path: Arc::new(PathX {
                krate: krate_id,
                segments: Arc::new(vec![Arc::new("increment".into())]),
            }),
        }),
        current_fun_attrs: Arc::new(FunctionAttrsX::default()),
    });
    let u8_typ = Arc::new(TypX::Int(IntRange::U(8)));
    let x = SpannedTyped::new(&span, &u8_typ, ExpX::Var(var("x")));
    let result = SpannedTyped::new(&span, &u8_typ, ExpX::Var(var("result")));
    let sum = binary(crate::sst::BinaryOp::Arith(crate::sst::ArithOp::Add), x.clone(), integer(1));
    let fits =
        binary(crate::sst::BinaryOp::Inequality(InequalityOp::Le), sum.clone(), integer(255));
    let body = Spanned::new(
        span.clone(),
        StmX::Block(Arc::new(vec![
            Spanned::new(
                span.clone(),
                StmX::Assert(None, Some(error(&span, "overflow")), fits.clone()),
            ),
            Spanned::new(span.clone(), StmX::Assume(fits)),
            Spanned::new(
                span.clone(),
                StmX::Return {
                    assert_id: None,
                    base_error: error(&span, "postcondition"),
                    ret_exp: Some(sum.clone()),
                    inside_body: false,
                },
            ),
        ])),
    );
    let check = FuncCheckSst {
        reqs: Arc::new(vec![binary(
            crate::sst::BinaryOp::Inequality(InequalityOp::Le),
            x,
            integer(254),
        )]),
        return_opaque_type_eqs: Arc::new(vec![]),
        post_condition: Arc::new(PostConditionSst {
            dest: Some(var("result")),
            ens_exps: Arc::new(vec![binary(crate::sst::BinaryOp::Eq, result, sum)]),
            ens_spec_precondition_stms: Arc::new(vec![]),
            kind: PostConditionKind::Ensures,
        }),
        unwind: UnwindSst::MayUnwind,
        body,
        local_decls: Arc::new(vec![
            Arc::new(LocalDeclX {
                ident: var("x"),
                typ: u8_typ.clone(),
                kind: LocalDeclKind::Param { mutable: false },
            }),
            Arc::new(LocalDeclX {
                ident: var("result"),
                typ: u8_typ.clone(),
                kind: LocalDeclKind::Return,
            }),
        ]),
        local_decls_decreases_init: Arc::new(vec![]),
        statics: Arc::new(vec![]),
    };
    let pars =
        Arc::new(vec![Spanned::new(span, ParX { name: var("x"), typ: u8_typ, mode: Mode::Exec })]);
    (ctx, pars, check)
}

fn verifies(ctx: &Ctx, pars: &Pars, check: &FuncCheckSst) -> bool {
    verifies_using(ctx, pars, check, false)
}
fn verifies_using(ctx: &Ctx, pars: &Pars, check: &FuncCheckSst, nonlinear: bool) -> bool {
    let interface = Arc::new(VirMessageInterface {});
    let mut solver = air::context::Context::new(interface.clone(), SmtSolver::Z3);
    solver.set_solver_option("air_recommended_options", "true");
    if nonlinear {
        solver.set_solver_option("smt.arith.solver", "6");
    }
    solver.set_rlimit(3_000_000);
    for cmd in ctx
        .prelude(crate::prelude::PreludeConfig {
            arch_word_bits: ArchWordBits::Exactly(64),
            smt_encoding: SmtSolver::Z3.default_encoding(),
        })
        .iter()
        .chain(ctx.fuel().iter())
    {
        assert!(matches!(
            solver.command(&*interface, &Silent, cmd, Default::default()),
            ValidityResult::Valid(_)
        ));
    }
    let commands = crate::sst_to_air::body_stm_to_air(
        ctx,
        &span(),
        &Arc::new(vec![]),
        &Arc::new(vec![]),
        pars,
        check,
        &vec![],
        false,
        false,
        nonlinear,
    )
    .unwrap();
    let mut valid = true;
    for commands in commands {
        for command in commands.commands.iter() {
            match solver.command(&*interface, &Silent, command, Default::default()) {
                ValidityResult::Valid(_) => {}
                ValidityResult::Invalid(..) => valid = false,
                result => panic!("unexpected backend result: {result:?}"),
            }
            if matches!(**command, air::ast::CommandX::CheckValid(_)) {
                solver.finish_query();
            }
        }
    }
    valid
}

#[test]
fn correct_candidate_passes_all_checks() {
    let (ctx, pars, check) = fixture();
    assert!(verifies(&ctx, &pars, &check));
    for kind in [PrecisionCheck::Maximality, PrecisionCheck::Exactness] {
        let precision = precision_check_body(&span(), &pars[0].x.typ, &check, kind).unwrap();
        assert!(verifies(&ctx, &pars, &precision));
    }
}

fn counted_loop_fixture() -> (Ctx, Pars, FuncCheckSst) {
    counted_family_fixture(false, false, Some(1))
}
fn counted_family_fixture(
    subtract: bool,
    countdown: bool,
    step: Option<u32>,
) -> (Ctx, Pars, FuncCheckSst) {
    use crate::sst::{ArithOp as A, BinaryOp as B};
    let (ctx, _, mut check) = fixture();
    let typ = Arc::new(TypX::Int(IntRange::U(8)));
    let read = |n: &str| SpannedTyped::new(&span(), &typ, ExpX::Var(var(n)));
    let assignment = |n: &str, rhs: Exp, is_init| {
        Spanned::new(
            span(),
            StmX::Assign {
                lhs: Dest { dest: SpannedTyped::new(&span(), &typ, ExpX::VarLoc(var(n))), is_init },
                rhs,
            },
        )
    };
    let x = read("x");
    let n = read("n");
    let r = read("r");
    let i = read("i");
    let stride = step.map(integer).unwrap_or_else(|| read("step"));
    let product = |count: Exp| {
        if step == Some(1) { count } else { binary(B::Arith(A::Mul), count, stride.clone()) }
    };
    let amount = product(n.clone());
    let sum = binary(B::Arith(if subtract { A::Sub } else { A::Add }), x.clone(), amount.clone());
    let fits = |e| binary(B::Inequality(InequalityOp::Le), e, integer(255));
    let measure =
        if countdown { i.clone() } else { binary(B::Arith(A::Sub), n.clone(), i.clone()) };
    let label = Label { id: 0, name: None };
    let guard = if countdown {
        binary(B::Inequality(InequalityOp::Gt), i.clone(), integer(0))
    } else {
        binary(B::Inequality(InequalityOp::Lt), i.clone(), n.clone())
    };
    let exit = Spanned::new(
        span(),
        StmX::If(
            SpannedTyped::new(&span(), &bool_typ(), ExpX::Unary(UnaryOp::Not, guard)),
            Spanned::new(span(), StmX::BreakOrContinue { label: label.clone(), is_break: true }),
            None,
        ),
    );
    let next_r =
        binary(B::Arith(if subtract { A::Sub } else { A::Add }), r.clone(), stride.clone());
    let next_i = binary(B::Arith(if countdown { A::Sub } else { A::Add }), i.clone(), integer(1));
    let body = Spanned::new(
        span(),
        StmX::Block(Arc::new(vec![
            exit,
            Spanned::new(
                span(),
                StmX::Assert(
                    None,
                    Some(error(&span(), "accumulator overflow")),
                    if subtract {
                        binary(B::Inequality(InequalityOp::Ge), next_r.clone(), integer(0))
                    } else {
                        fits(next_r.clone())
                    },
                ),
            ),
            assignment("r", next_r, false),
            Spanned::new(
                span(),
                StmX::Assert(
                    None,
                    Some(error(&span(), "counter overflow")),
                    if countdown {
                        binary(B::Inequality(InequalityOp::Ge), next_i.clone(), integer(0))
                    } else {
                        fits(next_i.clone())
                    },
                ),
            ),
            assignment("i", next_i, false),
        ])),
    );
    let completed =
        if countdown { binary(B::Arith(A::Sub), n.clone(), i.clone()) } else { i.clone() };
    let invariants = [
        binary(B::Inequality(InequalityOp::Le), i.clone(), n.clone()),
        binary(
            B::Eq,
            r.clone(),
            binary(B::Arith(if subtract { A::Sub } else { A::Add }), x.clone(), product(completed)),
        ),
    ];
    let loop_stm = Spanned::new(
        span(),
        StmX::Loop {
            pre_stms: Arc::new(vec![]),
            loop_isolation: false,
            is_for_loop: false,
            id: 0,
            label,
            cond: None,
            body,
            invs: Arc::new(
                invariants
                    .into_iter()
                    .map(|inv| LoopInv { at_entry: true, at_exit: true, inv })
                    .collect(),
            ),
            decrease: Arc::new(vec![measure.clone()]),
            typ_inv_vars: Arc::new(vec![]),
            modified_vars: None,
            au_branch_bool: None,
            pre_modified_params_incl: None,
            pre_modified_params_excl: None,
        },
    );
    check.body = Spanned::new(
        span(),
        StmX::Block(Arc::new(vec![
            assignment("r", x.clone(), true),
            assignment("i", if countdown { n.clone() } else { integer(0) }, true),
            loop_stm,
            Spanned::new(
                span(),
                StmX::Return {
                    assert_id: None,
                    base_error: error(&span(), "postcondition"),
                    ret_exp: Some(r),
                    inside_body: false,
                },
            ),
        ])),
    );
    check.reqs = Arc::new(vec![if subtract {
        binary(B::Inequality(InequalityOp::Le), amount, x)
    } else {
        fits(sum.clone())
    }]);
    Arc::make_mut(&mut check.post_condition).ens_exps =
        Arc::new(vec![binary(B::Eq, read("result"), sum)]);
    check.local_decls = Arc::new(
        [
            ("x", LocalDeclKind::Param { mutable: false }),
            ("n", LocalDeclKind::Param { mutable: false }),
            ("r", LocalDeclKind::StmtLet { mutable: true }),
            ("i", LocalDeclKind::StmtLet { mutable: true }),
            ("result", LocalDeclKind::Return),
        ]
        .into_iter()
        .map(|(n, kind)| Arc::new(LocalDeclX { ident: var(n), typ: typ.clone(), kind }))
        .collect(),
    );
    if step.is_none() {
        Arc::make_mut(&mut check.local_decls).push(Arc::new(LocalDeclX {
            ident: var("step"),
            typ: typ.clone(),
            kind: LocalDeclKind::Param { mutable: false },
        }));
    }
    let (decls, _) =
        crate::recursion::mk_decreases_at_entry(&ctx, &span(), Some(0), &vec![measure]).unwrap();
    Arc::make_mut(&mut check.local_decls).extend(decls);
    let mut pars: Pars = Arc::new(
        ["x", "n"]
            .into_iter()
            .map(|n| {
                Spanned::new(span(), ParX { name: var(n), typ: typ.clone(), mode: Mode::Exec })
            })
            .collect(),
    );
    if step.is_none() {
        Arc::make_mut(&mut pars)
            .push(Spanned::new(span(), ParX { name: var("step"), typ, mode: Mode::Exec }));
    }
    (ctx, pars, check)
}

fn change_counted_loop(
    check: &FuncCheckSst,
    inv: Option<Exp>,
    measure: Option<Exp>,
) -> FuncCheckSst {
    let mut changed = check.clone();
    changed.body = crate::sst_visitor::map_stm_visitor(&check.body, &mut |stm| {
        let mut x = stm.x.clone();
        if let StmX::Loop { invs, decrease, .. } = &mut x {
            if let Some(inv) = &inv {
                Arc::make_mut(invs)[1].inv = inv.clone();
            }
            if let Some(measure) = &measure {
                *decrease = Arc::new(vec![measure.clone()]);
            }
        }
        Ok(Spanned::new(stm.span.clone(), x))
    })
    .unwrap();
    changed
}

#[test]
fn counted_loop_has_independent_soundness_maximality_and_exactness() {
    let (ctx, pars, check) = counted_loop_fixture();
    assert!(validate_query_kind(&check, None, &span()).is_ok());
    assert!(verifies(&ctx, &pars, &check));
    for kind in [PrecisionCheck::Maximality, PrecisionCheck::Exactness] {
        let precision = precision_check_body(&span(), &pars[0].x.typ, &check, kind).unwrap();
        assert!(verifies(&ctx, &pars, &precision));
    }
}

#[test]
fn counted_loop_restrictive_candidate_and_circular_invariant_cannot_prove_maximality() {
    let (ctx, pars, mut check) = counted_loop_fixture();
    check.reqs = Arc::new(vec![binary(
        crate::sst::BinaryOp::Inequality(InequalityOp::Lt),
        SpannedTyped::new(&span(), &pars[0].x.typ, ExpX::Var(var("x"))),
        integer(10),
    )]);
    // Keep the real safe bound as well, so soundness remains true.
    check.reqs = Arc::new(vec![counted_loop_fixture().2.reqs[0].clone(), check.reqs[0].clone()]);
    assert!(verifies(&ctx, &pars, &check));
    let maximality =
        precision_check_body(&span(), &pars[0].x.typ, &check, PrecisionCheck::Maximality).unwrap();
    assert!(!verifies(&ctx, &pars, &maximality));
    // Even if a proposed invariant includes the candidate precondition, its
    // initialization must be proved after that precondition is removed.
    let circular = change_counted_loop(&check, Some(check.reqs[1].clone()), None);
    let maximality =
        precision_check_body(&span(), &pars[0].x.typ, &circular, PrecisionCheck::Maximality)
            .unwrap();
    assert!(!verifies(&ctx, &pars, &maximality));
}

#[test]
fn counted_loop_bad_invariant_and_non_decreasing_measure_fail_ordinary_checks() {
    let (ctx, pars, check) = counted_loop_fixture();
    let bad_inv = SpannedTyped::new(&span(), &bool_typ(), ExpX::Const(Constant::Bool(false)));
    assert!(!verifies(&ctx, &pars, &change_counted_loop(&check, Some(bad_inv), None)));
    assert!(!verifies(&ctx, &pars, &change_counted_loop(&check, None, Some(integer(0)))));
}

#[test]
fn counted_loop_missing_measure_cannot_receive_an_ordinary_or_precision_query() {
    let (_, pars, mut check) = counted_loop_fixture();
    check.body = crate::sst_visitor::map_stm_visitor(&check.body, &mut |stm| {
        let mut x = stm.x.clone();
        if let StmX::Loop { decrease, .. } = &mut x {
            *decrease = Arc::new(vec![]);
        }
        Ok(Spanned::new(stm.span.clone(), x))
    })
    .unwrap();
    assert!(validate_query_kind(&check, None, &span()).is_err());
    for kind in [PrecisionCheck::Maximality, PrecisionCheck::Exactness] {
        assert!(precision_check_body(&span(), &pars[0].x.typ, &check, kind).is_err());
    }
}

#[test]
fn overpermissive_domain_fails_soundness() {
    let (ctx, pars, mut check) = fixture();
    check.reqs = Arc::new(vec![]);
    assert!(!verifies(&ctx, &pars, &check));
}

#[test]
fn restrictive_domain_is_sound_but_not_maximal() {
    let (ctx, pars, mut check) = fixture();
    check.reqs =
        Arc::new(vec![SpannedTyped::new(&span(), &bool_typ(), ExpX::Const(Constant::Bool(false)))]);
    assert!(verifies(&ctx, &pars, &check));
    let precision =
        precision_check_body(&span(), &pars[0].x.typ, &check, PrecisionCheck::Maximality).unwrap();
    assert!(!verifies(&ctx, &pars, &precision));
}

#[test]
fn weak_output_is_sound_but_not_exact() {
    let (ctx, pars, mut check) = fixture();
    Arc::make_mut(&mut check.post_condition).ens_exps =
        Arc::new(vec![SpannedTyped::new(&span(), &bool_typ(), ExpX::Const(Constant::Bool(true)))]);
    assert!(verifies(&ctx, &pars, &check));
    let precision =
        precision_check_body(&span(), &pars[0].x.typ, &check, PrecisionCheck::Exactness).unwrap();
    assert!(!verifies(&ctx, &pars, &precision));
}

#[test]
fn wrong_output_fails_soundness_and_exactness() {
    let (ctx, pars, mut check) = fixture();
    let result = SpannedTyped::new(&span(), &pars[0].x.typ, ExpX::Var(var("result")));
    Arc::make_mut(&mut check.post_condition).ens_exps =
        Arc::new(vec![binary(crate::sst::BinaryOp::Eq, result, integer(0))]);
    assert!(!verifies(&ctx, &pars, &check));
    let precision =
        precision_check_body(&span(), &pars[0].x.typ, &check, PrecisionCheck::Exactness).unwrap();
    assert!(!verifies(&ctx, &pars, &precision));
}

#[test]
fn precision_queries_cannot_replace_or_mutate_the_ordinary_safety_query() {
    let (ctx, pars, mut ordinary) = fixture();
    // Admit the overflowing x=255. The normal body must continue to reject it
    // after generating and even successfully running a precision query.
    ordinary.reqs = Arc::new(vec![]);
    assert!(!verifies(&ctx, &pars, &ordinary));
    let before = format!("{ordinary:?}");
    for kind in [PrecisionCheck::Maximality, PrecisionCheck::Exactness] {
        let precision = precision_check_body(&span(), &pars[0].x.typ, &ordinary, kind).unwrap();
        assert!(validate_query_kind(&precision, None, &span()).is_err());
        assert!(validate_query_kind(&precision, Some(kind), &span()).is_ok());
        assert!(precision_check_body(&span(), &pars[0].x.typ, &precision, kind).is_err());
        assert_eq!(format!("{ordinary:?}"), before);
        // Maximality is true for P=true, despite ordinary soundness failing.
        // This result must never be recorded as a soundness receipt.
        if matches!(kind, PrecisionCheck::Maximality) {
            assert!(verifies(&ctx, &pars, &precision));
        }
        assert!(!verifies(&ctx, &pars, &ordinary));
    }
}

#[test]
fn weak_range_is_not_exact_over_arbitrary_outputs() {
    let (ctx, pars, mut ordinary) = fixture();
    let ret = SpannedTyped::new(&span(), &pars[0].x.typ, ExpX::Var(var("result")));
    Arc::make_mut(&mut ordinary.post_condition).ens_exps =
        Arc::new(vec![binary(crate::sst::BinaryOp::Inequality(InequalityOp::Le), integer(0), ret)]);
    assert!(verifies(&ctx, &pars, &ordinary));
    let exact = precision_check_body(&span(), &pars[0].x.typ, &ordinary, PrecisionCheck::Exactness)
        .unwrap();
    assert!(!verifies(&ctx, &pars, &exact));
}

#[test]
fn counted_step_families_check_all_obligations_and_reject_faults() {
    use crate::sst::BinaryOp as B;
    for subtract in [false, true] {
        for countdown in [false, true] {
            for step in [Some(0), Some(2), Some(255), None] {
                let (ctx, pars, check) = counted_family_fixture(subtract, countdown, step);
                let valid = |c: &FuncCheckSst| verifies_using(&ctx, &pars, c, step.is_none());
                let precision = |c: &FuncCheckSst, kind| {
                    precision_check_body(&span(), &pars[0].x.typ, c, kind).unwrap()
                };
                assert!(valid(&check), "soundness: {subtract} {countdown} {step:?}");
                for kind in [PrecisionCheck::Maximality, PrecisionCheck::Exactness] {
                    assert!(
                        valid(&precision(&check, kind)),
                        "{kind:?}: {subtract} {countdown} {step:?}"
                    );
                }
                // Faults are independent of synthesis, source recognition, and cache decoding.
                let truth =
                    |b| SpannedTyped::new(&span(), &bool_typ(), ExpX::Const(Constant::Bool(b)));
                assert!(!valid(&change_counted_loop(&check, Some(truth(false)), None)));
                assert!(!valid(&change_counted_loop(&check, None, Some(integer(0)))));
                let mut restrictive = check.clone();
                let read_x = SpannedTyped::new(&span(), &pars[0].x.typ, ExpX::Var(var("x")));
                let restriction = binary(B::Inequality(InequalityOp::Lt), read_x, integer(10));
                Arc::make_mut(&mut restrictive.reqs).push(restriction.clone());
                assert!(valid(&restrictive));
                assert!(!valid(&precision(&restrictive, PrecisionCheck::Maximality)));
                let circular = change_counted_loop(&restrictive, Some(restriction), None);
                assert!(!valid(&precision(&circular, PrecisionCheck::Maximality)));
                if step != Some(0) {
                    let mut permissive = check.clone();
                    permissive.reqs = Arc::new(vec![]);
                    assert!(!valid(&permissive));
                }
                let mut weak = check.clone();
                Arc::make_mut(&mut weak.post_condition).ens_exps = Arc::new(vec![truth(true)]);
                assert!(valid(&weak));
                assert!(!valid(&precision(&weak, PrecisionCheck::Exactness)));
                let mut wrong = check.clone();
                let result = SpannedTyped::new(&span(), &pars[0].x.typ, ExpX::Var(var("result")));
                Arc::make_mut(&mut wrong.post_condition).ens_exps =
                    Arc::new(vec![binary(B::Eq, result, integer(0))]);
                assert!(!valid(&wrong));
                assert!(!valid(&precision(&wrong, PrecisionCheck::Exactness)));
            }
        }
    }
}
