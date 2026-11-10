//! Read-only diagnostic prototype. Never registers a contract or emits a .vir API.
#![feature(rustc_private)]
extern crate rustc_abi;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate vir;

use rustc_hir::def_id::DefId;
use rustc_middle::{mir, ty::{self, TyCtxt, TypeVisitableExt}};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use vir::{ast::*, ast_util::{bool_typ, const_int_from_u128, str_unique_var}, def::Spanned};

type Reject = &'static str;

// Private typed observation: this is NOT VIR and NOT a certificate/proof token.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Primitive {
    ReturnBoolToU64 { copied_local: usize, assigned_local: usize },
}

// Fresh IDs/internal spans do not masquerade as HIR nodes. The original source
// origin is diagnostic data only; the real HIR call mapping is a missing gate.
static NEXT: AtomicU64 = AtomicU64::new(1);
fn span(origin: &str) -> vir::messages::Span {
    vir::messages::Span { raw_span: Arc::new(()), id: NEXT.fetch_add(1, Ordering::Relaxed),
        data: vec![], as_string: origin.into() }
}
fn uint64() -> Typ { Arc::new(TypX::Int(IntRange::U(64))) }
fn expression(origin: &str, t: &Typ, x: ExprX) -> vir::ast::Expr {
    SpannedTyped::new(&span(origin), t, x)
}

// A real ordinary VIR function, deliberately not inserted into any verifier
// registry. Building this AST does not perform mode/WF/body proofs.
fn translate(p: &Primitive, origin: &str) -> Function {
    let Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 0 } = p else {
        panic!("unsupported diagnostic observation");
    };
    let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
    let b = str_unique_var(&format!("foreign_input_{nonce}"), VarIdentDisambiguate::NoBodyParam);
    let r = str_unique_var(&format!("foreign_result_{nonce}"), VarIdentDisambiguate::NoBodyParam);
    let param = |name, typ| Spanned::new(span(origin), ParamX {
        name, typ, mode: Mode::Exec, user_mut: false, unwrapped_info: None });
    let input = param(b.clone(), bool_typ());
    let ret = param(r.clone(), uint64());
    let result = |c| expression(origin, &uint64(), ExprX::If(c,
        expression(origin, &uint64(), ExprX::Const(const_int_from_u128(1))),
        Some(expression(origin, &uint64(), ExprX::Const(const_int_from_u128(0))))));
    let place = SpannedTyped::new(&span(origin), &bool_typ(), PlaceX::Local(b.clone()));
    let read = expression(origin, &bool_typ(), ExprX::ReadPlace(place, UnfinalizedReadKind {
        preliminary_kind: ReadKind::Copy, id: NEXT.fetch_add(1, Ordering::Relaxed) }));
    let body = result(read);
    let ensure = expression(origin, &bool_typ(), ExprX::Binary(BinaryOp::Eq(Mode::Spec),
        expression(origin, &uint64(), ExprX::Var(r)),
        result(expression(origin, &bool_typ(), ExprX::Var(b)))));
    let visibility = Visibility { restricted_to: None };
    Spanned::new(span(origin), FunctionX {
        name: Arc::new(FunX { path: Arc::new(PathX { krate: CrateId::Internal,
            segments: Arc::new(vec![Arc::new(format!("foreign_diagnostic_{nonce}"))]) }) }),
        proxy: None, kind: FunctionKind::Static, visibility: visibility.clone(),
        body_visibility: BodyVisibility::Visibility(visibility), opaqueness: Opaqueness::Opaque,
        owning_module: None, mode: Mode::Exec, typ_params: Arc::new(vec![]),
        typ_bounds: Arc::new(vec![]), params: Arc::new(vec![input]),
        outer_ret: ret.clone(), inner_ret: ret, require: Arc::new(vec![]),
        ensure: (Arc::new(vec![ensure]), Arc::new(vec![])), returns: None,
        decrease: Arc::new(vec![]), decrease_when: None, decrease_by: None,
        fndef_axioms: None, mask_spec: None, atomic_update: None, unwind_spec: None,
        item_kind: ItemKind::Function,
        attrs: Arc::new(FunctionAttrsX { uses_ghost_blocks: true, ..Default::default() }),
        body: Some(body), return_opaque_type_eqs: Arc::new(vec![]),
        extra_dependencies: vec![], hidden: Arc::new(vec![]),
    })
}

fn is_u64(t: &Typ) -> bool { matches!(&**t, TypX::Int(IntRange::U(64))) }
fn literal(e: &vir::ast::Expr, n: u128) -> bool {
    is_u64(&e.typ) && matches!(&e.x, ExprX::Const(c) if *c == const_int_from_u128(n))
}

// Check the *original* typed MIR against actual ordinary VIR, independently of
// both the recognizer's returned descriptor and the proposed postcondition.
fn corresponds_vir(body: &mir::Body<'_>, f: &Function) -> bool {
    if body.arg_count != 1 || body.local_decls.len() != 2 || body.basic_blocks.len() != 1
        || f.x.mode != Mode::Exec || f.x.params.len() != 1 || f.x.attrs.is_external_body
        || !f.x.typ_params.is_empty() || !f.x.typ_bounds.is_empty()
        || !matches!(f.x.kind, FunctionKind::Static) || f.x.fndef_axioms.is_some() {
        return false;
    }
    let input = &f.x.params[0].x;
    if input.mode != Mode::Exec || !matches!(&*input.typ, TypX::Bool)
        || !is_u64(&f.x.inner_ret.x.typ) || !is_u64(&f.x.outer_ret.x.typ)
        || f.x.inner_ret.x.name != f.x.outer_ret.x.name || input.user_mut
        || f.x.inner_ret.x.mode != Mode::Exec || f.x.outer_ret.x.mode != Mode::Exec {
        return false;
    }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    let [s] = bb.statements.as_slice() else { return false };
    let mir::StatementKind::Assign(a) = &s.kind else { return false };
    let mir::Rvalue::Cast(mir::CastKind::IntToInt, mir::Operand::Copy(p), t) = &a.1 else {
        return false;
    };
    if bb.is_cleanup || !matches!(bb.terminator().kind, mir::TerminatorKind::Return)
        || a.0.local != mir::RETURN_PLACE || !a.0.projection.is_empty()
        || p.local != mir::Local::from_usize(1) || !p.projection.is_empty()
        || !matches!(t.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[a.0.local].ty.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[p.local].ty.kind(), ty::Bool) { return false; }
    let Some(e) = &f.x.body else { return false };
    let ExprX::If(c, one, Some(zero)) = &e.x else { return false };
    let ExprX::ReadPlace(place, read) = &c.x else { return false };
    is_u64(&e.typ) && literal(one, 1) && literal(zero, 0)
        && matches!(&*c.typ, TypX::Bool) && matches!(&*place.typ, TypX::Bool)
        && matches!(&place.x, PlaceX::Local(n) if *n == input.name)
        && matches!(read.preliminary_kind, ReadKind::Copy)
}

// Proposal validation is a separate syntactic gate, NOT a solver proof.
fn full_domain_proposal(f: &Function) -> bool {
    if !f.x.require.is_empty() || !f.x.ensure.1.is_empty() { return false; }
    let [e] = f.x.ensure.0.as_slice() else { return false };
    let ExprX::Binary(BinaryOp::Eq(Mode::Spec), r, v) = &e.x else { return false };
    let ExprX::If(b, one, Some(zero)) = &v.x else { return false };
    f.x.params.len() == 1 && matches!(&*e.typ, TypX::Bool)
        && matches!(&r.x, ExprX::Var(n) if *n == f.x.inner_ret.x.name)
        && is_u64(&r.typ) && is_u64(&v.typ) && matches!(&*b.typ, TypX::Bool)
        && matches!(&b.x, ExprX::Var(n) if *n == f.x.params[0].x.name)
        && literal(one, 1) && literal(zero, 0)
}

fn vir_controls(body: &mir::Body<'_>, original: &Function) {
    if !corresponds_vir(body, original) || !full_domain_proposal(original) {
        panic!("original diagnostic VIR mismatch");
    }
    // Malformed/wrong proposals keep the source body correspondence healthy.
    for fault in 0..3 {
        let mut x = original.x.clone();
        match fault {
            0 => x.ensure.0 = Arc::new(vec![]),
            1 => x.require = Arc::new(vec![expression("control", &bool_typ(), ExprX::Const(Constant::Bool(false)))]),
            2 => x.ensure.0 = Arc::new(vec![expression("control", &bool_typ(), ExprX::Const(Constant::Bool(false)))]),
            _ => unreachable!(),
        }
        let f = Spanned::new(span("control"), x);
        if !corresponds_vir(body, &f) || full_domain_proposal(&f) {
            panic!("proposal control failed {fault}");
        }
    }
    for fault in 0..5 {
        let mut x = original.x.clone();
        match fault {
            0 => x.mode = Mode::Spec,
            1 => x.params = Arc::new(vec![Spanned::new(span("control"), ParamX {
                mode: Mode::Spec, ..x.params[0].x.clone() })]),
            2 => x.body = None,
            3 => {
                let e = x.body.as_ref().unwrap();
                let ExprX::If(c, a, Some(b)) = &e.x else { unreachable!() };
                x.body = Some(expression("control", &uint64(), ExprX::If(c.clone(), b.clone(), Some(a.clone()))));
            }
            4 => {
                let e = x.body.as_ref().unwrap();
                let ExprX::If(c, a, b) = &e.x else { unreachable!() };
                let ExprX::ReadPlace(p, read) = &c.x else { unreachable!() };
                let mut read = read.clone(); read.preliminary_kind = ReadKind::Move;
                let c = expression("control", &bool_typ(), ExprX::ReadPlace(p.clone(), read));
                x.body = Some(expression("control", &uint64(), ExprX::If(c, a.clone(), b.clone())));
            }
            _ => unreachable!(),
        }
        if corresponds_vir(body, &Spanned::new(span("control"), x)) {
            panic!("VIR control incorrectly admitted {fault}");
        }
    }
    println!("VIR_STRUCTURAL_CONTROLS 8 diagnostic_only_not_proofs");
}

fn semantic_instance<'tcx>(
    tcx: TyCtxt<'tcx>, env: ty::TypingEnv<'tcx>, f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Result<ty::Instance<'tcx>, Reject> {
    let core = tcx.lang_items().sized_trait().ok_or("missing_core_lang_item")?.krate;
    let from = tcx.get_diagnostic_item(rustc_span::Symbol::intern("From"))
        .ok_or("missing_From_diagnostic_item")?;
    if from.krate != core { return Err("From_not_in_semantic_core"); }
    let methods: Vec<_> = tcx.associated_items(from).in_definition_order()
        .filter(|i| tcx.def_kind(i.def_id) == rustc_hir::def::DefKind::AssocFn).collect();
    if methods.len() != 1 { return Err("unsupported_From_trait_interface"); }
    let method = methods[0].def_id;
    // MIR may name the trait item or an already resolved impl item.
    if f != method && (tcx.def_kind(f) != rustc_hir::def::DefKind::AssocFn
        || tcx.associated_item(f).trait_item_def_id() != Some(method)) {
        return Err("callee_not_core_From_method");
    }
    if args.has_non_region_param() || args.has_infer() || args.has_escaping_bound_vars() {
        return Err("unresolved_substitutions");
    }
    let instance = ty::Instance::try_resolve(tcx, env, f, args)
        .map_err(|_| "instance_resolution_failed")?.ok_or("instance_unavailable")?;
    let ty::InstanceKind::Item(def) = instance.def else { return Err("not_Item_instance"); };
    if def.krate != core || !instance.args.is_empty() {
        return Err("non_core_or_nonempty_instance_args");
    }
    if tcx.associated_item(def).trait_item_def_id() != Some(method) {
        return Err("resolved_item_not_original_trait_method");
    }
    let tr = tcx.normalize_erasing_regions(env,
        tcx.impl_trait_ref(tcx.parent(def)).instantiate(tcx, instance.args));
    if tr.def_id != from || tr.args.len() != 2
        || !matches!(tr.self_ty().kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(tr.args.type_at(1).kind(), ty::Bool) {
        return Err("wrong_trait_input_or_Self");
    }
    let poly = tcx.normalize_erasing_regions(env, tcx.fn_sig(def).instantiate(tcx, instance.args));
    if !poly.bound_vars().is_empty() { return Err("bound_signature"); }
    let sig = poly.skip_binder();
    if sig.inputs().len() != 1 || !matches!(sig.inputs()[0].kind(), ty::Bool)
        || !matches!(sig.output().kind(), ty::Uint(ty::UintTy::U64))
        || sig.c_variadic() || sig.safety() != rustc_hir::Safety::Safe
        || sig.abi() != rustc_abi::ExternAbi::Rust {
        return Err("wrong_signature_or_ABI");
    }
    if !tcx.is_mir_available(def) { return Err("foreign_MIR_unavailable"); }
    Ok(instance)
}

fn inspect<'tcx>(tcx: TyCtxt<'tcx>, env: ty::TypingEnv<'tcx>, body: &mir::Body<'tcx>)
    -> Result<Primitive, Reject>
{
    if body.arg_count != 1 || body.local_decls.len() != 2 || body.basic_blocks.len() != 1 {
        return Err("local_argument_or_block_count");
    }
    if !matches!(body.local_decls[mir::RETURN_PLACE].ty.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[mir::Local::from_usize(1)].ty.kind(), ty::Bool) {
        return Err("local_types");
    }
    if body.local_decls.iter().any(|d| d.ty.needs_drop(tcx, env)) {
        return Err("needs_drop");
    }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    if bb.is_cleanup || bb.statements.len() != 1 { return Err("cleanup_or_statement_count"); }
    if !matches!(bb.terminator().kind, mir::TerminatorKind::Return) {
        return Err("unsupported_terminator");
    }
    let mir::StatementKind::Assign(a) = &bb.statements[0].kind else {
        return Err("unsupported_statement");
    };
    let (place, value) = &**a;
    if place.local != mir::RETURN_PLACE || !place.projection.is_empty() {
        return Err("assignment_place");
    }
    let mir::Rvalue::Cast(mir::CastKind::IntToInt, mir::Operand::Copy(input), out) = value else {
        return Err("unsupported_rvalue_cast_or_operand");
    };
    if input.local != mir::Local::from_usize(1) || !input.projection.is_empty()
        || !matches!(out.kind(), ty::Uint(ty::UintTy::U64)) {
        return Err("input_place_or_cast_type");
    }
    Ok(Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 0 })
}

// Independent direct typed correspondence to the observation, not a whitelist flag.
// It deliberately grants no contract and does not claim ordinary VIR validation.
fn corresponds(body: &mir::Body<'_>, p: &Primitive) -> bool {
    if body.basic_blocks.len() != 1 { return false; }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    let [s] = bb.statements.as_slice() else { return false };
    let mir::StatementKind::Assign(a) = &s.kind else { return false };
    match (&a.0, &a.1, &bb.terminator().kind, p) {
        (target, mir::Rvalue::Cast(mir::CastKind::IntToInt, mir::Operand::Copy(source), t),
         mir::TerminatorKind::Return, Primitive::ReturnBoolToU64 { copied_local, assigned_local }) => {
            *copied_local == source.local.as_usize() && *assigned_local == target.local.as_usize()
                && *copied_local == 1 && *assigned_local == 0
                && source.projection.is_empty() && target.projection.is_empty()
                && matches!(t.kind(), ty::Uint(ty::UintTy::U64))
                && !bb.is_cleanup && body.arg_count == 1 && body.local_decls.len() == 2
                && matches!(body.local_decls[source.local].ty.kind(), ty::Bool)
                && matches!(body.local_decls[target.local].ty.kind(), ty::Uint(ty::UintTy::U64))
        }
        _ => false,
    }
}

fn shape_controls<'tcx>(tcx: TyCtxt<'tcx>, env: ty::TypingEnv<'tcx>, original: &mir::Body<'tcx>) {
    let mut count = 0;
    for fault in 0..10 {
        let mut b = original.clone();
        match fault {
            0 => b.arg_count = 0,
            1 => { b.local_decls.push(b.local_decls[mir::RETURN_PLACE].clone()); }
            2 => {
                let extra = b.basic_blocks[mir::START_BLOCK].clone();
                b.basic_blocks.as_mut().push(extra);
            }
            3 => b.basic_blocks.as_mut()[mir::START_BLOCK].is_cleanup = true,
            4 => b.basic_blocks.as_mut()[mir::START_BLOCK].statements.clear(),
            5 => {
                let s = b.basic_blocks[mir::START_BLOCK].statements[0].clone();
                b.basic_blocks.as_mut()[mir::START_BLOCK].statements.push(s);
            }
            6 => b.basic_blocks.as_mut()[mir::START_BLOCK].terminator_mut().kind = mir::TerminatorKind::Unreachable,
            7 => b.local_decls[mir::RETURN_PLACE].ty = tcx.types.u32,
            8 => b.local_decls[mir::Local::from_usize(1)].ty = tcx.types.u8,
            9 => {
                let mir::StatementKind::Assign(a) = &mut b.basic_blocks.as_mut()[mir::START_BLOCK].statements[0].kind else { unreachable!() };
                let mir::Rvalue::Cast(_, operand, _) = &mut a.1 else { unreachable!() };
                let mir::Operand::Copy(place) = operand else { unreachable!() };
                *operand = mir::Operand::Move(*place);
            }
            _ => unreachable!(),
        }
        let rejected = inspect(tcx, env, &b).is_err();
        let mismatch = !corresponds(&b, &Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 0 });
        if !rejected || !mismatch { panic!("typed shape control {fault} incorrectly admitted"); }
        println!("SHAPE_CONTROL {fault} rejected_by_admission_and_correspondence");
        count += 1;
    }
    for wrong in [Primitive::ReturnBoolToU64 { copied_local: 0, assigned_local: 0 },
                  Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 1 }] {
        if corresponds(original, &wrong) { panic!("wrong local mapping accepted"); }
        count += 1;
    }
    println!("TYPED_CONTROLS {count}");
}

struct Query;
impl rustc_driver::Callbacks for Query {
    fn after_analysis<'tcx>(&mut self, _: &rustc_interface::interface::Compiler, tcx: TyCtxt<'tcx>)
        -> rustc_driver::Compilation
    {
        let roots: Vec<_> = tcx.hir_body_owners().filter(|id|
            tcx.def_kind(id.to_def_id()) == rustc_hir::def::DefKind::Fn
                && tcx.item_name(id.to_def_id()).as_str().starts_with("probe_")).collect();
        if roots.is_empty() { panic!("no probe roots"); }
        for root in roots {
            let def = root.to_def_id();
            let env = ty::TypingEnv::non_body_analysis(tcx, def);
            let caller = tcx.optimized_mir(def);
            let calls: Vec<_> = caller.basic_blocks.iter().filter_map(|bb| {
                if let mir::TerminatorKind::Call { func, .. } = &bb.terminator().kind {
                    Some(func.ty(&caller.local_decls, tcx))
                } else { None }
            }).collect();
            let name = tcx.item_name(def);
            println!("ROOT {name}");
            if calls.len() != 1 { println!("REJECT direct_call_count"); continue; }
            let ty::FnDef(f, args) = *calls[0].kind() else {
                println!("REJECT not_direct_FnDef_call"); continue;
            };
            let Some(args) = args.no_bound_vars() else {
                println!("REJECT bound_call_arguments"); continue;
            };
            let instance = match semantic_instance(tcx, env, f, args) {
                Ok(i) => i,
                Err(e) => { println!("REJECT {e}"); continue; }
            };
            let body = instance.instantiate_mir_and_normalize_erasing_regions(tcx, env,
                ty::EarlyBinder::bind(tcx, tcx.instance_mir(instance.def).clone()));
            println!("IDENTITY {:?} {:?} {:?} {:?} {:?}", tcx.def_path_hash(instance.def_id()),
                tcx.stable_crate_id(instance.def_id().krate), tcx.crate_hash(instance.def_id().krate),
                tcx.sess.target.llvm_target, tcx.fn_sig(instance.def_id()));
            println!("RAW_FOREIGN_MIR {body:#?}");
            match inspect(tcx, env, &body) {
                Ok(p) => {
                    if !corresponds(&body, &p) { panic!("typed original correspondence mismatch"); }
                    println!("OBSERVATION {p:?}");
                    println!("ORIGIN {}", tcx.sess.source_map().span_to_diagnostic_string(
                        body.basic_blocks[mir::START_BLOCK].statements[0].source_info.span));
                    shape_controls(tcx, env, &body);
                    let origin = tcx.sess.source_map().span_to_diagnostic_string(
                        body.basic_blocks[mir::START_BLOCK].statements[0].source_info.span);
                    let f = translate(&p, &origin);
                    vir_controls(&body, &f);
                    println!("RAW_ORDINARY_VIR {f:#?}");
                    println!("BLOCKED ordinary_VIR_registration_proof_and_certified_import_missing");
                }
                Err(e) => println!("REJECT {e}"),
            }
        }
        println!("CONTRACT_AVAILABLE false\nFOREIGN_BRIDGE_PROVED false\nPUBLICATION false");
        rustc_driver::Compilation::Stop
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    rustc_driver::run_compiler(&args, &mut Query);
}
