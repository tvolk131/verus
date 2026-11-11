//! Untrusted generic packing proposals for local checked-foreign callers.
//! Every fact is a closed conditional theorem checked by the ordinary BV backend
//! and used verbatim. No callee summary, application premise or assertion changes.
use num_bigint::BigInt;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use vir::{
    ast::*,
    ast_util::{bool_typ, int_typ},
    def::Spanned,
    sst::{self, Exp, ExpX, FuncCheckSst, Stm, StmX},
};

// Exact identity for the closed subset emitted below. This is intentionally not
// general SST equivalence: unsupported shapes are always retained. Spans carry
// diagnostics only here; no formula or variable identity is normalized.
#[derive(Debug, PartialEq, Eq, Hash)]
enum KeyTyp {
    Bool,
    Int(IntRange),
}
#[derive(Debug, PartialEq, Eq, Hash)]
struct ClosedKey(KeyTyp, KeyExp);
#[derive(Debug, PartialEq, Eq, Hash)]
enum KeyExp {
    Int(BigInt),
    Var(VarIdent),
    Binary(sst::BinaryOp, Box<ClosedKey>, Box<ClosedKey>),
    Forall(Quant, Vec<(VarIdent, KeyTyp)>, Vec<Vec<ClosedKey>>, Box<ClosedKey>),
}
fn key_typ(t: &Typ) -> Option<KeyTyp> {
    match &**t {
        TypX::Bool => Some(KeyTyp::Bool),
        TypX::Int(r @ (IntRange::Int | IntRange::U(8 | 16 | 32 | 64))) => {
            Some(KeyTyp::Int(*r))
        }
        _ => None,
    }
}
fn closed_key(e: &Exp) -> Option<ClosedKey> {
    fn visit(e: &Exp, bound: &[(VarIdent, KeyTyp)], budget: &mut usize) -> Option<ClosedKey> {
        *budget = budget.checked_sub(1)?;
        let t = key_typ(&e.typ)?;
        let x = match &e.x {
            ExpX::Const(Constant::Int(n)) => KeyExp::Int(n.clone()),
            ExpX::Var(v) if bound.iter().any(|(name, typ)| name == v && *typ == t) => {
                KeyExp::Var(v.clone())
            }
            ExpX::Binary(op @ (sst::BinaryOp::And | sst::BinaryOp::Implies
                | sst::BinaryOp::Eq | sst::BinaryOp::Inequality(_)
                | sst::BinaryOp::Arith(_) | sst::BinaryOp::Bitwise(_)), a, b) => {
                KeyExp::Binary(*op, Box::new(visit(a, bound, budget)?), Box::new(visit(b, bound, budget)?))
            }
            ExpX::Bind(bnd, body) if bound.is_empty() => {
                let sst::BndX::Quant(q, binders, triggers, None) = &bnd.x else { return None };
                if *q != vir::ast_util::QUANT_FORALL || binders.is_empty() || binders.len() > 3 {
                    return None;
                }
                let mut vars = Vec::new();
                for b in binders.iter() {
                    if vars.iter().any(|(name, _)| name == &b.name) { return None; }
                    vars.push((b.name.clone(), key_typ(&b.a)?));
                }
                let triggers = triggers.iter().map(|trig| {
                    trig.iter().map(|e| visit(e, &vars, budget)).collect::<Option<Vec<_>>>()
                }).collect::<Option<Vec<_>>>()?;
                KeyExp::Forall(*q, vars.iter().map(|(v, t)| {
                    // KeyTyp is small; keep the binder's complete supported type.
                    (v.clone(), match t { KeyTyp::Bool => KeyTyp::Bool, KeyTyp::Int(r) => KeyTyp::Int(*r) })
                }).collect(), triggers, Box::new(visit(body, &vars, budget)?))
            }
            _ => return None,
        };
        Some(ClosedKey(t, x))
    }
    visit(e, &[], &mut 4096)
}

fn number(e: &Exp, functions: &HashMap<Fun, Function>) -> Option<u32> {
    let n = match &e.x {
        ExpX::Const(Constant::Int(n)) => n,
        ExpX::Unary(UnaryOp::Clip { .. }, a) => return number(a, functions),
        ExpX::Call(sst::CallFun::Fun(f, None), typs, args)
            if typs.is_empty() && args.is_empty() =>
        {
            let f = functions.get(f)?;
            let mut e = f.x.body.as_ref()?;
            for _ in 0..8 {
                match &e.x {
                    ExprX::Block(stms, Some(tail)) if stms.is_empty() => e = tail,
                    _ => break,
                }
            }
            let ExprX::Const(Constant::Int(n)) = &e.x else { return None };
            n
        }
        _ => return None,
    };
    u32::try_from(n).ok()
}
fn mask_number(e: &Exp, functions: &HashMap<Fun, Function>) -> Option<BigInt> {
    match &e.x {
        ExpX::Const(Constant::Int(n)) => Some(n.clone()),
        ExpX::Unary(UnaryOp::Clip { .. }, a) => mask_number(a, functions),
        ExpX::Call(sst::CallFun::Fun(f, None), typs, args)
            if typs.is_empty() && args.is_empty() =>
        {
            let f = functions.get(f)?;
            if !matches!(f.x.item_kind, ItemKind::Const) {
                return None;
            }
            constant(f.x.body.as_ref()?, &mut 64).map(|(_, n)| n)
        }
        _ => None,
    }
}
fn expression(
    e: &Exp,
    functions: &HashMap<Fun, Function>,
    shapes: &mut BTreeSet<(u32, u32)>,
    budget: &mut usize,
) {
    if *budget == 0 {
        return;
    }
    *budget -= 1;
    match &e.x {
        ExpX::Binary(op, a, b) => {
            if let sst::BinaryOp::Bitwise(BitwiseOp::Shl(
                IntegerTypeBitwidth::Width(w @ (8 | 16 | 32 | 64)),
                false,
            )) = op
            {
                if let Some(k) = number(b, functions).filter(|k| *k < *w) {
                    shapes.insert((*w, k));
                }
            }
            if matches!(op, sst::BinaryOp::Bitwise(_))
                && let TypX::Int(IntRange::U(w @ (8 | 16 | 32 | 64))) = &*a.typ
            {
                // Width/zero is also a bounded activation hint for mask/shift
                // constants in readers, which need not contain a left shift.
                shapes.insert((*w, 0));
                if matches!(op, sst::BinaryOp::Bitwise(BitwiseOp::BitAnd))
                    && let Some(value) = mask_number(b, functions)
                    && let Some((lo, hi)) = mask_field(&value, *w)
                {
                    shapes.insert((*w, lo));
                    if hi < *w {
                        shapes.insert((*w, hi));
                    }
                }
                if matches!(op, sst::BinaryOp::Bitwise(BitwiseOp::Shr))
                    && let Some(k) = number(b, functions).filter(|k| *k < *w)
                {
                    shapes.insert((*w, k));
                }
            }
            expression(a, functions, shapes, budget);
            expression(b, functions, shapes, budget);
        }
        ExpX::Unary(_, a) | ExpX::UnaryOpr(_, a) => expression(a, functions, shapes, budget),
        ExpX::If(c, a, b) => {
            expression(c, functions, shapes, budget);
            expression(a, functions, shapes, budget);
            expression(b, functions, shapes, budget);
        }
        _ => (),
    }
}
fn statements(
    s: &Stm,
    functions: &HashMap<Fun, Function>,
    shapes: &mut BTreeSet<(u32, u32)>,
    budget: &mut usize,
) {
    if *budget == 0 {
        return;
    }
    *budget -= 1;
    match &s.x {
        StmX::Block(ss) => {
            for s in ss.iter() {
                statements(s, functions, shapes, budget);
            }
        }
        StmX::If(c, a, b) => {
            expression(c, functions, shapes, budget);
            statements(a, functions, shapes, budget);
            if let Some(b) = b {
                statements(b, functions, shapes, budget);
            }
        }
        StmX::Assign { rhs, .. } => expression(rhs, functions, shapes, budget),
        StmX::Return { ret_exp: Some(e), .. } => expression(e, functions, shapes, budget),
        StmX::Assert(_, _, e) | StmX::Assume(e) => expression(e, functions, shapes, budget),
        // No traversal into loops or isolated proof/closure scopes is required.
        _ => (),
    }
}
/// Formula construction only: all variables are freshly bound finite unsigned
/// values. Discovery never supplies application expressions or assumptions.
struct Arithmetic<'a> {
    span: &'a vir::messages::Span,
    w: u32,
    t: Typ,
    names: [VarIdent; 3],
}
impl<'a> Arithmetic<'a> {
    fn new(span: &'a vir::messages::Span, w: u32) -> Self {
        Self {
            span,
            w,
            t: Arc::new(TypX::Int(IntRange::U(w))),
            names: ["packing_low", "packing_high", "packing_upper"]
                .map(|s| VarIdent(Arc::new(s.into()), VarIdentDisambiguate::AirLocal)),
        }
    }
    fn v(&self, i: usize) -> Exp {
        SpannedTyped::new(self.span, &self.t, ExpX::Var(self.names[i].clone()))
    }
    fn n(&self, n: BigInt) -> Exp {
        SpannedTyped::new(self.span, &int_typ(), ExpX::Const(Constant::Int(n)))
    }
    fn b(&self, op: sst::BinaryOp, a: Exp, b: Exp) -> Exp {
        SpannedTyped::new(self.span, &bool_typ(), ExpX::Binary(op, a, b))
    }
    fn bit(&self, op: BitwiseOp, a: Exp, b: Exp) -> Exp {
        SpannedTyped::new(self.span, &self.t, ExpX::Binary(sst::BinaryOp::Bitwise(op), a, b))
    }
    fn shift(&self, a: Exp, k: u32) -> Exp {
        self.bit(BitwiseOp::Shl(IntegerTypeBitwidth::Width(self.w), false), a, self.n(k.into()))
    }
    fn shr(&self, a: Exp, k: u32) -> Exp {
        self.bit(BitwiseOp::Shr, a, self.n(k.into()))
    }
    fn and(&self, a: Exp, b: Exp) -> Exp {
        self.b(sst::BinaryOp::And, a, b)
    }
    fn eq(&self, a: Exp, b: Exp) -> Exp {
        self.b(sst::BinaryOp::Eq, a, b)
    }
    fn within(&self, i: usize, bits: u32) -> Exp {
        self.and(
            self.b(sst::BinaryOp::Inequality(InequalityOp::Le), self.n(0.into()), self.v(i)),
            self.b(
                sst::BinaryOp::Inequality(InequalityOp::Lt),
                self.v(i),
                self.n(BigInt::from(1u8) << bits),
            ),
        )
    }
    fn closed(&self, count: usize, trigger: Exp, domain: Exp, conclusion: Exp) -> Exp {
        let binders = Arc::new(
            self.names[..count]
                .iter()
                .map(|name| Arc::new(VarBinderX { name: name.clone(), a: self.t.clone() }))
                .collect(),
        );
        let binder = Spanned::new(
            self.span.clone(),
            sst::BndX::Quant(
                vir::ast_util::QUANT_FORALL,
                binders,
                Arc::new(vec![Arc::new(vec![trigger])]),
                None,
            ),
        );
        SpannedTyped::new(
            self.span,
            &bool_typ(),
            ExpX::Bind(binder, self.b(sst::BinaryOp::Implies, domain, conclusion)),
        )
    }
    fn split(&self, k: u32) -> Vec<Exp> {
        if k == 0 {
            return [self.shift(self.v(0), 0), self.shr(self.v(0), 0)]
                .map(|e| self.closed(1, e.clone(), self.within(0, self.w), self.eq(e, self.v(0))))
                .into();
        }
        let factor = BigInt::from(1u8) << k;
        let shifted = self.shift(self.v(1), k);
        let packed = self.bit(BitwiseOp::BitOr, self.v(0), shifted);
        let domain = self.and(self.within(0, k), self.within(1, self.w - k));
        let product = SpannedTyped::new(
            self.span,
            &int_typ(),
            ExpX::Binary(
                sst::BinaryOp::Arith(sst::ArithOp::Mul),
                self.n(factor.clone()),
                self.v(1),
            ),
        );
        let sum = SpannedTyped::new(
            self.span,
            &int_typ(),
            ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Add), self.v(0), product),
        );
        let masked = self.bit(
            BitwiseOp::BitAnd,
            packed.clone(),
            self.n((BigInt::from(1u8) << self.w) - &factor),
        );
        let positive = |a| self.b(sst::BinaryOp::Inequality(InequalityOp::Gt), a, self.n(0.into()));
        vec![
            self.closed(2, packed.clone(), domain.clone(), self.eq(packed.clone(), sum)),
            self.closed(2, packed.clone(), domain, self.eq(positive(masked), positive(self.v(1)))),
            self.closed(
                2,
                packed.clone(),
                self.and(self.within(0, k), self.within(1, 1)),
                self.eq(
                    positive(self.bit(BitwiseOp::BitAnd, packed.clone(), self.n(factor.clone()))),
                    positive(self.v(1)),
                ),
            ),
            self.closed(
                2,
                packed.clone(),
                self.and(self.within(0, k), self.within(1, self.w)),
                self.eq(self.bit(BitwiseOp::BitAnd, packed, self.n(&factor - 1)), self.v(0)),
            ),
        ]
    }
    fn field(&self, lo: u32, hi: u32) -> Exp {
        let mask = (BigInt::from(1u8) << hi) - (BigInt::from(1u8) << lo);
        let masked = self.bit(BitwiseOp::BitAnd, self.v(0), self.n(mask));
        let extracted = if lo == 0 { masked } else { self.shr(masked, lo) };
        self.closed(
            1,
            extracted.clone(),
            self.within(0, self.w),
            self.b(
                sst::BinaryOp::Inequality(InequalityOp::Lt),
                extracted,
                self.n(BigInt::from(1u8) << (hi - lo)),
            ),
        )
    }
    fn three_fields(&self, lo: u32, hi: u32) -> [Exp; 2] {
        let lower = self.bit(BitwiseOp::BitOr, self.v(0), self.shift(self.v(1), lo));
        let packed = self.bit(BitwiseOp::BitOr, lower, self.shift(self.v(2), hi));
        let domain =
            self.and(self.within(0, lo), self.and(self.within(1, hi - lo), self.within(2, self.w)));
        let low_mask = (BigInt::from(1u8) << lo) - 1;
        let mid_mask = (BigInt::from(1u8) << hi) - (BigInt::from(1u8) << lo);
        [
            self.eq(self.bit(BitwiseOp::BitAnd, packed.clone(), self.n(low_mask)), self.v(0)),
            self.eq(
                self.shr(self.bit(BitwiseOp::BitAnd, packed.clone(), self.n(mid_mask)), lo),
                self.v(1),
            ),
        ]
        .map(|c| self.closed(3, packed.clone(), domain.clone(), c))
    }
}

// Recognizers only select closed theorem shapes. A noncontiguous/empty/outside
// mask supplies no proposal, and cannot affect the logical acceptance boundary.
fn mask_field(value: &BigInt, w: u32) -> Option<(u32, u32)> {
    if value <= &BigInt::from(0u8) || value >= &(BigInt::from(1u8) << w) {
        return None;
    }
    for lo in 0..w {
        for hi in lo + 1..=w {
            if *value == (BigInt::from(1u8) << hi) - (BigInt::from(1u8) << lo) {
                return Some((lo, hi));
            }
        }
    }
    None
}

// Constant folding is only a proposal. Its input expression, rather than a
// call to a guessed constant, is checked verbatim by BV and used verbatim.
fn constant(e: &vir::ast::Expr, budget: &mut usize) -> Option<(Exp, BigInt)> {
    *budget = budget.checked_sub(1)?;
    let (x, value) = match &e.x {
        ExprX::Block(ss, Some(tail)) if ss.is_empty() => return constant(tail, budget),
        ExprX::Const(Constant::Int(n)) => (ExpX::Const(Constant::Int(n.clone())), n.clone()),
        ExprX::Binary(BinaryOp::Bitwise(op, _), a, b) => {
            let (a, av) = constant(a, budget)?;
            let (mut b, bv) = constant(b, budget)?;
            let TypX::Int(IntRange::U(w @ (8 | 16 | 32 | 64))) = &*e.typ else { return None };
            let v = match op {
                BitwiseOp::BitAnd => av & bv,
                BitwiseOp::BitOr => av | bv,
                BitwiseOp::BitXor => av ^ bv,
                BitwiseOp::Shl(..) | BitwiseOp::Shr => {
                    let k = u32::try_from(&bv).ok().filter(|k| *k < *w)?;
                    // Finite unsigned shift literals keep BV conversion bounded.
                    b = SpannedTyped::new(
                        &b.span,
                        &Arc::new(TypX::Int(IntRange::U(64))),
                        b.x.clone(),
                    );
                    if matches!(op, BitwiseOp::Shl(..)) { av << k } else { av >> k }
                }
            } & ((BigInt::from(1u8) << w) - 1);
            (ExpX::Binary(sst::BinaryOp::Bitwise(*op), a, b), v)
        }
        _ => return None,
    };
    Some((SpannedTyped::new(&e.span, &e.typ, x), value))
}

pub(crate) fn prepare(
    check: &FuncCheckSst,
    functions: &HashMap<Fun, Function>,
    krate: &CrateId,
) -> FuncCheckSst {
    let mut shapes = BTreeSet::new();
    statements(&check.body, functions, &mut shapes, &mut 4096);
    let mut stms = vec![];
    let mut constants: Vec<_> = functions
        .values()
        .filter(|f| {
            &f.x.name.path.krate == krate
                && matches!(f.x.item_kind, ItemKind::Const)
                && f.x.params.is_empty()
        })
        .collect();
    constants.sort_by(|a, b| a.x.name.cmp(&b.x.name));
    let mut proposed = vec![];
    let mut fields = BTreeSet::new();
    if !shapes.is_empty() {
        for f in constants.into_iter().take(32) {
            if let Some(body) = &f.x.body
                && let Some((e, value)) = constant(body, &mut 64)
                && !matches!(e.x, ExpX::Const(_))
            {
                if let TypX::Int(IntRange::U(w @ (8 | 16 | 32 | 64))) = &*e.typ
                    && let Some((lo, hi)) = mask_field(&value, *w)
                {
                    fields.insert((*w, lo, hi));
                }
                let rhs = SpannedTyped::new(&e.span, &e.typ, ExpX::Const(Constant::Int(value)));
                proposed.push(SpannedTyped::new(
                    &e.span,
                    &bool_typ(),
                    ExpX::Binary(sst::BinaryOp::Eq, e.clone(), rhs),
                ));
            }
        }
    }

    // At most eight width/shift shapes, 32 constants and 16 mask fields.
    // Pairwise laws use only the admitted bounded shifts of the same width.
    let shapes: Vec<_> = shapes.into_iter().take(8).collect();
    for &(w, k) in &shapes {
        let a = Arithmetic::new(&check.body.span, w);
        proposed.extend(a.split(k));
        if k > 0 {
            proposed.push(a.field(0, k));
            proposed.push(a.field(k, w));
            for &(w2, hi) in &shapes {
                if w2 == w && hi > k {
                    proposed.push(a.field(k, hi));
                    proposed.extend(a.three_fields(k, hi));
                }
            }
        }
    }
    for (w, lo, hi) in fields.into_iter().take(16) {
        proposed.push(Arithmetic::new(&check.body.span, w).field(lo, hi));
    }
    for fact in proposed {
        // The exact same Arc is checked and used. There is no unchecked
        // theorem, and failure of any proposal fails ordinary verification.
        stms.push(Spanned::new(
            fact.span.clone(),
            StmX::AssertBitVector {
                requires: Arc::new(vec![]),
                ensures: Arc::new(vec![fact.clone()]),
            },
        ));
        stms.push(Spanned::new(fact.span.clone(), StmX::Assume(fact)));
    }
    stms.push(check.body.clone());
    let mut result = check.clone();
    result.body = Spanned::new(check.body.span.clone(), StmX::Block(Arc::new(stms)));
    result
}
