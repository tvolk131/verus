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
                if let Some(k) = number(b, functions).filter(|k| *k > 0 && *k < *w) {
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
        StmX::Assert(_, _, e) | StmX::Assume(e) => expression(e, functions, shapes, budget),
        // No traversal into loops or isolated proof/closure scopes is required.
        _ => (),
    }
}
fn facts(span: &vir::messages::Span, w: u32, k: u32) -> [Exp; 2] {
    let t = Arc::new(TypX::Int(IntRange::U(w)));
    let names = ["packing_low", "packing_high"]
        .map(|s| VarIdent(Arc::new(s.into()), VarIdentDisambiguate::AirLocal));
    let v = |i: usize| SpannedTyped::new(span, &t, ExpX::Var(names[i].clone()));
    let n = |x: BigInt| SpannedTyped::new(span, &int_typ(), ExpX::Const(Constant::Int(x)));
    let b = |op, a, b| SpannedTyped::new(span, &bool_typ(), ExpX::Binary(op, a, b));
    let typed = |op, a, b| SpannedTyped::new(span, &t, ExpX::Binary(op, a, b));
    let within = |v: Exp, max: BigInt| {
        b(
            sst::BinaryOp::And,
            b(sst::BinaryOp::Inequality(InequalityOp::Le), n(0.into()), v.clone()),
            b(sst::BinaryOp::Inequality(InequalityOp::Le), v, n(max)),
        )
    };
    let factor = BigInt::from(1u8) << k;
    let domain = b(
        sst::BinaryOp::And,
        within(v(0), &factor - 1),
        within(v(1), (BigInt::from(1u8) << (w - k)) - 1),
    );
    let shifted = typed(
        sst::BinaryOp::Bitwise(BitwiseOp::Shl(IntegerTypeBitwidth::Width(w), false)),
        v(1),
        n(k.into()),
    );
    let packed = typed(sst::BinaryOp::Bitwise(BitwiseOp::BitOr), v(0), shifted);
    let product = SpannedTyped::new(
        span,
        &int_typ(),
        ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Mul), n(factor.clone()), v(1)),
    );
    let sum = SpannedTyped::new(
        span,
        &int_typ(),
        ExpX::Binary(sst::BinaryOp::Arith(sst::ArithOp::Add), v(0), product),
    );
    let mask = (BigInt::from(1u8) << w) - &factor;
    let masked = typed(sst::BinaryOp::Bitwise(BitwiseOp::BitAnd), packed.clone(), n(mask));
    let positive = |a| b(sst::BinaryOp::Inequality(InequalityOp::Gt), a, n(0.into()));
    let conclusions = [
        b(sst::BinaryOp::Eq, packed.clone(), sum),
        b(sst::BinaryOp::Eq, positive(masked), positive(v(1))),
    ];
    conclusions.map(|c| {
        let binders = Arc::new(
            names
                .iter()
                .map(|name| Arc::new(VarBinderX { name: name.clone(), a: t.clone() }))
                .collect(),
        );
        let triggers = Arc::new(vec![Arc::new(vec![packed.clone()])]);
        let binder = Spanned::new(
            span.clone(),
            sst::BndX::Quant(vir::ast_util::QUANT_FORALL, binders, triggers, None),
        );
        SpannedTyped::new(
            span,
            &bool_typ(),
            ExpX::Bind(binder, b(sst::BinaryOp::Implies, domain.clone(), c)),
        )
    })
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
    if !shapes.is_empty() {
        for f in constants.into_iter().take(32) {
            if let Some(body) = &f.x.body
                && let Some((e, value)) = constant(body, &mut 64)
                && !matches!(e.x, ExpX::Const(_))
            {
                let rhs = SpannedTyped::new(&e.span, &e.typ, ExpX::Const(Constant::Int(value)));
                proposed.push(SpannedTyped::new(
                    &e.span,
                    &bool_typ(),
                    ExpX::Binary(sst::BinaryOp::Eq, e.clone(), rhs),
                ));
            }
        }
    }

    for (w, k) in shapes.into_iter().take(4) {
        proposed.extend(facts(&check.body.span, w, k));
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
