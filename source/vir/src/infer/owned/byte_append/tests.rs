use super::*;
use crate::infer::tests::{function, input};

fn fun(krate: CrateId, parts: &[&str]) -> Fun {
    Arc::new(FunX {
        path: Arc::new(PathX {
            krate,
            segments: Arc::new(parts.iter().map(|part| Arc::new((*part).into())).collect()),
        }),
    })
}

fn extend() -> Function {
    let base = function("append_model", input("x"));
    let mut model = base.x.clone();
    let types: Vec<_> =
        ["T", "A"].map(|name| Arc::new(TypX::TypParam(Arc::new(name.into())))).into();
    model.name = fun(CrateId::Alloc, &["vec", "impl&%3", "extend_from_slice"]);
    model.proxy = Some(crate::def::Spanned {
        span: base.span.clone(),
        x: fun(
            CrateId::Vstd,
            &["std_specs", "vec", "_verus_external_fn_specification_test_extend__from__slice"],
        )
        .path
        .clone(),
    });
    model.typ_params = Arc::new(["T", "A"].map(|name| Arc::new(name.into())).into());
    let mut bounds: Vec<_> = types
        .iter()
        .map(|typ| {
            Arc::new(GenericBoundX::Trait(
                TraitId::Sizedness(Sizedness::Sized),
                Arc::new(vec![typ.clone()]),
            ))
        })
        .collect();
    for (index, path) in [
        (0, fun(CrateId::Core, &["clone", "Clone"]).path.clone()),
        (1, fun(CrateId::Core, &["alloc", "Allocator"]).path.clone()),
    ] {
        bounds.push(Arc::new(GenericBoundX::Trait(
            TraitId::Path(path),
            Arc::new(vec![types[index].clone()]),
        )));
    }
    model.typ_bounds = Arc::new(bounds);
    let vector = Arc::new(TypX::Datatype(
        Dt::Path(fun(CrateId::Alloc, &["vec", "Vec"]).path.clone()),
        Arc::new(types.clone()),
        Arc::new(vec![]),
    ));
    let arguments = [
        Arc::new(TypX::MutRef(vector)),
        interfaces::reference(interfaces::primitive(Primitive::Slice, vec![types[0].clone()])),
    ];
    model.params = Arc::new(
        model
            .params
            .iter()
            .zip(arguments)
            .map(|(parameter, typ)| {
                let mut value = parameter.x.clone();
                value.typ = typ;
                parameter.new_x(value)
            })
            .collect(),
    );
    let mut result = model.inner_ret.x.clone();
    result.typ = crate::ast_util::unit_typ();
    model.inner_ret = model.inner_ret.new_x(result);
    model.outer_ret = model.inner_ret.clone();
    model.body = None;
    model.require = Arc::new(vec![]);
    model.ensure = (Arc::new(vec![boolean(&base.span, true).expr; 2]), Arc::new(vec![]));
    Arc::make_mut(&mut model.attrs).infer = false;
    base.new_x(model)
}

fn borrowed_model_fixture() -> Function {
    let base = extend();
    let mut model = base.x.clone();
    model.name = fun(CrateId::Alloc, &["vec", "impl&%10", "deref"]);
    model.proxy = Some(crate::def::Spanned {
        span: base.span.clone(),
        x: fun(CrateId::Vstd, &["std_specs", "vec", "_verus_external_fn_specification_test_deref"])
            .path
            .clone(),
    });
    let TypX::MutRef(vector) = &*model.params[0].x.typ else { unreachable!() };
    let vector = vector.clone();
    model.kind = FunctionKind::ForeignTraitMethodImpl {
        method: fun(CrateId::Core, &["ops", "deref", "Deref", "deref"]),
        impl_path: fun(CrateId::Alloc, &["vec", "impl&%10"]).path.clone(),
        trait_path: fun(CrateId::Core, &["ops", "deref", "Deref"]).path.clone(),
        trait_typ_args: Arc::new(vec![vector.clone()]),
    };
    model.typ_bounds = Arc::new(
        model
            .typ_bounds
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 2)
            .map(|(_, bound)| bound.clone())
            .collect(),
    );
    let mut parameter = model.params[0].x.clone();
    parameter.typ = interfaces::reference(vector);
    model.params = Arc::new(vec![model.params[0].new_x(parameter)]);
    let mut result = model.inner_ret.x.clone();
    result.typ = base.x.params[1].x.typ.clone();
    model.inner_ret = model.inner_ret.new_x(result);
    model.outer_ret = model.inner_ret.clone();
    model.ensure.0 = Arc::new(vec![boolean(&base.span, true).expr]);
    base.new_x(model)
}

#[test]
fn borrowed_vec_model_rejects_identity_signature_effect_and_bound_mutants() {
    let canonical = borrowed_model_fixture();
    assert_eq!(models::model(&canonical), Some(Model::BorrowedBytes));
    let FunctionKind::ForeignTraitMethodImpl { method, impl_path, trait_path, trait_typ_args } =
        &canonical.x.kind
    else {
        unreachable!()
    };
    let mut before_lowering = canonical.x.clone();
    before_lowering.kind = FunctionKind::TraitMethodImpl {
        method: method.clone(),
        impl_path: impl_path.clone(),
        trait_path: trait_path.clone(),
        trait_typ_args: trait_typ_args.clone(),
        inherit_body_from: None,
    };
    assert_eq!(models::model(&canonical.new_x(before_lowering)), Some(Model::BorrowedBytes));
    for mutation in 0..20 {
        let mut changed = canonical.x.clone();
        match mutation {
            0 => {
                changed.name =
                    fun(CrateId::Id(Arc::new("lookalike".into()), 1), &["vec", "impl&%10", "deref"])
            }
            1 => changed.proxy = None,
            2 => {
                changed.proxy.as_mut().unwrap().x = fun(
                    CrateId::Vstd,
                    &["std_specs", "vec", "_verus_external_fn_specification_test_as__slice"],
                )
                .path
                .clone()
            }
            3 => changed.mode = Mode::Spec,
            4 => {
                changed.inner_ret = changed
                    .inner_ret
                    .new_x(ParamX { mode: Mode::Spec, ..changed.inner_ret.x.clone() })
            }
            5 => changed.body = Some(boolean(&canonical.span, true).expr),
            6 => changed.require = Arc::new(vec![boolean(&canonical.span, true).expr]),
            7 => Arc::make_mut(&mut changed.attrs).is_unsafe = true,
            8 => Arc::make_mut(&mut changed.attrs).is_async = true,
            9 => changed.kind = FunctionKind::Static,
            10 => changed.typ_bounds = Arc::new(vec![]),
            11 => changed.typ_params = Arc::new(vec![]),
            12 => changed.ensure.0 = Arc::new(vec![]),
            13 => changed.params = Arc::new(vec![]),
            14 => {
                changed.params = Arc::new(vec![
                    changed.params[0]
                        .new_x(ParamX { user_mut: true, ..changed.params[0].x.clone() }),
                ])
            }
            15 => {
                changed.params = Arc::new(vec![changed.params[0].new_x(ParamX {
                    typ: Arc::new(TypX::MutRef(
                        arrays::readonly_base(&changed.params[0].x.typ).clone(),
                    )),
                    ..changed.params[0].x.clone()
                })])
            }
            16 => {
                if let FunctionKind::ForeignTraitMethodImpl { method, .. } = &mut changed.kind {
                    *method = fun(CrateId::Core, &["ops", "deref", "DerefMut", "deref_mut"]);
                }
            }
            17 => {
                if let FunctionKind::ForeignTraitMethodImpl { impl_path, .. } = &mut changed.kind {
                    *impl_path = fun(CrateId::Alloc, &["vec", "impl&%11"]).path.clone();
                }
            }
            18 => {
                if let FunctionKind::ForeignTraitMethodImpl { trait_typ_args, .. } =
                    &mut changed.kind
                {
                    *trait_typ_args = Arc::new(vec![]);
                }
            }
            19 => {
                if let FunctionKind::ForeignTraitMethodImpl { trait_path, .. } = &mut changed.kind {
                    *trait_path = fun(CrateId::Core, &["ops", "deref", "DerefMut"]).path.clone();
                }
            }
            _ => unreachable!(),
        }
        assert_eq!(models::model(&canonical.new_x(changed)), None, "mutation {mutation}");
    }
}

#[test]
fn canonical_append_model_declines_independent_identity_signature_and_mode_mutants() {
    let canonical = extend();
    assert_eq!(models::model(&canonical), Some(Model::ExtendBytes));
    for mutation in 0..13 {
        let mut changed = canonical.x.clone();
        match mutation {
            0 => {
                changed.name = fun(
                    CrateId::Id(Arc::new("lookalike".into()), 1),
                    &["vec", "impl&%3", "extend_from_slice"],
                )
            }
            1 => changed.proxy = None,
            2 => {
                changed.proxy = Some(crate::def::Spanned {
                    span: canonical.span.clone(),
                    x: fun(
                        CrateId::Vstd,
                        &["std_specs", "vec", "_verus_external_fn_specification_test_append"],
                    )
                    .path
                    .clone(),
                })
            }
            3 => changed.mode = Mode::Spec,
            4 => changed.body = Some(input("x")),
            5 => changed.require = Arc::new(vec![boolean(&canonical.span, true).expr]),
            6 => changed.typ_bounds = Arc::new(changed.typ_bounds[..3].to_vec()),
            7 => Arc::make_mut(&mut changed.attrs).is_unsafe = true,
            8 => Arc::make_mut(&mut changed.attrs).is_async = true,
            9 => {
                let mut parameter = changed.params[1].x.clone();
                parameter.typ = Arc::new(TypX::MutRef(interfaces::primitive(
                    Primitive::Slice,
                    vec![input("x").typ.clone()],
                )));
                Arc::make_mut(&mut changed.params)[1] = changed.params[1].new_x(parameter);
            }
            10 => changed.ensure.0 = Arc::new(vec![boolean(&canonical.span, true).expr]),
            11 => {
                let mut result = changed.inner_ret.x.clone();
                result.mode = Mode::Spec;
                changed.inner_ret = changed.inner_ret.new_x(result);
                changed.outer_ret = changed.inner_ret.clone();
            }
            12 => {
                let mut parameter = changed.params[1].x.clone();
                parameter.mode = Mode::Proof;
                Arc::make_mut(&mut changed.params)[1] = changed.params[1].new_x(parameter);
            }
            _ => unreachable!(),
        }
        assert_eq!(models::model(&canonical.new_x(changed)), None, "mutation {mutation}");
    }
}

#[test]
fn borrowed_call_requires_matching_resolution_inert_byte_types_and_distinct_roots() {
    let source = function("borrowed_oracle", input("x"));
    let model = borrowed_model_fixture();
    let all = HashMap::from([(model.x.name.clone(), model.clone())]);
    let span = &source.span;
    let byte = Arc::new(TypX::Int(IntRange::U(8)));
    let global = Arc::new(TypX::Datatype(
        Dt::Path(fun(CrateId::Alloc, &["alloc", "Global"]).path.clone()),
        Arc::new(vec![]),
        Arc::new(vec![]),
    ));
    let vector = Arc::new(TypX::Datatype(
        Dt::Path(fun(CrateId::Alloc, &["vec", "Vec"]).path.clone()),
        Arc::new(vec![byte.clone(), global.clone()]),
        Arc::new(vec![]),
    ));
    let slice = interfaces::reference(interfaces::primitive(Primitive::Slice, vec![byte.clone()]));
    let destination = VarIdent(Arc::new("destination".into()), VarIdentDisambiguate::AirLocal);
    let suffix = VarIdent(Arc::new("suffix".into()), VarIdentDisambiguate::AirLocal);
    let contents =
        atom(SpannedTyped::new(span, &sequence_type(byte.clone()), ExprX::Var(suffix.clone())))
            .unwrap();
    let mut state = State {
        locals: HashMap::from([(
            suffix.clone(),
            Val { identity: None, typ: vector.clone(), node: Node::Seq(contents.clone()) },
        )]),
        objects: HashMap::new(),
        assigned: std::collections::HashSet::new(),
        active: boolean(span, true),
    };
    let mut engine = Engine {
        f: &source,
        all: &all,
        ds: &[],
        requirements: vec![],
        exits: vec![],
        plans: vec![],
        callbacks: vec![],
        remaining: FORMULA_BUDGET,
    };
    let argument = SpannedTyped::new(
        span,
        &interfaces::reference(vector.clone()),
        ExprX::ReadPlace(
            SpannedTyped::new(span, &vector, PlaceX::Local(suffix.clone())),
            UnfinalizedReadKind { preliminary_kind: ReadKind::ImmutBor, id: 0 },
        ),
    );
    let FunctionKind::ForeignTraitMethodImpl { method, impl_path, .. } = &model.x.kind else {
        unreachable!()
    };
    let canonical = SpannedTyped::new(
        span,
        &slice,
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::DynamicResolved {
                    resolved: model.x.name.clone(),
                    typs: Arc::new(vec![byte.clone(), global.clone()]),
                    impl_paths: Arc::new(vec![]),
                    is_trait_default: false,
                },
                method.clone(),
                Arc::new(vec![vector.clone()]),
                Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![argument]),
            post_args: None,
            body: None,
        },
    );
    let observed = engine.borrowed_bytes(&mut state, &canonical, &destination).unwrap();
    assert_eq!(format!("{:?}", observed.expr), format!("{:?}", contents.expr));
    assert!(engine.requirements.is_empty());
    assert_eq!(state.locals.len(), 1);
    assert!(engine.borrowed_bytes(&mut state, &canonical, &suffix).is_err());
    for mutation in 0..10 {
        let mut changed = canonical.x.clone();
        let ExprX::Call { target: CallTarget::Fun(kind, name, types, impls, attrs), args, .. } =
            &mut changed
        else {
            unreachable!()
        };
        match mutation {
            0 => *kind = CallTargetKind::Dynamic,
            1 => {
                if let CallTargetKind::DynamicResolved { is_trait_default, .. } = kind {
                    *is_trait_default = true;
                }
            }
            2 => *name = fun(CrateId::Core, &["ops", "deref", "DerefMut", "deref_mut"]),
            3 => *impls = Arc::new(vec![]),
            4 => attrs.const_var = true,
            5 => attrs.assume_external_allowed = true,
            6 => *types = Arc::new(vec![]),
            7 => {
                *args = Arc::new(vec![SpannedTyped::new(
                    span,
                    &interfaces::reference(vector.clone()),
                    ExprX::Var(suffix.clone()),
                )])
            }
            8 => {
                if let CallTargetKind::DynamicResolved { typs, .. } = kind {
                    *typs = Arc::new(vec![
                        byte.clone(),
                        Arc::new(TypX::TypParam(Arc::new("CustomAllocator".into()))),
                    ]);
                }
            }
            9 => {
                let reference = interfaces::reference(vector.clone());
                *args = Arc::new(vec![SpannedTyped::new(
                    span,
                    &reference,
                    ExprX::ReadPlace(
                        SpannedTyped::new(span, &reference, PlaceX::Local(suffix.clone())),
                        UnfinalizedReadKind { preliminary_kind: ReadKind::ImmutBor, id: 0 },
                    ),
                )]);
            }
            _ => unreachable!(),
        }
        assert!(
            engine.borrowed_bytes(&mut state, &canonical.new_x(changed), &destination).is_err(),
            "mutation {mutation}"
        );
    }
}

fn concat_model() -> Function {
    let base = function("concat_model", input("x"));
    let mut model = base.x.clone();
    let element = Arc::new(TypX::TypParam(Arc::new("T".into())));
    let sequence = sequence_type(element.clone());
    model.name = fun(CrateId::Vstd, &["seq", "Seq", "add"]);
    model.typ_params = Arc::new(vec![Arc::new("T".into())]);
    model.typ_bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
        TraitId::Sizedness(Sizedness::Sized),
        Arc::new(vec![element]),
    ))]);
    model.mode = Mode::Spec;
    model.require = Arc::new(vec![]);
    model.params = Arc::new(
        model
            .params
            .iter()
            .map(|parameter| {
                let mut value = parameter.x.clone();
                value.typ = sequence.clone();
                value.mode = Mode::Spec;
                parameter.new_x(value)
            })
            .collect(),
    );
    let mut result = model.inner_ret.x.clone();
    result.typ = sequence;
    result.mode = Mode::Spec;
    model.inner_ret = model.inner_ret.new_x(result);
    model.outer_ret = model.inner_ret.clone();
    Arc::make_mut(&mut model.attrs).infer = false;
    base.new_x(model)
}

#[test]
fn canonical_concat_has_two_equal_sequence_arguments_not_an_overloaded_add() {
    let canonical = concat_model();
    assert_eq!(models::model(&canonical), Some(Model::Concat));
    for mutation in 0..4 {
        let mut changed = canonical.x.clone();
        match mutation {
            0 => changed.name = fun(CrateId::Core, &["ops", "arith", "Add", "add"]),
            1 => changed.params = Arc::new(changed.params[..1].to_vec()),
            2 => changed.typ_bounds = Arc::new(vec![]),
            3 => changed.outer_ret = function("bad", input("x")).x.outer_ret.clone(),
            _ => unreachable!(),
        }
        assert_eq!(models::model(&canonical.new_x(changed)), None);
    }
}

fn slice_view_model() -> Function {
    let base = extend();
    let mut model = base.x.clone();
    let element = Arc::new(TypX::TypParam(model.typ_params[0].clone()));
    let slice = interfaces::primitive(Primitive::Slice, vec![element.clone()]);
    let implementation = fun(CrateId::Vstd, &["slice", "impl&%0"]).path.clone();
    model.name = fun(CrateId::Vstd, &["slice", "impl&%0", "view"]);
    model.proxy = None;
    model.mode = Mode::Spec;
    model.kind = FunctionKind::TraitMethodImpl {
        method: fun(CrateId::Vstd, &["view", "View", "view"]),
        impl_path: implementation,
        trait_path: fun(CrateId::Vstd, &["view", "View"]).path.clone(),
        trait_typ_args: Arc::new(vec![slice.clone()]),
        inherit_body_from: None,
    };
    model.typ_params = Arc::new(vec![model.typ_params[0].clone()]);
    model.typ_bounds = Arc::new(vec![model.typ_bounds[0].clone()]);
    let mut parameter = model.params[0].x.clone();
    parameter.typ = interfaces::reference(slice);
    parameter.mode = Mode::Spec;
    model.params = Arc::new(vec![model.params[0].new_x(parameter)]);
    let mut result = model.inner_ret.x.clone();
    result.typ = sequence_type(element);
    result.mode = Mode::Spec;
    model.inner_ret = model.inner_ret.new_x(result);
    model.outer_ret = model.inner_ret.clone();
    model.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    base.new_x(model)
}

#[test]
fn slice_observation_requires_the_actual_view_implementation() {
    let canonical = slice_view_model();
    assert_eq!(models::model(&canonical), Some(Model::SliceView));
    for mutation in 0..5 {
        let mut changed = canonical.x.clone();
        match mutation {
            0 => changed.kind = FunctionKind::Static,
            1 => {
                changed.name =
                    fun(CrateId::Id(Arc::new("fake".into()), 1), &["slice", "impl&%0", "view"])
            }
            2 => changed.body = Some(input("x")),
            3 => changed.typ_bounds = Arc::new(vec![]),
            4 => changed.mode = Mode::Exec,
            _ => unreachable!(),
        }
        assert_eq!(models::model(&canonical.new_x(changed)), None);
    }
}

#[test]
fn generated_append_state_matches_native_u8_execution_on_small_domains() {
    let source = function("append_oracle", input("x"));
    let span = source.span.clone();
    let byte = Arc::new(TypX::Int(IntRange::U(8)));
    let global = Arc::new(TypX::Datatype(
        Dt::Path(fun(CrateId::Alloc, &["alloc", "Global"]).path.clone()),
        Arc::new(vec![]),
        Arc::new(vec![]),
    ));
    let vector = Arc::new(TypX::Datatype(
        Dt::Path(fun(CrateId::Alloc, &["vec", "Vec"]).path.clone()),
        Arc::new(vec![byte.clone(), global.clone()]),
        Arc::new(vec![]),
    ));
    let slice = interfaces::reference(interfaces::primitive(Primitive::Slice, vec![byte.clone()]));
    let destination = VarIdent(Arc::new("buffer".into()), VarIdentDisambiguate::AirLocal);
    let prefix_name = VarIdent(Arc::new("prefix".into()), VarIdentDisambiguate::AirLocal);
    let tail_name = source.x.params[1].x.name.clone();
    let prefix = SpannedTyped::new(&span, &sequence_type(byte.clone()), ExprX::Var(prefix_name));
    let tail = SpannedTyped::new(&span, &slice, ExprX::Var(tail_name.clone()));
    let mut state = State {
        locals: HashMap::new(),
        objects: HashMap::new(),
        assigned: std::collections::HashSet::new(),
        active: boolean(&span, true),
    };
    state.locals.insert(
        destination.clone(),
        Val { identity: None, typ: vector.clone(), node: Node::Seq(atom(prefix).unwrap()) },
    );
    state.locals.insert(tail_name.clone(), Val::atom(atom(tail).unwrap()));
    let models = [extend(), slice_view_model(), concat_model()];
    let all = models.iter().map(|model| (model.x.name.clone(), model.clone())).collect();
    let mut engine = Engine {
        f: &source,
        all: &all,
        ds: &[],
        requirements: vec![],
        exits: vec![],
        plans: vec![],
        callbacks: vec![],
        remaining: FORMULA_BUDGET,
    };
    let receiver = SpannedTyped::new(
        &span,
        &Arc::new(TypX::MutRef(vector.clone())),
        ExprX::TwoPhaseBorrowMut(SpannedTyped::new(
            &span,
            &vector,
            PlaceX::Local(destination.clone()),
        )),
    );
    let borrowed = SpannedTyped::new(
        &span,
        &interfaces::reference(slice.clone()),
        ExprX::ReadPlace(
            SpannedTyped::new(&span, &slice, PlaceX::Local(tail_name)),
            UnfinalizedReadKind { preliminary_kind: ReadKind::ImmutBor, id: 0 },
        ),
    );
    let arguments = Arc::new(vec![receiver, borrowed]);
    let call = SpannedTyped::new(
        &span,
        &crate::ast_util::unit_typ(),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                models[0].x.name.clone(),
                Arc::new(vec![byte, global]),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: arguments.clone(),
            post_args: None,
            body: None,
        },
    );
    engine.extend_bytes(&mut state, &call, &arguments).unwrap();
    assert!(engine.requirements.is_empty());
    assert!(state.locals.get(&source.x.params[1].x.name).is_some());
    if let ExprX::Call { target: CallTarget::Fun(kind, name, types, impls, attributes), .. } =
        &call.x
    {
        for wrong_types in [
            vec![Arc::new(TypX::Int(IntRange::U(16))), types[1].clone()],
            vec![types[0].clone(), Arc::new(TypX::TypParam(Arc::new("Allocator".into())))],
        ] {
            let changed = call.new_x(ExprX::Call {
                target: CallTarget::Fun(
                    kind.clone(),
                    name.clone(),
                    Arc::new(wrong_types),
                    impls.clone(),
                    attributes.clone(),
                ),
                args: arguments.clone(),
                post_args: None,
                body: None,
            });
            assert!(engine.extend_bytes(&mut state, &changed, &arguments).is_err());
        }
    }
    let Node::Seq(candidate) = &state.locals[&destination].node else {
        panic!("sequence state missing")
    };
    fn evaluate(
        expression: &Expr,
        prefix: &[u8],
        suffix: &[u8],
        all: &HashMap<Fun, Function>,
    ) -> Vec<u8> {
        match &expression.x {
            ExprX::Var(name) if name.0.as_str() == "prefix" => prefix.to_vec(),
            ExprX::Var(_) => suffix.to_vec(),
            ExprX::Call { target: CallTarget::Fun(kind, name, ..), args, .. } => {
                let name = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                    resolved
                } else {
                    name
                };
                match models::model(&all[name]).unwrap() {
                    Model::SliceView => evaluate(&args[0], prefix, suffix, all),
                    Model::Concat => evaluate(&args[0], prefix, suffix, all)
                        .into_iter()
                        .chain(evaluate(&args[1], prefix, suffix, all))
                        .collect(),
                    other => panic!("unexpected model {other:?}"),
                }
            }
            other => panic!("unexpected generated term {other:?}"),
        }
    }
    let mut domain = vec![vec![]];
    for length in 1..=4 {
        for encoding in 0..3usize.pow(length) {
            let mut value = encoding;
            let mut bytes = Vec::new();
            for _ in 0..length {
                bytes.push([0, 1, 255][value % 3]);
                value /= 3;
            }
            domain.push(bytes);
        }
    }
    for prefix in &domain {
        for suffix in &domain {
            let mut native = prefix.clone();
            native.extend_from_slice(suffix);
            assert_eq!(evaluate(&candidate.expr, prefix, suffix, &all), native);
        }
    }
}
