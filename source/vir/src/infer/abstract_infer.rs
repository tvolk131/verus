//! Untrusted, bounded interval candidate generation. Every safety and range
//! claim is proved against the unchanged executable by ordinary verification.
use super::*;
use num_traits::{Signed, Zero};

mod safety;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Range {
    Unit,
    IntTop,
    Int(BigInt, BigInt),
    Bool(Option<bool>),
}
impl Range {
    fn full(typ: &Typ, word_bits: u32) -> Result<Self, VirErr> {
        match &**typ {
            TypX::Bool => Ok(Self::Bool(None)),
            TypX::Int(IntRange::Int) => Ok(Self::IntTop),
            TypX::Int(range) if arithmetic::bounds_at(*range, word_bits).is_some() => {
                let (min, max) = arithmetic::bounds_at(*range, word_bits).unwrap();
                Ok(Self::Int(min, max))
            }
            _ => Err(unsupported(
                &Span {
                    raw_span: Arc::new(()),
                    id: 0,
                    data: vec![],
                    as_string: "interval type".into(),
                },
                "interval analysis requires scalar types",
            )),
        }
    }
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unit, Self::Unit) => Self::Unit,
            (Self::IntTop, Self::Int(..) | Self::IntTop) | (Self::Int(..), Self::IntTop) => {
                Self::IntTop
            }
            (Self::Int(a, b), Self::Int(c, d)) => Self::Int(a.min(c), b.max(d)),
            (Self::Bool(a), Self::Bool(b)) => Self::Bool(if a == b { a } else { None }),
            _ => unreachable!("typed scalar branches"),
        }
    }
    fn restrict(self, typ: &Typ, word_bits: u32) -> Self {
        if let TypX::Int(range) = &**typ
            && arithmetic::bounds_at(*range, word_bits).is_some()
        {
            if self == Self::IntTop {
                return Self::full(typ, word_bits).unwrap();
            }
            let Self::Int(min, max) = Self::full(typ, word_bits).unwrap() else { unreachable!() };
            if let Self::Int(a, b) = self {
                let (a, b) = (a.max(min.clone()), b.min(max.clone()));
                // No successful execution in this abstract interval; widening
                // keeps the candidate conservative. Soundness must still prove
                // executable safety under the proposed precondition.
                return if a <= b { Self::Int(a, b) } else { Self::Int(min, max) };
            }
        }
        self
    }
}

// Split away zero and retain the closest negative/positive denominators.
// Taking only the original endpoints can miss the extrema near -1 and 1.
fn divide(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt, truncating: bool) -> Range {
    let mut divisors = Vec::new();
    if c < &BigInt::zero() {
        divisors.extend([c.clone(), d.clone().min(-BigInt::one())]);
    }
    if d > &BigInt::zero() {
        divisors.extend([c.clone().max(BigInt::one()), d.clone()]);
    }
    let mut values = Vec::new();
    for x in [a, b] {
        for y in &divisors {
            let mut q = x / y;
            if !truncating && x % y < BigInt::zero() {
                if y > &BigInt::zero() {
                    q -= 1;
                } else {
                    q += 1;
                }
            }
            values.push(q);
        }
    }
    match (values.iter().min(), values.iter().max()) {
        (Some(min), Some(max)) => Range::Int(min.clone(), max.clone()),
        // No nonzero divisor. Safety is checked separately; remain conservative.
        _ => Range::IntTop,
    }
}

fn remainder(a: &BigInt, b: &BigInt, c: &BigInt, d: &BigInt, truncating: bool) -> Range {
    let magnitude = c.abs().max(d.abs());
    if magnitude.is_zero() {
        return Range::IntTop;
    }
    let limit = magnitude - BigInt::one();
    if truncating {
        Range::Int(
            a.clone().min(BigInt::zero()).max(-limit.clone()),
            b.clone().max(BigInt::zero()).min(limit),
        )
    } else {
        let max = if a >= &BigInt::zero() { b.clone().min(limit) } else { limit };
        Range::Int(BigInt::zero(), max)
    }
}

struct Interpreter<'a> {
    locals: HashMap<VarIdent, Range>,
    callees: &'a HashMap<Fun, Function>,
    remaining: usize,
    word_bits: u32,
}
impl Interpreter<'_> {
    fn spend(&mut self, span: &Span) -> Result<(), VirErr> {
        if self.remaining == 0 {
            return Err(error(span, "Verus Infer incomplete: interval analysis budget exceeded"));
        }
        self.remaining -= 1;
        Ok(())
    }
    fn place(&mut self, place: &Place) -> Result<Range, VirErr> {
        self.spend(&place.span)?;
        match &place.x {
            PlaceX::Local(name) => self
                .locals
                .get(name)
                .cloned()
                .ok_or_else(|| unsupported(&place.span, "unknown interval local")),
            PlaceX::Temporary(expr) => self.expr(expr),
            _ => Err(unsupported(&place.span, "nonscalar interval place")),
        }
    }
    fn expr(&mut self, e: &Expr) -> Result<Range, VirErr> {
        self.spend(&e.span)?;
        let word_bits = self.word_bits;
        let unknown = || Range::full(&e.typ, word_bits);
        if let Some(n) = arithmetic::negative_literal(e) {
            return Ok(Range::Int(n.clone(), n));
        }
        if let Some(n) = arithmetic::arch_value(e, self.word_bits) {
            return Ok(Range::Int(n.clone(), n));
        }
        let value = match &e.x {
            ExprX::Const(Constant::Int(n)) => Range::Int(n.clone(), n.clone()),
            ExprX::Const(Constant::Bool(b)) => Range::Bool(Some(*b)),
            ExprX::Var(v) => self
                .locals
                .get(v)
                .cloned()
                .ok_or_else(|| unsupported(&e.span, "unknown interval variable"))?,
            ExprX::ReadPlace(p, _) => self.place(p)?,
            ExprX::NeverToAny(inner) => self.expr(inner)?,
            ExprX::Unary(UnaryOp::BitNot(_), a) => {
                self.expr(a)?;
                unknown()?
            }
            ExprX::Unary(UnaryOp::Not, a) => match self.expr(a)? {
                Range::Bool(b) => Range::Bool(b.map(|b| !b)),
                _ => return Err(unsupported(&e.span, "nonboolean interval negation")),
            },
            ExprX::Unary(UnaryOp::Clip { range, .. }, a) => {
                let value = self.expr(a)?;
                let Some((min, max)) = arithmetic::bounds_at(*range, self.word_bits) else {
                    return unknown();
                };
                match value {
                    Range::Int(a, b) if a >= min && b <= max => Range::Int(a, b),
                    // A wrapping cast can cross the destination's signed boundary.
                    // Intersecting the input would drop valid wrapped results.
                    _ => Range::Int(min, max),
                }
            }
            ExprX::Logical(op, a, b) => {
                let Range::Bool(a) = self.expr(a)? else {
                    return unknown();
                };
                let Range::Bool(b) = self.expr(b)? else {
                    return unknown();
                };
                Range::Bool(match op {
                    LogicalOp::And => {
                        if a == Some(false) || b == Some(false) {
                            Some(false)
                        } else if a == Some(true) && b == Some(true) {
                            Some(true)
                        } else {
                            None
                        }
                    }
                    LogicalOp::Or => {
                        if a == Some(true) || b == Some(true) {
                            Some(true)
                        } else if a == Some(false) && b == Some(false) {
                            Some(false)
                        } else {
                            None
                        }
                    }
                    LogicalOp::Implies => {
                        if a == Some(false) || b == Some(true) {
                            Some(true)
                        } else if a == Some(true) && b == Some(false) {
                            Some(false)
                        } else {
                            None
                        }
                    }
                })
            }
            ExprX::Binary(op, a, b) => {
                let a = self.expr(a)?;
                let b = self.expr(b)?;
                match (a, b) {
                    (Range::Int(a, b), Range::Int(c, d)) => match op {
                        BinaryOp::Arith(ArithOp::Add(_)) => Range::Int(a + c, b + d),
                        BinaryOp::Arith(ArithOp::Sub(_)) => Range::Int(a - d, b - c),
                        BinaryOp::Arith(ArithOp::Mul(_)) => {
                            let products = [&a * &c, &a * &d, &b * &c, &b * &d];
                            Range::Int(
                                products.iter().min().unwrap().clone(),
                                products.iter().max().unwrap().clone(),
                            )
                        }
                        BinaryOp::Arith(ArithOp::TruncatingDiv(..)) => divide(&a, &b, &c, &d, true),
                        BinaryOp::Arith(ArithOp::EuclideanDiv(_)) => divide(&a, &b, &c, &d, false),
                        BinaryOp::Arith(ArithOp::TruncatingMod(..)) => {
                            remainder(&a, &b, &c, &d, true)
                        }
                        BinaryOp::Arith(ArithOp::EuclideanMod(_)) => {
                            remainder(&a, &b, &c, &d, false)
                        }
                        BinaryOp::Eq(_) => Range::Bool(if b < c || d < a {
                            Some(false)
                        } else if a == b && c == d && a == c {
                            Some(true)
                        } else {
                            None
                        }),
                        BinaryOp::Ne => Range::Bool(if b < c || d < a {
                            Some(true)
                        } else if a == b && c == d && a == c {
                            Some(false)
                        } else {
                            None
                        }),
                        BinaryOp::Inequality(op) => Range::Bool(match op {
                            InequalityOp::Lt => {
                                if b < c {
                                    Some(true)
                                } else if a >= d {
                                    Some(false)
                                } else {
                                    None
                                }
                            }
                            InequalityOp::Le => {
                                if b <= c {
                                    Some(true)
                                } else if a > d {
                                    Some(false)
                                } else {
                                    None
                                }
                            }
                            InequalityOp::Gt => {
                                if a > d {
                                    Some(true)
                                } else if b <= c {
                                    Some(false)
                                } else {
                                    None
                                }
                            }
                            InequalityOp::Ge => {
                                if a >= d {
                                    Some(true)
                                } else if b < c {
                                    Some(false)
                                } else {
                                    None
                                }
                            }
                        }),
                        _ => return unknown(),
                    },
                    (Range::Bool(a), Range::Bool(b)) => Range::Bool(match (a, b) {
                        (Some(a), Some(b)) => match op {
                            BinaryOp::Eq(_) => Some(a == b),
                            BinaryOp::Ne | BinaryOp::BoolXor => Some(a != b),
                            BinaryOp::BoolAndNoSC => Some(a && b),
                            BinaryOp::BoolOrNoSC => Some(a || b),
                            _ => None,
                        },
                        _ => None,
                    }),
                    _ => return unknown(),
                }
            }
            ExprX::If(c, a, None) => {
                self.expr(c)?;
                self.expr(a)?;
                Range::Unit
            }
            ExprX::If(c, a, Some(b)) => match self.expr(c)? {
                Range::Bool(Some(true)) => self.expr(a)?,
                Range::Bool(Some(false)) => self.expr(b)?,
                _ => self.expr(a)?.join(self.expr(b)?),
            },
            ExprX::Block(stmts, tail) => {
                let saved = self.locals.clone();
                for stmt in stmts.iter() {
                    self.spend(&stmt.span)?;
                    match &stmt.x {
                        StmtX::Expr(e) => {
                            self.expr(e)?;
                        }
                        StmtX::Decl { pattern, init: Some(init), .. } => {
                            let PatternX::Var(binding) = &pattern.x else {
                                return Err(unsupported(&stmt.span, "interval destructuring"));
                            };
                            let value = self.place(init)?;
                            self.locals.insert(binding.name.clone(), value);
                        }
                        _ => return Err(unsupported(&stmt.span, "interval statement")),
                    }
                }
                let value = match tail {
                    Some(tail) => self.expr(tail)?,
                    None => Range::Unit,
                };
                self.locals = saved;
                value
            }
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                args,
                post_args: None,
                body: None,
            } => {
                let callee = self
                    .callees
                    .get(name)
                    .ok_or_else(|| unsupported(&e.span, "missing inferred interval callee"))?
                    .clone();
                if super::is_checked_arithmetic(&callee) {
                    return Err(unsupported(
                        &e.span,
                        "enum expressions currently require exact inference",
                    ));
                }
                let mut locals = HashMap::new();
                for (param, arg) in callee.x.params.iter().zip(args.iter()) {
                    locals.insert(param.x.name.clone(), self.expr(arg)?);
                }
                if let Some(model) = integer_models::recognize(&callee) {
                    if model.policy == integer_models::Policy::Checked {
                        return Err(unsupported(
                            &e.span,
                            "enum expressions currently require exact inference",
                        ));
                    }
                    let terms = callee
                        .x
                        .params
                        .iter()
                        .map(|p| Term {
                            view: None,
                            expr: SpannedTyped::new(
                                &p.span,
                                &p.x.typ,
                                ExprX::Var(p.x.name.clone()),
                            ),
                            size: 1,
                        })
                        .collect();
                    let formula = integer_models::value(&e.span, &callee, terms)?;
                    let mut child = Interpreter {
                        locals,
                        callees: self.callees,
                        remaining: self.remaining,
                        word_bits: self.word_bits,
                    };
                    let result = child.expr(&formula.expr)?;
                    self.remaining = child.remaining;
                    return Ok(result.restrict(&e.typ, self.word_bits));
                }
                if callee.x.attrs.infer_allow_approx {
                    range_from_summary(&callee, self.word_bits)?
                } else {
                    let ExprX::Binary(BinaryOp::Eq(_), _, output) = &callee.x.ensure.0[0].x else {
                        return Err(unsupported(&e.span, "invalid exact callee summary"));
                    };
                    let mut child = Interpreter {
                        locals,
                        callees: self.callees,
                        remaining: self.remaining,
                        word_bits: self.word_bits,
                    };
                    let range =
                        child.expr(output)?.restrict(&callee.x.inner_ret.x.typ, self.word_bits);
                    self.remaining = child.remaining;
                    range
                }
            }
            _ => return Err(unsupported(&e.span, "expression outside interval subset")),
        };
        Ok(value.restrict(&e.typ, self.word_bits))
    }
}

fn number(span: &Span, value: BigInt) -> Expr {
    SpannedTyped::new(span, &int_typ(), ExprX::Const(Constant::Int(value)))
}
fn relation(span: &Span, op: BinaryOp, a: Expr, b: Expr) -> Expr {
    SpannedTyped::new(span, &bool_typ(), ExprX::Binary(op, a, b))
}
fn postcondition(function: &Function, range: Range) -> Expr {
    let span = &function.span;
    let ret = SpannedTyped::new(
        span,
        &function.x.inner_ret.x.typ,
        ExprX::Var(function.x.inner_ret.x.name.clone()),
    );
    match range {
        Range::Unit | Range::IntTop => unreachable!("typed scalar return"),
        Range::Bool(None) => boolean(span, true).expr,
        Range::Bool(Some(b)) => {
            relation(span, BinaryOp::Eq(Mode::Spec), ret, boolean(span, b).expr)
        }
        Range::Int(a, b) if a == b => {
            relation(span, BinaryOp::Eq(Mode::Spec), ret, number(span, a))
        }
        Range::Int(a, b) => SpannedTyped::new(
            span,
            &bool_typ(),
            ExprX::Logical(
                LogicalOp::And,
                relation(
                    span,
                    BinaryOp::Inequality(InequalityOp::Le),
                    number(span, a),
                    ret.clone(),
                ),
                relation(span, BinaryOp::Inequality(InequalityOp::Le), ret, number(span, b)),
            ),
        ),
    }
}

pub(super) fn infer(
    function: &Function,
    callees: &HashMap<Fun, Function>,
) -> Result<Contract, VirErr> {
    // Keep width-specific intervals and safety domains. Merge only pure, typed
    // formulas, with the usual expanded-size budget, then let Verus prove them.
    let a = infer_on_arch(function, callees, 32)?;
    let b = infer_on_arch(function, callees, 64)?;
    let span = &function.span;
    let selector = summary::binary(
        span,
        BinaryOp::Eq(Mode::Spec),
        arithmetic::arch_bits(span),
        term(span, &int_typ(), ExprX::Const(Constant::Int(32.into())), 1)?,
    )?;
    let vars = function
        .x
        .params
        .iter()
        .chain(std::iter::once(&function.x.inner_ret))
        .map(|p| {
            (
                p.x.name.clone(),
                Term {
                    view: None,
                    expr: SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
                    size: 1,
                },
            )
        })
        .collect();
    let merge = |a: &Expr, b: &Expr| -> Result<Expr, VirErr> {
        let a_term = instantiate(a, &vars)?;
        if format!("{a:?}") == format!("{b:?}") {
            return Ok(a_term.expr);
        }
        Ok(summary::conditional(span, selector.clone(), a_term, instantiate(b, &vars)?)?.expr)
    };
    Contract::new(merge(&a.pre, &b.pre)?, merge(&a.post, &b.post)?)
}

fn infer_on_arch(
    function: &Function,
    callees: &HashMap<Fun, Function>,
    word_bits: u32,
) -> Result<Contract, VirErr> {
    validate_signature(function)?;
    let exact = if !uses_approximate_callee(function, callees) {
        infer_exact_with_callees(function, callees).ok()
    } else {
        None
    };
    let pre = match exact {
        Some(exact) => exact.pre,
        None => safety::infer(function, callees, word_bits)?.expr,
    };
    let locals = function
        .x
        .params
        .iter()
        .map(|p| Ok((p.x.name.clone(), Range::full(&p.x.typ, word_bits)?)))
        .collect::<Result<HashMap<_, _>, VirErr>>()?;
    let body = function
        .x
        .body
        .as_ref()
        .ok_or_else(|| unsupported(&function.span, "missing interval body"))?;
    let output = Interpreter { locals, callees, remaining: FORMULA_BUDGET, word_bits }
        .expr(body)?
        .restrict(&function.x.inner_ret.x.typ, word_bits);
    Contract::new(pre, postcondition(function, output))
}

pub(super) fn range_from_summary(function: &Function, word_bits: u32) -> Result<Range, VirErr> {
    let invalid = || unsupported(&function.span, "invalid sound interval summary");
    if function.x.ensure.0.len() != 1 || !function.x.ensure.1.is_empty() {
        return Err(invalid());
    }
    let ret = |e: &Expr| matches!(&e.x, ExprX::Var(v) if v == &function.x.inner_ret.x.name);
    let mut post = &function.x.ensure.0[0];
    if let ExprX::If(c, a, Some(b)) = &post.x {
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), left, right) = &c.x else {
            return Err(invalid());
        };
        if !matches!(
            &left.x,
            ExprX::UnaryOpr(UnaryOpr::IntegerTypeBound(IntegerTypeBoundKind::ArchWordBits), _)
        ) || arithmetic::arch_value(left, word_bits).is_none()
            || !matches!(&right.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::from(32))
        {
            return Err(invalid());
        }
        post = if word_bits == 32 { a } else { b };
    }
    match &post.x {
        ExprX::Const(Constant::Bool(true))
            if matches!(&*function.x.inner_ret.x.typ, TypX::Bool) =>
        {
            Ok(Range::Bool(None))
        }
        ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) if ret(a) => match &b.x {
            ExprX::Const(Constant::Bool(b))
                if matches!(&*function.x.inner_ret.x.typ, TypX::Bool) =>
            {
                Ok(Range::Bool(Some(*b)))
            }
            ExprX::Const(Constant::Int(n))
                if matches!(&*function.x.inner_ret.x.typ, TypX::Int(_)) =>
            {
                Ok(Range::Int(n.clone(), n.clone()))
            }
            _ => Err(invalid()),
        },
        ExprX::Logical(LogicalOp::And, a, b) => match (&a.x, &b.x) {
            (
                ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), lower, result),
                ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), result2, upper),
            ) if ret(result) && ret(result2) => match (&lower.x, &upper.x) {
                (ExprX::Const(Constant::Int(a)), ExprX::Const(Constant::Int(b)))
                    if a <= b && matches!(&*function.x.inner_ret.x.typ, TypX::Int(_)) =>
                {
                    Ok(Range::Int(a.clone(), b.clone()))
                }
                _ => Err(invalid()),
            },
            _ => Err(invalid()),
        },
        _ => Err(invalid()),
    }
}

pub(super) fn validate_summary(function: &Function) -> Result<(), VirErr> {
    let f = &function.x;
    if !f.attrs.infer || f.body.is_some() || f.require.len() != 1 {
        return Err(unsupported(&function.span, "invalid imported sound summary"));
    }
    let mut signature = f.clone();
    signature.require = Arc::new(vec![]);
    signature.ensure = (Arc::new(vec![]), Arc::new(vec![]));
    validate_signature(&function.new_x(signature))?;
    range_from_summary(function, 32)?;
    range_from_summary(function, 64)?;
    let params = f
        .params
        .iter()
        .map(|p| {
            (
                p.x.name.clone(),
                Term {
                    view: None,
                    expr: SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
                    size: 1,
                },
            )
        })
        .collect();
    let pre = instantiate(&f.require[0], &params)?;
    if !matches!(&*pre.expr.typ, TypX::Bool) {
        return Err(unsupported(&function.span, "nonboolean sound precondition"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
