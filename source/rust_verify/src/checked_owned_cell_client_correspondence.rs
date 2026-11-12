//! Test-only whole executable-body comparison. This does NOT grant authority,
//! certify contracts, or replace lifetime/erasure/state queries.
use super::*;
use rustc_hir::{self as hir, ExprKind as H, def::Res};
use rustc_middle::ty::{
    self,
    adjustment::{Adjust, AutoBorrow, DerefAdjustKind},
};
#[derive(Clone, Debug, PartialEq, Eq)]
enum T {
    U64,
    Bool,
    Cell,
    Mut(Box<T>),
    Tuple(Vec<T>),
}
impl T {
    fn copy(&self) -> bool {
        match self {
            Self::U64 | Self::Bool => true,
            Self::Tuple(ts) => ts.iter().all(T::copy),
            _ => false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct E(T, K);
#[derive(Clone, Debug, PartialEq, Eq)]
enum K {
    Var(usize),
    Call(usize, Vec<E>),
    Deref(Box<E>),
    Borrow(bool, Box<E>),
    Tuple(Vec<E>),
    Assign(Box<E>, Box<E>),
    Block(Vec<S>, Option<Box<E>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum S {
    Let(usize, bool, T, E),
    Expr(E),
}
type R<X> = Result<X, &'static str>;
struct Compare<'tcx> {
    tcx: TyCtxt<'tcx>,
    cell: DefId,
    types: &'tcx ty::TypeckResults<'tcx>,
    methods: Vec<(DefId, vir::ast::Path)>,
    headers: HashSet<hir::HirId>,
    hvars: HashMap<hir::HirId, usize>,
    vvars: HashMap<vir::ast::VarIdent, usize>,
}
impl<'tcx> Compare<'tcx> {
    fn ht(&self, t: ty::Ty<'tcx>) -> R<T> {
        match t.kind() {
            ty::Uint(ty::UintTy::U64) => Ok(T::U64),
            ty::Bool => Ok(T::Bool),
            ty::Tuple(ts) => Ok(T::Tuple(ts.iter().map(|t| self.ht(t)).collect::<R<_>>()?)),
            ty::Adt(d, ts)
                if d.did() == self.cell
                    && ts.len() == 1
                    && matches!(ts.type_at(0).kind(), ty::Uint(ty::UintTy::U64)) =>
            {
                Ok(T::Cell)
            }
            ty::Ref(_, t, hir::Mutability::Mut) => Ok(T::Mut(Box::new(self.ht(*t)?))),
            _ => Err("HIR type outside complete comparison grammar"),
        }
    }
    fn vt(&self, t: &vir::ast::Typ) -> R<T> {
        match &**t {
            TypX::Int(IntRange::U(64)) => Ok(T::U64),
            TypX::Bool => Ok(T::Bool),
            TypX::MutRef(t) => Ok(T::Mut(Box::new(self.vt(t)?))),
            TypX::Datatype(Dt::Tuple(n), ts, is) if *n == ts.len() && is.is_empty() => {
                Ok(T::Tuple(ts.iter().map(|t| self.vt(t)).collect::<R<_>>()?))
            }
            TypX::Datatype(Dt::Path(p), ts, is)
                if *p
                    == crate::rust_to_vir_base::def_id_to_vir_path_option(
                        self.tcx, None, self.cell,
                    )
                    .ok_or("actual native path absent")?
                    && ts.len() == 1
                    && is.is_empty()
                    && self.vt(&ts[0])? == T::U64 =>
            {
                Ok(T::Cell)
            }
            _ => Err("actual emitted type outside comparison grammar"),
        }
    }
    fn he(&mut self, e: &'tcx hir::Expr<'tcx>) -> R<E> {
        let typ = self.ht(self.types.expr_ty(e))?;
        let k = match e.kind {
            H::DropTemps(x) => return self.he(x),
            H::Path(ref p) => {
                let Res::Local(id) = self.types.qpath_res(p, e.hir_id) else {
                    return Err("nonlocal value");
                };
                K::Var(*self.hvars.get(&id).ok_or("unknown HIR binding")?)
            }
            H::Block(b, None) => {
                let mut ss = vec![];
                for s in b.stmts {
                    match s.kind {
                        hir::StmtKind::Let(l) => {
                            if l.els.is_some() {
                                return Err("let else");
                            }
                            let hir::PatKind::Binding(mode, id, _, None) = l.pat.kind else {
                                return Err("HIR pattern");
                            };
                            if mode.0 != rustc_ast::ByRef::No {
                                return Err("reference binding");
                            }
                            let init = self.he(l.init.ok_or("uninitialized local")?)?;
                            let t = self.ht(self.types.pat_ty(l.pat))?;
                            if t != init.0 {
                                return Err("HIR initializer type");
                            }
                            let key = self.hvars.len();
                            if self.hvars.insert(id, key).is_some() {
                                return Err("duplicate HIR binding");
                            }
                            ss.push(S::Let(key, mode.1 == hir::Mutability::Mut, t, init));
                        }
                        hir::StmtKind::Expr(x) | hir::StmtKind::Semi(x) => {
                            if !self.headers.contains(&x.hir_id) {
                                ss.push(S::Expr(self.he(x)?));
                            }
                        }
                        _ => return Err("uncompared executable HIR item"),
                    }
                }
                K::Block(ss, b.expr.map(|x| self.he(x).map(Box::new)).transpose()?)
            }
            H::Tup(xs) => K::Tuple(xs.iter().map(|e| self.he(e)).collect::<R<_>>()?),
            H::Assign(a, b, _) => K::Assign(Box::new(self.he(a)?), Box::new(self.he(b)?)),
            H::Unary(hir::UnOp::Deref, x) => K::Deref(Box::new(self.he(x)?)),
            H::MethodCall(_, receiver, args, _) => {
                let native = self
                    .types
                    .type_dependent_def_id(e.hir_id)
                    .ok_or("unresolved original method")?;
                let index = self
                    .methods
                    .iter()
                    .position(|(id, _)| *id == native)
                    .ok_or("non-native method")?;
                let mut r = self.he(receiver)?;
                for a in self.types.expr_adjustments(receiver) {
                    let t = self.ht(a.target)?;
                    r = E(
                        t,
                        match a.kind {
                            Adjust::Deref(DerefAdjustKind::Builtin) => K::Deref(Box::new(r)),
                            Adjust::Borrow(AutoBorrow::Ref(
                                ty::adjustment::AutoBorrowMutability::Mut {
                                    allow_two_phase_borrow,
                                },
                            )) => K::Borrow(
                                matches!(
                                    allow_two_phase_borrow,
                                    ty::adjustment::AllowTwoPhase::Yes
                                ),
                                Box::new(r),
                            ),
                            _ => return Err("uncompared receiver adjustment"),
                        },
                    );
                }
                let mut values = vec![r];
                for a in args {
                    values.push(self.he(a)?);
                }
                K::Call(index, values)
            }
            H::Call(c, args) => {
                let ty::FnDef(id, ts) = self.types.expr_ty(c).kind() else {
                    return Err("non-direct original call");
                };
                let ts = ts.skip_binder();
                if ts.len() != 1 || !matches!(ts.type_at(0).kind(), ty::Uint(ty::UintTy::U64)) {
                    return Err("native type arguments");
                }
                let index = self
                    .methods
                    .iter()
                    .position(|(d, _)| d == id)
                    .ok_or("non-native direct call")?;
                K::Call(index, args.iter().map(|a| self.he(a)).collect::<R<_>>()?)
            }
            _ => return Err("uncompared executable HIR expression"),
        };
        Ok(E(typ, k))
    }
    fn vp(&mut self, p: &Place) -> R<E> {
        let typ = self.vt(&p.typ)?;
        let k = match &p.x {
            PlaceX::Local(v) => K::Var(*self.vvars.get(v).ok_or("unknown actual binding")?),
            PlaceX::DerefMut(p) => K::Deref(Box::new(self.vp(p)?)),
            PlaceX::Temporary(e) => {
                let e = self.ve(e)?;
                if e.0 != typ {
                    return Err("temporary type");
                }
                return Ok(e);
            }
            _ => return Err("uncompared actual place"),
        };
        Ok(E(typ, k))
    }
    fn ve(&mut self, e: &Expr) -> R<E> {
        let typ = self.vt(&e.typ)?;
        let k = match &e.x {
            ExprX::ReadPlace(p, r) => {
                let p = self.vp(p)?;
                if typ != p.0
                    || !match r.preliminary_kind {
                        ReadKind::Copy => p.0.copy(),
                        ReadKind::Move => !p.0.copy(),
                        _ => false,
                    }
                {
                    return Err("actual read kind/type");
                }
                return Ok(p);
            }
            ExprX::TwoPhaseBorrowMut(p) => K::Borrow(true, Box::new(self.vp(p)?)),
            ExprX::BorrowMut(p) => K::Borrow(false, Box::new(self.vp(p)?)),
            ExprX::Block(ss, tail) => {
                let mut stmts = vec![];
                for s in ss.iter() {
                    match &s.x {
                        StmtX::Expr(e) => stmts.push(S::Expr(self.ve(e)?)),
                        StmtX::Decl {
                            pattern,
                            mode,
                            init: Some(init),
                            els: None,
                            assert_irrefutable: false,
                        } => {
                            if !matches!(mode, Some((Mode::Exec, DeclProph::Default))) {
                                return Err("actual local mode");
                            }
                            let PatternX::Var(b) = &pattern.x else {
                                return Err("actual binding pattern");
                            };
                            if b.by_ref != ByRef::No || self.vt(&b.typ)? != self.vt(&pattern.typ)? {
                                return Err("actual binding type");
                            }
                            let init = self.vp(init)?;
                            let t = self.vt(&b.typ)?;
                            if t != init.0 {
                                return Err("actual initializer type");
                            }
                            let key = self.vvars.len();
                            if self.vvars.insert(b.name.clone(), key).is_some() {
                                return Err("duplicate actual binding");
                            }
                            stmts.push(S::Let(
                                key,
                                b.user_mut.ok_or("actual local mutability")?,
                                t,
                                init,
                            ));
                        }
                        _ => return Err("uncompared actual statement"),
                    }
                }
                K::Block(stmts, tail.as_ref().map(|e| self.ve(e).map(Box::new)).transpose()?)
            }
            ExprX::Assign { place, rhs, op: None, typ: t, resolve: false } => {
                if self.vt(t)? != self.vt(&place.typ)? {
                    return Err("assignment type");
                }
                K::Assign(Box::new(self.vp(place)?), Box::new(self.ve(rhs)?))
            }
            ExprX::Ctor(Dt::Tuple(n), variant, fields, None) => {
                if *n != fields.len() || *variant != vir::def::prefix_tuple_variant(*n) {
                    return Err("actual tuple shape");
                }
                let mut values = vec![];
                for (i, b) in fields.iter().enumerate() {
                    if b.name != vir::def::positional_field_ident(i) {
                        return Err("actual tuple index");
                    }
                    values.push(self.ve(&b.a)?);
                }
                K::Tuple(values)
            }
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, fun, ts, is, attrs),
                args,
                post_args: None,
                body: None,
            } => {
                if ts.len() != 1
                    || self.vt(&ts[0])? != T::U64
                    || !is.is_empty()
                    || !matches!(attrs.autospec, AutospecUsage::Final)
                    || attrs.const_var
                    || attrs.assume_external_allowed
                {
                    return Err("actual native call flags/types");
                }
                let index = self
                    .methods
                    .iter()
                    .position(|(_, p)| *p == fun.path)
                    .ok_or("rewritten or unknown actual native target")?;
                K::Call(index, args.iter().map(|a| self.ve(a)).collect::<R<_>>()?)
            }
            _ => return Err("uncompared actual expression"),
        };
        Ok(E(typ, k))
    }
}
pub(super) fn check<'tcx>(
    tcx: TyCtxt<'tcx>,
    cell: DefId,
    owner: rustc_hir::def_id::LocalDefId,
    f: &Function,
    headers: HashSet<hir::HirId>,
) -> R<()> {
    let body = tcx.hir_body_owned_by(owner);
    let types = tcx.typeck(owner);
    let mut methods = vec![];
    for op in [Operation::New, Operation::IntoInner, Operation::GetMut] {
        let id = native_method(tcx, cell, op)?;
        signature(tcx, cell, id, op)?;
        methods.push((
            id,
            crate::rust_to_vir_base::def_id_to_vir_path_option(tcx, None, id)
                .ok_or("native method path")?,
        ));
    }
    if f.x.params.len() != body.params.len()
        || f.x.mode != Mode::Exec
        || f.x.proxy.is_some()
        || f.x.attrs.is_external_body
        || !f.x.typ_params.is_empty()
        || !f.x.typ_bounds.is_empty()
    {
        return Err("actual original signature/role");
    }
    let expected = crate::rust_to_vir_base::def_id_to_vir_path_option(tcx, None, owner.to_def_id())
        .ok_or("actual owner path")?;
    if f.x.name.path != expected {
        return Err("actual original owner");
    }
    let mut c = Compare {
        tcx,
        cell,
        types,
        methods,
        headers,
        hvars: HashMap::new(),
        vvars: HashMap::new(),
    };
    for (i, (p, v)) in body.params.iter().zip(f.x.params.iter()).enumerate() {
        let hir::PatKind::Binding(mode, id, _, None) = p.pat.kind else {
            return Err("source parameter binding");
        };
        if mode.0 != rustc_ast::ByRef::No
            || v.x.user_mut != (mode.1 == hir::Mutability::Mut)
            || v.x.mode != Mode::Exec
            || c.ht(types.pat_ty(p.pat))? != c.vt(&v.x.typ)?
        {
            return Err("actual/source parameter type");
        }
        c.hvars.insert(id, i);
        c.vvars.insert(v.x.name.clone(), i);
    }
    if c.ht(types.expr_ty(body.value))? != c.vt(&f.x.inner_ret.x.typ)?
        || c.vt(&f.x.outer_ret.x.typ)? != c.vt(&f.x.inner_ret.x.typ)?
    {
        return Err("actual/source return type");
    }
    let source = c.he(body.value)?;
    let actual = c.ve(f.x.body.as_ref().ok_or("no actual body")?)?;
    if source != actual {
        eprintln!("C1 complete source={source:?}\nC1 complete actual={actual:?}");
        return Err("whole typed runtime body/call/place/store/load mismatch");
    }
    Ok(())
}
