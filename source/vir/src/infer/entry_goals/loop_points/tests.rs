use super::*;
use crate::ast::{VarBinderX, VarIdentDisambiguate};

fn variable(name: &str, typ: Typ) -> Exp {
    SpannedTyped::new(
        &crate::infer::tests::span(),
        &typ,
        ExpX::Var(VarIdent(Arc::new(name.into()), VarIdentDisambiguate::AirLocal)),
    )
}

fn integer(name: &str) -> Exp {
    variable(name, crate::ast_util::int_typ())
}

fn name(expression: &Exp) -> VarIdent {
    let ExpX::Var(name) = &expression.x else { panic!() };
    name.clone()
}

fn declaration(expression: &Exp) -> sst::LocalDecl {
    Arc::new(sst::LocalDeclX {
        ident: name(expression),
        typ: expression.typ.clone(),
        kind: LocalDeclKind::StmtLet { mutable: true },
    })
}

fn freeze(expression: &Exp, roots: &[Exp], live: bool) -> Result<(Exp, Vec<Capture>), Decline> {
    freeze_using(expression, roots, live, &HashMap::new())
}

fn freeze_using(
    expression: &Exp,
    roots: &[Exp],
    live: bool,
    datatypes: &HashMap<Dt, crate::ast::Datatype>,
) -> Result<(Exp, Vec<Capture>), Decline> {
    let declarations = roots.iter().map(|root| (name(root), declaration(root))).collect();
    let live = if live { roots.iter().map(name).collect() } else { Live::new() };
    let mut captures = Vec::new();
    let mut snapshot = Snapshot {
        datatypes,
        declarations: &declarations,
        live: &live,
        names: &mut roots.iter().map(name).collect(),
        captures: &mut captures,
        budget: &mut Budget::new(),
        scope: VisitorScopeMap::new(),
    };
    let frozen = snapshot.visit_exp(expression)?;
    Ok((frozen, captures))
}

fn quantifier(bound: &Exp, body: &Exp, triggers: Vec<Vec<Exp>>) -> Exp {
    let binder = Spanned::new(
        bound.span.clone(),
        BndX::Quant(
            QUANT_FORALL,
            Arc::new(vec![Arc::new(VarBinderX { name: name(bound), a: bound.typ.clone() })]),
            Arc::new(triggers.into_iter().map(Arc::new).collect()),
            None,
        ),
    );
    SpannedTyped::new(&body.span, &Arc::new(TypX::Bool), ExpX::Bind(binder, body.clone()))
}

#[test]
fn captures_only_typed_live_roots_and_uses_location_initializers() {
    let root = integer("live");
    let (frozen, captures) = freeze(&root, &[root.clone()], true).unwrap();
    assert_eq!(captures.len(), 1);
    assert!(types_equal(&captures[0].rhs.typ, &captures[0].declaration.typ));
    assert!(types_equal(&frozen.typ, &captures[0].declaration.typ));
    let initializer =
        crate::ast_to_sst::init_var(&root.span, &captures[0].declaration.ident, &captures[0].rhs);
    let StmX::Assign { lhs, rhs } = &initializer.x else { panic!() };
    assert!(lhs.is_init);
    assert!(matches!(&lhs.dest.x, ExpX::VarLoc(actual) if actual == &name(&frozen)));
    assert!(types_equal(&lhs.dest.typ, &rhs.typ));
    assert_eq!(freeze(&root, &[root.clone()], false).err(), Some("missing_live_root"));
    assert_eq!(freeze(&root, &[], true).err(), Some("missing_live_root"));
}

#[test]
fn mutable_reference_current_captures_the_value_not_the_token() {
    let root = variable("receiver", Arc::new(TypX::MutRef(crate::ast_util::int_typ())));
    let current = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Boxed(crate::ast_util::int_typ())),
        ExpX::Unary(UnaryOp::MutRefCurrent, root.clone()),
    );
    let (_, captures) = freeze(&current, &[root.clone()], true).unwrap();
    assert_eq!(captures.len(), 1);
    assert!(captures[0].current);
    assert!(Arc::ptr_eq(&captures[0].rhs, &current));
    assert_eq!(freeze(&root, &[root.clone()], true).err(), Some("unsupported_value_type"));
    let wrong = current.new_typ(&Arc::new(TypX::Bool));
    assert_eq!(freeze(&wrong, &[root.clone()], true).err(), Some("capture_type_mismatch"));
}

#[test]
fn let_rhs_remains_parallel_and_guards_triggers_are_not_hoisted() {
    let root = integer("root");
    let bound = integer("query");
    let local = integer("prefix");
    let guarded_call = root.new_x(ExpX::Binary(
        sst::BinaryOp::Arith(sst::ArithOp::EuclideanDiv),
        root.clone(),
        bound.clone(),
    ));
    let equality = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Bool),
        ExpX::Binary(sst::BinaryOp::Eq, guarded_call.clone(), local.clone()),
    );
    let guard = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Bool),
        ExpX::Binary(sst::BinaryOp::Eq, bound.clone(), root.clone()),
    );
    let body = guard.new_x(ExpX::Binary(sst::BinaryOp::Implies, guard.clone(), equality));
    let forall = quantifier(&bound, &body, vec![vec![guarded_call], vec![guard.clone()]]);
    let binder = Spanned::new(
        root.span.clone(),
        BndX::Let(Arc::new(vec![Arc::new(VarBinderX { name: name(&local), a: root.clone() })])),
    );
    let original = forall.new_x(ExpX::Bind(binder, forall.clone()));
    let (frozen, captures) = freeze(&original, &[root.clone()], true).unwrap();
    assert_eq!(captures.len(), 1);
    assert!(matches!(captures[0].rhs.x, ExpX::Var(_)));
    let ExpX::Bind(prefix, inner) = &frozen.x else { panic!() };
    let BndX::Let(bindings) = &prefix.x else { panic!() };
    assert_eq!(name(&bindings[0].a), name(&captures[0].read));
    let ExpX::Bind(quantified, guarded) = &inner.x else { panic!() };
    let BndX::Quant(_, binders, triggers, _) = &quantified.x else { panic!() };
    assert_eq!(binders[0].name, name(&bound));
    assert_eq!(triggers.len(), 2);
    assert!(matches!(guarded.x, ExpX::Binary(sst::BinaryOp::Implies, _, _)));
    let replacement = integer("prefix");
    let instance = instantiate(
        &frozen,
        &replacement,
        name(&integer("unique_placeholder")),
        &mut Budget::new(),
    )
    .unwrap();
    let ExpX::Bind(renamed, body) = &instance.x else { panic!() };
    let BndX::Let(renamed) = &renamed.x else { panic!() };
    assert_ne!(renamed[0].name, name(&replacement));
    assert!(matches!(body.x, ExpX::Binary(sst::BinaryOp::Implies, _, _)));
    assert!(crate::sst_util::free_vars_exp(&instance).contains_key(&name(&replacement)));
}

#[test]
fn locations_future_old_and_unsupported_types_decline_without_panics() {
    let root = integer("value");
    for expression in [
        root.new_x(ExpX::VarLoc(name(&root))),
        root.new_x(ExpX::VarAt(name(&root), VarAt::Pre)),
        root.new_x(ExpX::Old(Arc::new("old".into()), name(&root))),
        root.new_x(ExpX::Unary(
            UnaryOp::MutRefFuture(crate::ast::MutRefFutureSourceName::Final),
            root.clone(),
        )),
    ] {
        assert!(freeze(&expression, &[root.clone()], true).is_err());
    }
    let reference =
        variable("reference", Arc::new(TypX::Decorate(TypDecoration::Ref, None, root.typ.clone())));
    assert_eq!(
        freeze(&reference, &[reference.clone()], true).err(),
        Some("unsupported_value_type")
    );
    let pointer = variable(
        "pointer",
        Arc::new(TypX::Primitive(Primitive::Ptr, Arc::new(vec![root.typ.clone()]))),
    );
    assert_eq!(freeze(&pointer, &[pointer.clone()], true).err(), Some("unsupported_value_type"));
}

#[test]
fn condition_setup_policy_and_output_budget_are_explicit() {
    let root = integer("root");
    let empty = Spanned::new(root.span.clone(), StmX::Block(Arc::new(vec![])));
    assert!(empty_setup(&empty));
    assert!(empty_setup(&empty.new_x(StmX::Block(Arc::new(vec![empty.clone()])))));
    assert!(!empty_setup(&crate::ast_to_sst::init_var(&root.span, &name(&root), &root)));
    let formula = quantifier(&root, &root, vec![]);
    let mut budget = Budget { remaining: 1, depth: 0 };
    assert_eq!(
        instantiate(&formula, &root, name(&integer("temporary")), &mut budget).err(),
        Some("budget_exhausted")
    );
    assert_eq!(budget.depth, 0);
}

#[test]
fn bound_mutable_current_is_never_hoisted_to_the_header() {
    let root = variable("bound_receiver", Arc::new(TypX::MutRef(crate::ast_util::int_typ())));
    let current = SpannedTyped::new(
        &root.span,
        &crate::ast_util::int_typ(),
        ExpX::Unary(UnaryOp::MutRefCurrent, root.clone()),
    );
    let body = quantifier(&root, &current, vec![]);
    assert_eq!(freeze(&body, &[root.clone()], true).err(), Some("unsupported_current_root"));
}

fn context() -> Ctx {
    use crate::ast::*;
    use crate::context::{GlobalCtx, WarningCtx};
    let span = crate::infer::tests::span();
    let krate_id = CrateId::Id(Arc::new("loop_test".into()), 1);
    let module = Spanned::new(
        span.clone(),
        ModuleX {
            path: Arc::new(PathX { krate: krate_id.clone(), segments: Arc::new(vec![]) }),
            reveals: None,
        },
    );
    let inferred_fixture =
        crate::infer::tests::function("ordinary_constructor", crate::infer::tests::input("x"));
    let mut function = inferred_fixture.x.clone();
    let mut attributes = function.attrs.as_ref().clone();
    attributes.infer = false;
    function.attrs = Arc::new(attributes);
    let function = inferred_fixture.new_x(function);
    let krate = Arc::new(KrateX {
        functions: vec![function.clone()],
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
        krate_id,
        span,
        10.0,
        Arc::new(std::sync::Mutex::new(None)),
        Arc::new(std::sync::Mutex::new(None)),
        Arc::new(WarningCtx { fun_warn_configs: HashMap::new() }),
        air::context::SmtSolver::Z3,
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
    ctx.fun = crate::ast_to_sst_func::mk_fun_ctx(&function, false);
    ctx
}

fn proof(serial: u64, root: &Exp) -> (Vec<Stm>, sst::LocalDecl) {
    let local = integer(&format!("proof_{serial}"));
    let bound = integer(&format!("query_{serial}"));
    let original = Arc::new(sst::LocalDeclX {
        ident: name(&local),
        typ: local.typ.clone(),
        kind: LocalDeclKind::AssertByVar { native: false },
    });
    let declared = Arc::new(sst::LocalDeclX {
        kind: LocalDeclKind::AssertByVar { native: true },
        ..original.as_ref().clone()
    });
    let binders = Arc::new(vec![Arc::new(VarBinderX { name: name(&bound), a: bound.typ.clone() })]);
    let id = Arc::new(vec![serial]);
    let body = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Bool),
        ExpX::Binary(sst::BinaryOp::Eq, bound.clone(), root.clone()),
    );
    let provenance = Arc::new(sst::AssertByProvenance {
        locals: Arc::new(vec![original]),
        binders: binders.clone(),
        assert_id: Some(id.clone()),
    });
    let quantifier = body.new_x(ExpX::Bind(
        Spanned::new(
            body.span.clone(),
            BndX::Quant(QUANT_FORALL, binders, Arc::new(vec![]), Some(provenance)),
        ),
        body.clone(),
    ));
    let asserted =
        crate::sst_util::subst_exp(&HashMap::new(), &HashMap::from([(name(&bound), local)]), &body);
    let final_assertion = Spanned::new(root.span.clone(), StmX::Assert(Some(id), None, asserted));
    let deadend = final_assertion.new_x(StmX::DeadEnd(
        final_assertion.new_x(StmX::Block(Arc::new(vec![final_assertion.clone()]))),
    ));
    (vec![deadend, final_assertion.new_x(StmX::Assume(quantifier))], declared)
}

fn loop_statement(id: u64, root: &Exp, body: Vec<Stm>, isolated: bool) -> Stm {
    let bound = integer("invariant_query");
    let proposition = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Bool),
        ExpX::Binary(sst::BinaryOp::Eq, bound.clone(), root.clone()),
    );
    Spanned::new(
        root.span.clone(),
        StmX::Loop {
            loop_isolation: isolated,
            pre_stms: Arc::new(vec![]),
            is_for_loop: false,
            id,
            label: crate::ast::Label { id: id as usize, name: None },
            cond: None,
            body: Spanned::new(root.span.clone(), StmX::Block(Arc::new(body))),
            invs: Arc::new(vec![sst::LoopInv {
                at_entry: true,
                at_exit: false,
                inv: quantifier(&bound, &proposition, vec![]),
            }]),
            decrease: Arc::new(vec![]),
            typ_inv_vars: Arc::new(vec![]),
            modified_vars: None,
            au_branch_bool: None,
            pre_modified_params_incl: None,
            pre_modified_params_excl: None,
        },
    )
}

fn check(
    body: Vec<Stm>,
    root: &Exp,
    mut declarations: Vec<sst::LocalDecl>,
) -> (sst::Pars, FuncCheckSst) {
    declarations.push(Arc::new(sst::LocalDeclX {
        ident: name(root),
        typ: root.typ.clone(),
        kind: LocalDeclKind::Param { mutable: false },
    }));
    let parameters = Arc::new(vec![Spanned::new(
        root.span.clone(),
        sst::ParX { name: name(root), typ: root.typ.clone(), mode: crate::ast::Mode::Exec },
    )]);
    let check = FuncCheckSst {
        reqs: Arc::new(vec![]),
        return_opaque_type_eqs: Arc::new(vec![]),
        post_condition: Arc::new(sst::PostConditionSst {
            dest: None,
            ens_exps: Arc::new(vec![]),
            ens_spec_precondition_stms: Arc::new(vec![]),
            kind: sst::PostConditionKind::Ensures,
        }),
        unwind: sst::UnwindSst::MayUnwind,
        body: Spanned::new(root.span.clone(), StmX::Block(Arc::new(body))),
        local_decls: Arc::new(declarations),
        local_decls_decreases_init: Arc::new(vec![]),
        statics: Arc::new(vec![]),
    };
    (parameters, check)
}

#[test]
fn nested_unsupported_sibling_and_ended_loops_keep_exact_lexical_frames() {
    let ctx = context();
    let root = integer("root");
    let (first, first_decl) = proof(1, &root);
    let (inner, inner_decl) = proof(2, &root);
    let (last, last_decl) = proof(3, &root);
    let (sibling, sibling_decl) = proof(4, &root);
    let (ended, ended_decl) = proof(5, &root);
    let inner_loop = loop_statement(20, &root, inner, false);
    let mut outer_body = first;
    outer_body.push(inner_loop.clone());
    outer_body.extend(last);
    let outer = loop_statement(10, &root, outer_body, true);
    let sibling = loop_statement(30, &root, sibling, true);
    let mut body = vec![outer.clone(), sibling];
    body.extend(ended.clone());
    let (parameters, original) =
        check(body, &root, vec![first_decl, inner_decl, last_decl, sibling_decl, ended_decl]);
    let (result, observations) = decorate(&original, &parameters, &ctx);
    assert_eq!(result.local_decls.len(), original.local_decls.len() + 2);
    assert_eq!(
        observations.iter().map(|item| (item.loop_id, item.prepared)).collect::<Vec<_>>(),
        vec![(Some(20), 0), (Some(10), 2), (Some(30), 1)]
    );
    let StmX::Block(body) = &result.body.x else { panic!() };
    assert!(Arc::ptr_eq(&body[2], &ended[0]));
    assert!(Arc::ptr_eq(&body[3], &ended[1]));
    let StmX::Loop { body: outer_body, invs, pre_stms, .. } = &body[0].x else { panic!() };
    let StmX::Loop { invs: original_invs, pre_stms: original_pre, .. } = &outer.x else { panic!() };
    assert!(Arc::ptr_eq(invs, original_invs));
    assert!(Arc::ptr_eq(pre_stms, original_pre));
    let StmX::Block(prefix) = &outer_body.x else { panic!() };
    let StmX::Assign { lhs, rhs } = &prefix[0].x else { panic!() };
    assert!(matches!(lhs.dest.x, ExpX::VarLoc(_)));
    assert!(types_equal(&lhs.dest.typ, &rhs.typ));
    let StmX::Assert(None, None, checked) = &prefix[1].x else { panic!() };
    let StmX::Assume(used) = &prefix[2].x else { panic!() };
    assert!(Arc::ptr_eq(checked, used));
    let StmX::Block(original_body) = &prefix[3].x else { panic!() };
    let StmX::Loop { body, .. } = &original_body[2].x else { panic!() };
    let StmX::Loop { body: original_inner_body, .. } = &inner_loop.x else { panic!() };
    assert!(
        Arc::ptr_eq(body, original_inner_body)
            || matches!(&body.x, StmX::Block(children) if children.len() == 2)
    );
}

#[test]
fn duplicate_ids_and_condition_effects_do_not_prepare_partial_output() {
    let ctx = context();
    let root = integer("root");
    let (proof, declared) = proof(8, &root);
    let original_loop = loop_statement(1, &root, proof, true);
    let (parameters, original) =
        check(vec![original_loop.clone(), original_loop.clone()], &root, vec![declared.clone()]);
    let (result, observations) = decorate(&original, &parameters, &ctx);
    assert_eq!(result.local_decls.len(), original.local_decls.len());
    assert!(observations.iter().all(|item| item.prepared == 0));
    let mut effectful = original_loop.x.clone();
    let StmX::Loop { cond, .. } = &mut effectful else { panic!() };
    *cond = Some((
        crate::ast_to_sst::init_var(&root.span, &name(&root), &root),
        SpannedTyped::new(
            &root.span,
            &Arc::new(TypX::Bool),
            ExpX::Const(crate::ast::Constant::Bool(true)),
        ),
    ));
    let (parameters, original) = check(vec![original_loop.new_x(effectful)], &root, vec![declared]);
    let (result, observations) = decorate(&original, &parameters, &ctx);
    assert_eq!(result.local_decls.len(), original.local_decls.len());
    assert_eq!(observations[0].reason, Some("effectful_or_nonempty_condition_setup"));
}

#[test]
fn inspection_budget_rolls_back_the_entire_staged_function() {
    let ctx = context();
    let root = integer("root");
    let (proof, declared) = proof(10, &root);
    let original_loop = loop_statement(1, &root, proof, true);
    let mut body = vec![original_loop];
    body.extend((0..NODES).map(|_| Spanned::new(root.span.clone(), StmX::Assume(root.clone()))));
    let (parameters, original) = check(body, &root, vec![declared]);
    let (result, observations) = decorate(&original, &parameters, &ctx);
    assert!(Arc::ptr_eq(&result.body, &original.body));
    assert!(Arc::ptr_eq(&result.local_decls, &original.local_decls));
    assert_eq!(observations[0].reason, Some("budget_exhausted"));
    assert_eq!(observations[0].prepared, 0);
}

#[test]
fn ordinary_constructor_eligibility_never_relaxes_discovery() {
    let ctx = context();
    let function = ctx.func_map.get(&ctx.fun.as_ref().unwrap().current_fun).unwrap();
    let mut constructor = function.x.clone();
    let mut parameters = constructor.params.as_ref().clone();
    let mut parameter = parameters[0].x.clone();
    parameter.user_mut = true;
    parameters[0] = parameters[0].new_x(parameter);
    constructor.params = Arc::new(parameters);
    let constructor = function.new_x(constructor);
    assert!(crate::infer::eligibility::authored_loop_points(&constructor).is_none());
    assert!(matches!(
        crate::infer::eligibility::loop_assistance(&constructor),
        Some(crate::infer::report::AssistanceSkip::NonUnitReturn)
    ));
}

#[test]
fn abstract_associated_and_generic_values_use_existing_typed_snapshots() {
    use crate::ast::*;
    let path = Arc::new(PathX {
        krate: CrateId::Id(Arc::new("unrelated".into()), 2),
        segments: Arc::new(vec![Arc::new("Hidden".into())]),
    });
    let datatype = Dt::Path(path.clone());
    let definition = Spanned::new(
        crate::infer::tests::span(),
        DatatypeX {
            name: datatype.clone(),
            proxy: None,
            owning_module: None,
            visibility: Visibility { restricted_to: None },
            transparency: DatatypeTransparency::Never,
            typ_params: Arc::new(vec![(Arc::new("T".into()), AcceptRecursiveType::RejectInGround)]),
            typ_bounds: Arc::new(vec![]),
            variants: Arc::new(vec![]),
            mode: Mode::Exec,
            ext_equal: false,
            user_defined_invariant_fn: None,
            sized_constraint: None,
            destructor: false,
        },
    );
    let datatypes = HashMap::from([(datatype.clone(), definition)]);
    let generic = Arc::new(TypX::TypParam(Arc::new("Arbitrary".into())));
    let abstract_typ =
        Arc::new(TypX::Datatype(datatype, Arc::new(vec![generic.clone()]), Arc::new(vec![])));
    let associated = Arc::new(TypX::Projection {
        trait_typ_args: Arc::new(vec![abstract_typ.clone()]),
        trait_path: path.clone(),
        name: Arc::new("Item".into()),
    });
    let opaque =
        Arc::new(TypX::Opaque { def_path: path, args: Arc::new(vec![associated.clone()]) });
    for typ in [generic, abstract_typ, associated, opaque.clone(), Arc::new(TypX::Boxed(opaque))] {
        let root = variable("logical_value", typ);
        let (frozen, captures) = freeze_using(&root, &[root.clone()], true, &datatypes).unwrap();
        assert_eq!(captures.len(), 1);
        assert!(types_equal(&frozen.typ, &root.typ));
        assert!(Arc::ptr_eq(&captures[0].rhs, &root));
        let initialized = crate::ast_to_sst::init_var(
            &root.span,
            &captures[0].declaration.ident,
            &captures[0].rhs,
        );
        let StmX::Assign { lhs, rhs } = &initialized.x else { panic!() };
        assert!(matches!(lhs.dest.x, ExpX::VarLoc(_)));
        assert!(types_equal(&lhs.dest.typ, &rhs.typ));
    }
    let raw = Arc::new(TypX::Primitive(Primitive::Ptr, Arc::new(vec![crate::ast_util::int_typ()])));
    let bad = Arc::new(TypX::Datatype(
        datatypes.keys().next().unwrap().clone(),
        Arc::new(vec![raw]),
        Arc::new(vec![]),
    ));
    assert_eq!(value_type(&datatypes, &bad, &mut Budget::new(), 0), Err("unsupported_value_type"));
}

fn whole_assignment(root: &Exp) -> Stm {
    let location = root.new_x(ExpX::VarLoc(name(root)));
    let location = location.new_x(ExpX::Loc(location.clone()));
    Spanned::new(
        root.span.clone(),
        StmX::Assign {
            lhs: sst::Dest { dest: location, is_init: false },
            rhs: root.new_x(ExpX::Const(crate::ast::Constant::Int(0.into()))),
        },
    )
}

fn prepared_after(root: &Exp, prefix: Vec<Stm>) -> usize {
    let argument = integer("argument");
    let (proof, proof_decl) = proof(90, root);
    let mut body = prefix;
    body.push(loop_statement(90, root, proof, true));
    let (parameters, original) = check(body, &argument, vec![declaration(root), proof_decl]);
    let (_, observations) = decorate(&original, &parameters, &context());
    observations.iter().filter(|item| item.loop_id == Some(90)).map(|item| item.prepared).sum()
}

#[test]
fn lowered_noninit_loc_assignments_dominate_across_nested_straight_line_blocks() {
    let root = integer("ghost_snapshot");
    let assignment = whole_assignment(&root);
    let nested = assignment.new_x(StmX::Block(Arc::new(vec![assignment.clone()])));
    let nested = nested.new_x(StmX::Block(Arc::new(vec![nested.clone()])));
    assert_eq!(prepared_after(&root, vec![nested]), 1);
    let StmX::Assign { lhs, .. } = &assignment.x else { panic!() };
    let mismatch = assignment.new_x(StmX::Assign {
        lhs: lhs.clone(),
        rhs: variable("wrong_type", Arc::new(TypX::Bool)),
    });
    assert_eq!(prepared_after(&root, vec![mismatch]), 0);
}

#[test]
fn branch_joins_require_assignment_on_every_reaching_arm() {
    let root = integer("branch_snapshot");
    let assignment = whole_assignment(&root);
    let condition = SpannedTyped::new(
        &root.span,
        &Arc::new(TypX::Bool),
        ExpX::Const(crate::ast::Constant::Bool(true)),
    );
    let branch = assignment.new_x(StmX::If(
        condition.clone(),
        assignment.clone(),
        Some(assignment.new_x(StmX::Block(Arc::new(vec![assignment.clone()])))),
    ));
    assert_eq!(prepared_after(&root, vec![branch]), 1);
    let one_arm = assignment.new_x(StmX::If(condition, assignment.clone(), None));
    assert_eq!(prepared_after(&root, vec![one_arm]), 0);
}

#[test]
fn proof_deadends_loop_bodies_and_partial_locations_never_initialize_roots() {
    let root = integer("not_live");
    let assignment = whole_assignment(&root);
    assert_eq!(prepared_after(&root, vec![assignment.new_x(StmX::DeadEnd(assignment.clone()))]), 0);
    let inner = loop_statement(7, &root, vec![assignment.clone()], true);
    assert_eq!(prepared_after(&root, vec![inner]), 0);
    for location in [
        root.new_x(ExpX::Unary(UnaryOp::MutRefCurrent, root.clone())),
        root.new_x(ExpX::UnaryOpr(crate::ast::UnaryOpr::Unbox(root.typ.clone()), root.clone())),
        root.new_x(ExpX::UnaryOpr(
            crate::ast::UnaryOpr::Field(crate::ast::FieldOpr {
                datatype: Dt::Tuple(1),
                variant: Arc::new("Tuple".into()),
                field: Arc::new("0".into()),
                get_variant: false,
                check: crate::ast::VariantCheck::None,
            }),
            root.new_x(ExpX::VarLoc(name(&root))),
        )),
    ] {
        let partial = assignment.new_x(StmX::Assign {
            lhs: sst::Dest { dest: location, is_init: false },
            rhs: root.clone(),
        });
        assert_eq!(prepared_after(&root, vec![partial]), 0);
    }
}
