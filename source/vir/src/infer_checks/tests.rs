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
    let interface = Arc::new(VirMessageInterface {});
    let mut solver = air::context::Context::new(interface.clone(), SmtSolver::Z3);
    solver.set_solver_option("air_recommended_options", "true");
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
        false,
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
