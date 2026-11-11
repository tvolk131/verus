//! Stage1 local carrier only. Separate from bool certificates and recognition.
//! No source call may be redirected until the actual full Function has passed
//! independent correspondence and its helper enters the ordinary body pipeline.
use rustc_middle::{mir, ty};
use vir::ast::{Fun, Function};

type Reject = &'static str;

struct ConcreteOrderingInstances<'tcx> {
    carrier: ty::Instance<'tcx>,
    get: ty::Instance<'tcx>,
    primitive: ty::Instance<'tcx>,
    nonzero: ty::Ty<'tcx>,
    ordering: ty::Ty<'tcx>,
}

/// No public constructor: this is an operational check result, not a proof.
struct CheckedExpansion {
    statement: usize,
    formal: mir::Local,
    result: mir::Local,
}

/// Must bind existing imported functions by semantic source/model identities.
struct OrderingModelBindings {
    get: Fun,
    spec_get: Fun,
    view: Fun,
    primitive: Fun,
    obeys_cmp_spec: Fun,
    cmp_spec: Fun,
}

use rustc_hir::def_id::DefId;
use rustc_middle::ty::{TyCtxt, TypeVisitableExt};
use rustc_span::Symbol;

fn demand(ok: bool, reason: Reject) -> Result<(), Reject> {
    if ok { Ok(()) } else { Err(reason) }
}

fn item<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Result<ty::Instance<'tcx>, Reject> {
    demand(
        !args.has_non_region_param() && !args.has_infer() && !args.has_escaping_bound_vars(),
        "unresolved_instance_arguments",
    )?;
    let i = ty::Instance::try_resolve(tcx, env, f, args)
        .map_err(|_| "instance_resolution_error")?
        .ok_or("instance_unavailable")?;
    demand(matches!(i.def, ty::InstanceKind::Item(_)), "non_Item_instance")?;
    demand(tcx.is_mir_available(i.def_id()), "missing_instance_MIR")?;
    Ok(i)
}

fn method(tcx: TyCtxt<'_>, owner: DefId, name: &str) -> Result<DefId, Reject> {
    let mut found = tcx.associated_items(owner).in_definition_order().filter(|a| {
        tcx.def_kind(a.def_id) == rustc_hir::def::DefKind::AssocFn && a.ident(tcx).as_str() == name
    });
    let id = found.next().ok_or("missing_semantic_owner_method")?.def_id;
    demand(found.next().is_none(), "ambiguous_semantic_owner_method")?;
    Ok(id)
}

fn signature<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    i: ty::Instance<'tcx>,
    inputs: &[ty::Ty<'tcx>],
    output: ty::Ty<'tcx>,
) -> Result<(), Reject> {
    let poly = tcx.normalize_erasing_regions(env, tcx.fn_sig(i.def_id()).instantiate(tcx, i.args));
    // Only ordinary shared-reference lifetime binders are admitted. Normalize
    // after validating the binders, never erase a higher-ranked type/const need.
    demand(
        poly.bound_vars().iter().all(|v| matches!(v, ty::BoundVariableKind::Region(_))),
        "non_region_signature_binder",
    )?;
    let sig = tcx.normalize_erasing_late_bound_regions(env, poly);
    demand(
        sig.inputs() == inputs
            && sig.output() == output
            && !sig.c_variadic()
            && sig.safety() == rustc_hir::Safety::Safe
            && sig.abi() == rustc_abi::ExternAbi::Rust,
        "signature_ABI_or_safety",
    )
}

fn resolve_ordering<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Result<ConcreteOrderingInstances<'tcx>, Reject> {
    let core = tcx.lang_items().sized_trait().ok_or("missing_core")?.krate;
    let ord = tcx.get_diagnostic_item(Symbol::intern("Ord")).ok_or("missing_Ord")?;
    let nz = tcx.get_diagnostic_item(Symbol::intern("NonZero")).ok_or("missing_NonZero")?;
    demand(ord.krate == core && nz.krate == core, "non_core_identity")?;
    let cmp = method(tcx, ord, "cmp")?;
    demand(
        f == cmp
            || (tcx.def_kind(f) == rustc_hir::def::DefKind::AssocFn
                && tcx.associated_item(f).trait_item_def_id() == Some(cmp)),
        "not_Ord_cmp",
    )?;
    let carrier = item(tcx, env, f, args)?;
    demand(
        carrier.def_id().krate == core
            && tcx.associated_item(carrier.def_id()).trait_item_def_id() == Some(cmp)
            && carrier.args == tcx.mk_args(&[tcx.types.u64.into()]),
        "wrong_carrier_instance",
    )?;
    let nonzero = ty::Ty::new_adt(tcx, tcx.adt_def(nz), tcx.mk_args(&[tcx.types.u64.into()]));
    let tr = tcx.normalize_erasing_regions(
        env,
        tcx.impl_trait_ref(tcx.parent(carrier.def_id())).instantiate(tcx, carrier.args),
    );
    demand(
        tr.def_id == ord && tr.args.len() == 1 && tr.self_ty() == nonzero,
        "wrong_carrier_trait_ref",
    )?;
    let trait_sig = tcx.normalize_erasing_late_bound_regions(
        env,
        tcx.normalize_erasing_regions(
            env,
            tcx.fn_sig(cmp).instantiate(tcx, tcx.mk_args(&[nonzero.into()])),
        ),
    );
    let ordering = trait_sig.output();
    let ty::Adt(adt, enum_args) = ordering.kind() else { return Err("non_ADT_Ord_result") };
    demand(
        adt.did().krate == core
            && enum_args.is_empty()
            && adt.is_enum()
            && adt.variants().len() == 3
            && adt
                .variants()
                .iter()
                .zip(["Less", "Equal", "Greater"])
                .all(|(v, name)| v.name.as_str() == name && v.fields.is_empty()),
        "wrong_Ord_result_enum",
    )?;
    let shared = |t| ty::Ty::new_imm_ref(tcx, tcx.lifetimes.re_erased, t);
    signature(tcx, env, carrier, &[shared(nonzero), shared(nonzero)], ordering)?;
    let mut getters = vec![];
    for &imp in tcx.inherent_impls(nz) {
        if let Ok(get) = method(tcx, imp, "get") {
            let i = item(tcx, env, get, tcx.mk_args(&[tcx.types.u64.into()]))?;
            demand(
                i.def_id().krate == core && tcx.trait_of_assoc(i.def_id()).is_none(),
                "non_inherent_core_get",
            )?;
            let self_ty =
                tcx.normalize_erasing_regions(env, tcx.type_of(imp).instantiate(tcx, i.args));
            demand(self_ty == nonzero, "wrong_get_Self")?;
            signature(tcx, env, i, &[nonzero], tcx.types.u64)?;
            getters.push(i);
        }
    }
    demand(getters.len() == 1, "missing_or_ambiguous_get")?;
    let primitive = item(tcx, env, cmp, tcx.mk_args(&[tcx.types.u64.into()]))?;
    demand(
        primitive.def_id().krate == core
            && primitive.args.is_empty()
            && tcx.associated_item(primitive.def_id()).trait_item_def_id() == Some(cmp),
        "wrong_primitive_instance",
    )?;
    signature(tcx, env, primitive, &[shared(tcx.types.u64), shared(tcx.types.u64)], ordering)?;
    Ok(ConcreteOrderingInstances { carrier, get: getters[0], primitive, nonzero, ordering })
}

/// This closed typed alphabet loses no executable operation in the supported
/// bodies. Unsupported statements, projections, operands and exits reject.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Op {
    Live(usize),
    Dead(usize),
    Copy { to: usize, from: usize, deref: bool },
    Transmute { to: usize, from: usize },
    Shared { to: usize, from: usize },
    Cmp { to: usize, left: usize, right: usize },
}
fn local(p: &mir::Place<'_>) -> Result<usize, Reject> {
    demand(p.projection.is_empty(), "unexpected_place_projection")?;
    Ok(p.local.as_usize())
}
fn operations<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &mir::Body<'tcx>,
    block: usize,
) -> Result<Vec<Op>, Reject> {
    body.basic_blocks[mir::BasicBlock::from_usize(block)]
        .statements
        .iter()
        .map(|s| {
            Ok(match &s.kind {
                mir::StatementKind::StorageLive(l) => Op::Live(l.as_usize()),
                mir::StatementKind::StorageDead(l) => Op::Dead(l.as_usize()),
                mir::StatementKind::Assign(a) => {
                    let to = local(&a.0)?;
                    match &a.1 {
                        mir::Rvalue::Use(mir::Operand::Copy(p), mir::WithRetag::Yes) => {
                            let deref = p.projection.as_ref() == [mir::ProjectionElem::Deref];
                            demand(p.projection.is_empty() || deref, "copy_projection")?;
                            Op::Copy { to, from: p.local.as_usize(), deref }
                        }
                        mir::Rvalue::Cast(mir::CastKind::Transmute, mir::Operand::Copy(p), ty)
                            if *ty == tcx.types.u64 =>
                        {
                            Op::Transmute { to, from: local(p)? }
                        }
                        mir::Rvalue::Ref(r, mir::BorrowKind::Shared, p)
                            if *r == tcx.lifetimes.re_erased =>
                        {
                            Op::Shared { to, from: local(p)? }
                        }
                        mir::Rvalue::BinaryOp(mir::BinOp::Cmp, a) => {
                            let (mir::Operand::Move(l), mir::Operand::Move(r)) = &**a else {
                                return Err("Cmp_operands");
                            };
                            Op::Cmp { to, left: local(l)?, right: local(r)? }
                        }
                        _ => return Err("unsupported_rvalue_or_operand"),
                    }
                }
                _ => return Err("unsupported_statement"),
            })
        })
        .collect()
}
fn properties<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    body: &mir::Body<'tcx>,
    instance: ty::Instance<'tcx>,
    types: &[ty::Ty<'tcx>],
    args: usize,
    blocks: usize,
) -> Result<(), Reject> {
    demand(
        matches!(body.phase, mir::MirPhase::Runtime(mir::RuntimePhase::Optimized))
            && body.source.instance == instance.def
            && body.source.promoted.is_none()
            && body.coroutine.is_none()
            && body.spread_arg.is_none()
            && body.injection_phase.is_none()
            && body.tainted_by_errors.is_none()
            && body.coverage_info_hi.is_none()
            && body.function_coverage_info.is_none()
            && body.required_consts.as_ref().is_some_and(|v| v.is_empty())
            && body.arg_count == args
            && body.basic_blocks.len() == blocks
            && body.local_decls.len() == types.len()
            && body.user_type_annotations.is_empty(),
        "body_properties_or_counts",
    )?;
    for (d, expected) in body.local_decls.iter().zip(types) {
        // Exact recursive whitelist: u64, semantic NZ<u64>, semantic Ordering,
        // or shared erased refs thereof. Equality also excludes open generic
        // aliases, constants, escaping variables and unsupported projections.
        demand(
            d.ty == *expected
                && d.user_ty.is_none()
                && !d.ty.has_non_region_param()
                && !d.ty.has_infer()
                && !d.ty.has_escaping_bound_vars()
                && !d.ty.needs_drop(tcx, env)
                && tcx.type_is_copy_modulo_regions(env, d.ty),
            "local_type_Copy_or_drop",
        )?;
    }
    demand(
        body.basic_blocks.iter().all(|b| !b.is_cleanup && b.terminator.is_some()),
        "cleanup_or_missing_terminator",
    )
}
fn empty_mentions(body: &mir::Body<'_>) -> Result<(), Reject> {
    demand(body.mentioned_items.as_ref().is_some_and(|v| v.is_empty()), "unexpected_mentioned_item")
}
fn returns(body: &mir::Body<'_>, block: usize) -> Result<(), Reject> {
    demand(
        matches!(
            body.basic_blocks[mir::BasicBlock::from_usize(block)].terminator().kind,
            mir::TerminatorKind::Return
        ),
        "unexpected_return_or_exit",
    )
}
fn carrier_ops() -> [Vec<Op>; 2] {
    use Op::*;
    [
        vec![
            Live(3),
            Live(4),
            Live(5),
            Copy { to: 5, from: 1, deref: true },
            Transmute { to: 4, from: 5 },
            Shared { to: 3, from: 4 },
            Dead(5),
            Live(6),
            Live(7),
            Live(8),
            Copy { to: 8, from: 2, deref: true },
            Transmute { to: 7, from: 8 },
            Dead(8),
            Shared { to: 6, from: 7 },
        ],
        vec![Dead(7), Dead(6), Dead(4), Dead(3)],
    ]
}

fn inspect_bodies<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    ids: &ConcreteOrderingInstances<'tcx>,
    carrier: &mir::Body<'tcx>,
    get: &mir::Body<'tcx>,
    primitive: &mir::Body<'tcx>,
) -> Result<[CheckedExpansion; 2], Reject> {
    let u = tcx.types.u64;
    let r = |t| ty::Ty::new_imm_ref(tcx, tcx.lifetimes.re_erased, t);
    properties(
        tcx,
        env,
        carrier,
        ids.carrier,
        &[ids.ordering, r(ids.nonzero), r(ids.nonzero), r(u), u, ids.nonzero, r(u), u, ids.nonzero],
        2,
        2,
    )?;
    properties(tcx, env, get, ids.get, &[u, ids.nonzero], 1, 1)?;
    properties(tcx, env, primitive, ids.primitive, &[ids.ordering, r(u), r(u), u, u], 2, 1)?;
    // Generic origin flag is deliberately not used here. All actual types and
    // constants/targets are checked independently in this closed whole shape.
    empty_mentions(get)?;
    empty_mentions(primitive)?;
    returns(get, 0)?;
    returns(primitive, 0)?;
    returns(carrier, 1)?;
    demand(operations(tcx, get, 0)? == [Op::Transmute { to: 0, from: 1 }], "whole_get_body")?;
    use Op::*;
    demand(
        operations(tcx, primitive, 0)?
            == [
                Live(3),
                Copy { to: 3, from: 1, deref: true },
                Live(4),
                Copy { to: 4, from: 2, deref: true },
                Cmp { to: 0, left: 3, right: 4 },
                Dead(4),
                Dead(3),
            ],
        "whole_primitive_dependency",
    )?;
    for (b, ops) in carrier_ops().iter().enumerate() {
        demand(operations(tcx, carrier, b)? == *ops, "whole_carrier_operations")?;
    }
    let term = &carrier.basic_blocks[mir::START_BLOCK].terminator().kind;
    let mir::TerminatorKind::Call { func, args, destination, target, unwind, .. } = term else {
        return Err("carrier_primitive_call");
    };
    demand(
        local(destination)? == 0
            && *target == Some(mir::BasicBlock::from_usize(1))
            && *unwind == mir::UnwindAction::Continue
            && args.len() == 2,
        "call_return_CFG_or_unwind",
    )?;
    for (a, l) in args.iter().zip([3, 6]) {
        demand(
            matches!(&a.node,mir::Operand::Move(p) if local(p) == Ok(l)),
            "primitive_argument_order",
        )?;
    }
    demand(
        matches!(func, mir::Operand::Constant(c)
        if c.user_ty.is_none() && matches!(c.const_,mir::Const::Val(mir::ConstValue::ZeroSized,_))),
        "callee_not_literal_function_item",
    )?;
    let ty::FnDef(def, args) = *func.ty(&carrier.local_decls, tcx).kind() else {
        return Err("non_FnDef_callee");
    };
    let args = args.no_bound_vars().ok_or("callee_bound_args")?;
    demand(item(tcx, env, def, args)? == ids.primitive, "nested_primitive_identity")?;
    let mentions = carrier.mentioned_items.as_ref().ok_or("missing_mentions")?;
    demand(mentions.len() == 1, "carrier_mentioned_count")?;
    let mir::MentionedItem::Fn(t) = mentions[0].node else { return Err("non_function_mention") };
    let ty::FnDef(d, a) = *t.kind() else { return Err("non_FnDef_mention") };
    demand(
        item(tcx, env, d, a.no_bound_vars().ok_or("mention_bound_args")?)? == ids.primitive,
        "mentioned_primitive_identity",
    )?;
    Ok([
        check_get_expansion(tcx, carrier, get, 4, 5, 4)?,
        check_get_expansion(tcx, carrier, get, 11, 8, 7)?,
    ])
}

fn check_get_expansion<'tcx>(
    tcx: TyCtxt<'tcx>,
    carrier: &mir::Body<'tcx>,
    get: &mir::Body<'tcx>,
    statement: usize,
    formal: usize,
    result: usize,
) -> Result<CheckedExpansion, Reject> {
    demand(formal != result && formal > 2 && result > 2, "expansion_map_not_injective")?;
    demand(
        formal < carrier.local_decls.len()
            && result < carrier.local_decls.len()
            && statement < carrier.basic_blocks[mir::START_BLOCK].statements.len(),
        "expansion_map_out_of_bounds",
    )?;
    let original = operations(tcx, get, 0)?;
    demand(
        original.len() == 1
            && get.basic_blocks.len() == 1
            && get.local_decls.len() == 2
            && get.arg_count == 1,
        "expansion_get_count",
    )?;
    returns(get, 0)?;
    let mapped = original
        .iter()
        .map(|op| match op {
            Op::Transmute { to: 0, from: 1 } => Ok(Op::Transmute { to: result, from: formal }),
            _ => Err("expansion_unsupported_get_operation"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ops = operations(tcx, carrier, 0)?;
    demand(ops.get(statement..statement + 1) == Some(mapped.as_slice()), "exact_get_expansion")?;
    demand(
        carrier.local_decls[mir::Local::from_usize(formal)].ty
            == get.local_decls[mir::Local::from_usize(1)].ty
            && carrier.local_decls[mir::Local::from_usize(result)].ty
                == get.local_decls[mir::RETURN_PLACE].ty,
        "typed_expansion_substitution",
    )?;
    Ok(CheckedExpansion {
        statement,
        formal: mir::Local::from_usize(formal),
        result: mir::Local::from_usize(result),
    })
}

use crate::context::Context;
use std::sync::Arc;
use vir::ast::*;
use vir::ast_util::{bool_typ, str_unique_var};
use vir::def::Spanned;

fn bind_models(
    ctxt: &Context<'_>,
    ids: &ConcreteOrderingInstances<'_>,
    imported: &[Krate],
) -> Result<OrderingModelBindings, Reject> {
    let fs: Vec<_> = imported.iter().flat_map(|k| k.functions.iter()).collect();
    let by_id = |i: ty::Instance<'_>| -> Result<&Function, Reject> {
        let path = ctxt.def_id_to_vir_path(i.def_id());
        let mut found = fs.iter().filter(|f| f.x.name.path == path);
        let f = *found.next().ok_or("missing_ordinary_external_model")?;
        demand(found.next().is_none(), "duplicate_ordinary_external_model")?;
        demand(
            f.x.mode == Mode::Exec
                && f.x.body.is_none()
                && f.x.proxy.as_ref().is_some_and(|p| p.x.krate == CrateId::Vstd),
            "external_model_proxy_not_vstd",
        )?;
        Ok(f)
    };
    let get = by_id(ids.get)?;
    let primitive = by_id(ids.primitive)?;
    let spec_get = get.x.attrs.autospec.as_ref().ok_or("get_autospec_binding_missing")?;
    let spec = fs.iter().find(|f| &f.x.name == spec_get).ok_or("spec_get_definition_missing")?;
    demand(
        spec.x.mode == Mode::Spec
            && spec.x.attrs.inline
            && spec.x.body.is_some()
            && spec.x.name.path.krate == CrateId::Vstd
            && spec.x.params.len() == 1
            && spec.x.require.is_empty(),
        "spec_get_definition_shape",
    )?;
    let mut spec_body = spec.x.body.as_ref().unwrap();
    while let ExprX::Block(stmts, Some(tail)) = &spec_body.x {
        demand(stmts.is_empty(), "spec_get_added_effect")?;
        spec_body = tail;
    }
    let ExprX::Call { target: CallTarget::Fun(_, view, ..), args, post_args: None, body: None } =
        &spec_body.x
    else {
        return Err("spec_get_View_call_missing");
    };
    demand(view.path.krate == CrateId::Vstd && args.len() == 1, "spec_get_View_signature")?;
    let view = view.clone();
    let ord_path = ctxt.def_id_to_vir_path(
        ctxt.tcx.get_diagnostic_item(Symbol::intern("Ord")).ok_or("missing_Ord")?,
    );
    let ord = imported
        .iter()
        .flat_map(|k| k.traits.iter())
        .find(|t| t.x.name == ord_path)
        .ok_or("ordinary_Ord_trait_model_missing")?;
    demand(
        ord.x.proxy.as_ref().is_some_and(|p| p.x.krate == CrateId::Vstd),
        "Ord_external_trait_proxy",
    )?;
    let (implementation, _) =
        ord.x.external_trait_extension.as_ref().ok_or("Ord_extension_missing")?;
    let u64t = Arc::new(TypX::Int(IntRange::U(64)));
    let find_impl = |name: &str| -> Result<Fun, Reject> {
        let mut found = fs.iter().filter(|f| {
            matches!(&f.x.kind,FunctionKind::TraitMethodImpl{trait_path,trait_typ_args,..}
                if trait_path == implementation && trait_typ_args.len() == 1
                && same_serial(&trait_typ_args[0],&u64t))
                && f.x.mode == Mode::Spec
                && f.x.name.path.last_segment().as_str() == name
                && f.x.name.path.krate == CrateId::Vstd
                && f.x.body.is_some()
        });
        let f = found.next().ok_or("actual_u64_OrdSpecImpl_missing")?;
        demand(found.next().is_none(), "actual_u64_OrdSpecImpl_ambiguous")?;
        if name == "obeys_cmp_spec" {
            let mut b = f.x.body.as_ref().unwrap();
            while let ExprX::Block(s, Some(t)) = &b.x {
                demand(s.is_empty(), "obeys_cmp_spec_effect")?;
                b = t;
            }
            demand(matches!(&b.x, ExprX::Const(Constant::Bool(true))), "obeys_cmp_spec_not_true")?;
        }
        Ok(f.x.name.clone())
    };
    Ok(OrderingModelBindings {
        get: get.x.name.clone(),
        spec_get: spec_get.clone(),
        view,
        primitive: primitive.x.name.clone(),
        obeys_cmp_spec: find_impl("obeys_cmp_spec")?,
        cmp_spec: find_impl("cmp_spec")?,
    })
}

fn model_target<'tcx>(
    ctxt: &Context<'tcx>,
    caller: DefId,
    origin: rustc_span::Span,
    i: ty::Instance<'tcx>,
    name: &Fun,
) -> Result<CallTarget, VirErr> {
    // Exactly the existing frontend's type/trait dependency collection; no
    // arbitrary impl-path placeholder and no assume-external fallback.
    let mut types = vec![];
    for a in i.args.iter() {
        let ty::GenericArgKind::Type(t) = a.kind() else {
            return crate::util::err_span(origin, "local ordering non-type model argument");
        };
        types.push(ctxt.mid_ty_to_vir(caller, origin, &t, None)?);
    }
    let paths = crate::rust_to_vir_base::get_impl_paths(
        ctxt.tcx,
        &ctxt.verus_items,
        caller,
        i.def_id(),
        i.args,
        None,
        origin,
    )?;
    let attrs = CallTargetAttrs {
        autospec: AutospecUsage::Final,
        const_var: false,
        assume_external_allowed: false,
    };
    if ctxt.tcx.def_kind(i.def_id()) == rustc_hir::def::DefKind::AssocFn {
        if let Some(method) = ctxt.tcx.associated_item(i.def_id()).trait_item_def_id() {
            let env = ty::TypingEnv::non_body_analysis(ctxt.tcx, caller);
            let tr = ctxt.tcx.normalize_erasing_regions(
                env,
                ctxt.tcx.impl_trait_ref(ctxt.tcx.parent(i.def_id())).instantiate(ctxt.tcx, i.args),
            );
            let mut trait_types = vec![];
            for a in tr.args.iter() {
                let ty::GenericArgKind::Type(t) = a.kind() else {
                    return crate::util::err_span(origin, "local ordering non-type trait argument");
                };
                trait_types.push(ctxt.mid_ty_to_vir(caller, origin, &t, None)?);
            }
            let trait_paths = crate::rust_to_vir_base::get_impl_paths(
                ctxt.tcx,
                &ctxt.verus_items,
                caller,
                method,
                tr.args,
                None,
                origin,
            )?;
            return Ok(CallTarget::Fun(
                CallTargetKind::DynamicResolved {
                    resolved: name.clone(),
                    typs: Arc::new(types),
                    impl_paths: paths,
                    is_trait_default: false,
                },
                Arc::new(FunX { path: ctxt.def_id_to_vir_path(method) }),
                Arc::new(trait_types),
                trait_paths,
                attrs,
            ));
        }
    }
    Ok(CallTarget::Fun(CallTargetKind::Static, name.clone(), Arc::new(types), paths, attrs))
}

fn translate<'tcx>(
    ctxt: &Context<'tcx>,
    caller: DefId,
    origin: rustc_span::Span,
    ids: &ConcreteOrderingInstances<'tcx>,
    models: &OrderingModelBindings,
) -> Result<Function, VirErr> {
    let span = || {
        let mut s = ctxt.spans.to_air_span(origin);
        s.raw_span = Arc::new(());
        s
    };
    let names: Vec<_> = (0..9)
        .map(|i| {
            str_unique_var(
                &format!("local_ord_{}_{}", span().id, i),
                VarIdentDisambiguate::NoBodyParam,
            )
        })
        .collect();
    let u = Arc::new(TypX::Int(IntRange::U(64)));
    let nz = ctxt.mid_ty_to_vir(caller, origin, &ids.nonzero, None)?;
    let ordering = ctxt.mid_ty_to_vir(caller, origin, &ids.ordering, None)?;
    let shared = |t: Typ| Arc::new(TypX::Decorate(TypDecoration::Ref, None, t));
    let types = [
        ordering.clone(),
        shared(nz.clone()),
        shared(nz.clone()),
        shared(u.clone()),
        u.clone(),
        nz.clone(),
        shared(u.clone()),
        u.clone(),
        nz.clone(),
    ];
    let e = |t: &Typ, x| SpannedTyped::new(&span(), t, x);
    let read = |i: usize, t: &Typ, kind| {
        e(
            t,
            ExprX::ReadPlace(
                SpannedTyped::new(&span(), &types[i], PlaceX::Local(names[i].clone())),
                UnfinalizedReadKind { preliminary_kind: kind, id: ctxt.unique_read_kind_id() },
            ),
        )
    };
    let get = model_target(ctxt, caller, origin, ids.get, &models.get)?;
    let primitive = model_target(ctxt, caller, origin, ids.primitive, &models.primitive)?;
    let call = |t: &Typ, target: CallTarget, args: Vec<Expr>| {
        e(t, ExprX::Call { target, args: Arc::new(args), post_args: None, body: None })
    };
    let decl = |i: usize, init: Expr| {
        Spanned::new(
            span(),
            StmtX::Decl {
                pattern: SpannedTyped::new(
                    &span(),
                    &types[i],
                    PatternX::Var(PatternBinding {
                        name: names[i].clone(),
                        by_ref: ByRef::No,
                        typ: types[i].clone(),
                        user_mut: Some(false),
                        copy: true,
                    }),
                ),
                mode: Some((Mode::Exec, DeclProph::Default)),
                init: Some(SpannedTyped::new(&span(), &init.typ, PlaceX::Temporary(init.clone()))),
                els: None,
                assert_irrefutable: false,
            },
        )
    };
    // Keep both genuine NZ copies and all shared reads/borrows in source order.
    // Exact MIR checks justify only unobservable fresh Copy/no-drop storage.
    let statements = vec![
        decl(5, read(1, &nz, ReadKind::Copy)),
        decl(4, call(&u, get.clone(), vec![read(5, &nz, ReadKind::Copy)])),
        decl(3, read(4, &types[3], ReadKind::ImmutBor)),
        decl(8, read(2, &nz, ReadKind::Copy)),
        decl(7, call(&u, get, vec![read(8, &nz, ReadKind::Copy)])),
        decl(6, read(7, &types[6], ReadKind::ImmutBor)),
    ];
    let body = e(
        &ordering,
        ExprX::Block(
            Arc::new(statements),
            Some(call(
                &ordering,
                primitive,
                vec![read(3, &types[3], ReadKind::Move), read(6, &types[6], ReadKind::Move)],
            )),
        ),
    );
    // Contract proposal, never a source proof or primitive Cmp lowering.
    // The normal get and primitive Ord contracts must prove this.
    let spec_target = match model_target(ctxt, caller, origin, ids.get, &models.get)? {
        CallTarget::Fun(k, _, t, p, a) => CallTarget::Fun(k, models.spec_get.clone(), t, p, a),
        _ => return crate::util::err_span(origin, "local ordering get target unsupported"),
    };
    let value =
        |i: usize| call(&u, spec_target.clone(), vec![e(&nz, ExprX::Var(names[i].clone()))]);
    let variant = |name: &str| {
        let TypX::Datatype(dt, _, _) = &*ordering else { unreachable!() };
        e(&ordering, ExprX::Ctor(dt.clone(), Arc::new(name.into()), Arc::new(vec![]), None))
    };
    let numeric = e(
        &ordering,
        ExprX::If(
            e(
                &bool_typ(),
                ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), value(1), value(2)),
            ),
            variant("Less"),
            Some(e(
                &ordering,
                ExprX::If(
                    e(
                        &bool_typ(),
                        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Gt), value(1), value(2)),
                    ),
                    variant("Greater"),
                    Some(variant("Equal")),
                ),
            )),
        ),
    );
    let ensure = e(
        &bool_typ(),
        ExprX::Binary(
            BinaryOp::Eq(Mode::Spec),
            e(&ordering, ExprX::Var(names[0].clone())),
            numeric,
        ),
    );
    let param = |i: usize| {
        Spanned::new(
            span(),
            ParamX {
                name: names[i].clone(),
                typ: types[i].clone(),
                mode: Mode::Exec,
                user_mut: false,
                unwrapped_info: None,
            },
        )
    };
    let root = crate::rust_to_vir::get_root_module_path(ctxt);
    let name =
        Arc::new(FunX { path: root.push_segment(Arc::new("checked_nonzero_u64_ord%".into())) });
    let visibility = Visibility { restricted_to: Some(root.clone()) };
    Ok(Spanned::new(
        span(),
        FunctionX {
            name,
            proxy: None,
            kind: FunctionKind::Static,
            visibility: visibility.clone(),
            body_visibility: BodyVisibility::Visibility(visibility),
            opaqueness: Opaqueness::Opaque,
            owning_module: Some(root),
            mode: Mode::Exec,
            typ_params: Arc::new(vec![]),
            typ_bounds: Arc::new(vec![]),
            params: Arc::new(vec![param(1), param(2)]),
            outer_ret: param(0),
            inner_ret: param(0),
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
            extra_dependencies: vec![
                models.view.clone(),
                models.obeys_cmp_spec.clone(),
                models.cmp_spec.clone(),
            ],
            hidden: Arc::new(vec![]),
        },
    ))
}

// Correspondence requires complete structural identity, including recursively
// embedded datatype/dyn implementation paths. General VIR types_equal deliberately
// omits those witnesses and is too weak for this independent checker.
fn same_serial<T: serde::Serialize>(a: &T, b: &T) -> bool {
    matches!((serde_json::to_value(a),serde_json::to_value(b)),(Ok(a),Ok(b)) if a==b)
}

/// Independent readback of the ACTUAL Function. This does not rebuild a proposed
/// Function and compare hashes. Every supported body node is parsed against the
/// original MIR's local types, operands, sequencing and checked expansions.
fn check_ordering_function<'tcx>(
    ctxt: &Context<'tcx>,
    caller: DefId,
    origin: rustc_span::Span,
    ids: &ConcreteOrderingInstances<'tcx>,
    carrier: &mir::Body<'tcx>,
    get: &mir::Body<'tcx>,
    primitive: &mir::Body<'tcx>,
    models: &OrderingModelBindings,
    f: &Function,
) -> Result<(), VirErr> {
    let reject = |s| {
        crate::util::err_span::<(), _>(
            origin,
            &format!("local ordering actual Function correspondence: {s}"),
        )
        .unwrap_err()
    };
    // Recheck the original bodies; no descriptor or hash supplies validity.
    let expansions = inspect_bodies(
        ctxt.tcx,
        ty::TypingEnv::non_body_analysis(ctxt.tcx, caller),
        ids,
        carrier,
        get,
        primitive,
    )
    .map_err(reject)?;
    if expansions.iter().zip([(4, 5, 4), (11, 8, 7)]).any(|(e, (s, a, b))| {
        e.statement != s || e.formal.as_usize() != a || e.result.as_usize() != b
    }) {
        return Err(reject("expansion map"));
    }
    let x = &f.x;
    let root = crate::rust_to_vir::get_root_module_path(ctxt);
    if x.owning_module.as_ref() != Some(&root)
        || x.name.path != root.push_segment(Arc::new("checked_nonzero_u64_ord%".into()))
    {
        return Err(reject("function/module identity"));
    }
    let default_attrs = FunctionAttrsX { uses_ghost_blocks: true, ..Default::default() };
    if x.mode != Mode::Exec
        || !matches!(x.kind, FunctionKind::Static)
        || x.proxy.is_some()
        || !x.typ_params.is_empty()
        || !x.typ_bounds.is_empty()
        || x.params.len() != 2
        || !x.require.is_empty()
        || x.ensure.0.len() != 1
        || !x.ensure.1.is_empty()
        || x.returns.is_some()
        || !x.decrease.is_empty()
        || x.decrease_when.is_some()
        || x.decrease_by.is_some()
        || x.fndef_axioms.is_some()
        || x.mask_spec.is_some()
        || x.atomic_update.is_some()
        || x.unwind_spec.is_some()
        || !x.return_opaque_type_eqs.is_empty()
        || x.extra_dependencies
            != vec![models.view.clone(), models.obeys_cmp_spec.clone(), models.cmp_spec.clone()]
        || !x.hidden.is_empty()
        || !same_serial(&*x.attrs, &default_attrs)
        || !matches!(x.item_kind, ItemKind::Function)
        || !matches!(x.opaqueness, Opaqueness::Opaque)
        || x.visibility.restricted_to != x.owning_module
        || !same_serial(&x.body_visibility, &BodyVisibility::Visibility(x.visibility.clone()))
    {
        return Err(reject("signature/contracts/attributes"));
    }
    let mut names: Vec<Option<VarIdent>> = vec![None; 9];
    for (i, p) in [(0, &x.inner_ret), (1, &x.params[0]), (2, &x.params[1])] {
        let t = ctxt.mid_ty_to_vir(
            caller,
            origin,
            &carrier.local_decls[mir::Local::from_usize(i)].ty,
            None,
        )?;
        if !same_serial(&p.x.typ, &t)
            || p.x.mode != Mode::Exec
            || p.x.user_mut
            || p.x.unwrapped_info.is_some()
        {
            return Err(reject("parameter types/modes"));
        }
        names[i] = Some(p.x.name.clone());
    }
    if !same_serial(&x.inner_ret.x, &x.outer_ret.x) {
        return Err(reject("return parameter"));
    }
    let body = x.body.as_ref().ok_or_else(|| reject("body missing"))?;
    let read_ids = std::cell::RefCell::new(std::collections::HashSet::new());
    vir::ast_visitor::map_expr_visitor(body, &|e| {
        if let ExprX::ReadPlace(_, k) = &e.x {
            if !read_ids.borrow_mut().insert(k.id) {
                return Err(reject("reused read identity"));
            }
        }
        Ok(e.clone())
    })?;
    let ExprX::Block(stmts, Some(tail)) = &body.x else { return Err(reject("whole block")) };
    if stmts.len() != 6 || !same_serial(&body.typ, &x.inner_ret.x.typ) {
        return Err(reject("whole block type/count"));
    }
    let mut initializers = vec![];
    for (stmt, i) in stmts.iter().zip([5, 4, 3, 8, 7, 6]) {
        let StmtX::Decl {
            pattern,
            mode: Some((Mode::Exec, DeclProph::Default)),
            init: Some(init),
            els: None,
            assert_irrefutable: false,
        } = &stmt.x
        else {
            return Err(reject("declaration effect"));
        };
        let PatternX::Var(binding) = &pattern.x else { return Err(reject("declaration pattern")) };
        let t = ctxt.mid_ty_to_vir(
            caller,
            origin,
            &carrier.local_decls[mir::Local::from_usize(i)].ty,
            None,
        )?;
        if !same_serial(&pattern.typ, &t)
            || !same_serial(&binding.typ, &t)
            || !matches!(binding.by_ref, ByRef::No)
            || binding.user_mut != Some(false)
            || !binding.copy
        {
            return Err(reject("local binding type/mode"));
        }
        let PlaceX::Temporary(e) = &init.x else { return Err(reject("local initializer place")) };
        if !same_serial(&init.typ, &e.typ) || !same_serial(&e.typ, &t) {
            return Err(reject("local initializer type"));
        }
        names[i] = Some(binding.name.clone());
        initializers.push(e);
    }
    let names: Vec<_> =
        names.into_iter().collect::<Option<Vec<_>>>().ok_or_else(|| reject("incomplete map"))?;
    if names.iter().collect::<std::collections::HashSet<_>>().len() != 9 {
        return Err(reject("noninjective map"));
    }
    let read = |e: &Expr, i: usize, kind: ReadKind| -> bool {
        let ExprX::ReadPlace(p, k) = &e.x else { return false };
        let Ok(t) = ctxt.mid_ty_to_vir(
            caller,
            origin,
            &carrier.local_decls[mir::Local::from_usize(i)].ty,
            None,
        ) else {
            return false;
        };
        let output = if matches!(kind, ReadKind::ImmutBor) {
            Arc::new(TypX::Decorate(TypDecoration::Ref, None, t.clone()))
        } else if i == 1 || i == 2 {
            let TypX::Decorate(TypDecoration::Ref, None, nz) = &*t else { return false };
            nz.clone()
        } else {
            t.clone()
        };
        matches!(&p.x,PlaceX::Local(n) if n==&names[i])
            && same_serial(&k.preliminary_kind, &kind)
            && same_serial(&p.typ, &t)
            && same_serial(&e.typ, &output)
    };
    let get_target = model_target(ctxt, caller, origin, ids.get, &models.get)?;
    let primitive_target = model_target(ctxt, caller, origin, ids.primitive, &models.primitive)?;
    let call = |e: &Expr, target: &CallTarget, inputs: &[(usize, ReadKind)]| -> bool {
        let ExprX::Call { target: actual, args, post_args: None, body: None } = &e.x else {
            return false;
        };
        same_serial(actual, target)
            && args.len() == inputs.len()
            && args.iter().zip(inputs).all(|(e, (i, k))| read(e, *i, *k))
    };
    if !read(initializers[0], 1, ReadKind::Copy)
        || !call(initializers[1], &get_target, &[(5, ReadKind::Copy)])
        || !read(initializers[2], 4, ReadKind::ImmutBor)
        || !read(initializers[3], 2, ReadKind::Copy)
        || !call(initializers[4], &get_target, &[(8, ReadKind::Copy)])
        || !read(initializers[5], 7, ReadKind::ImmutBor)
        || !call(tail, &primitive_target, &[(3, ReadKind::Move), (6, ReadKind::Move)])
        || !same_serial(&tail.typ, &x.inner_ret.x.typ)
    {
        return Err(reject("read/borrow/call target or operand order"));
    }
    // Check the complete exact numeric contract separately from the body.
    let mut spec_target = get_target;
    let CallTarget::Fun(_, name, ..) = &mut spec_target else { return Err(reject("spec target")) };
    *name = models.spec_get.clone();
    let u = Arc::new(TypX::Int(IntRange::U(64)));
    let value = |e: &Expr, i: usize| -> bool {
        let ExprX::Call { target, args, post_args: None, body: None } = &e.x else { return false };
        same_serial(&e.typ, &u)
            && same_serial(target, &spec_target)
            && args.len() == 1
            && matches!(&args[0].x,ExprX::Var(n) if n==&names[i])
            && ctxt
                .mid_ty_to_vir(caller, origin, &ids.nonzero, None)
                .is_ok_and(|t| same_serial(&args[0].typ, &t))
    };
    let relation = |e: &Expr, op: InequalityOp| -> bool {
        matches!(&e.x,ExprX::Binary(BinaryOp::Inequality(actual),a,b)
            if *actual==op && value(a,1) && value(b,2))
            && same_serial(&e.typ, &bool_typ())
    };
    let variant = |e: &Expr, name: &str| -> bool {
        matches!((&e.x,&*x.inner_ret.x.typ),(ExprX::Ctor(dt,v,fields,None),TypX::Datatype(expected,..))
            if dt==expected && v.as_str()==name && fields.is_empty())
            && same_serial(&e.typ, &x.inner_ret.x.typ)
    };
    let ensure = &x.ensure.0[0];
    let ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, numeric) = &ensure.x else {
        return Err(reject("exact ensures equality"));
    };
    let ExprX::If(lt, less, Some(other)) = &numeric.x else {
        return Err(reject("numeric Less condition"));
    };
    let ExprX::If(gt, greater, Some(equal)) = &other.x else {
        return Err(reject("numeric Greater condition"));
    };
    if !same_serial(&ensure.typ, &bool_typ())
        || !same_serial(&result.typ, &x.inner_ret.x.typ)
        || !matches!(&result.x,ExprX::Var(n) if n==&names[0])
        || !same_serial(&numeric.typ, &x.inner_ret.x.typ)
        || !same_serial(&other.typ, &x.inner_ret.x.typ)
        || !relation(lt, InequalityOp::Lt)
        || !relation(gt, InequalityOp::Gt)
        || !variant(less, "Less")
        || !variant(greater, "Greater")
        || !variant(equal, "Equal")
    {
        return Err(reject("full-domain numeric contract"));
    }
    Ok(())
}

pub(crate) struct LocalCall<'tcx> {
    caller: DefId,
    source: &'tcx rustc_hir::Expr<'tcx>,
    instance: ty::Instance<'tcx>,
    helper: Fun,
}
struct Calls<'a, 'tcx> {
    ctxt: &'a Context<'tcx>,
    caller: DefId,
    types: &'tcx ty::TypeckResults<'tcx>,
    found: Vec<(&'tcx rustc_hir::Expr<'tcx>, ConcreteOrderingInstances<'tcx>)>,
}
fn actual_call<'tcx>(
    types: &ty::TypeckResults<'tcx>,
    e: &'tcx rustc_hir::Expr<'tcx>,
) -> Option<(DefId, ty::GenericArgsRef<'tcx>, Vec<&'tcx rustc_hir::Expr<'tcx>>)> {
    use rustc_hir::ExprKind;
    match e.kind {
        ExprKind::Call(callee, args) => {
            let ExprKind::Path(ref q) = callee.kind else { return None };
            if !matches!(
                types.qpath_res(q, callee.hir_id),
                rustc_hir::def::Res::Def(
                    rustc_hir::def::DefKind::Fn | rustc_hir::def::DefKind::AssocFn,
                    _
                )
            ) {
                return None;
            }
            let ty::FnDef(f, a) = types.expr_ty(callee).kind() else { return None };
            Some((*f, a.no_bound_vars()?, args.iter().collect()))
        }
        ExprKind::MethodCall(_, receiver, args, _) => Some((
            types.type_dependent_def_id(e.hir_id)?,
            types.node_args(e.hir_id),
            std::iter::once(receiver).chain(args.iter()).collect(),
        )),
        _ => None,
    }
}
fn source_arguments<'tcx>(
    tcx: TyCtxt<'tcx>,
    types: &ty::TypeckResults<'tcx>,
    args: &[&rustc_hir::Expr<'tcx>],
    nz: ty::Ty<'tcx>,
) -> Result<(), Reject> {
    demand(args.len() == 2, "source_argument_count")?;
    for a in args {
        let adjusted = tcx.erase_and_anonymize_regions(types.expr_ty_adjusted(a));
        demand(
            adjusted == ty::Ty::new_imm_ref(tcx, tcx.lifetimes.re_erased, nz),
            "source_argument_type",
        )?;
        for adj in types.expr_adjustments(a) {
            use ty::adjustment::{Adjust, AutoBorrow, AutoBorrowMutability};
            demand(
                matches!(
                    adj.kind,
                    Adjust::Deref(ty::adjustment::DerefAdjustKind::Builtin)
                        | Adjust::Borrow(AutoBorrow::Ref(AutoBorrowMutability::Not))
                ),
                "source_coercion_or_overloaded_deref",
            )?;
        }
    }
    Ok(())
}
impl<'tcx> rustc_hir::intravisit::Visitor<'tcx> for Calls<'_, 'tcx> {
    fn visit_expr(&mut self, e: &'tcx rustc_hir::Expr<'tcx>) {
        if let Some((f, a, args)) = actual_call(self.types, e) {
            let env = ty::TypingEnv::non_body_analysis(self.ctxt.tcx, self.caller);
            if let Ok(ids) = resolve_ordering(self.ctxt.tcx, env, f, a) {
                // Do not redirect unsupported adjustments; ordinary translation
                // still handles the original source call without a new fact.
                if source_arguments(self.ctxt.tcx, self.types, &args, ids.nonzero).is_ok() {
                    self.found.push((e, ids));
                }
            }
        }
        rustc_hir::intravisit::walk_expr(self, e);
    }
}
fn bodies<'tcx>(
    tcx: TyCtxt<'tcx>,
    env: ty::TypingEnv<'tcx>,
    ids: &ConcreteOrderingInstances<'tcx>,
) -> [mir::Body<'tcx>; 3] {
    [ids.carrier, ids.get, ids.primitive].map(|i| {
        i.instantiate_mir_and_normalize_erasing_regions(
            tcx,
            env,
            ty::EarlyBinder::bind(tcx, tcx.instance_mir(i.def).clone()),
        )
    })
}

pub(crate) fn register<'tcx>(
    ctxt: &Context<'tcx>,
    state: &mut crate::rust_to_vir::State,
    infos: &mut Vec<crate::rust_to_vir_func::FunctionOrConstInfo<'tcx>>,
    imported: &[Krate],
) -> Result<(), VirErr> {
    use crate::rust_to_vir_func::{CheckItemFnEither, FunctionOrConstInfo};
    use rustc_hir::intravisit::Visitor;
    if !ctxt.cmd_line_args.infer_goals {
        return Ok(());
    }
    let mut found = vec![];
    for info in infos.iter() {
        let FunctionOrConstInfo::Function(f) = info else { continue };
        if f.vattrs.external_body {
            continue;
        }
        let CheckItemFnEither::BodyId(body) = f.body_id else { continue };
        let mut calls = Calls {
            ctxt,
            caller: f.def_id,
            types: ctxt.tcx.typeck(f.def_id.expect_local()),
            found: vec![],
        };
        calls.visit_body(ctxt.tcx.hir_body(body));
        found.extend(calls.found.into_iter().map(|(e, ids)| (f.def_id, e, ids)));
    }
    let mut helper = None;
    for (caller, e, ids) in found {
        if !ctxt.cmd_line_args.can_certify_inference() || ctxt.cmd_line_args.no_erasure_check {
            return crate::util::err_span(
                e.span,
                "local ordering requires complete ordinary source/body/lifetime/trait/erasure checks",
            );
        }
        let [carrier, get, primitive] =
            bodies(ctxt.tcx, ty::TypingEnv::non_body_analysis(ctxt.tcx, caller), &ids);
        inspect_bodies(
            ctxt.tcx,
            ty::TypingEnv::non_body_analysis(ctxt.tcx, caller),
            &ids,
            &carrier,
            &get,
            &primitive,
        )
        .map_err(|r| {
            crate::util::err_span::<(), _>(e.span, &format!("local ordering MIR unsupported: {r}"))
                .unwrap_err()
        })?;
        let models = bind_models(ctxt, &ids, imported).map_err(|r| {
            crate::util::err_span::<(), _>(
                e.span,
                &format!("local ordering model binding unsupported: {r}"),
            )
            .unwrap_err()
        })?;
        if helper.is_none() {
            helper = Some(translate(ctxt, caller, e.span, &ids, &models)?);
        }
        let f = helper.as_ref().unwrap();
        check_ordering_function(
            ctxt, caller, e.span, &ids, &carrier, &get, &primitive, &models, f,
        )?;
        ctxt.local_ordering_calls.borrow_mut().insert(
            e.hir_id,
            LocalCall { caller, source: e, instance: ids.carrier, helper: f.x.name.clone() },
        );
        state.insert_fun_warn_config(ctxt, &f.x.name, caller);
    }
    if let Some(full) = helper {
        infos.push(FunctionOrConstInfo::BodyBackedForeign {
            stub: vir::ast_util::function_to_stub(&full),
            full,
        });
    }
    Ok(())
}

pub(crate) fn direct_helper<'tcx>(
    bctx: &crate::context::BodyCtxt<'tcx>,
    e: &rustc_hir::Expr<'tcx>,
    f: DefId,
    args: ty::GenericArgsRef<'tcx>,
) -> Option<Fun> {
    let calls = bctx.ctxt.local_ordering_calls.borrow();
    let c = calls.get(&e.hir_id)?;
    if c.caller != bctx.fun_id || !std::ptr::eq(c.source, e) {
        return None;
    }
    let env = ty::TypingEnv::non_body_analysis(bctx.ctxt.tcx, bctx.fun_id);
    let ids = resolve_ordering(bctx.ctxt.tcx, env, f, args).ok()?;
    let (source_f, source_args, actual) = actual_call(bctx.types, c.source)?;
    if source_f != f || source_args != args || ids.carrier != c.instance {
        return None;
    }
    source_arguments(bctx.ctxt.tcx, bctx.types, &actual, ids.nonzero).ok()?;
    Some(c.helper.clone())
}

pub(crate) fn validate_actual<'tcx>(
    ctxt: &Context<'tcx>,
    f: &Function,
    imported: &[Krate],
) -> Result<(), VirErr> {
    let calls = ctxt.local_ordering_calls.borrow();
    let Some(c) = calls.values().find(|c| c.helper == f.x.name) else { return Ok(()) };
    let env = ty::TypingEnv::non_body_analysis(ctxt.tcx, c.caller);
    let (def, args, _) = actual_call(ctxt.tcx.typeck(c.caller.expect_local()), c.source)
        .ok_or_else(|| crate::util::error("local ordering original direct call missing"))?;
    let fail = |r| crate::util::err_span::<(), _>(c.source.span, r).unwrap_err();
    let ids = resolve_ordering(ctxt.tcx, env, def, args).map_err(fail)?;
    let [carrier, get, primitive] = bodies(ctxt.tcx, env, &ids);
    let models = bind_models(ctxt, &ids, imported).map_err(fail)?;
    check_ordering_function(
        ctxt,
        c.caller,
        c.source.span,
        &ids,
        &carrier,
        &get,
        &primitive,
        &models,
        f,
    )
}

#[cfg(test)]
#[path = "checked_nonzero_ord_tests.rs"]
mod tests;
