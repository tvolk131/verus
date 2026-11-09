#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;

include!("../allocation_selection160.rs");
include!("../native_continuation160.rs");

use rustc_middle::mir::{BinOp, Body, Local, Operand, Place, ProjectionElem, Rvalue, StatementKind, TerminatorKind, RETURN_PLACE};
use rustc_middle::ty::{self, Ty, TyCtxt};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, VecDeque};
use std::path::PathBuf;

struct Callbacks {
    output: PathBuf,
    pending: bool,
    source_root: PathBuf,
    kernel170: Option<(PathBuf, String)>,
}

struct TypedStore<'tcx> {
    instance: ty::Instance<'tcx>,
    pointer: Local,
    value: Local,
    base: Local,
    index: Local,
    record: Value,
}

fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn instantiate<'tcx>(tcx: TyCtxt<'tcx>, instance: ty::Instance<'tcx>, typ: Ty<'tcx>) -> Ty<'tcx> {
    tcx.normalize_erasing_regions(ty::TypingEnv::fully_monomorphized(), ty::EarlyBinder::bind(tcx, typ).instantiate(tcx, instance.args))
}

fn unique_definition<'tcx>(body: &'tcx Body<'tcx>, local: Local) -> (usize, usize, &'tcx Rvalue<'tcx>) {
    let mut found = Vec::new();
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (index, statement) in data.statements.iter().enumerate() {
            if let StatementKind::Assign(assignment) = &statement.kind {
                if assignment.0.as_local() == Some(local) {
                    found.push((block.as_usize(), index, &assignment.1));
                }
            }
        }
    }
    assert_eq!(found.len(), 1, "ambiguous or absent local definition");
    found[0]
}

fn tuple_component<'tcx>(tcx: TyCtxt<'tcx>, instance: ty::Instance<'tcx>, body: &Body<'tcx>, place: Place<'tcx>, field: usize) {
    assert_eq!(place.projection.len(), 3, "exact Option tuple projection required");
    let typ = instantiate(tcx, instance, body.local_decls[place.local].ty);
    let ty::Adt(definition, arguments) = typ.kind() else { panic!("Option ADT required") };
    assert_eq!(tcx.crate_name(definition.did().krate).as_str(), "core");
    assert_eq!(tcx.item_name(definition.did()).as_str(), "Option");
    let ProjectionElem::Downcast(_, variant) = place.projection[0] else { panic!("Some projection required") };
    assert_eq!(definition.variant(variant).name.as_str(), "Some");
    let ProjectionElem::Field(tuple_field, _) = place.projection[1] else { panic!("Some field required") };
    assert_eq!(tuple_field.as_usize(), 0);
    let ProjectionElem::Field(component, _) = place.projection[2] else { panic!("tuple component required") };
    assert_eq!(component.as_usize(), field);
    let ty::Tuple(components) = arguments.type_at(0).kind() else { panic!("tuple type required") };
    assert_eq!(components.len(), 2);
    assert_eq!(components[0], tcx.types.usize);
    assert_eq!(components[1], tcx.types.u8);
}

fn select_store<'tcx>(tcx: TyCtxt<'tcx>, instance: ty::Instance<'tcx>, expected_iterator: Ty<'tcx>) -> TypedStore<'tcx> {
    assert_eq!(tcx.crate_name(instance.def_id().krate).as_str(), "alloc");
    assert_eq!(tcx.item_name(instance.def_id()).as_str(), "from_iter_exact");
    assert_eq!(instance.args.len(), 2);
    assert_eq!(instance.args.type_at(0), tcx.types.u8);
    assert_eq!(instance.args.type_at(1), expected_iterator, "exact original call's iterator type required");
    let ty::Adt(iterator, arguments) = instance.args.type_at(1).kind() else { panic!("RepeatN required") };
    assert_eq!(tcx.crate_name(iterator.did().krate).as_str(), "core");
    assert_eq!(tcx.item_name(iterator.did()).as_str(), "RepeatN");
    assert_eq!(arguments.type_at(0), tcx.types.u8);
    let body = tcx.instance_mir(instance.def);
    let mut candidates = Vec::new();
    for (block, data) in body.basic_blocks.iter_enumerated() {
        for (index, statement) in data.statements.iter().enumerate() {
            let StatementKind::Assign(assignment) = &statement.kind else { continue };
            let (destination, effect) = &**assignment;
            if !destination.projection.iter().any(|projection| matches!(projection, ProjectionElem::Deref)) {
                continue;
            }
            assert_eq!(destination.projection.as_ref(), &[ProjectionElem::Deref]);
            assert!(!data.is_cleanup, "cleanup not a normal store cut");
            let pointer = destination.local;
            let pointer_type = instantiate(tcx, instance, body.local_decls[pointer].ty);
            let ty::RawPtr(pointee, mutability) = pointer_type.kind() else { panic!("raw pointer required") };
            assert_eq!(*pointee, tcx.types.u8);
            assert!(mutability.is_mut());
            assert_eq!(instantiate(tcx, instance, destination.ty(&body.local_decls, tcx).ty), tcx.types.u8);
            let Rvalue::Use(Operand::Copy(value_place), retag) = effect else { panic!("exact Copy byte required") };
            assert!(retag.yes(), "only observed scalar-u8 retag form is proposed");
            let value = value_place.as_local().expect("unprojected u8 operand");
            assert_eq!(instantiate(tcx, instance, value_place.ty(&body.local_decls, tcx).ty), tcx.types.u8);
            let (pointer_block, pointer_index, offset) = unique_definition(body, pointer);
            assert_eq!(pointer_block, block.as_usize());
            assert!(pointer_index < index);
            let Rvalue::BinaryOp(BinOp::Offset, operands) = offset else { panic!("typed Offset required") };
            let (Operand::Copy(base), Operand::Copy(offset_index)) = &**operands else { panic!("exact pointer/index operands required") };
            assert_eq!(instantiate(tcx, instance, base.ty(&body.local_decls, tcx).ty), pointer_type);
            assert_eq!(instantiate(tcx, instance, offset_index.ty(&body.local_decls, tcx).ty), tcx.types.usize);
            let offset_local = offset_index.as_local().expect("index local");
            let (value_block, value_index, value_definition) = unique_definition(body, value);
            let (index_block, index_index, index_definition) = unique_definition(body, offset_local);
            assert_eq!(value_block, pointer_block);
            assert_eq!(index_block, pointer_block);
            assert!(value_index < pointer_index && index_index < pointer_index);
            let Rvalue::Use(Operand::Move(value_source), _) = value_definition else { panic!("Some byte source required") };
            let Rvalue::Use(Operand::Copy(index_source), _) = index_definition else { panic!("Some index source required") };
            tuple_component(tcx, instance, body, *value_source, 1);
            tuple_component(tcx, instance, body, *index_source, 0);
            assert_eq!(value_source.local, index_source.local);
            let predecessors = body.basic_blocks.predecessors();
            assert_eq!(predecessors[block].len(), 1, "unique Some control predecessor required");
            let predecessor = predecessors[block][0];
            assert!(!body.basic_blocks[predecessor].is_cleanup);
            let TerminatorKind::SwitchInt { discr, targets } = &body.basic_blocks[predecessor].terminator().kind else { panic!("Some discriminant guard required") };
            let Operand::Move(discriminant) = discr else { panic!("discriminant operand required") };
            let (guard_block, _, guard) = unique_definition(body, discriminant.as_local().expect("discriminant local"));
            assert_eq!(guard_block, predecessor.as_usize());
            let Rvalue::Discriminant(option_place) = guard else { panic!("actual Option discriminant required") };
            assert_eq!(option_place.as_local(), Some(value_source.local));
            let option_type = instantiate(tcx, instance, body.local_decls[value_source.local].ty);
            let ty::Adt(option_definition, _) = option_type.kind() else { unreachable!() };
            let ProjectionElem::Downcast(_, some_variant) = value_source.projection[0] else { unreachable!() };
            let incoming = targets.iter().filter(|(_, target)| *target == block).collect::<Vec<_>>();
            assert_eq!(incoming.len(), 1);
            assert_eq!(incoming[0].0, option_definition.discriminant_for_variant(tcx, some_variant).val);
            for between in &data.statements[index_index.min(value_index)..index] {
                if let StatementKind::Assign(assignment) = &between.kind {
                    assert_ne!(assignment.0.local, value_source.local, "tuple source overwritten before store");
                }
            }
            let scope = &body.source_scopes[statement.source_info.scope];
            let (inlined, _) = scope.inlined.expect("actual inlined ptr::write source owner required");
            assert_eq!(tcx.crate_name(inlined.def_id().krate).as_str(), "core");
            assert_eq!(tcx.item_name(inlined.def_id()).as_str(), "write");
            let write_arguments = tcx.normalize_erasing_regions(ty::TypingEnv::fully_monomorphized(), ty::EarlyBinder::bind(tcx, inlined.args).instantiate(tcx, instance.args));
            assert_eq!(write_arguments.len(), 1);
            assert_eq!(write_arguments.type_at(0), tcx.types.u8);
            candidates.push(TypedStore { instance, pointer, value, base: base.as_local().unwrap(), index: offset_local, record: json!({
                "instance": format!("{instance:?}"), "substitutions": format!("{:?}", instance.args),
                "owner": tcx.def_path_str(instance.def_id()), "block_observation": format!("{block:?}"),
                "statement_observation": index, "destination": format!("{destination:?}"),
                "destination_type": "u8", "pointer_type": format!("{pointer_type:?}"),
                "operand": format!("{effect:?}"), "operand_type": "u8",
                "copy_retag": format!("{retag:?}"), "retag_scope": "u8 scalar only; no pointer-bearing operand",
                "pointer_definition": format!("{offset:?}"), "index_source": format!("{index_source:?}"),
                "byte_source": format!("{value_source:?}"), "source_span": format!("{:?}", statement.source_info.span),
                "inlined_write_definition": format!("{:?}", inlined.def_id()),
                "inlined_write_substitutions": format!("{write_arguments:?}"),
                "normal_guard": format!("{:?}", body.basic_blocks[predecessor]),
                "source_scope": format!("{:?}", body.source_scopes[statement.source_info.scope]),
                "owner_mir_sha256": digest(format!("{body:#?}").as_bytes()), "owner_mir": format!("{body:#?}"),
            }) });
        }
    }
    assert_eq!(candidates.len(), 1, "ambiguous or absent typed native byte store");
    candidates.remove(0)
}

impl rustc_driver::Callbacks for Callbacks {
    fn after_analysis<'tcx>(&mut self, compiler: &rustc_interface::interface::Compiler, tcx: TyCtxt<'tcx>) -> rustc_driver::Compilation {
        let roots = tcx.hir_body_owners().filter(|owner| tcx.def_kind(owner.to_def_id()) == rustc_hir::def::DefKind::Fn
            && tcx.item_name(owner.to_def_id()).as_str() == "zero_filled_arc").collect::<Vec<_>>();
        assert_eq!(roots.len(), 1);
        let root = ty::Instance::mono(tcx, roots[0].to_def_id());
        let root_span = tcx.def_span(root.def_id());
        let root_file = tcx.sess.source_map().lookup_char_pos(root_span.lo()).file;
        let root_path = PathBuf::from(root_file.name.prefer_local_unconditionally().to_string()).canonicalize().expect("local original source");
        assert_eq!(root_path, self.source_root.join("tree_store/page_store/cached_file.rs").canonicalize().unwrap(), "exact selected original source owner");
        let root_body = tcx.instance_mir(root.def);
        assert_eq!(root_body.arg_count, 1);
        assert_eq!(root_body.local_decls[root_body.args_iter().next().unwrap()].ty, tcx.types.usize);
        let root_return = root_body.local_decls[RETURN_PLACE].ty;
        let ty::Adt(arc_definition, arc_arguments) = root_return.kind() else { panic!("Arc return required") };
        assert_eq!(tcx.crate_name(arc_definition.did().krate).as_str(), "alloc");
        assert_eq!(tcx.item_name(arc_definition.did()).as_str(), "Arc");
        let ty::Slice(element) = arc_arguments.type_at(0).kind() else { panic!("slice payload required") };
        assert_eq!(*element, tcx.types.u8);
        let mut entries = Vec::new();
        for data in root_body.basic_blocks.iter() {
            if let TerminatorKind::Call { func, args, destination, .. } = &data.terminator().kind {
                if destination.ty(&root_body.local_decls, tcx).ty != root_return { continue; }
                assert_eq!(args.len(), 1);
                let iterator_type = args[0].node.ty(&root_body.local_decls, tcx);
                let function_type = func.ty(&root_body.local_decls, tcx);
                let ty::FnDef(definition, arguments) = *function_type.kind() else { panic!("direct original call required") };
                let callee = ty::Instance::try_resolve(tcx, ty::TypingEnv::fully_monomorphized(), definition, arguments.skip_binder()).unwrap().expect("original call instance");
                entries.push((callee, iterator_type));
            }
        }
        assert_eq!(entries.len(), 1, "unique actual original Arc-returning call required");
        let (entry, expected_iterator) = entries[0];
        let mut queue = VecDeque::from([(entry, 0usize, Vec::new())]);
        let mut traversed = 0usize;
        let mut stores = Vec::new();
        while let Some((instance, depth, mut ancestors)) = queue.pop_front() {
            traversed += 1;
            assert!(traversed <= 100 && depth <= 12, "bounded source connection exhausted");
            assert!(!ancestors.contains(&instance), "recursive source connection unsupported");
            ancestors.push(instance);
            if tcx.item_name(instance.def_id()).as_str() == "from_iter_exact" {
                stores.push(select_store(tcx, instance, expected_iterator));
                continue;
            }
            let body = tcx.instance_mir(instance.def);
            for data in body.basic_blocks.iter() {
                let TerminatorKind::Call { func, .. } = &data.terminator().kind else { continue };
                let function_type = instantiate(tcx, instance, func.ty(&body.local_decls, tcx));
                let ty::FnDef(definition, arguments) = *function_type.kind() else { continue };
                let resolved = ty::Instance::try_resolve(tcx, ty::TypingEnv::fully_monomorphized(), definition, arguments.skip_binder()).expect("typed call resolution");
                if let Some(callee) = resolved {
                    if tcx.is_mir_available(callee.def_id()) {
                        let callee_body = tcx.instance_mir(callee.def);
                        let callee_return = instantiate(tcx, callee, callee_body.local_decls[RETURN_PLACE].ty);
                        if callee_return != root_return { continue; }
                        queue.push_back((callee, depth + 1, ancestors.clone()));
                    }
                }
            }
        }
        assert_eq!(stores.len(), 1, "one actual resolved source-connected native occurrence required");
        let store = stores.remove(0);
        let allocation = allocation_cut156(tcx, &store);
        #[cfg(feature = "native-semantics170-inspect")]
        if let Some((library, library_sha256)) = &self.kernel170 {
            let selected = select_original160(tcx, &store, &allocation);
            let body = tcx.instance_mir(selected.owner.def);
            let runtime_path = PathBuf::from("/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/lib");
            let mut runtimes = std::fs::read_dir(runtime_path).unwrap().map(|entry| entry.unwrap().path())
                .filter(|path| path.file_name().unwrap().to_string_lossy().starts_with("liballoc-")
                    && path.extension().is_some_and(|extension| extension == "rlib")).collect::<Vec<_>>();
            assert_eq!(runtimes.len(), 1);
            let runtime = runtimes.pop().unwrap();
            let align_type = instantiate(tcx, selected.owner, selected.align.ty(&body.local_decls, tcx));
            let alignment_method_mir_sha256 = rust_verify::native_adapter164::interpret_alignment170(tcx, align_type)
                .expect("interpret actual Alignment::as_usize MIR before numeric semantic operand");
            let abi = format!("{:?}",tcx.fn_sig(selected.callee.def_id()).instantiate(tcx,selected.callee.args).skip_binder().abi());
            assert_eq!(abi, "Rust");
            assert_eq!(tcx.crate_name(selected.callee.def_id().krate).as_str(), "alloc");
            assert_eq!(tcx.item_name(selected.callee.def_id()).as_str(), "__rust_alloc");
            let source = vir::native_resource::kernel170::Source170 {
                root_source_sha256: digest(&std::fs::read(&root_path).unwrap()),
                owner_mir_sha256: digest(format!("{body:#?}").as_bytes()),
                owner: format!("{:?}", selected.owner), callee: "alloc::alloc::__rust_alloc".into(),
                runtime_sha256: digest(&std::fs::read(&runtime).unwrap()),
                target: tcx.sess.opts.target_triple.to_string(),
                abi: format!("{abi}; usize, core::mem::Alignment -> *mut u8"),
                alignment: "core::mem::Alignment::as_usize(actual operand)".into(),
                alignment_method_mir_sha256,
                block: selected.block.as_usize(), normal: selected.continuation.as_usize(),
                native_destination: selected.result.as_local().expect("plain selected native result").as_usize(),
                size_operand: format!("{:?}", selected.size), align_operand: format!("{:?}", selected.align),
            };
            let inspection = rust_verify::native_kernel170::inspect170(source, library, library_sha256);
            let mut report = match inspection { Ok(report) => report,
                Err(error) => json!({"inspection_error": error, "native_export":"REFUSED", "operative_issuance":0}) };
            report["original_source_linkage"] = json!({"root_path":root_path,
                "root_instance":format!("{root:?}"),"root_mir":format!("{root_body:#?}"),
                "original_allocation_and_payload_projection":allocation.record,
                "actual_store_projection":store.record,"selected_owner_mir":format!("{body:#?}"),
                "selected_callee":format!("{:?}",selected.callee),"actual_abi":abi,
                "actual_unwind":selected.unwind,"runtime_alloc_rlib":runtime,
                "scope":"conditional selected primitive; actual caller/frontier/non-null/layout flow not accepted"});
            std::fs::write(self.output.with_extension("kernel170.json"), serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            compiler.sess.dcx().err("NATIVE170_EXPORT_REFUSED: pre-review structural candidate only; no solver, original permission or native body credit");
            return rustc_driver::Compilation::Stop;
        }
        let pending = lower_original160(tcx, &store, &allocation, self.pending);
        std::fs::write(self.output.with_extension("pending.json"), serde_json::to_vec_pretty(&pending).unwrap()).unwrap();
        compiler.sess.dcx().err("CONTINUATION160_EXPORT_REFUSED: typed candidate only; native dynamic event/spec-pointer interpretation unavailable; PENDING_REVIEW; no resource authority or theorem credit");
        rustc_driver::Compilation::Stop
    }
}

fn main() -> std::process::ExitCode {
    let mut arguments = std::env::args().collect::<Vec<_>>();
    assert_eq!(arguments[1], "--continuation160-output");
    let output = PathBuf::from(arguments.remove(2));
    arguments.remove(1);
    let pending = if arguments.get(1).is_some_and(|argument| argument == "--lower-continuation160") {
        arguments.remove(1);
        true
    } else { false };
    let kernel170 = if arguments.get(1).is_some_and(|argument| argument == "--inspect-kernel170-vstd") {
        arguments.remove(1);
        let path = PathBuf::from(arguments.remove(1));
        let digest = arguments.remove(1);
        Some((path, digest))
    } else { None };
    let source_root = PathBuf::from(arguments.last().unwrap()).parent().unwrap().canonicalize().unwrap();
    let result = rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(&arguments, &mut Callbacks { output, pending, source_root, kernel170 }));
    if result.is_ok() { std::process::ExitCode::SUCCESS } else { std::process::ExitCode::FAILURE }
}
