use super::*;
use crate::ast::*;
use crate::ast_util::{bool_typ, int_typ, unit_typ};
use crate::def::Spanned;
use crate::messages::Span;
use std::sync::Arc;

fn span() -> Span {
    crate::messages::Span {
        raw_span: Arc::new(()),
        id: 0,
        data: vec![],
        as_string: "affine fixture".into(),
    }
}

fn name(text: &str) -> VarIdent {
    VarIdent(Arc::new(text.into()), VarIdentDisambiguate::RustcId(0))
}

fn fixture() -> Function {
    let span = span();
    let param = |text: &str, typ: Typ| {
        Spanned::new(
            span.clone(),
            ParamX {
                name: name(text),
                typ,
                mode: Mode::Exec,
                user_mut: false,
                unwrapped_info: None,
            },
        )
    };
    let mut params: Vec<_> =
        (0..10).map(|index| param(&format!("descriptor_{index}"), int_typ())).collect();
    params.push(param("wrong", int_typ()));
    params.push(param("frame", int_typ()));
    params.push(param("condition", bool_typ()));
    let ret = param("result", unit_typ());
    Spanned::new(
        span.clone(),
        FunctionX {
            name: Arc::new(FunX {
                path: Arc::new(PathX {
                    krate: CrateId::Id(Arc::new("affine_test".into()), 1),
                    segments: Arc::new(vec![Arc::new("fixture".into())]),
                }),
            }),
            proxy: None,
            kind: FunctionKind::Static,
            visibility: Visibility { restricted_to: None },
            body_visibility: BodyVisibility::Uninterpreted,
            opaqueness: Opaqueness::Opaque,
            owning_module: None,
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(params),
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
            attrs: Arc::new(FunctionAttrsX::default()),
            body: Some(SpannedTyped::new(&span, &unit_typ(), ExprX::Block(Arc::new(vec![]), None))),
            return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![],
            hidden: Arc::new(vec![]),
        },
    )
}

fn krate(function: Function) -> Krate {
    let module = Spanned::new(
        span(),
        ModuleX {
            path: Arc::new(PathX {
                krate: function.x.name.path.krate.clone(),
                segments: Arc::new(vec![]),
            }),
            reveals: None,
        },
    );
    Arc::new(KrateX {
        functions: vec![function],
        reveal_groups: vec![],
        datatypes: vec![],
        traits: vec![],
        trait_impls: vec![],
        assoc_type_impls: vec![],
        modules: vec![module],
        external_fns: vec![],
        external_types: vec![],
        path_as_rust_names: vec![],
        opaque_types: vec![],
        arch: Arch { word_bits: ArchWordBits::Exactly(64) },
    })
}

#[test]
fn marked_crates_are_refused_before_publication_or_native_admission() {
    let function = fixture();
    let mut data = function.x.clone();
    data.attrs = Arc::new(FunctionAttrsX {
        infer_candidate_id: Some(format!("{MARKER_PREFIX}v1:affine")),
        ..FunctionAttrsX::default()
    });
    data.body = None;
    let marked = krate(function.new_x(data));
    let encoded = serde_json::to_vec(&marked).unwrap();
    let marked: Krate = serde_json::from_slice(&encoded).unwrap();
    for operation in ["import", "export", "native source admission"] {
        let error = reject_crate(&marked, operation).unwrap_err();
        assert!(error.note.contains(operation));
    }
    assert!(reject_crate(&krate(fixture()), "export").is_ok());
}

#[test]
fn pure_and_safe_api_contexts_are_refused() {
    let function = fixture();
    let mut data = function.x.clone();
    data.attrs = Arc::new(FunctionAttrsX {
        infer_candidate_id: Some(format!("{MARKER_PREFIX}v1:affine")),
        ..FunctionAttrsX::default()
    });
    data.mode = Mode::Spec;
    assert!(check_lowering(&function.new_x(data.clone()), false).is_err());
    data.mode = Mode::Exec;
    assert!(check_lowering(&function.new_x(data), true).is_err());
}

#[cfg(not(feature = "experimental-native-resource"))]
#[test]
fn lowering_is_default_off() {
    let function = fixture();
    let mut data = function.x.clone();
    data.attrs = Arc::new(FunctionAttrsX {
        infer_candidate_id: Some(format!("{MARKER_PREFIX}v1:affine")),
        ..FunctionAttrsX::default()
    });
    assert!(check_lowering(&function.new_x(data), false).is_err());
}

#[cfg(feature = "experimental-native-resource")]
mod checked {
    use super::super::affine::*;
    use super::*;
    use crate::context::{Ctx, GlobalCtx, WarningCtx};
    use air::context::{SmtEncoding, SmtSolver, ValidityResult};
    use air::messages::MessageInterface;
    use std::collections::{HashMap, HashSet};

    fn descriptor() -> Descriptor {
        std::array::from_fn(|index| name(&format!("descriptor_{index}")))
    }

    fn event(kind: EventKind) -> Event {
        static NEXT_EVENT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let mut span = span();
        span.id = NEXT_EVENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        span.as_string = format!("affine event {}", span.id);
        Event { span, kind }
    }

    fn transfer(source: Slot, destination: Slot) -> Event {
        event(EventKind::Transfer { source, destination, expected: descriptor() })
    }

    fn use_slot(slot: Slot) -> Event {
        event(EventKind::Use { slot, expected: descriptor() })
    }

    fn consume(slot: Slot) -> Event {
        event(EventKind::Consume { slot, expected: descriptor() })
    }

    fn branch(then_events: Vec<Event>, else_events: Vec<Event>) -> Event {
        event(EventKind::If { condition: name("condition"), then_events, else_events })
    }

    fn program(events: Vec<Event>, returned_slot: Option<Slot>) -> Program {
        Program { incoming: descriptor(), frame: name("frame"), events, returned_slot }
    }

    fn global(krate: &Krate, simplified: bool) -> GlobalCtx {
        GlobalCtx::new(
            krate,
            krate.functions[0].x.name.path.krate.clone(),
            span(),
            10.0,
            Arc::new(std::sync::Mutex::new(None)),
            Arc::new(std::sync::Mutex::new(None)),
            Arc::new(WarningCtx { fun_warn_configs: HashMap::new() }),
            SmtSolver::Z3,
            simplified,
            false,
            false,
            false,
            false,
        )
        .unwrap()
    }

    fn check(function: Function, expected_error: Option<&str>) {
        check_at(function, expected_error, None);
    }

    fn check_at(function: Function, expected_error: Option<&str>, expected_span: Option<u64>) {
        let source = krate(function);
        let mut initial_global = global(&source, false);
        let simplified = crate::ast_simplify::simplify_krate(&mut initial_global, &source).unwrap();
        assert!(is_marked(&simplified.functions[0]));
        let mut ctx = Ctx::new(
            &simplified,
            global(&simplified, true),
            simplified.modules[0].clone(),
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
        let reporter = air::messages::Reporter {};
        let functions = simplified.functions.iter().map(|f| f.x.name.clone()).collect();
        let sst =
            crate::ast_to_sst_crate::ast_to_sst_krate(&mut ctx, &reporter, &functions, &simplified)
                .unwrap();
        let function = &sst.functions[0];
        assert!(function.x.attrs.infer_candidate_id.as_deref().unwrap().starts_with(MARKER_PREFIX));
        ctx.fun = crate::ast_to_sst_func::mk_fun_ctx(function, false);
        let checks = crate::sst_to_air_func::func_sst_to_air(
            &ctx,
            function,
            function.x.exec_proof_check.as_ref().unwrap(),
        )
        .unwrap();
        let interface = Arc::new(crate::messages::VirMessageInterface {});
        let mut solver = air::context::Context::new(interface.clone(), SmtSolver::Z3);
        solver.set_solver_option("air_recommended_options", "true");
        let prelude = ctx.prelude(crate::prelude::PreludeConfig {
            arch_word_bits: ArchWordBits::Exactly(64),
            smt_encoding: SmtEncoding::Z3,
        });
        for command in prelude.iter().chain(
            crate::datatype_to_air::datatypes_and_primitives_to_air(&ctx, &simplified.datatypes)
                .iter(),
        ) {
            assert!(matches!(
                solver.command(&*interface, &reporter, command, Default::default()),
                ValidityResult::Valid(..)
            ));
        }
        let mut checked = 0;
        let mut failures = Vec::new();
        let mut failure_spans = Vec::new();
        for group in checks.iter() {
            for command in group.commands.iter() {
                if matches!(&**command, air::ast::CommandX::CheckValid(_)) {
                    eprintln!("native-resource CheckValid expected={expected_error:?}");
                }
                let result = solver.command(&*interface, &reporter, command, Default::default());
                if matches!(&**command, air::ast::CommandX::CheckValid(_)) {
                    checked += 1;
                    match result {
                        ValidityResult::Valid(..) => {}
                        ValidityResult::Invalid(_, Some(message), _) => {
                            let diagnostic =
                                message.downcast_ref::<crate::messages::MessageX>().unwrap();
                            failure_spans.extend(diagnostic.spans.iter().map(|span| span.id));
                            failures.extend(interface.all_msgs(&message));
                        }
                        result => panic!("unexpected resource backend result: {result:?}"),
                    }
                    solver.finish_query();
                } else {
                    assert!(matches!(result, ValidityResult::Valid(..)), "{result:?}");
                }
            }
        }
        assert_eq!(checked, 1, "must submit one real CheckValid query");
        match expected_error {
            None => assert!(failures.is_empty(), "{failures:?}"),
            Some(label) => assert!(
                failures.iter().any(|message| message.contains(label)),
                "expected {label}, got {failures:?}"
            ),
        }
        if let Some(expected_span) = expected_span {
            assert!(
                failure_spans.contains(&expected_span),
                "expected event {expected_span}, got {failure_spans:?}"
            );
        }
    }

    fn run(events: Vec<Event>, returned_slot: Option<Slot>, expected_error: Option<&str>) {
        check(instrument(&fixture(), &program(events, returned_slot)).unwrap(), expected_error);
    }

    #[test]
    fn symbolic_move_use_consume_and_frame() {
        run(
            vec![
                transfer(Slot::Input, Slot::Output),
                use_slot(Slot::Output),
                consume(Slot::Output),
            ],
            None,
            None,
        );
    }

    #[test]
    fn symbolic_branch_join_preserves_authority() {
        run(
            vec![
                branch(
                    vec![transfer(Slot::Input, Slot::Output), transfer(Slot::Output, Slot::Input)],
                    vec![],
                ),
                use_slot(Slot::Input),
                consume(Slot::Input),
            ],
            None,
            None,
        );
    }

    #[test]
    fn symbolic_return_and_early_return_check_exit_shape() {
        run(vec![transfer(Slot::Input, Slot::Output)], Some(Slot::Output), None);
        run(
            vec![
                branch(vec![consume(Slot::Input), event(EventKind::Return)], vec![]),
                consume(Slot::Input),
            ],
            None,
            None,
        );
    }

    #[test]
    fn stale_source_and_duplicate_transfer_fail_in_backend() {
        run(
            vec![transfer(Slot::Input, Slot::Output), use_slot(Slot::Input)],
            Some(Slot::Output),
            Some("NRE100_SOURCE_LIVE"),
        );
        run(
            vec![transfer(Slot::Input, Slot::Output), transfer(Slot::Input, Slot::Output)],
            Some(Slot::Output),
            Some("NRE100_SOURCE_LIVE"),
        );
        run(vec![consume(Slot::Input), consume(Slot::Input)], None, Some("NRE100_SOURCE_LIVE"));
    }

    #[test]
    fn self_transfer_fails_in_backend() {
        run(
            vec![transfer(Slot::Input, Slot::Input)],
            Some(Slot::Input),
            Some("NRE105_DISTINCT_SLOTS"),
        );
    }

    #[test]
    fn asymmetric_branch_join_cannot_reset_live_state() {
        for slot in [Slot::Input, Slot::Output] {
            let usage = use_slot(slot);
            let occurrence = usage.span.id;
            let program = program(
                vec![
                    branch(vec![transfer(Slot::Input, Slot::Output)], vec![]),
                    usage,
                    branch(vec![transfer(Slot::Output, Slot::Input)], vec![]),
                    consume(Slot::Input),
                ],
                None,
            );
            check_at(
                instrument(&fixture(), &program).unwrap(),
                Some("NRE100_SOURCE_LIVE"),
                Some(occurrence),
            );
        }
    }

    #[test]
    fn every_descriptor_component_is_checked_symbolically() {
        for field in 0..DESCRIPTOR_FIELDS {
            let mut expected = descriptor();
            expected[field] = name("wrong");
            let function = fixture();
            let mut data = function.x.clone();
            let variable = |name| SpannedTyped::new(&span(), &int_typ(), ExprX::Var(name));
            data.require = Arc::new(vec![SpannedTyped::new(
                &span(),
                &bool_typ(),
                ExprX::Binary(
                    BinaryOp::Ne,
                    variable(descriptor()[field].clone()),
                    variable(name("wrong")),
                ),
            )]);
            let program = program(
                vec![event(EventKind::Use { slot: Slot::Input, expected }), consume(Slot::Input)],
                None,
            );
            check(
                instrument(&function.new_x(data), &program).unwrap(),
                Some("NRE102_DESCRIPTOR_IDENTITY"),
            );
        }
    }

    #[test]
    fn wrong_normal_and_early_exit_shape_fail_in_backend() {
        run(
            vec![transfer(Slot::Input, Slot::Output)],
            Some(Slot::Input),
            Some("NRE103_EXIT_SHAPE"),
        );
        run(
            vec![branch(vec![event(EventKind::Return)], vec![]), consume(Slot::Input)],
            None,
            Some("NRE103_EXIT_SHAPE"),
        );
    }

    #[test]
    fn invalid_operand_and_nonempty_source_are_refused() {
        let mut invalid = program(vec![], Some(Slot::Input));
        invalid.incoming[0] = name("undeclared");
        assert!(instrument(&fixture(), &invalid).is_err());
        let mut wrong_type = program(vec![], Some(Slot::Input));
        wrong_type.incoming[0] = name("condition");
        assert!(instrument(&fixture(), &wrong_type).is_err());
        let valid = program(vec![], Some(Slot::Input));
        let generated = instrument(&fixture(), &valid).unwrap();
        assert!(instrument(&generated, &valid).is_err());
        let original = fixture();
        let mut nonempty = original.x.clone();
        nonempty.body = Some(SpannedTyped::new(&span(), &unit_typ(), ExprX::Return(None)));
        assert!(instrument(&original.new_x(nonempty), &valid).is_err());
        let mut source = fixture().x.clone();
        source.mode = Mode::Spec;
        assert!(instrument(&fixture().new_x(source), &valid).is_err());
    }

    #[test]
    fn structural_budget_is_not_silently_truncated() {
        let too_many =
            program((0..257).map(|_| use_slot(Slot::Input)).collect(), Some(Slot::Input));
        assert!(instrument(&fixture(), &too_many).is_err());
    }

    #[test]
    fn distinct_name_descriptor_binding_checks_transfer_and_consume() {
        for consume_only in [false, true] {
            for equal in [true, false] {
                let function = fixture();
                let mut data = function.x.clone();
                let mut parameters = (*data.params).clone();
                parameters.push(Spanned::new(
                    span(),
                    ParamX {
                        name: name("affine"),
                        typ: int_typ(),
                        mode: Mode::Exec,
                        user_mut: false,
                        unwrapped_info: None,
                    },
                ));
                data.params = Arc::new(parameters);
                let original =
                    SpannedTyped::new(&span(), &int_typ(), ExprX::Var(descriptor()[0].clone()));
                let alias = SpannedTyped::new(&span(), &int_typ(), ExprX::Var(name("affine")));
                let condition = if equal {
                    crate::ast_util::mk_eq(&span(), &original, &alias)
                } else {
                    SpannedTyped::new(
                        &span(),
                        &bool_typ(),
                        ExprX::Binary(BinaryOp::Ne, original, alias),
                    )
                };
                data.require = Arc::new(vec![condition]);
                let mut expected = descriptor();
                expected[0] = name("affine");
                let events = if consume_only {
                    vec![event(EventKind::Consume { slot: Slot::Input, expected })]
                } else {
                    vec![
                        event(EventKind::Transfer {
                            source: Slot::Input,
                            destination: Slot::Output,
                            expected: expected.clone(),
                        }),
                        event(EventKind::Use { slot: Slot::Output, expected: expected.clone() }),
                        event(EventKind::Consume { slot: Slot::Output, expected }),
                    ]
                };
                let program = program(events, None);
                check(
                    instrument(&function.new_x(data), &program).unwrap(),
                    if equal { None } else { Some("NRE102_DESCRIPTOR_IDENTITY") },
                );
            }
        }
    }

    #[test]
    fn operand_modes_frame_condition_and_expected_bindings_are_checked() {
        let function = fixture();
        for mode in [Mode::Spec, Mode::Exec] {
            let mut data = function.x.clone();
            let mut parameters = (*data.params).clone();
            let mut parameter = parameters[0].x.clone();
            parameter.mode = mode;
            parameter.user_mut = mode == Mode::Exec;
            parameters[0] = parameters[0].new_x(parameter);
            data.params = Arc::new(parameters);
            assert!(
                instrument(&function.new_x(data), &program(vec![], Some(Slot::Input))).is_err()
            );
        }
        let mut invalid_frame = program(vec![], Some(Slot::Input));
        invalid_frame.frame = name("condition");
        assert!(instrument(&function, &invalid_frame).is_err());
        let mut expected = descriptor();
        expected[0] = name("undeclared");
        let invalid_expected =
            program(vec![event(EventKind::Consume { slot: Slot::Input, expected })], None);
        assert!(instrument(&function, &invalid_expected).is_err());
        let invalid_condition = program(
            vec![event(EventKind::If {
                condition: name("frame"),
                then_events: vec![],
                else_events: vec![],
            })],
            Some(Slot::Input),
        );
        assert!(instrument(&function, &invalid_condition).is_err());
        let mut data = function.x.clone();
        let mut parameters = (*data.params).clone();
        let mut parameter = parameters[0].x.clone();
        parameter.name =
            VarIdent(Arc::new("affine".into()), VarIdentDisambiguate::NativeResource(0));
        parameters[0] = parameters[0].new_x(parameter);
        data.params = Arc::new(parameters);
        assert!(instrument(&function.new_x(data), &program(vec![], Some(Slot::Input))).is_err());
    }

    #[test]
    fn structural_depth_limit_is_not_silently_truncated() {
        let mut events = vec![];
        for _ in 0..34 {
            events = vec![branch(events, vec![])];
        }
        assert!(instrument(&fixture(), &program(events, Some(Slot::Input))).is_err());
    }
}
