//! Untrusted interval lemmas, checked by the existing bit-vector decision procedure.
//!
//! Guessed operand ranges are PREMISES, never assumptions. Each complete lemma
//! is shared verbatim by its isolated check and its use in the ordinary query.
//! Explicit finite-type premises bridge BV variables and mathematical integers.
//! This pass decorates SST after precision transformation; it changes no source
//! expressions, safety assertions, assignments, or precision obligations.
use super::*;
use crate::def::Spanned;
use crate::sst::{self, Exp, ExpX, FuncCheckSst, Stm, StmX};
use abstract_infer::Range;

type Bounds = (BigInt, BigInt);
type Env = HashMap<VarIdent, Bounds>;

fn binary(op: sst::BinaryOp, a: Exp, b: Exp) -> Exp {
    SpannedTyped::new(&a.span, &bool_typ(), ExpX::Binary(op, a.clone(), b))
}
fn number(e: &Exp, n: BigInt) -> Exp {
    SpannedTyped::new(&e.span, &int_typ(), ExpX::Const(Constant::Int(n)))
}
fn within(e: &Exp, (lo, hi): Bounds) -> Exp {
    binary(
        sst::BinaryOp::And,
        binary(sst::BinaryOp::Inequality(InequalityOp::Le), number(e, lo), e.clone()),
        binary(sst::BinaryOp::Inequality(InequalityOp::Le), e.clone(), number(e, hi)),
    )
}
fn full(typ: &Typ, bits: u32) -> Option<Bounds> {
    let TypX::Int(r) = &**typ else { return None };
    arithmetic::bounds_at(*r, bits)
}

// This conversion only PROPOSES lemmas about pure published formulas. Any bug
// here still has to produce a universally valid BV theorem, used verbatim by
// the ordinary proof; it cannot install a callee fact or change a call.
pub(super) fn summary_exp(
    e: &Expr,
    vars: &HashMap<VarIdent, Exp>,
    budget: &mut usize,
) -> Option<Exp> {
    *budget = budget.checked_sub(1)?;
    let x = match &e.x {
        ExprX::Var(v) => return vars.get(v).cloned(),
        ExprX::Const(c @ (Constant::Int(_) | Constant::Bool(_))) => ExpX::Const(c.clone()),
        ExprX::Unary(op @ (UnaryOp::Not | UnaryOp::Clip { .. } | UnaryOp::BitNot(_)), a) => {
            ExpX::Unary(*op, summary_exp(a, vars, budget)?)
        }
        ExprX::UnaryOpr(op @ UnaryOpr::IntegerTypeBound(_), a) => {
            ExpX::UnaryOpr(op.clone(), summary_exp(a, vars, budget)?)
        }
        ExprX::If(c, a, Some(b)) => ExpX::If(
            summary_exp(c, vars, budget)?,
            summary_exp(a, vars, budget)?,
            summary_exp(b, vars, budget)?,
        ),
        ExprX::Logical(op, a, b) => ExpX::Binary(
            match op {
                LogicalOp::And => sst::BinaryOp::And,
                LogicalOp::Or => sst::BinaryOp::Or,
                LogicalOp::Implies => sst::BinaryOp::Implies,
            },
            summary_exp(a, vars, budget)?,
            summary_exp(b, vars, budget)?,
        ),
        ExprX::Binary(op, a, b) => ExpX::Binary(
            match op {
                BinaryOp::Eq(_) => sst::BinaryOp::Eq,
                BinaryOp::Ne => sst::BinaryOp::Ne,
                BinaryOp::Inequality(op) => sst::BinaryOp::Inequality(*op),
                BinaryOp::Bitwise(op, _) => sst::BinaryOp::Bitwise(*op),
                BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)) => {
                    sst::BinaryOp::Arith(sst::ArithOp::Add)
                }
                BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)) => {
                    sst::BinaryOp::Arith(sst::ArithOp::Sub)
                }
                BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Allow)) => {
                    sst::BinaryOp::Arith(sst::ArithOp::Mul)
                }
                BinaryOp::Arith(ArithOp::EuclideanDiv(Div0Behavior::Allow)) => {
                    sst::BinaryOp::Arith(sst::ArithOp::EuclideanDiv)
                }
                BinaryOp::Arith(ArithOp::EuclideanMod(Div0Behavior::Allow)) => {
                    sst::BinaryOp::Arith(sst::ArithOp::EuclideanMod)
                }
                _ => return None,
            },
            summary_exp(a, vars, budget)?,
            summary_exp(b, vars, budget)?,
        ),
        _ => return None,
    };
    Some(SpannedTyped::new(&e.span, &e.typ, x))
}
fn bounds(e: &Exp, env: &Env, bits: u32, budget: &mut usize) -> Option<Bounds> {
    *budget = budget.checked_sub(1)?;
    let value = match &e.x {
        ExpX::Const(Constant::Int(n)) => (n.clone(), n.clone()),
        ExpX::Var(v) | ExpX::VarAt(v, VarAt::Pre) => {
            env.get(v).cloned().or_else(|| full(&e.typ, bits))?
        }
        ExpX::Unary(UnaryOp::BitNot(width), a) => {
            let (lo, hi) = bounds(a, env, bits, budget)?;
            match width {
                Some(_) => {
                    let (_, max) = full(&e.typ, bits)?;
                    (&max - hi, max - lo)
                }
                None => (-hi - 1, -lo - 1),
            }
        }
        ExpX::Unary(UnaryOp::Clip { range, .. }, a) => {
            let (lo, hi) = arithmetic::bounds_at(*range, bits)?;
            let (a, b) = bounds(a, env, bits, budget)?;
            let Range::Int(a, b) = abstract_infer::numeric::clip(a, b, lo, hi) else {
                return None;
            };
            (a, b)
        }
        ExpX::Binary(sst::BinaryOp::Arith(op), a, b) => {
            let (a, b, c, d) = {
                let left = bounds(a, env, bits, budget)?;
                let right = bounds(b, env, bits, budget)?;
                (left.0, left.1, right.0, right.1)
            };
            match op {
                sst::ArithOp::Add => (a + c, b + d),
                sst::ArithOp::Sub => (a - d, b - c),
                sst::ArithOp::Mul => {
                    let values = [&a * &c, &a * &d, &b * &c, &b * &d];
                    (values.iter().min()?.clone(), values.iter().max()?.clone())
                }
                _ => return full(&e.typ, bits),
            }
        }
        ExpX::Binary(sst::BinaryOp::Bitwise(op), a, b) => {
            let (a, b) = (bounds(a, env, bits, budget)?, bounds(b, env, bits, budget)?);
            let Range::Int(lo, hi) = abstract_infer::numeric::bitwise(
                *op,
                Range::Int(a.0, a.1),
                Range::Int(b.0, b.1),
                &e.typ,
                bits,
            ) else {
                return None;
            };
            (lo, hi)
        }
        _ => return full(&e.typ, bits),
    };
    // Proposals are bounded even for mathematical arithmetic expressions.
    if value.0.bits().max(value.1.bits()) > 4096 {
        return None;
    }
    Some(value)
}

// Only a small pure fragment is passed to the BV converter. In particular,
// never reinterpret a call, array access, snapshot, or unbounded integer variable.
fn normalize(e: &Exp, leaves: &mut Vec<Exp>, budget: &mut usize) -> Option<Exp> {
    *budget = budget.checked_sub(1)?;
    let x = match &e.x {
        ExpX::Var(_) | ExpX::VarAt(_, VarAt::Pre) => {
            full(&e.typ, 32)?;
            let v = match &e.x {
                ExpX::Var(v) | ExpX::VarAt(v, VarAt::Pre) => v,
                _ => unreachable!(),
            };
            let e = e.new_x(ExpX::Var(v.clone()));
            leaves.push(e.clone());
            return Some(e);
        }
        ExpX::Const(Constant::Int(_) | Constant::Bool(_)) => return Some(e.clone()),
        ExpX::Unary(op @ (UnaryOp::Clip { .. } | UnaryOp::BitNot(_) | UnaryOp::Not), a) => {
            ExpX::Unary(*op, normalize(a, leaves, budget)?)
        }
        ExpX::Binary(op @ (sst::BinaryOp::Bitwise(_) | sst::BinaryOp::Arith(_)), a, b) => {
            let left = normalize(a, leaves, budget)?;
            let mut right = normalize(b, leaves, budget)?;
            if matches!(op, sst::BinaryOp::Bitwise(BitwiseOp::Shl(..) | BitwiseOp::Shr)) {
                let range = match &*right.typ {
                    TypX::Int(IntRange::I(bits)) => Some(IntRange::U(*bits)),
                    TypX::Int(IntRange::ISize) => Some(IntRange::USize),
                    _ => None,
                };
                if let Some(range) = range {
                    // BV translation requires an unsigned shift amount. This
                    // changes the proposed theorem on BOTH check and use sides.
                    // The ordinary source proof must still connect its original
                    // amount to this cast using the existing shift safety check.
                    right = SpannedTyped::new(
                        &right.span,
                        &Arc::new(TypX::Int(range)),
                        ExpX::Unary(UnaryOp::Clip { range, truncate: true }, right.clone()),
                    );
                }
            }
            ExpX::Binary(*op, left, right)
        }
        _ => return None,
    };
    Some(e.new_x(x))
}

fn lemma(e: &Exp, env: &Env, bits: u32) -> Option<Exp> {
    let args: Vec<&Exp> = match &e.x {
        ExpX::Binary(sst::BinaryOp::Bitwise(_), a, b) => vec![a, b],
        ExpX::Unary(UnaryOp::Clip { .. } | UnaryOp::BitNot(_), a) => vec![a],
        _ => return None,
    };
    let mut input_ranges =
        args.iter().map(|a| bounds(a, env, bits, &mut 512)).collect::<Option<Vec<_>>>()?;
    if matches!(e.x, ExpX::Binary(sst::BinaryOp::Bitwise(BitwiseOp::Shl(..) | BitwiseOp::Shr), ..))
    {
        let (width, _) = bitwise::shape(&e.typ)?;
        let width = match width {
            IntegerTypeBitwidth::Width(n) => n,
            IntegerTypeBitwidth::ArchWordSize => bits,
        };
        input_ranges[1].0 = input_ranges[1].0.clone().max(0.into());
        input_ranges[1].1 = input_ranges[1].1.clone().min((width - 1).into());
    }
    if matches!(e.x, ExpX::Unary(UnaryOp::Clip { .. }, _))
        && let Some((min, max)) = full(&e.typ, bits)
        && input_ranges[0].0 >= min
        && input_ranges[0].1 <= max
    {
        return None;
    }
    let range = bounds(e, env, bits, &mut 512)?;
    if Some(range.clone()) == full(&e.typ, bits) {
        return None;
    }
    let mut leaves = vec![];
    let e = normalize(e, &mut leaves, &mut 512)?;
    let args: Vec<&Exp> = match &e.x {
        ExpX::Binary(_, a, b) => vec![a, b],
        ExpX::Unary(_, a) => vec![a],
        _ => unreachable!(),
    };
    let mut premise = SpannedTyped::new(&e.span, &bool_typ(), ExpX::Const(Constant::Bool(true)));
    for (arg, range) in args.into_iter().zip(input_ranges) {
        premise = binary(sst::BinaryOp::And, premise, within(arg, range));
    }
    for leaf in leaves {
        premise = binary(sst::BinaryOp::And, premise, within(&leaf, full(&leaf.typ, bits)?));
    }
    // Select the same architecture in both integer and BV encodings. The BV
    // backend independently specializes and checks both pointer models.
    let arch = SpannedTyped::new(
        &e.span,
        &int_typ(),
        ExpX::UnaryOpr(
            UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits),
            number(&e, 0.into()),
        ),
    );
    premise = binary(
        sst::BinaryOp::And,
        binary(sst::BinaryOp::Eq, arch, number(&e, bits.into())),
        premise,
    );
    Some(binary(sst::BinaryOp::Implies, premise, within(&e, range)))
}

pub(super) fn checked(fact: Exp) -> Result<Stm, VirErr> {
    let span = fact.span.clone();
    let pair = Spanned::new(
        span.clone(),
        StmX::Block(Arc::new(vec![
            Spanned::new(
                span.clone(),
                StmX::AssertBitVector {
                    requires: Arc::new(vec![]),
                    ensures: Arc::new(vec![fact.clone()]),
                },
            ),
            Spanned::new(span, StmX::Assume(fact)),
        ])),
    );
    validate_pair(&pair)?;
    Ok(pair)
}

fn validate_pair(stm: &Stm) -> Result<(), VirErr> {
    if let StmX::Block(stms) = &stm.x
        && stms.len() == 2
        && let StmX::AssertBitVector { requires, ensures } = &stms[0].x
        && requires.is_empty()
        && ensures.len() == 1
        && let StmX::Assume(used) = &stms[1].x
        && Arc::ptr_eq(&ensures[0], used)
    {
        return Ok(());
    }
    Err(error(&stm.span, "Verus Infer: bit-vector hint is missing its identical checked theorem"))
}

struct Proposals<'a> {
    callees: &'a HashMap<Fun, Function>,
    envs: [Env; 2],
    definitions: HashMap<VarIdent, Exp>,
    remaining: usize,
    visits: usize,
}
impl Proposals<'_> {
    fn expression(&mut self, e: &Exp, hints: &mut Vec<Stm>) -> Result<(), VirErr> {
        if self.visits == 0 {
            return Ok(());
        }
        self.visits -= 1;
        match &e.x {
            ExpX::If(c, a, b) => {
                self.expression(c, hints)?;
                let before = self.envs.clone();
                self.refine(c, true);
                self.expression(a, hints)?;
                self.envs = before.clone();
                self.refine(c, false);
                self.expression(b, hints)?;
                self.envs = before;
            }
            ExpX::Unary(_, a) | ExpX::UnaryOpr(_, a) => self.expression(a, hints)?,
            ExpX::Binary(_, a, b) | ExpX::BinaryOpr(_, a, b) => {
                self.expression(a, hints)?;
                self.expression(b, hints)?;
            }
            _ => {}
        }
        for (i, bits) in [32, 64].into_iter().enumerate() {
            if self.remaining > 0
                && let Some(fact) = lemma(e, &self.envs[i], bits)
            {
                hints.push(checked(fact)?);
                self.remaining -= 1;
            }
        }
        Ok(())
    }
    fn refine(&mut self, e: &Exp, truth: bool) {
        if self.visits == 0 {
            return;
        }
        self.visits -= 1;
        match &e.x {
            ExpX::Var(v) if self.definitions.contains_key(v) => {
                let definition = self.definitions[v].clone();
                self.refine(&definition, truth);
            }
            ExpX::Unary(UnaryOp::Not, a) => self.refine(a, !truth),
            ExpX::Binary(sst::BinaryOp::And, a, b) if truth => {
                self.refine(a, true);
                self.refine(b, true);
            }
            ExpX::Binary(sst::BinaryOp::Or, a, b) if !truth => {
                self.refine(a, false);
                self.refine(b, false);
            }
            ExpX::Binary(op @ (sst::BinaryOp::Inequality(_) | sst::BinaryOp::Eq), a, b) => {
                for (i, bits) in [32, 64].into_iter().enumerate() {
                    let Some((alo, ahi)) = bounds(a, &self.envs[i], bits, &mut 512) else {
                        continue;
                    };
                    let Some((blo, bhi)) = bounds(b, &self.envs[i], bits, &mut 512) else {
                        continue;
                    };
                    let op = match (*op, truth) {
                        (sst::BinaryOp::Inequality(op), true) => Some(op),
                        (sst::BinaryOp::Inequality(InequalityOp::Lt), false) => {
                            Some(InequalityOp::Ge)
                        }
                        (sst::BinaryOp::Inequality(InequalityOp::Le), false) => {
                            Some(InequalityOp::Gt)
                        }
                        (sst::BinaryOp::Inequality(InequalityOp::Gt), false) => {
                            Some(InequalityOp::Le)
                        }
                        (sst::BinaryOp::Inequality(InequalityOp::Ge), false) => {
                            Some(InequalityOp::Lt)
                        }
                        _ => None,
                    };
                    let (left, right) = match op {
                        Some(InequalityOp::Lt) => (
                            (alo.clone(), ahi.clone().min(&bhi - 1)),
                            (blo.clone().max(&alo + 1), bhi.clone()),
                        ),
                        Some(InequalityOp::Le) => (
                            (alo.clone(), ahi.clone().min(bhi.clone())),
                            (blo.clone().max(alo.clone()), bhi.clone()),
                        ),
                        Some(InequalityOp::Gt) => (
                            (alo.clone().max(&blo + 1), ahi.clone()),
                            (blo.clone(), bhi.clone().min(&ahi - 1)),
                        ),
                        Some(InequalityOp::Ge) => (
                            (alo.clone().max(blo.clone()), ahi.clone()),
                            (blo.clone(), bhi.clone().min(ahi.clone())),
                        ),
                        None if truth => {
                            let range = (alo.max(blo), ahi.min(bhi));
                            (range.clone(), range)
                        }
                        _ => continue,
                    };
                    for (e, range) in [(a, left), (b, right)] {
                        if range.0 <= range.1
                            && let ExpX::Var(v) | ExpX::VarAt(v, VarAt::Pre) = &e.x
                        {
                            let mut v = v.clone();
                            for _ in 0..128 {
                                self.envs[i].insert(v.clone(), range.clone());
                                let Some(definition) = self.definitions.get(&v) else { break };
                                let ExpX::Var(root) = &definition.x else { break };
                                if *root == v {
                                    break;
                                }
                                v = root.clone();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    fn stm(&mut self, stm: &Stm) -> Result<Stm, VirErr> {
        let mut hints = vec![];
        let x = match &stm.x {
            StmX::Block(stms) => {
                return Ok(stm.new_x(StmX::Block(Arc::new(
                    stms.iter().map(|s| self.stm(s)).collect::<Result<_, _>>()?,
                ))));
            }
            StmX::If(c, a, b) => {
                self.expression(c, &mut hints)?;
                let before = self.envs.clone();
                self.refine(c, true);
                let a = self.stm(a)?;
                self.envs = before.clone();
                self.refine(c, false);
                let b = b.as_ref().map(|s| self.stm(s)).transpose()?;
                self.envs = before;
                StmX::If(c.clone(), a, b)
            }
            StmX::Assign { lhs, rhs } => {
                self.expression(rhs, &mut hints)?;
                if let ExpX::Var(v) | ExpX::VarLoc(v) = &lhs.dest.x {
                    self.definitions.insert(v.clone(), rhs.clone());
                    for (i, bits) in [32, 64].into_iter().enumerate() {
                        let range = bounds(rhs, &self.envs[i], bits, &mut 512);
                        self.envs[i].remove(v);
                        if let Some(range) = range {
                            self.envs[i].insert(v.clone(), range);
                        }
                    }
                }
                stm.x.clone()
            }
            StmX::Assert(_, _, e) | StmX::Assume(e) => {
                self.expression(e, &mut hints)?;
                stm.x.clone()
            }
            StmX::Return { ret_exp: Some(e), .. } => {
                self.expression(e, &mut hints)?;
                stm.x.clone()
            }
            // Loops and calls invalidate guesses; no unrolling or executable changes.
            StmX::Call { fun: sst::CallTarget::Fun(name), args, dest: Some(dest), .. } => {
                if let Some(callee) = self.callees.get(name)
                    && let ExpX::Var(v) | ExpX::VarLoc(v) = &dest.dest.x
                {
                    let vars = callee
                        .x
                        .params
                        .iter()
                        .zip(args.iter())
                        .map(|(p, e)| (p.x.name.clone(), e.clone()))
                        .collect();
                    if let Some(formula) = abstract_infer::call_formula(callee)
                        && let Some(formula) = summary_exp(&formula, &vars, &mut 512)
                    {
                        self.expression(&formula, &mut hints)?;
                    }
                    for (i, bits) in [32, 64].into_iter().enumerate() {
                        let inputs = args
                            .iter()
                            .map(|e| {
                                bounds(e, &self.envs[i], bits, &mut 512)
                                    .map(|(a, b)| Range::Int(a, b))
                            })
                            .collect::<Option<Vec<_>>>();
                        let proposed = inputs.and_then(|inputs| {
                            abstract_infer::call_range(callee, inputs, self.callees, bits)
                        });
                        self.envs[i].remove(v);
                        if let Some(Range::Int(a, b)) = proposed {
                            self.envs[i].insert(v.clone(), (a, b));
                        }
                    }
                    self.definitions.remove(v);
                }
                stm.x.clone()
            }
            StmX::Loop { .. } | StmX::Call { .. } => {
                self.envs = Default::default();
                self.definitions.clear();
                stm.x.clone()
            }
            _ => stm.x.clone(),
        };
        if hints.is_empty() {
            return Ok(stm.new_x(x));
        }
        hints.push(stm.new_x(x));
        Ok(stm.new_x(StmX::Block(Arc::new(hints))))
    }
}

pub(crate) fn decorate(
    check: &FuncCheckSst,
    callees: &HashMap<Fun, Function>,
) -> Result<FuncCheckSst, VirErr> {
    // Eligibility excludes source proof blocks. Reject preexisting auxiliary BV
    // assertions rather than accepting a disconnected check/use pair.
    crate::sst_visitor::map_stm_visitor(&check.body, &mut |s| {
        if matches!(s.x, StmX::AssertBitVector { .. }) {
            return Err(error(&s.span, "Verus Infer: unexpected preexisting bit-vector hint"));
        }
        Ok(s.clone())
    })?;
    let mut proposals = Proposals {
        callees,
        envs: Default::default(),
        definitions: HashMap::new(),
        remaining: 128,
        visits: 16384,
    };
    let mut decorated = check.clone();
    decorated.body = proposals.stm(&check.body)?;
    Ok(decorated)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fact(value: bool) -> Exp {
        let span = crate::infer::tests::input("x").span.clone();
        SpannedTyped::new(&span, &bool_typ(), ExpX::Const(Constant::Bool(value)))
    }

    #[test]
    fn checked_hint_requires_an_identical_theorem_and_a_real_unconditional_query() {
        let used = fact(true);
        let pair = checked(used.clone()).unwrap();
        assert!(validate_pair(&pair).is_ok());
        let StmX::Block(stms) = &pair.x else { panic!() };
        for broken in [
            vec![stms[1].clone()],
            vec![stms[0].clone()],
            vec![stms[0].clone(), stms[1].new_x(StmX::Assume(fact(false)))],
            vec![
                stms[0].new_x(StmX::AssertBitVector {
                    requires: Arc::new(vec![fact(false)]),
                    ensures: Arc::new(vec![used.clone()]),
                }),
                stms[1].clone(),
            ],
            vec![
                stms[0].new_x(StmX::AssertBitVector {
                    requires: Arc::new(vec![]),
                    ensures: Arc::new(vec![]),
                }),
                stms[1].clone(),
            ],
        ] {
            assert!(validate_pair(&pair.new_x(StmX::Block(Arc::new(broken)))).is_err());
        }
    }

    #[test]
    fn hints_reject_unbounded_variables_calls_and_nonpre_snapshots() {
        let v = SpannedTyped::new(
            &fact(true).span,
            &int_typ(),
            ExpX::Var(VarIdent(Arc::new("arbitrary".into()), VarIdentDisambiguate::AirLocal)),
        );
        assert!(normalize(&v, &mut vec![], &mut 512).is_none());
        let old = v.new_x(ExpX::Old(
            Arc::new("snapshot".into()),
            VarIdent(Arc::new("arbitrary".into()), VarIdentDisambiguate::AirLocal),
        ));
        assert!(normalize(&old, &mut vec![], &mut 512).is_none());
    }
}
