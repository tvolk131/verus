//! Synthetic structural controls on in-memory clones. No core/toolchain patch,
//! source proof or native correspondence is inferred from these tests.
use super::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
struct Structural;
impl rustc_driver::Callbacks for Structural {
    fn after_analysis<'tcx>(
        &mut self,
        _: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        let local = tcx
            .hir_body_owners()
            .find(|id| tcx.item_name(id.to_def_id()).as_str() == "total")
            .unwrap();
        let caller = local.to_def_id();
        let spans = crate::spans::SpanContextX::new(
            tcx,
            tcx.stable_crate_id(rustc_span::def_id::LOCAL_CRATE),
            tcx.sess.source_map(),
            HashMap::new(),
            None,
        );
        let erasure = crate::context::ErasureInfo {
            local_hir_vir_ids: vec![],
            binder_hir_vir_ids: vec![],
            ctor_hir_vir_ids: vec![],
            resolved_calls: vec![],
            resolved_pats: vec![],
            direct_var_modes: vec![],
            ignored_functions: vec![],
            bodies: vec![],
            shadow_check: vec![],
            extra_erase_ast_ids: vec![],
            local_invariant_bodies: vec![],
        };
        let ctxt = Rc::new(crate::context::ContextX::new(
            Arc::new(crate::config::ArgsX::new()),
            tcx,
            Rc::new(RefCell::new(erasure)),
            spans,
            Arc::new(crate::verus_items::from_diagnostic_items(tcx)),
            false,
            crate::rust_to_vir_base::mk_crate_id(tcx, rustc_span::def_id::LOCAL_CRATE),
        ));
        let mut calls = Calls { ctxt: &ctxt, caller, types: tcx.typeck(local), found: vec![] };
        use rustc_hir::intravisit::Visitor;
        let body = tcx.hir_body_owned_by(local);
        calls.visit_body(body);
        assert_eq!(calls.found.len(), 1);
        let (e, ids) = calls.found.pop().unwrap();
        let env = ty::TypingEnv::non_body_analysis(tcx, caller);
        let ord = tcx.get_diagnostic_item(Symbol::intern("Ord")).unwrap();
        let cmp = method(tcx, ord, "cmp").unwrap();
        for t in [tcx.types.u32, tcx.types.i64, tcx.types.u128] {
            assert!(resolve_ordering(tcx, env, cmp, tcx.mk_args(&[t.into()])).is_err());
        }
        let [carrier, get, primitive] = bodies(tcx, env, &ids);
        inspect_bodies(tcx, env, &ids, &carrier, &get, &primitive).unwrap();
        // Actual concretely instantiated generic-origin flags are accepted.
        assert!(carrier.is_polymorphic && get.is_polymorphic);
        for (s, a, b) in [
            (4, 8, 4),
            (4, 5, 7),
            (11, 5, 7),
            (4, 5, 5),
            (3, 5, 4),
            (usize::MAX, 5, 4),
            (4, usize::MAX, 4),
        ] {
            assert!(check_get_expansion(tcx, &carrier, &get, s, a, b).is_err());
        }
        // Whole executable body mutation, independently of an expansion descriptor.
        for mutation in 0..13 {
            let mut c = carrier.clone();
            let blocks = c.basic_blocks.as_mut();
            match mutation {
                0 => blocks[mir::START_BLOCK].statements.swap(3, 10),
                1 => blocks[mir::START_BLOCK].statements.swap(4, 11),
                2 => {
                    let s = blocks[mir::START_BLOCK].statements[3].clone();
                    blocks[mir::START_BLOCK].statements.push(s);
                }
                3 => {
                    let b = blocks[mir::BasicBlock::from_usize(1)].clone();
                    blocks.push(b);
                }
                4 => blocks[mir::BasicBlock::from_usize(1)].is_cleanup = true,
                5 => blocks[mir::START_BLOCK].statements.swap(6, 3),
                6 => blocks[mir::START_BLOCK].statements.swap(12, 10),
                7 => {
                    let mir::TerminatorKind::Call { args, .. } =
                        &mut blocks[mir::START_BLOCK].terminator_mut().kind
                    else {
                        unreachable!()
                    };
                    args.swap(0, 1);
                }
                8 => {
                    let mir::TerminatorKind::Call { destination, .. } =
                        &mut blocks[mir::START_BLOCK].terminator_mut().kind
                    else {
                        unreachable!()
                    };
                    *destination = mir::Place::from(mir::Local::from_usize(4));
                }
                9 => {
                    let mir::TerminatorKind::Call { unwind, .. } =
                        &mut blocks[mir::START_BLOCK].terminator_mut().kind
                    else {
                        unreachable!()
                    };
                    *unwind = mir::UnwindAction::Unreachable;
                }
                10 => blocks[mir::START_BLOCK].terminator_mut().kind = mir::TerminatorKind::Return,
                11 => {
                    let s = blocks[mir::START_BLOCK].statements[3].clone();
                    blocks[mir::BasicBlock::from_usize(1)].statements.push(s);
                }
                12 => {
                    blocks[mir::START_BLOCK].statements[6].kind =
                        mir::StatementKind::StorageDead(mir::Local::from_usize(4))
                }
                _ => unreachable!(),
            }
            assert!(
                inspect_bodies(tcx, env, &ids, &c, &get, &primitive).is_err(),
                "MIR mutation {mutation}"
            );
        }
        let path = crate::test_verus_root().join("vstd.vir");
        let mut file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        let k: Krate =
            bincode_next::serde::decode_from_std_read(&mut file, bincode_next::config::legacy())
                .unwrap();
        let models = bind_models(&ctxt, &ids, &[k]).unwrap();
        let original = translate(&ctxt, caller, e.span, &ids, &models).unwrap();
        check_ordering_function(
            &ctxt, caller, e.span, &ids, &carrier, &get, &primitive, &models, &original,
        )
        .unwrap();
        // Embedded implementation witnesses are not diagnostic spans. Keep
        // nominal type/arguments/decoration unchanged while editing only paths.
        for change in 0..3 {
            for parameter in 0..2 {
                let mut x = original.x.clone();
                let mut param = x.params[parameter].x.clone();
                param.typ = altered_type_dependencies(&param.typ, change).unwrap();
                Arc::make_mut(&mut x.params)[parameter] = x.params[parameter].new_x(param);
                assert!(
                    check_ordering_function(
                        &ctxt,
                        caller,
                        e.span,
                        &ids,
                        &carrier,
                        &get,
                        &primitive,
                        &models,
                        &original.new_x(x),
                    )
                    .is_err(),
                    "parameter implementation paths {parameter}/{change}"
                );
            }
            for contract in [false, true] {
                let mut x = original.x.clone();
                let edits = std::cell::Cell::new(0);
                let mutate = |e: &Expr| -> Result<Expr, VirErr> {
                    if let Some(t) = altered_type_dependencies(&e.typ, change) {
                        edits.set(edits.get() + 1);
                        Ok(SpannedTyped::new(&e.span, &t, e.x.clone()))
                    } else {
                        Ok(e.clone())
                    }
                };
                if contract {
                    Arc::make_mut(&mut x.ensure.0)[0] =
                        vir::ast_visitor::map_expr_visitor(&x.ensure.0[0], &mutate).unwrap();
                } else {
                    x.body = Some(
                        vir::ast_visitor::map_expr_visitor(x.body.as_ref().unwrap(), &mutate)
                            .unwrap(),
                    );
                }
                assert!(edits.get() > 0);
                assert!(
                    check_ordering_function(
                        &ctxt,
                        caller,
                        e.span,
                        &ids,
                        &carrier,
                        &get,
                        &primitive,
                        &models,
                        &original.new_x(x),
                    )
                    .is_err(),
                    "nested implementation paths contract={contract}/{change}"
                );
            }
        }
        // Additional model/type/operand/constructor changes in the actual
        // Function, without changing the recognizer's expansion maps.
        for mutation in 0..8 {
            let mut x = original.x.clone();
            let mutate = |e: &Expr| -> Result<Expr, VirErr> {
                let mut changed = e.x.clone();
                match (&mut changed, mutation) {
                    (ExprX::Call { target: CallTarget::Fun(_, name, types, ..), .. }, 0)
                        if *name == models.get =>
                    {
                        *name = models.primitive.clone()
                    }
                    (ExprX::Call { target: CallTarget::Fun(_, name, types, ..), .. }, 1)
                        if *name == models.get =>
                    {
                        Arc::make_mut(types)[0] = Arc::new(TypX::Int(IntRange::U(32)))
                    }
                    (ExprX::Call { args, .. }, 2) if args.len() == 2 => {
                        Arc::make_mut(args).swap(0, 1)
                    }
                    (ExprX::ReadPlace(p, _), 3) if matches!(&p.x,PlaceX::Local(n) if *n==original.x.params[1].x.name) =>
                    {
                        *p = p.new_x(PlaceX::Local(original.x.params[0].x.name.clone()));
                    }
                    (ExprX::ReadPlace(_, kind), 4)
                        if matches!(kind.preliminary_kind, ReadKind::ImmutBor) =>
                    {
                        kind.preliminary_kind = ReadKind::Copy
                    }
                    (ExprX::ReadPlace(_, kind), 5) => kind.id = 0,
                    (ExprX::Ctor(_, variant, _, _), 6) if variant.as_str() == "Less" => {
                        *variant = Arc::new("Greater".into())
                    }
                    (ExprX::ReadPlace(_, _), 7) => {
                        return Ok(SpannedTyped::new(
                            &e.span,
                            &Arc::new(TypX::Int(IntRange::I(64))),
                            changed,
                        ));
                    }
                    _ => (),
                }
                Ok(e.new_x(changed))
            };
            if mutation == 6 {
                Arc::make_mut(&mut x.ensure.0)[0] =
                    vir::ast_visitor::map_expr_visitor(&x.ensure.0[0], &mutate).unwrap();
            } else {
                x.body = Some(
                    vir::ast_visitor::map_expr_visitor(x.body.as_ref().unwrap(), &mutate).unwrap(),
                );
            }
            assert!(
                check_ordering_function(
                    &ctxt,
                    caller,
                    e.span,
                    &ids,
                    &carrier,
                    &get,
                    &primitive,
                    &models,
                    &original.new_x(x)
                )
                .is_err(),
                "nested Function mutation {mutation}"
            );
        }
        for mutation in 0..12 {
            let mut x = original.x.clone();
            match mutation {
                0 => Arc::make_mut(&mut x.params)[1] = x.params[0].clone(),
                1 => x.inner_ret = x.params[0].clone(),
                2 => x.require = Arc::new(vec![x.ensure.0[0].clone()]),
                3 => {
                    x.ensure.0 = Arc::new(vec![SpannedTyped::new(
                        &e_span(&original),
                        &bool_typ(),
                        ExprX::Const(Constant::Bool(true)),
                    )])
                }
                4 => Arc::make_mut(&mut x.attrs).is_external_body = true,
                5 => Arc::make_mut(&mut x.attrs).exec_assume_termination = true,
                6 => x.unwind_spec = Some(UnwindSpec::NoUnwind),
                7 => x.body = None,
                8..=11 => {
                    let b = x.body.as_ref().unwrap();
                    let ExprX::Block(stmts, tail) = &b.x else { unreachable!() };
                    let mut stmts = (**stmts).clone();
                    match mutation {
                        8 => stmts.swap(0, 3),
                        9 => stmts[3] = stmts[0].clone(),
                        10 => stmts.push(stmts[0].clone()),
                        11 => stmts.swap(1, 4),
                        _ => unreachable!(),
                    }
                    x.body = Some(b.new_x(ExprX::Block(Arc::new(stmts), tail.clone())));
                }
                _ => unreachable!(),
            }
            assert!(
                check_ordering_function(
                    &ctxt,
                    caller,
                    e.span,
                    &ids,
                    &carrier,
                    &get,
                    &primitive,
                    &models,
                    &original.new_x(x)
                )
                .is_err(),
                "Function mutation {mutation}"
            );
        }
        rustc_driver::Compilation::Stop
    }
}
fn altered_type_dependencies(t: &Typ, change: usize) -> Option<Typ> {
    match &**t {
        TypX::Decorate(decoration, arg, inner) => Some(Arc::new(TypX::Decorate(
            decoration.clone(),
            arg.clone(),
            altered_type_dependencies(inner, change)?,
        ))),
        TypX::Boxed(inner) => {
            Some(Arc::new(TypX::Boxed(altered_type_dependencies(inner, change)?)))
        }
        TypX::Datatype(name, args, paths) if !paths.is_empty() => {
            let mut changed = paths.as_ref().clone();
            match change {
                0 => changed.clear(),
                1 => {
                    changed[0] = match &changed[0] {
                        ImplPath::TraitImplPath(path) => ImplPath::TraitImplPath(
                            path.push_segment(Arc::new("wrong_implementation".into())),
                        ),
                        ImplPath::FnDefImplPath(fun) => ImplPath::FnDefImplPath(Arc::new(FunX {
                            path: fun.path.push_segment(Arc::new("wrong_implementation".into())),
                        })),
                    };
                }
                2 => changed.push(changed[0].clone()),
                _ => unreachable!(),
            }
            Some(Arc::new(TypX::Datatype(name.clone(), args.clone(), Arc::new(changed))))
        }
        _ => None,
    }
}

fn e_span(f: &Function) -> vir::messages::Span {
    f.span.clone()
}
#[test]
fn local_ordering_whole_mir_and_actual_function_structural_controls() {
    let bundle = crate::test_verus_root();
    let path = std::env::temp_dir().join(format!("local-ord-structural-{}.rs", std::process::id()));
    std::fs::write(&path,"extern crate vstd; use std::num::NonZero; pub fn total(a:&NonZero<u64>,b:&NonZero<u64>)->core::cmp::Ordering{a.cmp(b)}").unwrap();
    let args = vec![
        "rustc".into(),
        path.display().to_string(),
        "--crate-type=lib".into(),
        "--edition=2021".into(),
        "--sysroot".into(),
        format!(
            "{}/toolchains/1.99.1-x86_64-unknown-linux-gnu",
            std::env::var("RUSTUP_HOME").unwrap()
        ),
        "--extern".into(),
        format!("vstd={}", bundle.join("libvstd.rlib").display()),
        "-L".into(),
        format!("dependency={}", bundle.display()),
    ];
    rustc_driver::run_compiler(&args, &mut Structural);
    std::fs::remove_file(path).unwrap();
}
