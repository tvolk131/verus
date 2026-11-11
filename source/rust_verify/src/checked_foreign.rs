//! Bounded trusted frontend lowering of an actual body, never an assumed law.
//! Full contract checking is performed by the ordinary Query / solver pipeline.
use crate::context::Context;
use crate::rust_to_vir_func::{CheckItemFnEither, FunctionOrConstInfo};
use rustc_hir::def_id::DefId;
use rustc_hir::{
    ExprKind,
    intravisit::{self, Visitor},
};
use rustc_middle::{
    mir,
    ty::{self, TyCtxt, TypeVisitableExt},
};
use std::sync::Arc;
use vir::{
    ast::*,
    ast_util::{bool_typ, const_int_from_u128, str_unique_var},
    def::Spanned,
};
/// Required, versioned evidence, retained independently of public function pruning.
/// Hashes bind inherited artifacts; they are not independent proof objects.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignIdentity {
    pub semantic_instance: String,
    pub typed_mir: String,
    pub compiler: String,
    pub core_artifact: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignCertificate {
    pub identity: ForeignIdentity,
    pub checked_function: String,
}

type Reject = &'static str;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Primitive {
    ReturnBoolToU64 { copied_local: usize, assigned_local: usize },
}

fn semantic_instance<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Result<ty::Instance<'tcx>, Reject> {
    let core = tcx.lang_items().sized_trait().ok_or("missing_core_lang_item")?.krate;
    let from = tcx
        .get_diagnostic_item(rustc_span::Symbol::intern("From"))
        .ok_or("missing_From_diagnostic_item")?;
    if from.krate != core {
        return Err("From_not_in_semantic_core");
    }
    let methods: Vec<_> = tcx
        .associated_items(from)
        .in_definition_order()
        .filter(|i| tcx.def_kind(i.def_id) == rustc_hir::def::DefKind::AssocFn)
        .collect();
    if methods.len() != 1 {
        return Err("unsupported_From_trait_interface");
    }
    let method = methods[0].def_id;
    // MIR may name the trait item or an already resolved impl item.
    if f != method
        && (tcx.def_kind(f) != rustc_hir::def::DefKind::AssocFn
            || tcx.associated_item(f).trait_item_def_id() != Some(method))
    {
        return Err("callee_not_core_From_method");
    }
    if args.has_non_region_param() || args.has_infer() || args.has_escaping_bound_vars() {
        return Err("unresolved_substitutions");
    }
    let instance = ty::Instance::try_resolve(tcx, env, f, args)
        .map_err(|_| "instance_resolution_failed")?
        .ok_or("instance_unavailable")?;
    let ty::InstanceKind::Item(def) = instance.def else {
        return Err("not_Item_instance");
    };
    if def.krate != core || !instance.args.is_empty() {
        return Err("non_core_or_nonempty_instance_args");
    }
    if tcx.associated_item(def).trait_item_def_id() != Some(method) {
        return Err("resolved_item_not_original_trait_method");
    }
    let tr = tcx.normalize_erasing_regions(
        env,
        tcx.impl_trait_ref(tcx.parent(def)).instantiate(tcx, instance.args),
    );
    if tr.def_id != from
        || tr.args.len() != 2
        || !matches!(tr.self_ty().kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(tr.args.type_at(1).kind(), ty::Bool)
    {
        return Err("wrong_trait_input_or_Self");
    }
    let poly = tcx.normalize_erasing_regions(env, tcx.fn_sig(def).instantiate(tcx, instance.args));
    if !poly.bound_vars().is_empty() {
        return Err("bound_signature");
    }
    let sig = poly.skip_binder();
    if sig.inputs().len() != 1
        || !matches!(sig.inputs()[0].kind(), ty::Bool)
        || !matches!(sig.output().kind(), ty::Uint(ty::UintTy::U64))
        || sig.c_variadic()
        || sig.safety() != rustc_hir::Safety::Safe
        || sig.abi() != rustc_abi::ExternAbi::Rust
    {
        return Err("wrong_signature_or_ABI");
    }
    if !tcx.is_mir_available(def) {
        return Err("foreign_MIR_unavailable");
    }
    Ok(instance)
}

fn inspect<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
) -> Result<Primitive, Reject> {
    if !matches!(body.phase, mir::MirPhase::Runtime(mir::RuntimePhase::Optimized))
        || body.coroutine.is_some()
        || body.spread_arg.is_some()
        || body.is_polymorphic
        || body.injection_phase.is_some()
        || body.tainted_by_errors.is_some()
        || body.coverage_info_hi.is_some()
        || body.function_coverage_info.is_some()
        || !body.required_consts.as_ref().is_some_and(|c| c.is_empty())
        || !body.mentioned_items.as_ref().is_some_and(|c| c.is_empty())
    {
        return Err("unsupported_body_properties");
    }
    // Spans/scopes, source names, user type annotations and debug info are
    // diagnostic metadata, not executable effects. Every executable node,
    // local type and potential drop is checked below, including unreachable BBs.
    if body.arg_count != 1 || body.local_decls.len() != 2 || body.basic_blocks.len() != 1 {
        return Err("local_argument_or_block_count");
    }
    if !matches!(body.local_decls[mir::RETURN_PLACE].ty.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[mir::Local::from_usize(1)].ty.kind(), ty::Bool)
    {
        return Err("local_types");
    }
    if body.local_decls.iter().any(|d| d.ty.needs_drop(tcx, env)) {
        return Err("needs_drop");
    }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    if bb.is_cleanup || bb.statements.len() != 1 {
        return Err("cleanup_or_statement_count");
    }
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
    if input.local != mir::Local::from_usize(1)
        || !input.projection.is_empty()
        || !matches!(out.kind(), ty::Uint(ty::UintTy::U64))
    {
        return Err("input_place_or_cast_type");
    }
    Ok(Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 0 })
}

// Independent MIR/local-map correspondence; this is a frontend check, not a body proof.
fn corresponds(body: &mir::Body<'_>, p: &Primitive) -> bool {
    if body.basic_blocks.len() != 1 {
        return false;
    }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    let [s] = bb.statements.as_slice() else { return false };
    let mir::StatementKind::Assign(a) = &s.kind else { return false };
    match (&a.0, &a.1, &bb.terminator().kind, p) {
        (
            target,
            mir::Rvalue::Cast(mir::CastKind::IntToInt, mir::Operand::Copy(source), t),
            mir::TerminatorKind::Return,
            Primitive::ReturnBoolToU64 { copied_local, assigned_local },
        ) => {
            *copied_local == source.local.as_usize()
                && *assigned_local == target.local.as_usize()
                && *copied_local == 1
                && *assigned_local == 0
                && source.projection.is_empty()
                && target.projection.is_empty()
                && matches!(t.kind(), ty::Uint(ty::UintTy::U64))
                && !bb.is_cleanup
                && body.arg_count == 1
                && body.local_decls.len() == 2
                && matches!(body.local_decls[source.local].ty.kind(), ty::Bool)
                && matches!(body.local_decls[target.local].ty.kind(), ty::Uint(ty::UintTy::U64))
        }
        _ => false,
    }
}

fn is_u64(t: &Typ) -> bool {
    matches!(&**t, TypX::Int(IntRange::U(64)))
}
fn literal(e: &vir::ast::Expr, n: u128) -> bool {
    is_u64(&e.typ) && matches!(&e.x, ExprX::Const(c) if *c == const_int_from_u128(n))
}

// Check the *original* typed MIR against actual ordinary VIR, independently of
// both the recognizer's returned descriptor and the proposed postcondition.
fn corresponds_vir(body: &mir::Body<'_>, f: &Function) -> bool {
    if body.arg_count != 1
        || body.local_decls.len() != 2
        || body.basic_blocks.len() != 1
        || f.x.mode != Mode::Exec
        || f.x.params.len() != 1
        || f.x.attrs.is_external_body
        || !f.x.typ_params.is_empty()
        || !f.x.typ_bounds.is_empty()
        || !matches!(f.x.kind, FunctionKind::Static)
        || f.x.fndef_axioms.is_some()
        || f.x.proxy.is_some()
        || f.x.returns.is_some()
        || !f.x.decrease.is_empty()
        || f.x.decrease_when.is_some()
        || f.x.decrease_by.is_some()
        || f.x.mask_spec.is_some()
        || f.x.atomic_update.is_some()
        || f.x.unwind_spec.is_some()
        || !f.x.return_opaque_type_eqs.is_empty()
        || !f.x.extra_dependencies.is_empty()
        || !f.x.hidden.is_empty()
    {
        return false;
    }
    let input = &f.x.params[0].x;
    if input.mode != Mode::Exec
        || !matches!(&*input.typ, TypX::Bool)
        || !is_u64(&f.x.inner_ret.x.typ)
        || !is_u64(&f.x.outer_ret.x.typ)
        || f.x.inner_ret.x.name != f.x.outer_ret.x.name
        || input.user_mut
        || input.unwrapped_info.is_some()
        || f.x.inner_ret.x.unwrapped_info.is_some()
        || f.x.outer_ret.x.unwrapped_info.is_some()
        || f.x.inner_ret.x.user_mut
        || f.x.outer_ret.x.user_mut
        || input.name == f.x.inner_ret.x.name
        || f.x.inner_ret.x.mode != Mode::Exec
        || f.x.outer_ret.x.mode != Mode::Exec
    {
        return false;
    }
    let bb = &body.basic_blocks[mir::START_BLOCK];
    let [s] = bb.statements.as_slice() else { return false };
    let mir::StatementKind::Assign(a) = &s.kind else { return false };
    let mir::Rvalue::Cast(mir::CastKind::IntToInt, mir::Operand::Copy(p), t) = &a.1 else {
        return false;
    };
    if bb.is_cleanup
        || !matches!(bb.terminator().kind, mir::TerminatorKind::Return)
        || a.0.local != mir::RETURN_PLACE
        || !a.0.projection.is_empty()
        || p.local != mir::Local::from_usize(1)
        || !p.projection.is_empty()
        || !matches!(t.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[a.0.local].ty.kind(), ty::Uint(ty::UintTy::U64))
        || !matches!(body.local_decls[p.local].ty.kind(), ty::Bool)
    {
        return false;
    }
    let Some(e) = &f.x.body else { return false };
    let ExprX::If(c, one, Some(zero)) = &e.x else { return false };
    let ExprX::ReadPlace(place, read) = &c.x else { return false };
    is_u64(&e.typ)
        && literal(one, 1)
        && literal(zero, 0)
        && matches!(&*c.typ, TypX::Bool)
        && matches!(&*place.typ, TypX::Bool)
        && matches!(&place.x, PlaceX::Local(n) if *n == input.name)
        && matches!(read.preliminary_kind, ReadKind::Copy)
}

fn translate(ctxt: &Context<'_>, origin: rustc_span::Span, p: &Primitive) -> Function {
    let Primitive::ReturnBoolToU64 { copied_local: 1, assigned_local: 0 } = p else {
        unreachable!("only inspected MIR can be lowered");
    };
    let span = || {
        let mut s = ctxt.spans.to_air_span(origin);
        s.raw_span = Arc::new(());
        s
    };
    let nonce = span().id;
    let b = str_unique_var(&format!("foreign_input_{nonce}"), VarIdentDisambiguate::NoBodyParam);
    let r = str_unique_var(&format!("foreign_result_{nonce}"), VarIdentDisambiguate::NoBodyParam);
    let u64t = Arc::new(TypX::Int(IntRange::U(64)));
    let e = |t: &Typ, x| SpannedTyped::new(&span(), t, x);
    let param = |name, typ| {
        Spanned::new(
            span(),
            ParamX { name, typ, mode: Mode::Exec, user_mut: false, unwrapped_info: None },
        )
    };
    let input = param(b.clone(), bool_typ());
    let ret = param(r.clone(), u64t.clone());
    // This is the existing ordinary bool IntToInt cast encoding.
    let cast = |c| {
        e(
            &u64t,
            ExprX::If(
                c,
                e(&u64t, ExprX::Const(const_int_from_u128(1))),
                Some(e(&u64t, ExprX::Const(const_int_from_u128(0)))),
            ),
        )
    };
    let place = SpannedTyped::new(&span(), &bool_typ(), PlaceX::Local(b.clone()));
    let body = cast(e(
        &bool_typ(),
        ExprX::ReadPlace(
            place,
            UnfinalizedReadKind {
                preliminary_kind: ReadKind::Copy,
                id: ctxt.unique_read_kind_id(),
            },
        ),
    ));
    // Independent ordinary contract proposal, NOT inference-summary synthesis.
    let ensure = e(
        &bool_typ(),
        ExprX::Binary(
            BinaryOp::Eq(Mode::Spec),
            e(&u64t, ExprX::Var(r)),
            cast(e(&bool_typ(), ExprX::Var(b))),
        ),
    );
    let root = crate::rust_to_vir::get_root_module_path(ctxt);
    let mut path = (*root).clone();
    Arc::make_mut(&mut path.segments).push(Arc::new("checked_foreign_bool_to_u64%".into()));
    let visibility = Visibility { restricted_to: Some(root.clone()) };
    Spanned::new(
        span(),
        FunctionX {
            name: Arc::new(FunX { path: Arc::new(path) }),
            proxy: None,
            kind: FunctionKind::Static,
            visibility: visibility.clone(),
            body_visibility: BodyVisibility::Visibility(visibility),
            opaqueness: Opaqueness::Opaque,
            owning_module: Some(root),
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(vec![input]),
            outer_ret: ret.clone(),
            inner_ret: ret,
            require: Arc::new(vec![]),
            ensure: (Arc::new(vec![ensure]), Arc::new(vec![])),
            returns: None,
            decrease: Arc::new(vec![]),
            decrease_when: None,
            decrease_by: None,
            fndef_axioms: None,
            mask_spec: None,
            atomic_update: None,
            unwind_spec: None,
            item_kind: ItemKind::Function,
            attrs: Arc::new(FunctionAttrsX { uses_ghost_blocks: true, ..Default::default() }),
            body: Some(body),
            return_opaque_type_eqs: Arc::new(vec![]),
            extra_dependencies: vec![],
            hidden: Arc::new(vec![]),
        },
    )
}

/// Only direct syntactic calls are eligible. No FnDef value/coercion mapping.
struct Calls<'a, 'tcx> {
    ctxt: &'a Context<'tcx>,
    caller: DefId,
    types: &'tcx ty::TypeckResults<'tcx>,
    found: Vec<(rustc_hir::HirId, ty::Instance<'tcx>, rustc_span::Span)>,
}
impl<'tcx> Visitor<'tcx> for Calls<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        let call = match expr.kind {
            ExprKind::Call(callee, _) if matches!(callee.kind, ExprKind::Path(_)) => {
                let ExprKind::Path(qpath) = &callee.kind else { unreachable!() };
                // A local holding a FnDef is still a function value. Only a
                // source path resolving directly to a function definition is
                // eligible; no value/coercion identity is rewritten.
                if matches!(
                    self.types.qpath_res(qpath, callee.hir_id),
                    rustc_hir::def::Res::Def(
                        rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn,
                        _
                    )
                ) {
                    match self.types.expr_ty(callee).kind() {
                        ty::FnDef(f, args) => args.no_bound_vars().map(|args| (*f, args)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            ExprKind::MethodCall(..) => self
                .types
                .type_dependent_def_id(expr.hir_id)
                .map(|f| (f, self.types.node_args(expr.hir_id))),
            _ => None,
        };
        if let Some((f, args)) = call {
            let env = ty::TypingEnv::non_body_analysis(self.ctxt.tcx, self.caller);
            if let Ok(instance) = semantic_instance(self.ctxt.tcx, env, f, args) {
                self.found.push((expr.hir_id, instance, expr.span));
            }
        }
        intravisit::walk_expr(self, expr);
    }
}

pub(crate) fn register<'tcx>(
    ctxt: &Context<'tcx>,
    state: &mut crate::rust_to_vir::State,
    infos: &mut Vec<FunctionOrConstInfo<'tcx>>,
) -> Result<(), VirErr> {
    if !ctxt.cmd_line_args.infer_goals {
        return Ok(());
    }
    let mut found = vec![];
    for info in infos.iter() {
        let (caller, body) = match info {
            FunctionOrConstInfo::Function(f) if !f.vattrs.external_body => match f.body_id {
                CheckItemFnEither::BodyId(body) => (f.def_id, body),
                _ => continue,
            },
            _ => continue,
        };
        let mut calls =
            Calls { ctxt, caller, types: ctxt.tcx.typeck(caller.expect_local()), found: vec![] };
        calls.visit_body(ctxt.tcx.hir_body(body));
        found.extend(calls.found.into_iter().map(|x| (caller, x)));
    }
    let mut full: Option<Function> = None;
    for (caller, (hir, instance, origin)) in found {
        let env = ty::TypingEnv::non_body_analysis(ctxt.tcx, caller);
        let body = instance.instantiate_mir_and_normalize_erasing_regions(
            ctxt.tcx,
            env,
            ty::EarlyBinder::bind(ctxt.tcx, ctxt.tcx.instance_mir(instance.def).clone()),
        );
        let primitive = inspect(ctxt.tcx, env, &body).map_err(|e| {
            crate::util::err_span::<(), _>(origin, &format!("checked foreign MIR unsupported: {e}"))
                .unwrap_err()
        })?;
        let function = full.get_or_insert_with(|| translate(ctxt, origin, &primitive));
        // Re-read original MIR and compare the actual full body independently
        // of both the observation descriptor and the proposed postcondition.
        if !corresponds_vir(&body, function) || !corresponds(&body, &primitive) {
            return crate::util::err_span(origin, "checked foreign MIR/VIR correspondence failed");
        }
        ctxt.checked_foreign_calls
            .borrow_mut()
            .insert(hir, (instance.def_id(), function.x.name.clone()));
        state.insert_fun_warn_config(ctxt, &function.x.name, caller);
    }
    if let Some(full) = full {
        infos.push(FunctionOrConstInfo::BodyBackedForeign {
            stub: vir::ast_util::function_to_stub(&full),
            full,
        });
    }
    Ok(())
}

pub(crate) fn direct_helper<'tcx>(
    bctx: &crate::context::BodyCtxt<'tcx>,
    expr: &rustc_hir::Expr<'tcx>,
    f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Option<Fun> {
    let calls = bctx.ctxt.checked_foreign_calls.borrow();
    let (expected, helper) = calls.get(&expr.hir_id)?;
    let env = ty::TypingEnv::non_body_analysis(bctx.ctxt.tcx, bctx.fun_id);
    let instance = semantic_instance(bctx.ctxt.tcx, env, f, args).ok()?;
    if instance.def_id() != *expected {
        return None;
    }
    Some(helper.clone())
}
