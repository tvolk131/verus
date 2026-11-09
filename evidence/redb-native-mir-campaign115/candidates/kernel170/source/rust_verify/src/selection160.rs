struct SelectedNative160<'tcx> {
    owner: ty::Instance<'tcx>,
    callee: ty::Instance<'tcx>,
    block: rustc_middle::mir::BasicBlock,
    size: Operand<'tcx>,
    align: Operand<'tcx>,
    result: Place<'tcx>,
    continuation: rustc_middle::mir::BasicBlock,
    unwind: String,
}

fn select_original160<'tcx>(tcx: TyCtxt<'tcx>, store: &TypedStore<'tcx>, allocation: &AllocationCut156) -> SelectedNative160<'tcx> {
    let mut queue = VecDeque::from([(store.instance, Vec::<Value>::new(), Vec::<ty::Instance<'tcx>>::new())]);
    let mut runtime = Vec::new();
    let mut explored = 0usize;
    while let Some((instance, path, ancestors)) = queue.pop_front() {
        explored += 1;
        assert!(explored <= 350 && ancestors.len() <= 14, "bounded exact call-path selection exhausted");
        assert!(!ancestors.contains(&instance), "recursive activation path unsupported");
        let mut next_ancestors = ancestors;
        next_ancestors.push(instance);
        for (callee, block, record) in resolved_calls156(tcx, instance) {
            let name = tcx.opt_item_name(callee.def_id()).map(|name| name.as_str().to_owned()).unwrap_or_default();
            if tcx.crate_name(callee.def_id().krate).as_str() != "alloc" { continue; }
            let mut next_path = path.clone();
            next_path.push(record);
            if name == "alloc_impl_runtime" {
                let body = tcx.instance_mir(instance.def);
                let data = body.basic_blocks.iter().nth(block).unwrap();
                let TerminatorKind::Call { args, .. } = &data.terminator().kind else { unreachable!() };
                assert_eq!(args.len(), 2);
                let Operand::Constant(zeroed) = &args[1].node else { panic!("selected invocation requires actual false operand") };
                assert_eq!(zeroed.const_.try_eval_scalar_int(tcx, ty::TypingEnv::fully_monomorphized()).expect("zeroed constant").to_u8(), 0);
                runtime.push((callee, next_path));
                continue;
            }
            if matches!(name.as_str(), "allocate_for_slice" | "allocate_for_layout" | "allocate" | "alloc_impl" | "call_once" | "")
                && tcx.is_mir_available(callee.def_id()) {
                queue.push_back((callee, next_path, next_ancestors.clone()));
            }
        }
    }
    assert_eq!(runtime.len(), 1, "one exact incoming runtime invocation path required; no instance-only deduplication");
    let (owner, path) = runtime.pop().unwrap();
    let body = tcx.instance_mir(owner.def);
    let mut leaves = Vec::new();
    for (block, data) in body.basic_blocks.iter_enumerated() {
        let TerminatorKind::Call { func, args, destination, target, unwind, .. } = &data.terminator().kind else { continue };
        let function_type = instantiate(tcx, owner, func.ty(&body.local_decls, tcx));
        let ty::FnDef(definition, arguments) = *function_type.kind() else { continue };
        if tcx.crate_name(definition.krate).as_str() != "alloc"
            || tcx.opt_item_name(definition).is_none_or(|name| name.as_str() != "__rust_alloc") { continue; }
        let callee = ty::Instance::try_resolve(tcx, ty::TypingEnv::fully_monomorphized(), definition, arguments.skip_binder()).unwrap().unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(instantiate(tcx, owner, args[0].node.ty(&body.local_decls, tcx)), tcx.types.usize);
        let layout_local = body.args_iter().next().expect("selected runtime Layout argument required");
        let layout_type = instantiate(tcx, owner, body.local_decls[layout_local].ty);
        let ty::Adt(layout_definition, layout_arguments) = *layout_type.kind() else { panic!("selected runtime argument must be Layout") };
        assert_eq!(Some(layout_definition.did()), tcx.lang_items().alloc_layout());
        let alignment_fields: Vec<_> = layout_definition.non_enum_variant().fields.iter()
            .filter(|field| field.name.as_str() == "align").collect();
        assert_eq!(alignment_fields.len(), 1, "actual Layout alignment field required");
        let alignment_type = instantiate(tcx, owner, alignment_fields[0].ty(tcx, layout_arguments).skip_norm_wip());
        assert_eq!(instantiate(tcx, owner, args[1].node.ty(&body.local_decls, tcx)), alignment_type);
        let ty::Adt(alignment_definition, _) = *alignment_type.kind() else { panic!("pinned alignment operand must retain its nominal type") };
        assert_eq!(tcx.crate_name(alignment_definition.did().krate).as_str(), "core");
        assert_eq!(tcx.item_name(alignment_definition.did()).as_str(), "Alignment");
        assert_eq!(instantiate(tcx, owner, destination.ty(&body.local_decls, tcx).ty), Ty::new_mut_ptr(tcx, tcx.types.u8));
        assert!(!data.is_cleanup);
        let continuation = target.expect("normal return edge required");
        assert!(!body.basic_blocks[continuation].is_cleanup);
        let scope = &body.source_scopes[data.terminator().source_info.scope];
        let (inline, _) = scope.inlined.expect("inlined alloc owner required");
        assert_eq!(tcx.crate_name(inline.def_id().krate).as_str(), "alloc");
        assert_eq!(tcx.item_name(inline.def_id()).as_str(), "alloc");
        leaves.push(SelectedNative160 { owner, callee, block, size: args[0].node.clone(), align: args[1].node.clone(), result: *destination, continuation, unwind: format!("{unwind:?}") });
    }
    assert_eq!(leaves.len(), 1, "one selected nonzeroed raw allocator leaf required");
    leaves.pop().unwrap()
}
