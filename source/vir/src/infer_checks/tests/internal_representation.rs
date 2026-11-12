//! Stage A diagnostic IR tests: actual ordinary backend; no source installation.
use super::*;
use crate::internal_repr::{CORE_REV_IDENTITY, InternalRepresentation};

fn path(parts: &[&str]) -> Path {
    Arc::new(PathX {
        krate: CrateId::Core,
        segments: Arc::new(parts.iter().map(|s| Arc::new(s.to_string())).collect()),
    })
}
fn owner() -> Path {
    path(&["iter", "adapters", "rev"])
}
fn definition() -> Datatype {
    let i = Arc::new("I".to_string());
    let mut d = DatatypeX {
        name: Dt::Path(path(&["iter", "adapters", "rev", "Rev"])),
        proxy: None,
        owning_module: Some(owner()),
        visibility: Visibility { restricted_to: None },
        transparency: DatatypeTransparency::Never,
        typ_params: Arc::new(vec![(i.clone(), AcceptRecursiveType::Reject)]),
        typ_bounds: Arc::new(vec![]),
        variants: Arc::new(vec![Variant {
            name: Arc::new("Rev".into()),
            ctor_style: CtorPrintStyle::Braces,
            fields: Arc::new(vec![crate::ast_util::ident_binder(
                &crate::def::field_ident_from_rust("iter"),
                &(
                    Arc::new(TypX::TypParam(i)),
                    Mode::Exec,
                    Visibility { restricted_to: Some(owner()) },
                ),
            )]),
        }]),
        mode: Mode::Exec,
        ext_equal: false,
        user_defined_invariant_fn: None,
        sized_constraint: None,
        destructor: false,
    };
    d.transparency = DatatypeTransparency::Internal(
        InternalRepresentation::core_rev(&d, CORE_REV_IDENTITY).unwrap(),
    );
    Spanned::new(span(), d)
}
fn typ(d: &Datatype, arg: Typ) -> Typ {
    Arc::new(TypX::Datatype(d.x.name.clone(), Arc::new(vec![arg]), Arc::new(vec![])))
}
fn field(d: &Datatype) -> FieldOpr {
    FieldOpr {
        datatype: d.x.name.clone(),
        variant: Arc::new("Rev".into()),
        field: crate::def::field_ident_from_rust("iter"),
        get_variant: false,
        check: VariantCheck::None,
    }
}
fn select(d: &Datatype, arg: Typ, e: Exp) -> Exp {
    SpannedTyped::new(&span(), &arg, ExpX::UnaryOpr(UnaryOpr::Field(field(d)), e))
}
fn construct(d: &Datatype, arg: Typ, e: Exp) -> Exp {
    SpannedTyped::new(
        &span(),
        &typ(d, arg),
        ExpX::Ctor(
            d.x.name.clone(),
            Arc::new("Rev".into()),
            Arc::new(vec![crate::ast_util::ident_binder(
                &crate::def::field_ident_from_rust("iter"),
                &e,
            )]),
        ),
    )
}
fn bare_krate(d: Datatype, module: Path) -> Krate {
    Arc::new(KrateX {
        functions: vec![],
        reveal_groups: vec![],
        datatypes: vec![d],
        traits: vec![],
        trait_impls: vec![],
        assoc_type_impls: vec![],
        modules: vec![Spanned::new(span(), ModuleX { path: module, reveals: None })],
        external_fns: vec![],
        external_types: vec![],
        path_as_rust_names: vec![],
        opaque_types: vec![],
        arch: Arch { word_bits: ArchWordBits::Exactly(64) },
    })
}

#[test]
fn internal_representation_preserves_source_access_and_old_modes() {
    let d = definition();
    let foreign = path(&["consumer"]);
    let child = path(&["iter", "adapters", "rev", "child"]);
    let k = bare_krate(d.clone(), foreign.clone());
    let p = d.x.name.expect_path();
    for m in [owner(), child] {
        assert!(crate::ast_util::is_transparent_to(&d.x.transparency, &m));
        assert!(crate::well_formed::internal_repr_test_source_access(&k, &p, Some(m), false));
    }
    assert!(!crate::well_formed::internal_repr_test_source_access(
        &k,
        &p,
        Some(foreign.clone()),
        false
    ));
    assert!(!crate::well_formed::internal_repr_test_source_access(&k, &p, Some(owner()), true));
    assert!(!crate::ast_util::is_transparent_to(&d.x.transparency, &foreign));
    for module in [owner(), foreign] {
        assert!(crate::datatype_to_air::is_datatype_encoded(&module, &d));
        assert!(!crate::ast_util::is_encoded_to(&DatatypeTransparency::Never, &module));
        let old = DatatypeTransparency::WhenVisible(Visibility { restricted_to: Some(owner()) });
        assert_eq!(
            crate::ast_util::is_encoded_to(&old, &module),
            crate::ast_util::is_transparent_to(&old, &module)
        );
    }
}

#[test]
fn internal_representation_rejects_wire_and_detects_without_helpers() {
    let d = definition();
    assert!(serde_json::to_string(&d.x).is_err());
    assert!(serde_json::from_str::<DatatypeTransparency>(r#"{"Internal":{}}"#).is_err());
    assert_eq!(serde_json::to_string(&DatatypeTransparency::Never).unwrap(), "\"Never\"");
    let k = bare_krate(d, path(&["consumer"]));
    assert!(k.functions.is_empty());
    assert!(crate::internal_repr::contains_internal_representation(&k));
}

#[test]
fn internal_representation_revalidates_actual_corrupted_datatype() {
    let d = definition();
    assert!(
        InternalRepresentation::core_rev(&d.x, (CORE_REV_IDENTITY.0, CORE_REV_IDENTITY.1, 0))
            .is_err()
    );
    for change in 0..13 {
        let mut x = d.x.clone();
        match change {
            0 => x.name = Dt::Path(path(&["lookalike", "Rev"])),
            1 => x.destructor = true,
            2 => x.ext_equal = true,
            3 => x.mode = Mode::Spec,
            4 => x.visibility = Visibility { restricted_to: Some(owner()) },
            5 => x.typ_params = Arc::new(vec![]),
            6 => Arc::make_mut(&mut x.typ_params)[0].1 = AcceptRecursiveType::Accept,
            7 => Arc::make_mut(&mut x.variants)[0].fields = Arc::new(vec![]),
            8 => {
                Arc::make_mut(
                    Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)
                        .get_mut(0)
                        .unwrap(),
                )
                .a
                .0 = bool_typ()
            }
            9 => {
                Arc::make_mut(
                    Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)
                        .get_mut(0)
                        .unwrap(),
                )
                .a
                .2 = Visibility { restricted_to: None }
            }
            10 => x.owning_module = Some(path(&["wrong_owner"])),
            11 => x.sized_constraint = Some(bool_typ()),
            12 => {
                Arc::make_mut(
                    Arc::make_mut(&mut Arc::make_mut(&mut x.variants)[0].fields)
                        .get_mut(0)
                        .unwrap(),
                )
                .name = Arc::new("wrong".into())
            }
            _ => unreachable!(),
        }
        let actual = Spanned::new(span(), x);
        assert!(crate::internal_repr::validate_datatype(&actual).is_err(), "corruption {change}");
    }
}

#[test]
fn internal_representation_generic_and_closed_types_are_one_nominal_sort() {
    let d = definition();
    let (ctx, _, _) = fixture_with_datatypes(vec![d.clone()]);
    for arg in [
        bool_typ(),
        Arc::new(TypX::Int(IntRange::U(64))),
        Arc::new(TypX::TypParam(Arc::new("T".into()))),
    ] {
        let t = typ(&d, arg);
        assert!(!crate::poly::typ_is_poly(&ctx, &t));
        assert!(crate::ast_util::types_equal(&crate::poly::coerce_typ_to_native(&ctx, &t), &t));
        assert!(
            matches!(&*crate::poly::coerce_typ_to_poly(&ctx,&t),TypX::Boxed(inner) if crate::ast_util::types_equal(inner,&t))
        );
    }
}

fn check_expression(
    d: &Datatype,
    args: Vec<(VarIdent, Typ)>,
    expression: Exp,
    already_encoded: bool,
) -> bool {
    let (ctx, _, mut check) = fixture_with_datatypes(vec![d.clone()]);
    check.reqs = Arc::new(vec![]);
    check.post_condition = Arc::new(PostConditionSst {
        dest: None,
        ens_exps: Arc::new(vec![]),
        ens_spec_precondition_stms: Arc::new(vec![]),
        kind: PostConditionKind::Ensures,
    });
    check.local_decls = Arc::new(
        args.iter()
            .map(|(n, t)| {
                Arc::new(LocalDeclX {
                    ident: n.clone(),
                    typ: t.clone(),
                    kind: LocalDeclKind::Param { mutable: false },
                })
            })
            .collect(),
    );
    check.body = Spanned::new(
        span(),
        StmX::Assert(None, Some(error(&span(), "internal nominal computation")), expression),
    );
    let check =
        if already_encoded { check } else { crate::poly::internal_repr_test_check(&ctx, &check) };
    let pars = Arc::new(
        args.iter()
            .map(|(n, t)| {
                Spanned::new(span(), ParX { name: n.clone(), typ: t.clone(), mode: Mode::Exec })
            })
            .collect(),
    );
    verifies(&ctx, &pars, &check)
}
#[test]
fn internal_representation_datatype_computation_equality_and_boxing_use_backend() {
    let d = definition();
    let u8t = Arc::new(TypX::Int(IntRange::U(8)));
    let rtyp = typ(&d, u8t.clone());
    let a = SpannedTyped::new(&span(), &u8t, ExpX::Var(var("a")));
    let b = SpannedTyped::new(&span(), &u8t, ExpX::Var(var("b")));
    let r = SpannedTyped::new(&span(), &rtyp, ExpX::Var(var("r")));
    let c = construct(&d, u8t.clone(), a.clone());
    let eq = |a, b| binary(crate::sst::BinaryOp::Eq, a, b);
    let args = vec![(var("a"), u8t.clone()), (var("b"), u8t.clone()), (var("r"), rtyp.clone())];
    assert!(check_expression(
        &d,
        args.clone(),
        eq(select(&d, u8t.clone(), c.clone()), a.clone()),
        false
    ));
    assert!(check_expression(
        &d,
        args.clone(),
        eq(construct(&d, u8t.clone(), select(&d, u8t.clone(), r.clone())), r.clone()),
        false
    ));
    assert!(check_expression(
        &d,
        args.clone(),
        eq(eq(c.clone(), construct(&d, u8t.clone(), b.clone())), eq(a.clone(), b.clone())),
        false
    ));
    let boxed = SpannedTyped::new(
        &span(),
        &Arc::new(TypX::Boxed(rtyp.clone())),
        ExpX::UnaryOpr(UnaryOpr::Box(rtyp.clone()), r.clone()),
    );
    let unboxed =
        SpannedTyped::new(&span(), &rtyp, ExpX::UnaryOpr(UnaryOpr::Unbox(rtyp.clone()), boxed));
    assert!(check_expression(&d, args.clone(), eq(unboxed, r), true));
    assert!(!check_expression(&d, args, eq(select(&d, u8t, c), b), false));
}

#[test]
fn internal_representation_ordinary_field_reborrow_and_writeback() {
    let d = definition();
    let (ctx, _, mut check) = fixture_with_datatypes(vec![d.clone()]);
    let u8t = Arc::new(TypX::Int(IntRange::U(8)));
    let rtyp = typ(&d, u8t.clone());
    let qtyp = Arc::new(TypX::MutRef(rtyp.clone()));
    let root = SpannedTyped::new(&span(), &qtyp, PlaceX::Local(var("q")));
    let deref = SpannedTyped::new(&span(), &rtyp, PlaceX::DerefMut(root));
    let p = SpannedTyped::new(&span(), &u8t, PlaceX::Field(field(&d), deref));
    let e = SpannedTyped::new(&span(), &Arc::new(TypX::MutRef(u8t.clone())), ExprX::BorrowMut(p));
    let (mut stms, writeback, reference, locals) =
        crate::ast_to_sst::internal_repr_test_borrow(&ctx, &Silent, &var("q"), &qtyp, &e);
    let q = SpannedTyped::new(&span(), &qtyp, ExpX::Var(var("q")));
    let saved = var("saved_outer_future");
    stms.insert(
        0,
        crate::ast_to_sst::init_var(
            &span(),
            &saved,
            &crate::sst_util::sst_mut_ref_future(&span(), &q),
        ),
    );
    let current = select(&d, u8t.clone(), crate::sst_util::sst_mut_ref_current(&span(), &q));
    let equality = binary(
        crate::sst::BinaryOp::Eq,
        current,
        crate::sst_util::sst_mut_ref_current(&span(), &reference),
    );
    stms.push(Spanned::new(
        span(),
        StmX::Assert(None, Some(error(&span(), "field reborrow current")), equality),
    ));
    stms.push(writeback);
    let after = select(&d, u8t, crate::sst_util::sst_mut_ref_current(&span(), &q));
    let equality = binary(
        crate::sst::BinaryOp::Eq,
        after,
        crate::sst_util::sst_mut_ref_future(&span(), &reference),
    );
    stms.push(Spanned::new(
        span(),
        StmX::Assert(None, Some(error(&span(), "field reborrow writeback")), equality),
    ));
    let saved_exp = SpannedTyped::new(&span(), &rtyp, ExpX::Var(saved.clone()));
    let unchanged = binary(
        crate::sst::BinaryOp::Eq,
        crate::sst_util::sst_mut_ref_future(&span(), &q),
        saved_exp,
    );
    stms.push(Spanned::new(
        span(),
        StmX::Assert(
            None,
            Some(error(&span(), "outer future preserved by field writeback")),
            unchanged,
        ),
    ));
    let mut locals = locals;
    locals.push(Arc::new(LocalDeclX {
        ident: saved,
        typ: rtyp,
        kind: LocalDeclKind::StmtLet { mutable: false },
    }));
    check.reqs = Arc::new(vec![]);
    check.local_decls = Arc::new(locals);
    check.post_condition = Arc::new(PostConditionSst {
        dest: None,
        ens_exps: Arc::new(vec![]),
        ens_spec_precondition_stms: Arc::new(vec![]),
        kind: PostConditionKind::Ensures,
    });
    check.body = Spanned::new(span(), StmX::Block(Arc::new(stms)));
    let pars =
        Arc::new(vec![Spanned::new(span(), ParX { name: var("q"), typ: qtyp, mode: Mode::Exec })]);
    let check = crate::poly::internal_repr_test_check(&ctx, &check);
    assert!(verifies(&ctx, &pars, &check));
    // Without the ordinary writeback statement, the asserted future field
    // relation is not available. No added selector/future axiom repairs it.
    let mut omitted = check.clone();
    omitted.body = crate::sst_visitor::map_stm_visitor(&check.body, &mut |stm| {
        Ok(if matches!(&stm.x,StmX::Assign{lhs,..} if !lhs.is_init) {
            Spanned::new(stm.span.clone(), StmX::Block(Arc::new(vec![])))
        } else {
            stm.clone()
        })
    })
    .unwrap();
    assert!(!verifies(&ctx, &pars, &omitted));
}

#[test]
fn internal_representation_resolvedness_follows_internal_fields_in_foreign_modules() {
    let d = definition();
    let foreign = path(&["consumer"]);
    let k = bare_krate(d.clone(), foreign.clone());
    let mut collect = crate::resolve_axioms::ResolvedTypeCollection::new(&foreign, &k);
    collect.visit_type(&typ(&d, Arc::new(TypX::MutRef(bool_typ()))));
    assert!(
        collect.finish().iter().any(
            |t| matches!(t,crate::resolve_axioms::ResolvableType::Datatype(dt) if dt==&d.x.name)
        )
    );
}

#[test]
fn internal_representation_pruning_retains_fields_and_never_makes_abstract_sorts() {
    let d = definition();
    let foreign = path(&["consumer"]);
    // A local selected definition roots the type in both diagnostic modules;
    // source field visibility remains the real private core owner.
    for module in [owner(), foreign] {
        let mut x = d.x.clone();
        x.owning_module = Some(module.clone());
        let repr = InternalRepresentation::core_rev(&x, CORE_REV_IDENTITY).unwrap();
        x.transparency = DatatypeTransparency::Internal(repr);
        let k = bare_krate(Spanned::new(span(), x), module.clone());
        let (pruned, info) = crate::prune::prune_krate_for_module_or_krate(
            &k,
            &CrateId::Core,
            None,
            Some(module.clone()),
            None,
            true,
            true,
        );
        assert_eq!(pruned.datatypes.len(), 1);
        assert_eq!(pruned.datatypes[0].x.variants[0].fields.len(), 1);
        assert!(info.mono_abstract_datatypes.unwrap().is_empty());
        assert!(crate::datatype_to_air::is_datatype_encoded(&module, &pruned.datatypes[0]));
        crate::internal_repr::validate_datatype(&pruned.datatypes[0]).unwrap();
    }
}
// Synthetic backend-boundary controls only: these metadata fixtures do not
// claim to pass the full source/WF pipeline or prove a native body.
fn owned_cell_definition() -> Datatype {
    let mut d = definition().x.clone();
    let name = path(&["cell", "UnsafeCell"]);
    let module = name.pop_segment();
    let param: Ident = Arc::new("T".into());
    let t = Arc::new(TypX::TypParam(param.clone()));
    d.name = Dt::Path(name);
    d.owning_module = Some(module.clone());
    d.transparency = DatatypeTransparency::Never;
    d.typ_params = Arc::new(vec![(param, AcceptRecursiveType::Reject)]);
    d.sized_constraint = Some(t.clone());
    d.variants = Arc::new(vec![Variant {
        name: Arc::new("UnsafeCell".into()),
        ctor_style: CtorPrintStyle::Braces,
        fields: Arc::new(vec![crate::ast_util::ident_binder(
            &crate::def::field_ident_from_rust("value"),
            &(t, Mode::Exec, Visibility { restricted_to: Some(module) }),
        )]),
    }]);
    d.transparency = DatatypeTransparency::Internal(
        InternalRepresentation::owned_unsafe_cell_u64(
            &d,
            crate::internal_repr::CORE_OWNED_CELL_IDENTITY,
        )
        .unwrap(),
    );
    Spanned::new(span(), d)
}

fn metadata_backend_accepts(k: &Krate, base: &Krate) -> bool {
    let global = GlobalCtx::new(
        base,
        CrateId::Core,
        span(),
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
    Ctx::new(
        k,
        global,
        Spanned::new(span(), ModuleX { path: path(&["consumer"]), reveals: None }),
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
    .is_ok()
}

#[test]
fn internal_representation_checks_owned_cell_types_in_all_metadata() {
    let d = owned_cell_definition();
    let base = bare_krate(d.clone(), path(&["consumer"]));
    let p = path(&["test_trait"]);
    let labels = [
        "trait implementation arguments",
        "trait implementation bounds",
        "associated implementation RHS",
        "associated implementation arguments",
        "associated implementation bounds",
        "trait declaration bounds",
        "trait associated-type bounds",
        "opaque parameters",
        "opaque bounds",
    ];
    let mut failures = vec![];
    for width in [32, 64] {
        let t = typ(&d, Arc::new(TypX::Int(IntRange::U(width))));
        let bounds = Arc::new(vec![Arc::new(GenericBoundX::Trait(
            TraitId::Path(p.clone()),
            Arc::new(vec![t.clone()]),
        ))]);
        for (placement, label) in labels.iter().enumerate() {
            let mut k = (*base).clone();
            match placement {
                0 | 1 => k.trait_impls.push(Spanned::new(
                    span(),
                    TraitImplX {
                        impl_path: p.clone(),
                        typ_params: Arc::new(vec![]),
                        typ_bounds: if placement == 1 { bounds.clone() } else { Arc::new(vec![]) },
                        trait_path: p.clone(),
                        trait_typ_args: Arc::new(if placement == 0 {
                            vec![t.clone()]
                        } else {
                            vec![]
                        }),
                        trait_typ_arg_impls: Spanned::new(span(), Arc::new(vec![])),
                        owning_module: None,
                        auto_imported: false,
                        external_trait_blanket: false,
                    },
                )),
                2 | 3 | 4 => k.assoc_type_impls.push(Spanned::new(
                    span(),
                    AssocTypeImplX {
                        name: Arc::new("A".into()),
                        impl_path: p.clone(),
                        typ_params: Arc::new(vec![]),
                        typ_bounds: if placement == 4 { bounds.clone() } else { Arc::new(vec![]) },
                        trait_path: p.clone(),
                        trait_typ_args: Arc::new(if placement == 3 {
                            vec![t.clone()]
                        } else {
                            vec![]
                        }),
                        typ: if placement == 2 { t.clone() } else { Arc::new(TypX::Bool) },
                        impl_paths: Arc::new(vec![]),
                    },
                )),
                5 | 6 => k.traits.push(Spanned::new(
                    span(),
                    TraitX {
                        name: p.clone(),
                        proxy: None,
                        visibility: Visibility { restricted_to: None },
                        typ_params: Arc::new(vec![]),
                        typ_bounds: if placement == 5 { bounds.clone() } else { Arc::new(vec![]) },
                        assoc_typs: Arc::new(vec![]),
                        assoc_typs_bounds: if placement == 6 {
                            bounds.clone()
                        } else {
                            Arc::new(vec![])
                        },
                        methods: Arc::new(vec![]),
                        is_unsafe: false,
                        dyn_compatible: None,
                        external_trait_extension: None,
                    },
                )),
                7 | 8 => k.opaque_types.push(Spanned::new(
                    span(),
                    OpaqueTypeX {
                        def_fun: Arc::new(FunX { path: p.clone() }),
                        name: p.clone(),
                        typ_params: Arc::new(if placement == 7 { vec![t.clone()] } else { vec![] }),
                        typ_bounds: if placement == 8 { bounds.clone() } else { Arc::new(vec![]) },
                    },
                )),
                _ => unreachable!(),
            }
            let k = Arc::new(k);
            let expected = width == 64;
            let direct = crate::internal_repr::validate_krate_uses(&k).is_ok();
            let backend = metadata_backend_accepts(&k, &base);
            if direct != expected || backend != expected {
                failures.push(format!(
                    "{label}: C<u{width}>, expected={expected}, direct={direct}, backend={backend}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
