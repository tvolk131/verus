//! Bounded, demand-driven proof proposals for ordinary callers of exact inference.
//!
//! Substitution is untrusted: we add only a universally CHECKED conditional
//! theorem, never a rewritten caller assertion or an assumed callee result.
//! The original assertion and every executable safety check remain in place.
//! The existing BV backend checks the identical theorem used by the normal
//! query. Failure of an auxiliary query fails verification, just like an
//! explicit `assert(...) by(bit_vector)`.
use super::*;
use crate::sst::{self, Exp, ExpX, FuncCheckSst, Stm, StmX};

const EXPANSION: usize = 2048;
const DEPTH: usize = 32;
const REQUESTS: usize = 8;
const VISITS: usize = 8192;

#[derive(Clone)]
enum Definition {
    Value(Exp),
    // Keep only the existing summary and actual arguments until a goal needs
    // this result. No summary expansion is performed merely because of a call.
    Call(Function, sst::Exps),
}

fn binary(op: sst::BinaryOp, a: Exp, b: Exp) -> Exp {
    SpannedTyped::new(&a.span, &bool_typ(), ExpX::Binary(op, a.clone(), b))
}

struct Expression<'a> {
    definitions: &'a HashMap<VarIdent, Definition>,
    remaining: usize,
    inferred: bool,
    bitwise: bool,
    premises: Vec<Exp>,
}

impl Expression<'_> {
    fn number(e: &Exp, n: BigInt) -> Exp {
        SpannedTyped::new(&e.span, &int_typ(), ExpX::Const(Constant::Int(n)))
    }

    // Return a conservative bit-width estimate as well as the proposed term.
    // This bounds bit-blasting before asking the existing converter to run.
    fn expand(&mut self, e: &Exp, depth: usize) -> Option<(Exp, u64)> {
        self.remaining = self.remaining.checked_sub(1)?;
        if depth == 0 {
            return None;
        }
        let next = depth - 1;
        let (x, width) = match &e.x {
            ExpX::Var(v) => {
                if let Some(def) = self.definitions.get(v) {
                    return match def {
                        Definition::Value(value) => self.expand(value, next),
                        Definition::Call(function, args) => {
                            let ExprX::Binary(BinaryOp::Eq(Mode::Spec), _, value) =
                                &function.x.ensure.0.first()?.x
                            else {
                                return None;
                            };
                            let vars = function
                                .x
                                .params
                                .iter()
                                .zip(args.iter())
                                .map(|(p, e)| (p.x.name.clone(), e.clone()))
                                .collect();
                            let value = super::bitvector_hints::summary_exp(
                                value,
                                &vars,
                                &mut self.remaining,
                            )?;
                            self.inferred = true;
                            self.expand(&value, next)
                        }
                    };
                }
                let width = match &*e.typ {
                    TypX::Bool => 1,
                    TypX::Int(r @ (IntRange::U(_) | IntRange::I(_))) => {
                        let (lo, hi) = arithmetic::bounds_at(*r, 64)?;
                        self.premises.push(binary(
                            sst::BinaryOp::And,
                            binary(
                                sst::BinaryOp::Inequality(InequalityOp::Le),
                                Self::number(e, lo),
                                e.clone(),
                            ),
                            binary(
                                sst::BinaryOp::Inequality(InequalityOp::Le),
                                e.clone(),
                                Self::number(e, hi),
                            ),
                        ));
                        arithmetic::bounds_at(*r, 64)?.1.bits() + 1
                    }
                    _ => return None,
                };
                (e.x.clone(), width)
            }
            ExpX::Const(Constant::Bool(_)) => (e.x.clone(), 1),
            ExpX::Const(Constant::Int(n)) => (e.x.clone(), n.bits() + 1),
            ExpX::Unary(op @ (UnaryOp::Not | UnaryOp::BitNot(_) | UnaryOp::Clip { .. }), a) => {
                let (a, w) = self.expand(a, next)?;
                (ExpX::Unary(*op, a), w.max(128))
            }
            ExpX::UnaryOpr(UnaryOpr::Box(_) | UnaryOpr::Unbox(_), a) => {
                return self.expand(a, next);
            }
            ExpX::Binary(op, a, b) => {
                let (a, aw) = self.expand(a, next)?;
                let (b, bw) = self.expand(b, next)?;
                let width = match op {
                    sst::BinaryOp::Arith(sst::ArithOp::Mul) => aw + bw,
                    sst::BinaryOp::Arith(sst::ArithOp::Add | sst::ArithOp::Sub) => aw.max(bw) + 1,
                    sst::BinaryOp::Arith(
                        sst::ArithOp::EuclideanDiv | sst::ArithOp::EuclideanMod,
                    ) => {
                        self.premises.push(binary(
                            sst::BinaryOp::Ne,
                            b.clone(),
                            Self::number(&b, 0.into()),
                        ));
                        aw.max(bw) + 1
                    }
                    sst::BinaryOp::Bitwise(
                        BitwiseOp::BitAnd | BitwiseOp::BitOr | BitwiseOp::BitXor,
                    ) => {
                        self.bitwise = true;
                        aw.max(bw)
                    }
                    sst::BinaryOp::And
                    | sst::BinaryOp::Or
                    | sst::BinaryOp::Xor
                    | sst::BinaryOp::Implies
                    | sst::BinaryOp::Eq
                    | sst::BinaryOp::Ne
                    | sst::BinaryOp::Inequality(_) => 1,
                    _ => return None,
                };
                (ExpX::Binary(*op, a, b), width)
            }
            ExpX::If(c, a, b) => {
                let (c, _) = self.expand(c, next)?;
                let (a, aw) = self.expand(a, next)?;
                let (b, bw) = self.expand(b, next)?;
                (ExpX::If(c, a, b), aw.max(bw))
            }
            _ => return None,
        };
        if width > 4096 {
            return None;
        }
        Some((e.new_x(x), width))
    }
}

struct Requests<'a> {
    callees: &'a HashMap<Fun, Function>,
    definitions: HashMap<VarIdent, Definition>,
    assumptions: Vec<Exp>,
    remaining: usize,
    visits: usize,
}

impl Requests<'_> {
    fn request(&mut self, goal: &Exp) -> Option<Exp> {
        if self.remaining == 0 {
            return None;
        }
        let mut e = Expression {
            definitions: &self.definitions,
            remaining: EXPANSION,
            inferred: false,
            bitwise: false,
            premises: vec![],
        };
        let (conclusion, _) = e.expand(goal, DEPTH)?;
        if !e.inferred || !e.bitwise {
            return None;
        }
        for assumption in &self.assumptions {
            // Unsupported assumptions are omitted, never guessed. Losing one
            // can make this tactic incomplete, but cannot justify a false fact.
            if let Some((assumption, _)) = e.expand(assumption, DEPTH) {
                e.premises.push(assumption);
            }
        }
        let mut fact = conclusion;
        for premise in e.premises.into_iter().rev() {
            fact = binary(sst::BinaryOp::Implies, premise, fact);
        }
        self.remaining -= 1;
        // A failed specialized proof should identify the requested assertion,
        // not a parameter inside an expanded callee summary.
        Some(SpannedTyped::new(&goal.span, &bool_typ(), fact.x.clone()))
    }

    fn stm(&mut self, stm: &Stm) -> Result<Stm, VirErr> {
        if self.visits == 0 {
            return Ok(stm.clone());
        }
        self.visits -= 1;
        match &stm.x {
            StmX::Block(stms) => Ok(stm.new_x(StmX::Block(Arc::new(
                stms.iter().map(|s| self.stm(s)).collect::<Result<_, _>>()?,
            )))),
            StmX::Assert(_, _, goal) => {
                if let Some(fact) = self.request(goal) {
                    return Ok(stm.new_x(StmX::Block(Arc::new(vec![
                        super::bitvector_hints::checked(fact)?,
                        stm.clone(),
                    ]))));
                }
                Ok(stm.clone())
            }
            StmX::Assume(e) => {
                if self.assumptions.len() < 32 {
                    self.assumptions.push(e.clone());
                }
                Ok(stm.clone())
            }
            StmX::Assign { lhs, rhs } => {
                if let ExpX::Var(v) | ExpX::VarLoc(v) = &lhs.dest.x {
                    self.definitions.insert(v.clone(), Definition::Value(rhs.clone()));
                } else {
                    self.definitions.clear();
                }
                Ok(stm.clone())
            }
            StmX::Call { fun: sst::CallTarget::Fun(name), args, dest: Some(dest), .. } => {
                let callee = self
                    .callees
                    .get(name)
                    .filter(|f| f.x.attrs.infer && !f.x.attrs.infer_allow_approx);
                if let (Some(callee), ExpX::Var(v) | ExpX::VarLoc(v)) = (callee, &dest.dest.x) {
                    self.definitions
                        .insert(v.clone(), Definition::Call(callee.clone(), args.clone()));
                } else {
                    self.definitions.clear();
                }
                Ok(stm.clone())
            }
            StmX::If(c, a, b) => {
                let definitions = self.definitions.clone();
                let assumptions = self.assumptions.clone();
                self.assumptions.push(c.clone());
                let a = self.stm(a)?;
                self.definitions = definitions;
                self.assumptions = assumptions.clone();
                self.assumptions.push(c.new_x(ExpX::Unary(UnaryOp::Not, c.clone())));
                let b = b.as_ref().map(|s| self.stm(s)).transpose()?;
                self.definitions.clear();
                self.assumptions = assumptions;
                Ok(stm.new_x(StmX::If(c.clone(), a, b)))
            }
            // Do not propagate guessed state through loops, proof scopes,
            // mutation-capable calls, or other unsupported control flow.
            _ => {
                self.definitions.clear();
                Ok(stm.clone())
            }
        }
    }
}

/// First experimental strategy: fixed-width scalar assertions depending on an
/// exact inferred result, whose expanded goal contains bitwise &, |, or ^.
/// This decorates ordinary callers only; precision queries are never inputs.
pub fn prepare(
    check: &FuncCheckSst,
    callees: &HashMap<Fun, Function>,
) -> Result<FuncCheckSst, VirErr> {
    if !matches!(check.post_condition.kind, sst::PostConditionKind::Ensures) {
        return Err(error(&check.body.span, "Verus Infer goals: expected an ordinary proof body"));
    }
    let mut requests = Requests {
        callees,
        definitions: HashMap::new(),
        assumptions: check.reqs.iter().take(32).cloned().collect(),
        remaining: REQUESTS,
        visits: VISITS,
    };
    let mut result = check.clone();
    result.body = requests.stm(&check.body)?;
    Ok(result)
}
