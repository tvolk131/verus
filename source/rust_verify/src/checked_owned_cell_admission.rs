//! Complete, conservative local typed-unit admission. No logical authority.
use super::*;
use crate::verus_items::{self, BinaryOpItem, BuiltinFunctionItem, SpecItem, VerusItem};
use rustc_hir::{
    self as hir, ExprKind as E,
    def::{DefKind, Res},
    def_id::LocalDefId,
};
use rustc_middle::ty::{
    TypeVisitableExt,
    adjustment::{Adjust, AutoBorrow, DerefAdjustKind},
};
use std::collections::{HashMap, HashSet};

/// Deliberately private and nonserializable. This is NOT a proof/field capability.
#[allow(dead_code)] // Scoped inventory retained for later phases; never an authority.
pub(super) struct AdmittedUnit {
    native_cell: DefId,
    functions: HashSet<DefId>,
    bodies: HashSet<LocalDefId>,
    headers: Vec<HeaderInventory>,
}
#[allow(dead_code)] // Exact header identity/erasure boundary; not a proof certificate.
struct HeaderInventory {
    caller: DefId,
    call: hir::HirId,
    closure: Option<LocalDefId>,
    erased_type_only: Option<hir::HirId>,
    predicates: Vec<hir::HirId>,
}
impl AdmittedUnit {
    pub(super) fn later_phase_refusal(self) -> VirErr {
        crate::util::error(format!(
            "experimental owned UnsafeCell: inactive: F1 admission passed; functions={}, bodies={}, headers={}; native body lowering, F2 diagnostic/field authority, F3 erasure and correspondence/state gates remain unavailable",
            self.functions.len(),
            self.bodies.len(),
            self.headers.len()
        ))
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Surface {
    Exec,
    Spec,
}
struct Unit<'tcx> {
    tcx: TyCtxt<'tcx>,
    cell: DefId,
    builtins: crate::verus_items::VerusItems,
    builtin_crate: rustc_hir::def_id::CrateNum,
    locals: HashMap<DefId, Surface>,
    constants: HashSet<DefId>,
    native: HashSet<DefId>,
    visited: HashSet<LocalDefId>,
    edges: HashMap<DefId, HashSet<DefId>>,
    headers: Vec<HeaderInventory>,
}
fn fail(span: rustc_span::Span, reason: impl std::fmt::Display) -> VirErr {
    crate::util::err_span::<(), _>(
        span,
        format!("experimental owned UnsafeCell admission: {reason}"),
    )
    .unwrap_err()
}
impl<'tcx> Unit<'tcx> {
    fn typ(
        &self,
        owner: DefId,
        t: Ty<'tcx>,
        surface: Surface,
        span: rustc_span::Span,
    ) -> Result<(), VirErr> {
        let env = ty::TypingEnv::non_body_analysis(self.tcx, owner);
        if t.has_non_region_param() || t.has_infer() {
            return Err(fail(span, "unresolved/generic/projection type"));
        }
        let t = self
            .tcx
            .try_normalize_erasing_regions(env, ty::Unnormalized::new_wip(t))
            .map_err(|_| fail(span, "type normalization failed"))?;
        let scalar = |t: Ty<'tcx>| -> bool { scalar(t) };
        let admitted = match surface {
            Surface::Spec => scalar(t),
            Surface::Exec => {
                scalar(t)
                    || is_cell(t, self.cell)
                    || matches!(t.kind(),ty::Ref(_,inner,hir::Mutability::Mut) if is_cell(*inner,self.cell) || is_u64(*inner))
            }
        };
        if !admitted {
            let reason = match (surface, t.kind()) {
                (Surface::Spec, _) => {
                    "spec surface/capture must be scalar or scalar tuple; cells/references/closures/quantifiers are forbidden"
                }
                (_, ty::Ref(_, _, hir::Mutability::Not)) => "shared reference/escape is forbidden",
                (_, ty::RawPtr(..)) => "raw pointer/escape is forbidden",
                (_, ty::FnDef(..) | ty::FnPtr(..)) => {
                    "executable function value/callback is forbidden"
                }
                (_, ty::Closure(..) | ty::Coroutine(..) | ty::CoroutineClosure(..)) => {
                    "executable closure/capture/coroutine is forbidden"
                }
                (_, ty::Adt(..)) => "non-native-u64 cell or user ADT/container is forbidden",
                (_, ty::Alias(..) | ty::Param(..) | ty::Dynamic(..)) => {
                    "generic/projection/opaque/dyn type is forbidden"
                }
                _ => "type is outside closed scalar/tuple/native-cell/exclusive-reference grammar",
            };
            return Err(fail(span, format!("{reason}: {t:?}")));
        }
        if t.needs_drop(self.tcx, env) {
            return Err(fail(span, "drop-bearing type is forbidden"));
        }
        Ok(())
    }
    fn typed(
        &self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        e: &'tcx hir::Expr<'tcx>,
        surface: Surface,
    ) -> Result<(), VirErr> {
        self.typ(owner, types.expr_ty(e), surface, e.span)?;
        for a in types.expr_adjustments(e) {
            self.typ(owner, a.target, surface, e.span)?;
            match &a.kind {
                Adjust::Deref(DerefAdjustKind::Builtin) => (),
                Adjust::Borrow(AutoBorrow::Ref(m))
                    if matches!(m, ty::adjustment::AutoBorrowMutability::Mut { .. })
                        && surface == Surface::Exec =>
                {
                    ()
                }
                _ => {
                    return Err(fail(
                        e.span,
                        "unsupported implicit coercion/autoref/autoderef adjustment",
                    ));
                }
            }
        }
        Ok(())
    }
    fn head(
        &self,
        types: &'tcx ty::TypeckResults<'tcx>,
        e: &'tcx hir::Expr<'tcx>,
    ) -> Result<(DefId, ty::GenericArgsRef<'tcx>), VirErr> {
        let E::Path(q) = &e.kind else {
            return Err(fail(e.span, "indirect/function-value callback call is forbidden"));
        };
        let Res::Def(DefKind::Fn | DefKind::AssocFn, id) = types.qpath_res(q, e.hir_id) else {
            return Err(fail(e.span, "indirect/function-value callback call is forbidden"));
        };
        let ty::FnDef(typed, args) = types.expr_ty(e).kind() else {
            return Err(fail(e.span, "direct call lacks actual FnDef type"));
        };
        if *typed != id || !types.expr_adjustments(e).is_empty() {
            return Err(fail(e.span, "coerced or mismatched direct callee"));
        }
        Ok((
            id,
            args.no_bound_vars()
                .ok_or_else(|| fail(e.span, "bound direct function substitutions"))?,
        ))
    }
    fn builtin(&self, id: DefId) -> Option<VerusItem> {
        if id.is_local() || id.krate != self.builtin_crate {
            return None;
        }
        let item = self.builtins.id_to_name.get(&id)?.clone();
        // Actual loaded baseline directive identity AND ABI/shape, never spelling.
        if self.tcx.def_kind(id) != DefKind::Fn || !self.tcx.visibility(id).is_public() {
            return None;
        }
        let sig = self.tcx.fn_sig(id).instantiate_identity().skip_norm_wip().skip_binder();
        let arity = match item {
            VerusItem::Spec(SpecItem::Ensures | SpecItem::Requires) => 1,
            VerusItem::BuiltinFunction(BuiltinFunctionItem::ConstrainType)
            | VerusItem::BinaryOp(_) => 2,
            _ => return Some(item),
        };
        if sig.inputs().len() != arity
            || sig.c_variadic()
            || sig.safety() != hir::Safety::Safe
            || sig.abi() != rustc_abi::ExternAbi::Rust
        {
            return None;
        }
        if arity == 1 && !sig.output().is_unit() {
            return None;
        }
        if matches!(item, VerusItem::BuiltinFunction(BuiltinFunctionItem::ConstrainType))
            && !matches!(sig.output().kind(), ty::Bool)
        {
            return None;
        }
        Some(item)
    }
    fn binding(&self, p: &hir::Pat<'_>) -> Result<hir::HirId, VirErr> {
        match p.kind {
            hir::PatKind::Binding(_, id, _, None) => Ok(id),
            _ => Err(fail(p.span, "function/header parameters require direct bindings")),
        }
    }
    fn local_path(
        &self,
        types: &ty::TypeckResults<'tcx>,
        e: &hir::Expr<'tcx>,
        expected: hir::HirId,
    ) -> bool {
        matches!(&e.kind,E::Path(q) if matches!(types.qpath_res(q,e.hir_id),Res::Local(id) if id==expected))
    }
    fn pattern(
        &self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        p: &'tcx hir::Pat<'tcx>,
        surface: Surface,
    ) -> Result<(), VirErr> {
        self.typ(owner, types.pat_ty(p), surface, p.span)?;
        match p.kind {
            hir::PatKind::Wild => Ok(()),
            hir::PatKind::Binding(mode, _, _, sub) => {
                if mode.0 != rustc_ast::ByRef::No {
                    return Err(fail(p.span, "reference binding pattern is forbidden"));
                }
                if let Some(p) = sub {
                    self.pattern(owner, types, p, surface)?;
                }
                Ok(())
            }
            hir::PatKind::Tuple(ps, _) => {
                for p in ps {
                    self.pattern(owner, types, p, surface)?;
                }
                Ok(())
            }
            _ => Err(fail(p.span, "unsupported pattern outside closed tuple/binding grammar")),
        }
    }
    fn callee(
        &mut self,
        owner: DefId,
        id: DefId,
        args: ty::GenericArgsRef<'tcx>,
        surface: Surface,
        span: rustc_span::Span,
    ) -> Result<(), VirErr> {
        if args.has_non_region_param() || args.has_infer() || args.has_escaping_bound_vars() {
            return Err(fail(span, "unresolved call substitutions"));
        }
        let env = ty::TypingEnv::non_body_analysis(self.tcx, owner);
        let i = ty::Instance::try_resolve(self.tcx, env, id, args)
            .map_err(|_| fail(span, "call resolution failed"))?
            .ok_or_else(|| fail(span, "abstract/dyn call is forbidden"))?;
        if !matches!(i.def,ty::InstanceKind::Item(d) if d==id) {
            return Err(fail(span, "virtual/abstract/non-original call instance is forbidden"));
        }
        if self.native.contains(&id) {
            if surface != Surface::Exec || args.len() != 1 || !is_u64(args.type_at(0)) {
                return Err(fail(
                    span,
                    "native cell calls are executable u64-only; cell-bearing specs forbidden",
                ));
            }
            return Ok(());
        }
        let Some(target) = self.locals.get(&id) else {
            return Err(fail(
                span,
                "imported/foreign/abstract callback or unsupported native operation is forbidden",
            ));
        };
        if (surface == Surface::Spec && *target != Surface::Spec)
            || (surface == Surface::Exec && *target != Surface::Exec)
        {
            return Err(fail(span, "cross-mode call is outside closed executable/spec grammar"));
        }
        self.edges.entry(owner).or_default().insert(id);
        Ok(())
    }
    /// Validate ONLY the exact compiler-erased type inference subtree. No call
    /// graph edge/capture fact comes from it; all real predicates are scanned.
    fn type_only(
        &self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        e: &'tcx hir::Expr<'tcx>,
        ret: hir::HirId,
        params: &[hir::HirId],
    ) -> Result<(), VirErr> {
        let E::Call(head, args) = e.kind else {
            return Err(fail(e.span, "canonical constrain_type call expected"));
        };
        let (id, _) = self.head(types, head)?;
        if self.builtin(id) != Some(VerusItem::BuiltinFunction(BuiltinFunctionItem::ConstrainType))
            || args.len() != 2
            || !self.local_path(types, &args[0], ret)
        {
            return Err(fail(e.span, "noncanonical builtin constrain_type shape/result binding"));
        }
        self.typ(owner, types.expr_ty(&args[0]), Surface::Spec, args[0].span)?;
        if !matches!(types.expr_ty(e).kind(), ty::Bool) {
            return Err(fail(e.span, "constrain_type must return bool"));
        }
        let E::Call(head, actuals) = args[1].kind else {
            return Err(fail(e.span, "canonical type-only direct self call expected"));
        };
        let (target, substs) = self.head(types, head)?;
        if target != owner
            || actuals.len() != params.len()
            || substs.has_non_region_param()
            || substs.has_infer()
        {
            return Err(fail(e.span, "type-only self call identity/arity/substitutions mismatch"));
        }
        for (a, p) in actuals.iter().zip(params) {
            if !self.local_path(types, a, *p) {
                return Err(fail(a.span, "type-only self call must use original parameter places"));
            }
            self.typed(owner, types, a, Surface::Exec)?;
        }
        self.typ(owner, types.expr_ty(&args[1]), Surface::Spec, args[1].span)?;
        Ok(())
    }
    fn header(
        &mut self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        call: &'tcx hir::Expr<'tcx>,
        params: &[hir::HirId],
    ) -> Result<bool, VirErr> {
        let E::Call(h, args) = call.kind else {
            return Ok(false);
        };
        let Ok((id, _)) = self.head(types, h) else {
            return Ok(false);
        };
        let Some(VerusItem::Spec(kind @ (SpecItem::Ensures | SpecItem::Requires))) =
            self.builtin(id)
        else {
            return Ok(false);
        };
        if args.len() != 1
            || !types.expr_adjustments(call).is_empty()
            || !types.expr_ty(call).is_unit()
        {
            return Err(fail(call.span, "noncanonical header call arity/type/adjustment"));
        }
        let mut types = types;
        let mut closure_id = None;
        let (value, ret) = if kind == SpecItem::Ensures {
            if let E::Closure(c) = args[0].kind {
                let body = self.tcx.hir_body(c.body);
                if body.params.len() != 1 {
                    return Err(fail(
                        args[0].span,
                        "ensures closure must have one scalar result binding",
                    ));
                }
                types = self.tcx.typeck(c.def_id);
                self.pattern(owner, types, body.params[0].pat, Surface::Spec)?;
                self.visited.insert(c.def_id);
                closure_id = Some(c.def_id);
                (body.value, Some(self.binding(body.params[0].pat)?))
            } else {
                (&args[0], None)
            }
        } else {
            (&args[0], None)
        };
        let E::Array(preds) = value.kind else {
            return Err(fail(value.span, "canonical header bool array expected"));
        };
        if !matches!(types.expr_ty(value).kind(),ty::Array(t,_) if matches!(t.kind(),ty::Bool))
            || !types.expr_adjustments(value).is_empty()
        {
            return Err(fail(
                value.span,
                "canonical header array must contain bools without coercions",
            ));
        }
        let mut start = 0;
        if let Some(ret) = ret {
            if preds.is_empty() {
                return Err(fail(
                    value.span,
                    "canonical ensures requires constrain_type and predicates",
                ));
            }
            self.type_only(owner, types, &preds[0], ret, params)?;
            start = 1;
        }
        for p in &preds[start..] {
            self.expr(owner, types, p, Surface::Spec)?;
            if !matches!(types.expr_ty(p).kind(), ty::Bool) {
                return Err(fail(p.span, "header predicate must be bool"));
            }
        }
        self.headers.push(HeaderInventory {
            caller: owner,
            call: call.hir_id,
            closure: closure_id,
            erased_type_only: if start == 1 { Some(preds[0].hir_id) } else { None },
            predicates: preds[start..].iter().map(|p| p.hir_id).collect(),
        });
        Ok(true)
    }
    fn quantifier(
        &mut self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        arg: &'tcx hir::Expr<'tcx>,
    ) -> Result<(), VirErr> {
        let closure = if let E::Call(head, args) = arg.kind {
            let (id, _) = self.head(types, head)?;
            if args.len() != 1
                || self.builtin(id) != Some(VerusItem::Expr(verus_items::ExprItem::ClosureToFnSpec))
            {
                return Err(fail(arg.span, "noncanonical scalar quantifier wrapper"));
            }
            &args[0]
        } else {
            arg
        };
        let E::Closure(c) = closure.kind else {
            return Err(fail(closure.span, "quantifier requires canonical scalar closure"));
        };
        let types = self.tcx.typeck(c.def_id);
        let body = self.tcx.hir_body(c.body);
        for p in body.params {
            self.pattern(owner, types, p.pat, Surface::Spec)?;
        }
        self.visited.insert(c.def_id);
        self.expr(owner, types, body.value, Surface::Spec)?;
        if !matches!(types.expr_ty(body.value).kind(), ty::Bool) {
            return Err(fail(body.value.span, "quantifier predicate must be bool"));
        }
        Ok(())
    }

    fn block(
        &mut self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        b: &'tcx hir::Block<'tcx>,
        surface: Surface,
        headers: Option<&[hir::HirId]>,
    ) -> Result<(), VirErr> {
        if !matches!(b.rules, hir::BlockCheckMode::DefaultBlock) {
            return Err(fail(b.span, "unsafe block is forbidden"));
        }
        let mut prefix = headers.is_some();
        for s in b.stmts {
            if let hir::StmtKind::Semi(e) | hir::StmtKind::Expr(e) = s.kind {
                if prefix && self.header(owner, types, e, headers.unwrap())? {
                    continue;
                }
            }
            prefix = false;
            match s.kind {
                hir::StmtKind::Let(l) => {
                    let local_mode = crate::attributes::get_var_mode(
                        if surface == Surface::Exec {
                            vir::ast::Mode::Exec
                        } else {
                            vir::ast::Mode::Spec
                        },
                        self.tcx.hir_attrs(s.hir_id),
                    );
                    let local_surface = if local_mode == vir::ast::Mode::Exec {
                        Surface::Exec
                    } else {
                        Surface::Spec
                    };
                    self.pattern(owner, types, l.pat, local_surface)?;
                    if l.els.is_some() {
                        return Err(fail(l.span, "let-else/cleanup control is unsupported"));
                    }
                    if let Some(i) = l.init {
                        self.expr(owner, types, i, local_surface)?;
                    }
                }
                hir::StmtKind::Expr(e) | hir::StmtKind::Semi(e) => {
                    self.expr(owner, types, e, surface)?
                }
                hir::StmtKind::Item(_) => (), // Independently inventoried/checked below, never skipped.
            }
        }
        if let Some(e) = b.expr {
            self.expr(owner, types, e, surface)?;
        }
        Ok(())
    }
    fn expr(
        &mut self,
        owner: DefId,
        types: &'tcx ty::TypeckResults<'tcx>,
        e: &'tcx hir::Expr<'tcx>,
        surface: Surface,
    ) -> Result<(), VirErr> {
        let attrs = self.tcx.hir_attrs(e.hir_id);
        let surface = if crate::attributes::get_ghost_block_opt(attrs).is_some()
            || matches!(
                crate::attributes::get_mode_opt(attrs),
                Some(vir::ast::Mode::Spec | vir::ast::Mode::Proof)
            ) {
            Surface::Spec
        } else {
            surface
        };
        // Diagnose forbidden executable constructs before their FnDef/closure type.
        match e.kind {
            E::Closure(..) => {
                return Err(fail(
                    e.span,
                    "executable closure/capture or noncanonical spec closure is forbidden",
                ));
            }
            E::Cast(..) | E::UnsafeBinderCast(..) => {
                return Err(fail(e.span, "cast/raw conversion is forbidden"));
            }
            E::InlineAsm(..) => return Err(fail(e.span, "inline assembly is forbidden")),
            _ => (),
        }
        self.typed(owner, types, e, surface)?;
        match e.kind {
            E::Lit(_) => Ok(()),
            E::Path(ref q) => match types.qpath_res(q, e.hir_id) {
                Res::Local(_) => Ok(()),
                Res::Def(DefKind::Const { .. }, id) if self.constants.contains(&id) => {
                    self.edges.entry(owner).or_default().insert(id);
                    Ok(())
                }
                _ => Err(fail(
                    e.span,
                    "unknown/imported value, static or function value is forbidden",
                )),
            },
            E::Tup(es) => {
                for e in es {
                    self.expr(owner, types, e, surface)?;
                }
                Ok(())
            }
            E::Call(head, args) => {
                let (id, substs) = self.head(types, head)?;
                if surface == Surface::Spec {
                    if let Some(VerusItem::Quant(_)) = self.builtin(id) {
                        if args.len() != 1 {
                            return Err(fail(e.span, "quantifier arity"));
                        }
                        return self.quantifier(owner, types, &args[0]);
                    }
                    if let Some(VerusItem::BinaryOp(
                        BinaryOpItem::Equality(_)
                        | BinaryOpItem::SpecOrd(_)
                        | BinaryOpItem::SpecArith(_)
                        | BinaryOpItem::SpecBitwise(_),
                    )) = self.builtin(id)
                    {
                        if args.len() != 2 {
                            return Err(fail(e.span, "scalar builtin binary arity"));
                        }
                        for a in args {
                            self.expr(owner, types, a, surface)?;
                        }
                        return Ok(());
                    }
                }
                self.callee(owner, id, substs, surface, e.span)?;
                for a in args {
                    self.expr(owner, types, a, surface)?;
                }
                Ok(())
            }
            E::MethodCall(_, recv, args, _) => {
                let id = types
                    .type_dependent_def_id(e.hir_id)
                    .ok_or_else(|| fail(e.span, "unresolved/abstract method"))?;
                self.callee(owner, id, types.node_args(e.hir_id), surface, e.span)?;
                self.expr(owner, types, recv, surface)?;
                for a in args {
                    self.expr(owner, types, a, surface)?;
                }
                Ok(())
            }
            E::Binary(_, a, b) => {
                if types.type_dependent_def_id(e.hir_id).is_some() {
                    return Err(fail(e.span, "overloaded operator/abstract callback is forbidden"));
                }
                self.expr(owner, types, a, surface)?;
                self.expr(owner, types, b, surface)
            }
            E::Unary(_, a) => {
                if types.type_dependent_def_id(e.hir_id).is_some() {
                    return Err(fail(e.span, "overloaded unary/deref callback is forbidden"));
                }
                self.expr(owner, types, a, surface)
            }
            E::Assign(a, b, _) if surface == Surface::Exec => {
                self.expr(owner, types, a, surface)?;
                self.expr(owner, types, b, surface)
            }
            E::AddrOf(hir::BorrowKind::Ref, hir::Mutability::Mut, a)
                if surface == Surface::Exec =>
            {
                self.expr(owner, types, a, surface)
            }
            E::Field(a, _) if matches!(types.expr_ty(a).kind(), ty::Tuple(_)) => {
                self.expr(owner, types, a, surface)
            }
            E::Block(b, None) => self.block(owner, types, b, surface, None),
            E::If(a, b, c) => {
                self.expr(owner, types, a, surface)?;
                self.expr(owner, types, b, surface)?;
                if let Some(c) = c {
                    self.expr(owner, types, c, surface)?;
                }
                Ok(())
            }
            E::DropTemps(a) | E::Type(a, _) => self.expr(owner, types, a, surface),
            _ => Err(fail(
                e.span,
                "expression/effect is outside the closed typed grammar (arrays/promotions, quantifiers, loops, drop/unwind and unknown operations are unsupported)",
            )),
        }
    }
}
fn scalar(t: Ty<'_>) -> bool {
    match t.kind() {
        ty::Bool | ty::Uint(ty::UintTy::U64) => true,
        ty::Tuple(ts) => ts.iter().all(scalar),
        _ => false,
    }
}

fn checked_item(tcx: TyCtxt<'_>, id: LocalDefId, default_verify: bool) -> Result<(), VirErr> {
    let mut current = Some(id.to_def_id());
    let mut checked = default_verify;
    while let Some(d) = current {
        let Some(d) = d.as_local() else {
            break;
        };
        let attrs = tcx.hir_attrs(tcx.local_def_id_to_hir_id(d));
        let a = crate::attributes::get_external_attrs(attrs, None)?;
        let v = crate::attributes::get_verifier_attrs(attrs, None)?;
        // rust_to_vir::check_item omits these bodies. Reject the markers on
        // every ancestor too, so admission cannot count an untranslated local.
        if v.internal_const_body || v.internal_reveal_fn {
            return Err(fail(
                tcx.def_span(id),
                "frontend-skipped const_body/reveal_fn local item or ancestor is forbidden",
            ));
        }
        // These items become frontend directives rather than ordinary checked
        // Functions (rust_to_vir::check_item / rust_to_vir_func::check_item_fn).
        // Conservatively reject the markers on ancestors as well.
        if v.reveal_group || v.item_broadcast_use || v.size_of_global || v.structural_const_wrapper
        {
            return Err(fail(
                tcx.def_span(id),
                "frontend directive/special-const local item or ancestor is forbidden",
            ));
        }
        if a.external
            || a.external_body
            || a.external_fn_specification
            || a.external_type_specification
            || a.external_trait_specification
            || v.trusted
        {
            return Err(fail(
                tcx.def_span(id),
                "external/abstract/skipped/trusted local item or ancestor is forbidden",
            ));
        }
        // Ordinary get_crate_items visits the root contents without applying
        // crate-owner opt-in flags. Keep root exclusions above, but only actual
        // item/module contexts (or the real default option) establish checking.
        if d != hir::CRATE_OWNER_ID.def_id {
            checked |= a.verify || a.verus_macro || a.sets_mode;
        }
        current = tcx.opt_parent(d.to_def_id());
    }
    if !checked {
        return Err(fail(
            tcx.def_span(id),
            "native/unverified/skipped local body is forbidden; all locals require ordinary checking",
        ));
    }
    Ok(())
}
fn no_generics(tcx: TyCtxt<'_>, id: LocalDefId) -> Result<(), VirErr> {
    let g = tcx.generics_of(id);
    if g.own_params.iter().any(|p| !matches!(p.kind, ty::GenericParamDefKind::Lifetime)) {
        return Err(fail(tcx.def_span(id), "local type/const generics are forbidden"));
    }
    let predicates = tcx.explicit_clauses_of(id);
    if predicates
        .clauses
        .iter()
        .any(|(p, _)| !matches!(p.kind().skip_binder(), ty::ClauseKind::RegionOutlives(_)))
    {
        return Err(fail(
            tcx.def_span(id),
            "local trait/type/projection bounds are forbidden; only region binders are admitted",
        ));
    }
    Ok(())
}
fn acyclic(
    edges: &HashMap<DefId, HashSet<DefId>>,
    id: DefId,
    active: &mut HashSet<DefId>,
    done: &mut HashSet<DefId>,
) -> bool {
    if done.contains(&id) {
        return true;
    }
    if !active.insert(id) {
        return false;
    }
    if let Some(es) = edges.get(&id) {
        for e in es {
            if !acyclic(edges, *e, active, done) {
                return false;
            }
        }
    }
    active.remove(&id);
    done.insert(id);
    true
}

struct SourceTypes<'tcx> {
    tcx: TyCtxt<'tcx>,
    error: Option<VirErr>,
}
impl<'tcx> hir::intravisit::Visitor<'tcx> for SourceTypes<'tcx> {
    type NestedFilter = rustc_middle::hir::nested_filter::All;
    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }
    fn visit_ty(&mut self, t: &'tcx hir::Ty<'tcx, hir::AmbigArg>) {
        if matches!(
            t.kind,
            hir::TyKind::Path(hir::QPath::TypeRelative(..) | hir::QPath::Resolved(Some(_), _))
        ) {
            self.error.get_or_insert_with(|| {
                fail(t.span, "source associated projection/qualified type is forbidden")
            });
        }
        hir::intravisit::walk_ty(self, t);
    }
}

pub(super) fn admit(tcx: TyCtxt<'_>, cell: DefId, args: &Args) -> Result<AdmittedUnit, VirErr> {
    use hir::intravisit::Visitor;
    let mut source_types = SourceTypes { tcx, error: None };
    source_types.visit_mod(tcx.hir_root_module(), rustc_span::DUMMY_SP, hir::CRATE_HIR_ID);
    if let Some(e) = source_types.error {
        return Err(e);
    }
    let anchor = tcx
        .get_diagnostic_item(rustc_span::Symbol::intern("verus::verus_builtin::int"))
        .filter(|id| !id.is_local())
        .ok_or_else(|| {
            fail(rustc_span::DUMMY_SP, "missing actual nonlocal builtin diagnostic anchor")
        })?;
    let mut u = Unit {
        tcx,
        cell,
        builtins: verus_items::from_diagnostic_items(tcx),
        builtin_crate: anchor.krate,
        locals: HashMap::new(),
        constants: HashSet::new(),
        native: HashSet::new(),
        visited: HashSet::new(),
        edges: HashMap::new(),
        headers: vec![],
    };
    for op in [Operation::New, Operation::IntoInner, Operation::GetMut] {
        u.native.insert(native_method(tcx, cell, op).map_err(|s| fail(rustc_span::DUMMY_SP, s))?);
    }
    // rustc's whole-crate owner inventory includes nested items, independent of
    // verification attributes, external filtering and proof selection.
    let items = tcx.hir_crate_items(());
    if items.opaques().next().is_some() {
        return Err(fail(rustc_span::DUMMY_SP, "opaque/projection local item is forbidden"));
    }
    for id in items.definitions() {
        match tcx.def_kind(id) {
            DefKind::Mod | DefKind::Use | DefKind::ExternCrate | DefKind::Macro(_) => (),
            DefKind::Fn => {
                checked_item(tcx, id, args.no_external_by_default)?;
                no_generics(tcx, id)?;
                let mode = crate::attributes::get_mode(
                    vir::ast::Mode::Exec,
                    tcx.hir_attrs(tcx.local_def_id_to_hir_id(id)),
                );
                let surface = if mode == vir::ast::Mode::Spec {
                    Surface::Spec
                } else if mode == vir::ast::Mode::Exec {
                    Surface::Exec
                } else {
                    return Err(fail(
                        tcx.def_span(id),
                        "proof-mode function is outside closed grammar",
                    ));
                };
                u.locals.insert(id.to_def_id(), surface);
                let sig = tcx.fn_sig(id).instantiate_identity().skip_norm_wip().skip_binder();
                if sig.c_variadic()
                    || sig.safety() != hir::Safety::Safe
                    || sig.abi() != rustc_abi::ExternAbi::Rust
                {
                    return Err(fail(tcx.def_span(id), "unsafe/foreign function ABI is forbidden"));
                }
                for t in sig.inputs() {
                    u.typ(id.to_def_id(), *t, surface, tcx.def_span(id))?;
                }
                let ret_mode = crate::attributes::get_ret_mode(
                    if surface == Surface::Exec {
                        vir::ast::Mode::Exec
                    } else {
                        vir::ast::Mode::Spec
                    },
                    tcx.hir_attrs(tcx.local_def_id_to_hir_id(id)),
                );
                u.typ(
                    id.to_def_id(),
                    sig.output(),
                    if ret_mode == vir::ast::Mode::Exec { Surface::Exec } else { Surface::Spec },
                    tcx.def_span(id),
                )?;
            }
            DefKind::TyAlias => {
                no_generics(tcx, id)?;
                u.typ(
                    id.to_def_id(),
                    tcx.type_of(id).instantiate_identity().skip_norm_wip(),
                    Surface::Exec,
                    tcx.def_span(id),
                )?;
            }
            DefKind::Const { .. } => {
                checked_item(tcx, id, args.no_external_by_default)?;
                no_generics(tcx, id)?;
                u.typ(
                    id.to_def_id(),
                    tcx.type_of(id).instantiate_identity().skip_norm_wip(),
                    Surface::Spec,
                    tcx.def_span(id),
                )?;
                u.constants.insert(id.to_def_id());
            }
            _ => {
                return Err(fail(
                    tcx.def_span(id),
                    format!(
                        "local item {:?} is forbidden (ADTs/traits/abstract callbacks/statics/drop/foreign items)",
                        tcx.def_kind(id)
                    ),
                ));
            }
        }
    }
    for id in tcx.hir_body_owners() {
        if !u.locals.contains_key(&id.to_def_id()) && !u.constants.contains(&id.to_def_id()) {
            continue;
        }
        let owner = id.to_def_id();
        let types = tcx.typeck(id);
        let body = tcx.hir_body_owned_by(id);
        let surface = *u.locals.get(&owner).unwrap_or(&Surface::Spec);
        let mut params = vec![];
        for p in body.params {
            let mode = crate::attributes::get_var_mode(
                if surface == Surface::Exec { vir::ast::Mode::Exec } else { vir::ast::Mode::Spec },
                tcx.hir_attrs(p.hir_id),
            );
            u.pattern(
                owner,
                types,
                p.pat,
                if mode == vir::ast::Mode::Exec { Surface::Exec } else { Surface::Spec },
            )?;
            params.push(u.binding(p.pat)?);
        }
        u.visited.insert(id);
        if let E::Block(b, None) = body.value.kind {
            u.typed(owner, types, body.value, surface)?;
            u.block(owner, types, b, surface, Some(&params))?;
        } else {
            u.expr(owner, types, body.value, surface)?;
        }
    }
    for id in tcx.hir_body_owners().chain(items.nested_bodies()) {
        if !u.visited.contains(&id) {
            return Err(fail(
                tcx.def_span(id),
                "unvisited nested owner/closure/anonymous const/promotion body is forbidden",
            ));
        }
    }
    if tcx.dcx().has_errors().is_some() {
        return Err(fail(
            rustc_span::DUMMY_SP,
            "rustc typechecking failed; typed admission is not established",
        ));
    }
    let mut done = HashSet::new();
    for id in u.locals.keys().chain(u.constants.iter()) {
        if !acyclic(&u.edges, *id, &mut HashSet::new(), &mut done) {
            return Err(fail(
                tcx.def_span(*id),
                "actual executable/spec local recursion is forbidden; canonical type-only self calls are not graph edges",
            ));
        }
    }
    Ok(AdmittedUnit {
        native_cell: cell,
        functions: u.locals.keys().copied().collect(),
        bodies: u.visited,
        headers: u.headers,
    })
}
