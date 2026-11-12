//! Inactive, closed owned-cell experiment. Validation is not translation authority.
//!
//! No descriptor, field access, helper, contract, serialization or runtime rewrite
//! is installed here. Until the whole-unit grammar, compiler field authority and
//! independent actual emitted-body correspondence are complete, every request
//! fails closed in ordinary and Infer modes, before imports/pruning/caches.
use crate::config::Args;
use rustc_hir::def_id::DefId;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::{
    mir,
    ty::{self, Ty, TyCtxt},
};
use rustc_trait_selection::infer::InferCtxtExt;
use vir::ast::VirErr;

const CORE: u64 = 17281971461272426270;
const CELL: u64 = 11949802078424953320;
const VALUE: u64 = 16821345337525786618;

type Reject = &'static str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    New,
    IntoInner,
    GetMut,
}

fn is_u64(t: Ty<'_>) -> bool {
    matches!(t.kind(), ty::Uint(ty::UintTy::U64))
}

fn is_cell(t: Ty<'_>, cell: DefId) -> bool {
    matches!(t.kind(), ty::Adt(a, args) if a.did() == cell && args.len() == 1 && is_u64(args.type_at(0)))
}

/// Validate the real loaded lang item and private field. This is deliberately
/// independent of the inactive Rev factory; layout or spelling is insufficient.
fn native_cell(tcx: TyCtxt<'_>) -> Result<DefId, Reject> {
    let cell = tcx.lang_items().unsafe_cell_type().ok_or("missing native unsafe_cell lang item")?;
    let core = tcx.lang_items().sized_trait().ok_or("missing native core Sized lang item")?.krate;
    let adt = tcx.adt_def(cell);
    if cell.krate != core
        || tcx.stable_crate_id(core).as_u64() != CORE
        || tcx.def_path_hash(cell).local_hash().as_u64() != CELL
        || !adt.is_unsafe_cell()
        || adt.repr().flags != rustc_abi::ReprFlags::IS_TRANSPARENT
        || adt.repr().align.is_some()
        || adt.repr().pack.is_some()
        || adt.repr().int.is_some()
        || adt.repr().scalable.is_some()
        || !adt.is_struct()
        || adt.variants().len() != 1
        || adt.destructor(tcx).is_some()
        || !tcx.visibility(cell).is_public()
    {
        return Err("loaded native UnsafeCell core/ADT identity or shape");
    }
    let generics = tcx.generics_of(cell);
    if generics.parent.is_some()
        || generics.count() != 1
        || generics.own_params.len() != 1
        || !matches!(
            generics.own_params[0].kind,
            ty::GenericParamDefKind::Type { has_default: false, .. }
        )
    {
        return Err("loaded native UnsafeCell generic declaration");
    }
    let variant = adt.non_enum_variant();
    if variant.fields.len() != 1 || variant.ctor.is_some() {
        return Err("loaded native UnsafeCell variant");
    }
    let field = variant.fields.iter().next().unwrap();
    let typ = tcx.type_of(field.did).instantiate_identity().skip_norm_wip();
    if tcx.def_path_hash(field.did).local_hash().as_u64() != VALUE
        || tcx.parent(field.did) != cell
        || field.name.as_str() != "value"
        || !matches!(field.vis, ty::Visibility::Restricted(m) if m.to_def_id() == tcx.parent(cell))
        || !matches!(typ.kind(), ty::Param(p) if p.index == 0 && p.name == generics.own_params[0].name)
    {
        return Err("loaded native UnsafeCell private field identity/type/visibility");
    }
    let param = Ty::new_param(tcx, 0, generics.own_params[0].name);
    if param.is_sized(tcx, ty::TypingEnv::non_body_analysis(tcx, cell)) {
        return Err("native UnsafeCell declaration lost ?Sized");
    }
    let concrete = Ty::new_adt(tcx, adt, tcx.mk_args(&[tcx.types.u64.into()]));
    let sync = tcx.lang_items().sync_trait().ok_or("missing native Sync lang item")?;
    let infcx = tcx.infer_ctxt().build(ty::TypingMode::PostAnalysis);
    if infcx.type_implements_trait(sync, [concrete].into_iter(), ty::ParamEnv::empty()).may_apply()
    {
        return Err("native UnsafeCell unexpectedly implements Sync or trait result is ambiguous");
    }
    Ok(cell)
}

/// Name selection is only within the actual native ADT's inherent impls;
/// resolved owner, substitutions, complete signature and MIR are checked below.
fn native_method(tcx: TyCtxt<'_>, cell: DefId, op: Operation) -> Result<DefId, Reject> {
    let name = match op {
        Operation::New => "new",
        Operation::IntoInner => "into_inner",
        Operation::GetMut => "get_mut",
    };
    let mut found = None;
    for imp in tcx.inherent_impls(cell) {
        if tcx.impl_is_of_trait(*imp) {
            continue;
        }
        let self_type = tcx.type_of(*imp).instantiate_identity().skip_norm_wip();
        if !matches!(self_type.kind(), ty::Adt(a, _) if a.did() == cell) {
            continue;
        }
        for item in tcx.associated_items(*imp).in_definition_order() {
            if tcx.item_name(item.def_id).as_str() == name {
                if found.replace(item.def_id).is_some() {
                    return Err("ambiguous native cell method");
                }
            }
        }
    }
    let method = found.ok_or("missing original native cell method")?;
    let expected = match op {
        Operation::New => 10297035969456002870,
        Operation::IntoInner => 14652106901632559644,
        Operation::GetMut => 13506035689665434230,
    };
    if tcx.def_path_hash(method).local_hash().as_u64() != expected
        || method.krate != cell.krate
        || tcx.def_kind(method) != rustc_hir::def::DefKind::AssocFn
        || !tcx.visibility(method).is_public()
        || !tcx.is_const_fn(method)
        || !tcx.is_mir_available(method)
        || !tcx.generics_of(method).own_params.is_empty()
        || tcx.generics_of(method).count() != 1
    {
        return Err("native cell method identity/generics/const/MIR metadata");
    }
    Ok(method)
}

/// Inspect regions before region erasure. get_mut must return the SAME bound
/// lifetime as its unique input borrow, never an erased or manufactured region.
fn signature<'tcx>(
    tcx: TyCtxt<'tcx>,
    cell: DefId,
    method: DefId,
    op: Operation,
) -> Result<ty::Instance<'tcx>, Reject> {
    let args = tcx.mk_args(&[tcx.types.u64.into()]);
    let poly = tcx.fn_sig(method).instantiate(tcx, args).skip_norm_wip();
    check_signature(cell, op, poly)?;
    let env = ty::TypingEnv::fully_monomorphized();
    let instance = ty::Instance::try_resolve(tcx, env, method, args)
        .map_err(|_| "native cell method resolution failed")?
        .ok_or("native cell instance absent")?;
    if !matches!(instance.def, ty::InstanceKind::Item(d) if d == method) {
        return Err("native cell method resolved to non-original item");
    }
    Ok(instance)
}

fn check_signature(cell: DefId, op: Operation, poly: ty::PolyFnSig<'_>) -> Result<(), Reject> {
    let sig = poly.skip_binder();
    if sig.inputs().len() != 1
        || sig.c_variadic()
        || sig.safety() != rustc_hir::Safety::Safe
        || sig.abi() != rustc_abi::ExternAbi::Rust
    {
        return Err("native cell method ABI/safety/arity");
    }
    let input = sig.inputs()[0];
    let output = sig.output();
    match op {
        Operation::New
            if poly.bound_vars().is_empty() && is_u64(input) && is_cell(output, cell) =>
        {
            ()
        }
        Operation::IntoInner
            if poly.bound_vars().is_empty() && is_cell(input, cell) && is_u64(output) =>
        {
            ()
        }
        Operation::GetMut => {
            let (
                ty::Ref(a, c, rustc_hir::Mutability::Mut),
                ty::Ref(b, n, rustc_hir::Mutability::Mut),
            ) = (input.kind(), output.kind())
            else {
                return Err("native get_mut exclusive reference signature");
            };
            if poly.bound_vars().len() != 1
                || a != b
                || !matches!(a.kind(), ty::ReBound(db, br) if db == ty::BoundVarIndexKind::Bound(ty::INNERMOST) && br.var.as_usize() == 0)
                || !is_cell(*c, cell)
                || !is_u64(*n)
            {
                return Err("native get_mut input/output region or referent");
            }
        }
        _ => return Err("native new/into_inner signature"),
    }
    Ok(())
}

/// Exhaustively validate the complete executable body, including unreachable
/// blocks. No cleanup/drop/call/cast or arbitrary one-field wrapper is accepted.
fn inspect<'tcx>(
    tcx: TyCtxt<'tcx>,
    cell: DefId,
    op: Operation,
    body: &mir::Body<'tcx>,
) -> Result<(), Reject> {
    if !matches!(body.phase, mir::MirPhase::Runtime(mir::RuntimePhase::Optimized))
        || body.coroutine.is_some()
        || body.spread_arg.is_some()
        || !body.is_polymorphic
        || body.injection_phase.is_some()
        || body.tainted_by_errors.is_some()
        || body.coverage_info_hi.is_some()
        || body.function_coverage_info.is_some()
        || !body.required_consts.as_ref().is_some_and(|c| c.is_empty())
        || !body.mentioned_items.as_ref().is_some_and(|c| c.is_empty())
        || body.arg_count != 1
        || body.local_decls.len() != 2
        || body.basic_blocks.len() != 1
    {
        return Err("native cell complete MIR body properties/locals/blocks");
    }
    let env = ty::TypingEnv::fully_monomorphized();
    for decl in &body.local_decls {
        if decl.ty.needs_drop(tcx, env) {
            return Err("native cell MIR local needs drop");
        }
    }
    let input = body.local_decls[mir::Local::from_usize(1)].ty;
    let output = body.local_decls[mir::RETURN_PLACE].ty;
    let bb = &body.basic_blocks[mir::START_BLOCK];
    let Some(terminator) = bb.terminator.as_ref() else {
        return Err("native cell MIR missing terminator");
    };
    if bb.is_cleanup || !matches!(terminator.kind, mir::TerminatorKind::Return) {
        return Err("native cell MIR cleanup/terminator");
    }
    let [statement] = bb.statements.as_slice() else {
        return Err("native cell MIR statement count");
    };
    let mir::StatementKind::Assign(assignment) = &statement.kind else {
        return Err("native cell MIR non-assignment");
    };
    let (target, rvalue) = &**assignment;
    if target.local != mir::RETURN_PLACE || !target.projection.is_empty() {
        return Err("native cell MIR return place");
    }
    match (op, rvalue) {
        (Operation::New, mir::Rvalue::Aggregate(kind, operands)) => {
            let mir::AggregateKind::Adt(adt, variant, args, user, field) = &**kind else {
                return Err("native new MIR non-ADT aggregate");
            };
            if *adt != cell
                || variant.as_usize() != 0
                || args.len() != 1
                || !is_u64(args.type_at(0))
                || user.is_some()
                || field.is_some()
                || !is_u64(input)
                || !is_cell(output, cell)
                || operands.len() != 1
            {
                return Err("native new MIR aggregate identity/type/field");
            }
            let p = match operands.iter().next().unwrap() {
                mir::Operand::Copy(p) | mir::Operand::Move(p) => p,
                _ => return Err("native new MIR operand mode"),
            };
            if p.local.as_usize() != 1 || !p.projection.is_empty() {
                return Err("native new MIR input place");
            }
        }
        (Operation::IntoInner, mir::Rvalue::Use(operand, mir::WithRetag::Yes)) => {
            let p = match operand {
                mir::Operand::Copy(p) | mir::Operand::Move(p) => p,
                _ => return Err("native into_inner MIR operand"),
            };
            if !is_cell(input, cell)
                || !is_u64(output)
                || p.local.as_usize() != 1
                || !matches!(p.projection.as_ref(), [mir::ProjectionElem::Field(f, t)] if f.as_usize() == 0 && is_u64(*t))
            {
                return Err("native into_inner MIR field/place/type");
            }
        }
        (
            Operation::GetMut,
            mir::Rvalue::Ref(region, mir::BorrowKind::Mut { kind: mir::MutBorrowKind::Default }, p),
        ) => {
            if !matches!(region.kind(), ty::ReErased)
                || !matches!(input.kind(), ty::Ref(r, c, rustc_hir::Mutability::Mut) if matches!(r.kind(),ty::ReErased) && is_cell(*c, cell))
                || !matches!(output.kind(), ty::Ref(r, n, rustc_hir::Mutability::Mut) if matches!(r.kind(),ty::ReErased) && is_u64(*n))
                || p.local.as_usize() != 1
                || !matches!(p.projection.as_ref(), [mir::ProjectionElem::Deref, mir::ProjectionElem::Field(f, t)] if f.as_usize() == 0 && is_u64(*t))
            {
                return Err("native get_mut MIR exclusive field/place/type");
            }
        }
        _ => return Err("native cell MIR operation differs from original method"),
    }
    Ok(())
}

pub(crate) fn check_request(
    tcx: TyCtxt<'_>,
    args: &Args,
    cargo_export: bool,
) -> Result<(), VirErr> {
    let fail = |s| crate::util::error(format!("experimental owned UnsafeCell: {s}"));
    if args.export.is_some() || cargo_export {
        return Err(fail(
            "local-only; selected VIR export is unsupported (including zero-helper units)",
        ));
    }
    if !args.can_certify_inference() || args.no_erasure_check || !args.no_cheating {
        return Err(fail(
            "requires --no-cheating and full body/lifetime/trait/erasure checking; partial verification is unsupported",
        ));
    }
    let cell = native_cell(tcx).map_err(fail)?;
    for op in [Operation::New, Operation::IntoInner, Operation::GetMut] {
        let method = native_method(tcx, cell, op).map_err(fail)?;
        let instance = signature(tcx, cell, method, op).map_err(fail)?;
        let env = ty::TypingEnv::fully_monomorphized();
        let body = instance.instantiate_mir_and_normalize_erasing_regions(
            tcx,
            env,
            ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()),
        );
        inspect(tcx, cell, op, &body).map_err(fail)?;
    }
    Err(fail(
        "inactive: complete typed-unit admission, compiler-owned field authority, emitted-body correspondence and final-input-state gates are not implemented",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Loaded;
    impl rustc_driver::Callbacks for Loaded {
        fn after_analysis<'tcx>(
            &mut self,
            _: &rustc_interface::interface::Compiler,
            tcx: TyCtxt<'tcx>,
        ) -> rustc_driver::Compilation {
            let cell = native_cell(tcx).unwrap();
            eprintln!(
                "loaded cell identity: {:?}, field: {:?}",
                tcx.def_path_hash(cell),
                tcx.def_path_hash(
                    tcx.adt_def(cell).non_enum_variant().fields.iter().next().unwrap().did
                )
            );
            let mut failures = vec![];
            for op in [Operation::New, Operation::IntoInner, Operation::GetMut] {
                let method = native_method(tcx, cell, op).unwrap();
                let instance = signature(tcx, cell, method, op).unwrap();
                let env = ty::TypingEnv::fully_monomorphized();
                let body = instance.instantiate_mir_and_normalize_erasing_regions(
                    tcx,
                    env,
                    ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()),
                );
                eprintln!(
                    "native {op:?} {:?}: phase={:?} polymorphic={} required={:?} mentioned={:?} args={} locals={:?} blocks={:?}",
                    tcx.def_path_hash(method),
                    body.phase,
                    body.is_polymorphic,
                    body.required_consts,
                    body.mentioned_items,
                    body.arg_count,
                    body.local_decls,
                    body.basic_blocks
                );
                if let Err(e) = inspect(tcx, cell, op, &body) {
                    failures.push((op, e));
                    continue;
                }
                for change in 0..16 {
                    let mut bad = body.clone();
                    match change {
                        0 => bad.arg_count = 0,
                        1 => {
                            bad.basic_blocks.as_mut()[mir::START_BLOCK].statements.clear();
                        }
                        2 => {
                            let s = bad.basic_blocks[mir::START_BLOCK].statements[0].clone();
                            bad.basic_blocks.as_mut()[mir::START_BLOCK].statements.push(s);
                        }
                        3 => bad.local_decls[mir::RETURN_PLACE].ty = tcx.types.bool,
                        4 => bad.local_decls[mir::Local::from_usize(1)].ty = tcx.types.bool,
                        5 => bad.basic_blocks.as_mut()[mir::START_BLOCK].is_cleanup = true,
                        6 => {
                            bad.basic_blocks.as_mut()[mir::START_BLOCK].terminator_mut().kind =
                                mir::TerminatorKind::Unreachable
                        }
                        7 => {
                            let mir::StatementKind::Assign(a) =
                                &mut bad.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind
                            else {
                                unreachable!()
                            };
                            a.0 = mir::Place::from(mir::Local::from_usize(1));
                        }
                        8 => {
                            let b = bad.basic_blocks[mir::START_BLOCK].clone();
                            bad.basic_blocks.as_mut().push(b);
                        }
                        9 => bad.injection_phase = Some(body.phase),
                        10 => bad.is_polymorphic = false,
                        11 => {
                            let mir::StatementKind::Assign(a) =
                                &mut bad.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind
                            else {
                                unreachable!()
                            };
                            match &mut a.1 {
                                mir::Rvalue::Aggregate(_, operands) => {
                                    operands.iter_mut().for_each(|o| {
                                        *o = mir::Operand::Copy(mir::Place::from(mir::RETURN_PLACE))
                                    })
                                }
                                mir::Rvalue::Use(o, _) => {
                                    *o = mir::Operand::Copy(mir::Place::from(
                                        mir::Local::from_usize(1),
                                    ))
                                }
                                mir::Rvalue::Ref(_, _, p) => {
                                    *p = mir::Place::from(mir::Local::from_usize(1))
                                }
                                _ => unreachable!(),
                            }
                        }
                        12 => {
                            let mir::StatementKind::Assign(a) =
                                &mut bad.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind
                            else {
                                unreachable!()
                            };
                            match &mut a.1 {
                                mir::Rvalue::Aggregate(_, operands) => operands.raw.clear(),
                                mir::Rvalue::Use(_, retag) => *retag = mir::WithRetag::No,
                                mir::Rvalue::Ref(r, _, _) => *r = tcx.lifetimes.re_static,
                                _ => unreachable!(),
                            }
                        }
                        13 => {
                            let mir::StatementKind::Assign(a) =
                                &mut bad.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind
                            else {
                                unreachable!()
                            };
                            match &mut a.1 {
                                mir::Rvalue::Aggregate(k, _) => **k = mir::AggregateKind::Tuple,
                                mir::Rvalue::Use(mir::Operand::Move(p), _) => {
                                    p.projection =
                                        tcx.mk_place_elems(&[mir::ProjectionElem::Field(
                                            rustc_abi::FieldIdx::from_usize(1),
                                            tcx.types.u64,
                                        )])
                                }
                                mir::Rvalue::Ref(_, _, p) => {
                                    p.projection = tcx.mk_place_elems(&[
                                        mir::ProjectionElem::Deref,
                                        mir::ProjectionElem::Field(
                                            rustc_abi::FieldIdx::from_usize(1),
                                            tcx.types.u64,
                                        ),
                                    ])
                                }
                                _ => unreachable!(),
                            }
                        }
                        14 => {
                            let mir::StatementKind::Assign(a) =
                                &mut bad.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind
                            else {
                                unreachable!()
                            };
                            match &mut a.1 {
                                mir::Rvalue::Aggregate(k, _) => {
                                    let mir::AggregateKind::Adt(d, ..) = &mut **k else {
                                        unreachable!()
                                    };
                                    *d = tcx.lang_items().sized_trait().unwrap();
                                }
                                mir::Rvalue::Use(mir::Operand::Move(p), _) => {
                                    p.projection =
                                        tcx.mk_place_elems(&[mir::ProjectionElem::Field(
                                            rustc_abi::FieldIdx::from_usize(0),
                                            tcx.types.bool,
                                        )])
                                }
                                mir::Rvalue::Ref(_, k, _) => *k = mir::BorrowKind::Shared,
                                _ => unreachable!(),
                            }
                        }
                        15 => bad.basic_blocks.as_mut()[mir::START_BLOCK].terminator = None,
                        _ => unreachable!(),
                    }
                    assert!(inspect(tcx, cell, op, &bad).is_err(), "{op:?} corruption {change}");
                }
                assert!(inspect(tcx, tcx.lang_items().sized_trait().unwrap(), op, &body).is_err());
                assert!(
                    inspect(
                        tcx,
                        cell,
                        match op {
                            Operation::New => Operation::IntoInner,
                            _ => Operation::New,
                        },
                        &body
                    )
                    .is_err()
                );
            }
            for name in ["separate", "static_output", "shared_input", "wrong_inner"] {
                let id = tcx
                    .hir_body_owners()
                    .find(|d| tcx.item_name(d.to_def_id()).as_str() == name)
                    .unwrap();
                let poly = tcx.fn_sig(id).instantiate_identity().skip_norm_wip();
                assert!(
                    check_signature(cell, Operation::GetMut, poly).is_err(),
                    "region/signature corruption {name}"
                );
            }
            assert!(failures.is_empty(), "native MIR failures: {failures:?}");
            rustc_driver::Compilation::Stop
        }
    }
    #[test]
    fn loaded_native_methods_and_complete_mir_corruptions() {
        let path =
            std::env::temp_dir().join(format!("owned-cell-loaded-{}.rs", std::process::id()));
        std::fs::write(
            &path,
            format!(
                "{}\n{}",
                include_str!(
                    "../../tools/research/redb_crash_protocol/native_state_link/owned_native.rs"
                ),
                r#"
fn separate<'a,'b>(_: &'a mut core::cell::UnsafeCell<u64>) -> &'b mut u64 {loop{}}
fn static_output(_: &mut core::cell::UnsafeCell<u64>) -> &'static mut u64 {loop{}}
fn shared_input(_: &core::cell::UnsafeCell<u64>) -> &mut u64 {loop{}}
fn wrong_inner(_: &mut core::cell::UnsafeCell<u64>) -> &mut bool {loop{}}
"#
            ),
        )
        .unwrap();
        let args = vec![
            "rustc".into(),
            path.display().to_string(),
            "--crate-type=lib".into(),
            "--edition=2024".into(),
            "--sysroot".into(),
            format!(
                "{}/toolchains/1.99.1-x86_64-unknown-linux-gnu",
                std::env::var("RUSTUP_HOME").unwrap()
            ),
        ];
        rustc_driver::run_compiler(&args, &mut Loaded);
        std::fs::remove_file(path).unwrap();
    }
}
