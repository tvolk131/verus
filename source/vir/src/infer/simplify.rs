//! Bounded, untrusted simplification of pure candidate formulas. The installer
//! binds the resulting formulas into the candidate and all normal proofs rerun.
//! Executable safety operations have already contributed their requirements.
use super::*;
use num_traits::{Signed, Zero};

fn nodes(e: &Expr) -> usize {
    let mut n = 0;
    crate::ast_visitor::expr_visitor_walk(e, &mut |_| {
        n += 1;
        crate::ast_visitor::VisitorControlFlow::<()>::Recurse
    });
    n
}
fn same(a: &Expr, b: &Expr) -> bool {
    if Arc::ptr_eq(a, b) {
        return true;
    }
    if !crate::ast_util::types_equal(&a.typ, &b.typ) {
        return false;
    }
    match (&a.x, &b.x) {
        (ExprX::Var(a), ExprX::Var(b)) => a == b,
        (ExprX::Const(a), ExprX::Const(b)) => a == b,
        (ExprX::Unary(a, x), ExprX::Unary(b, y)) => a == b && same(x, y),
        (ExprX::UnaryOpr(a, x), ExprX::UnaryOpr(b, y)) => {
            format!("{a:?}") == format!("{b:?}") && same(x, y)
        }
        (ExprX::Binary(a, x, y), ExprX::Binary(b, z, w)) => a == b && same(x, z) && same(y, w),
        (ExprX::Logical(a, x, y), ExprX::Logical(b, z, w)) => a == b && same(x, z) && same(y, w),
        (ExprX::If(c, a, Some(b)), ExprX::If(d, x, Some(y))) => {
            same(c, d) && same(a, x) && same(b, y)
        }
        (ExprX::ArrayLiteral(a), ExprX::ArrayLiteral(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| same(a, b))
        }
        (ExprX::Ctor(dt, v, fs, None), ExprX::Ctor(et, w, gs, None)) => {
            dt == et
                && v == w
                && fs.len() == gs.len()
                && fs.iter().zip(gs.iter()).all(|(a, b)| a.name == b.name && same(&a.a, &b.a))
        }
        _ => false,
    }
}
fn integer(e: &Expr) -> Option<&BigInt> {
    if let ExprX::Const(Constant::Int(n)) = &e.x { Some(n) } else { None }
}

fn entails(a: &Expr, b: &Expr) -> bool {
    if same(a, b) {
        return true;
    }
    let (ExprX::Binary(left, x, y), ExprX::Binary(right, z, w)) = (&a.x, &b.x) else {
        return false;
    };
    let right = if same(x, z) && same(y, w) {
        *right
    } else if same(x, w) && same(y, z) {
        flip(*right)
    } else {
        return false;
    };
    matches!(
        (*left, right),
        (BinaryOp::Inequality(InequalityOp::Lt), BinaryOp::Inequality(InequalityOp::Le))
            | (BinaryOp::Inequality(InequalityOp::Gt), BinaryOp::Inequality(InequalityOp::Ge))
            | (BinaryOp::Eq(_), BinaryOp::Inequality(InequalityOp::Le | InequalityOp::Ge))
    )
}
fn number(span: &Span, n: BigInt) -> Result<Term, VirErr> {
    term(span, &int_typ(), ExprX::Const(Constant::Int(n)), 1)
}
fn bounds(typ: &Typ) -> Option<(BigInt, BigInt)> {
    let TypX::Int(range) = &**typ else { return None };
    let (a, b) = arithmetic::bounds_at(*range, 32)?;
    let (c, d) = arithmetic::bounds_at(*range, 64)?;
    Some((a.min(c), b.max(d)))
}
fn flip(op: BinaryOp) -> BinaryOp {
    use InequalityOp::*;
    match op {
        BinaryOp::Inequality(Lt) => BinaryOp::Inequality(Gt),
        BinaryOp::Inequality(Le) => BinaryOp::Inequality(Ge),
        BinaryOp::Inequality(Gt) => BinaryOp::Inequality(Lt),
        BinaryOp::Inequality(Ge) => BinaryOp::Inequality(Le),
        op => op,
    }
}
fn negate(op: BinaryOp) -> BinaryOp {
    use InequalityOp::*;
    match op {
        BinaryOp::Eq(_) => BinaryOp::Ne,
        BinaryOp::Ne => BinaryOp::Eq(Mode::Spec),
        BinaryOp::Inequality(Lt) => BinaryOp::Inequality(Ge),
        BinaryOp::Inequality(Le) => BinaryOp::Inequality(Gt),
        BinaryOp::Inequality(Gt) => BinaryOp::Inequality(Le),
        BinaryOp::Inequality(Ge) => BinaryOp::Inequality(Lt),
        op => op,
    }
}
fn predicate(e: &Expr) -> Option<(Expr, BinaryOp, BigInt)> {
    let ExprX::Binary(op @ (BinaryOp::Inequality(_) | BinaryOp::Eq(_) | BinaryOp::Ne), a, b) = &e.x
    else {
        return None;
    };
    if let Some(n) = integer(b) {
        Some((a.clone(), *op, n.clone()))
    } else {
        integer(a).map(|n| (b.clone(), flip(*op), n.clone()))
    }
}
fn envelope(e: &Expr, facts: &[(Expr, bool)], budget: &mut usize) -> Option<(BigInt, BigInt)> {
    *budget = budget.checked_sub(1)?;
    let value = match &e.x {
        ExprX::Const(Constant::Int(n)) => Some((n.clone(), n.clone())),
        _ if arithmetic::arch_constant(e).is_some() => {
            let a = arithmetic::arch_value(e, 32)?;
            let b = arithmetic::arch_value(e, 64)?;
            Some((a.clone().min(b.clone()), a.max(b)))
        }
        ExprX::Var(_) => bounds(&e.typ),
        ExprX::Unary(UnaryOp::Length(ArrayKind::Slice), _) => {
            Some((0.into(), arithmetic::bounds_at(IntRange::USize, 64)?.1))
        }
        ExprX::Unary(UnaryOp::Clip { range, .. }, a) => {
            let (lo, hi) = bounds(&Arc::new(TypX::Int(*range)))?;
            let (min32, max32) = arithmetic::bounds_at(*range, 32)?;
            let (min64, max64) = arithmetic::bounds_at(*range, 64)?;
            match envelope(a, facts, budget) {
                Some((a, b)) if a >= min32.max(min64) && b <= max32.min(max64) => Some((a, b)),
                _ => Some((lo, hi)),
            }
        }
        ExprX::Binary(BinaryOp::Arith(op), a, b) => {
            let (a, b) = (envelope(a, facts, budget)?, envelope(b, facts, budget)?);
            match op {
                ArithOp::Add(OverflowBehavior::Allow) => Some((a.0 + b.0, a.1 + b.1)),
                ArithOp::Sub(OverflowBehavior::Allow) => Some((a.0 - b.1, a.1 - b.0)),
                ArithOp::Mul(OverflowBehavior::Allow) => {
                    let xs = [
                        a.0.clone() * b.0.clone(),
                        a.0 * b.1.clone(),
                        a.1.clone() * b.0,
                        a.1 * b.1,
                    ];
                    Some((xs.iter().min()?.clone(), xs.iter().max()?.clone()))
                }
                _ => None,
            }
        }
        ExprX::If(_, a, Some(b)) => {
            let a = envelope(a, facts, budget)?;
            let b = envelope(b, facts, budget)?;
            Some((a.0.min(b.0), a.1.max(b.1)))
        }
        _ => None,
    };
    let (mut lo, mut hi) = value?;
    for (fact, truth) in facts {
        if let Some((value, op, n)) = predicate(fact) {
            if same(&value, e) {
                match if *truth { op } else { negate(op) } {
                    BinaryOp::Inequality(InequalityOp::Le) => hi = hi.min(n),
                    BinaryOp::Inequality(InequalityOp::Lt) => hi = hi.min(n - 1),
                    BinaryOp::Inequality(InequalityOp::Ge) => lo = lo.max(n),
                    BinaryOp::Inequality(InequalityOp::Gt) => lo = lo.max(n + 1),
                    BinaryOp::Eq(_) => {
                        lo = lo.max(n.clone());
                        hi = hi.min(n);
                    }
                    _ => {}
                }
            }
        }
    }
    (lo <= hi && lo.bits() <= 4096 && hi.bits() <= 4096).then_some((lo, hi))
}

struct Linear {
    constant: BigInt,
    atoms: Vec<(Expr, BigInt)>,
    leaves: usize,
}
impl Linear {
    fn collect(&mut self, e: &Expr, factor: BigInt, budget: &mut usize) -> bool {
        let Some(next) = budget.checked_sub(1) else { return false };
        *budget = next;
        if let Some(n) = integer(e) {
            self.constant += factor * n;
            return self.constant.bits() <= 4096;
        }
        if let ExprX::Binary(BinaryOp::Arith(op), a, b) = &e.x {
            match op {
                ArithOp::Add(OverflowBehavior::Allow) => {
                    return self.collect(a, factor.clone(), budget)
                        && self.collect(b, factor, budget);
                }
                ArithOp::Sub(OverflowBehavior::Allow) => {
                    return self.collect(a, factor.clone(), budget)
                        && self.collect(b, -factor, budget);
                }
                ArithOp::Mul(OverflowBehavior::Allow) => {
                    if let Some(n) = integer(a) {
                        return self.collect(b, factor * n, budget);
                    }
                    if let Some(n) = integer(b) {
                        return self.collect(a, factor * n, budget);
                    }
                }
                _ => {}
            }
        }
        if !eligibility::integer(&e.typ) || factor.bits() > 4096 {
            return false;
        }
        self.leaves += 1;
        if let Some((_, c)) = self.atoms.iter_mut().find(|(a, _)| same(a, e)) {
            *c += factor;
            c.bits() <= 4096
        } else if self.atoms.len() < 32 {
            self.atoms.push((e.clone(), factor));
            true
        } else {
            false
        }
    }
    fn of(e: &Expr) -> Option<Self> {
        let mut value = Self { constant: 0.into(), atoms: vec![], leaves: 0 };
        if value.collect(e, 1.into(), &mut FORMULA_BUDGET.clone()) {
            value.atoms.retain(|(_, c)| !c.is_zero());
            Some(value)
        } else {
            None
        }
    }
    fn term(&self, span: &Span) -> Result<Term, VirErr> {
        let mut out = None;
        for (expr, c) in &self.atoms {
            let value = Term { view: None, expr: expr.clone(), size: nodes(expr) };
            let value = if c == &BigInt::one() {
                value
            } else {
                summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Allow)),
                    number(span, c.clone())?,
                    value,
                )?
            };
            out = Some(match out {
                None => value,
                Some(a) => summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
                    a,
                    value,
                )?,
            });
        }
        match out {
            None => number(span, self.constant.clone()),
            Some(a) if self.constant.is_zero() => Ok(a),
            Some(a) => summary::binary(
                span,
                BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
                a,
                number(span, self.constant.clone())?,
            ),
        }
    }
}
fn floor(n: &BigInt, d: &BigInt) -> BigInt {
    let q = n / d;
    if n % d < BigInt::zero() { q - 1 } else { q }
}

struct Simplifier {
    remaining: usize,
}
impl Simplifier {
    fn go(&mut self, e: &Expr, facts: &[(Expr, bool)]) -> Result<Term, VirErr> {
        self.remaining = self.remaining.checked_sub(1).ok_or_else(|| {
            error(&e.span, "Verus Infer incomplete: simplification budget exceeded")
        })?;
        let span = &e.span;
        if matches!(&*e.typ, TypX::Bool) {
            if let Some((_, truth)) = facts.iter().rev().find(|(f, _)| same(f, e)) {
                return Ok(boolean(span, *truth));
            }
        }
        let out = match &e.x {
            _ if arithmetic::arch_constant(e).is_some() => arithmetic::arch_constant(e).unwrap(),
            ExprX::Const(_) | ExprX::Var(_) => Term { view: None, expr: e.clone(), size: 1 },
            ExprX::Unary(UnaryOp::Not, a) => {
                let a = self.go(a, facts)?;
                if let ExprX::Const(Constant::Bool(b)) = a.expr.x {
                    boolean(span, !b)
                } else if let ExprX::Unary(UnaryOp::Not, b) = &a.expr.x {
                    self.go(b, facts)?
                } else {
                    summary::unary(span, UnaryOp::Not, a)?
                }
            }
            ExprX::Unary(op, a) => summary::unary(span, *op, self.go(a, facts)?)?,
            ExprX::Binary(op, a, b) => {
                let a = self.go(a, facts)?;
                let b = self.go(b, facts)?;
                self.binary(span, *op, a, b, facts)?
            }
            ExprX::Logical(op, a, b) => {
                let a = self.go(a, facts)?;
                let mut local = facts.to_vec();
                local.push((a.expr.clone(), *op != LogicalOp::Or));
                let b = self.go(b, &local)?;
                match op {
                    LogicalOp::And if entails(&a.expr, &b.expr) => a,
                    LogicalOp::And if entails(&b.expr, &a.expr) => b,
                    LogicalOp::Or if entails(&a.expr, &b.expr) => b,
                    LogicalOp::Or if entails(&b.expr, &a.expr) => a,
                    LogicalOp::And | LogicalOp::Or => summary::logical(span, *op, a, b)?,
                    LogicalOp::Implies if is_bool(&a, false) || is_bool(&b, true) => {
                        boolean(span, true)
                    }
                    LogicalOp::Implies if is_bool(&a, true) => b,
                    LogicalOp::Implies if is_bool(&b, false) => {
                        summary::unary(span, UnaryOp::Not, a)?
                    }
                    _ => summary::logical(span, *op, a, b)?,
                }
            }
            ExprX::If(c, a, Some(b)) => {
                let c = self.go(c, facts)?;
                let mut yes = facts.to_vec();
                yes.push((c.expr.clone(), true));
                let mut no = facts.to_vec();
                no.push((c.expr.clone(), false));
                let a = self.go(a, &yes)?;
                let b = self.go(b, &no)?;
                if same(&a.expr, &b.expr) {
                    a
                } else if is_bool(&a, true) && is_bool(&b, false) {
                    c
                } else if is_bool(&a, false) && is_bool(&b, true) {
                    summary::unary(span, UnaryOp::Not, c)?
                } else {
                    summary::conditional(span, c, a, b)?
                }
            }
            ExprX::ArrayLiteral(xs) => arrays::literal(
                span,
                &e.typ,
                xs.iter().map(|x| self.go(x, facts)).collect::<Result<Vec<_>, _>>()?,
            )?,
            ExprX::Ctor(_, variant, fields, None) => enums::ctor(
                span,
                &e.typ,
                variant,
                fields.first().map(|f| self.go(&f.a, facts)).transpose()?,
            )?,
            ExprX::UnaryOpr(UnaryOpr::IsVariant { variant, .. }, a) => {
                enums::is_variant(span, self.go(a, facts)?, variant)?
            }
            ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
                enums::field(span, self.go(a, facts)?, &op.variant)?
            }
            _ => return Err(unsupported(span, "expression outside pure simplification language")),
        };
        Ok(out)
    }
    fn binary(
        &mut self,
        span: &Span,
        op: BinaryOp,
        a: Term,
        b: Term,
        facts: &[(Expr, bool)],
    ) -> Result<Term, VirErr> {
        let raw = summary::binary(span, op, a.clone(), b.clone())?;
        if matches!(
            op,
            BinaryOp::Arith(
                ArithOp::Add(OverflowBehavior::Allow)
                    | ArithOp::Sub(OverflowBehavior::Allow)
                    | ArithOp::Mul(OverflowBehavior::Allow)
            )
        ) {
            if let Some(linear) = Linear::of(&raw.expr) {
                let out = linear.term(span)?;
                if out.size < raw.size || out.size == raw.size && linear.leaves > linear.atoms.len()
                {
                    return Ok(out);
                }
            }
        }
        if matches!(op, BinaryOp::Eq(_) | BinaryOp::Ne | BinaryOp::Inequality(_)) {
            if same(&a.expr, &b.expr) {
                return Ok(boolean(
                    span,
                    matches!(
                        op,
                        BinaryOp::Eq(_) | BinaryOp::Inequality(InequalityOp::Le | InequalityOp::Ge)
                    ),
                ));
            }
            if let (Some((lo, hi)), Some((min, max))) =
                (envelope(&a.expr, facts, &mut 512), envelope(&b.expr, facts, &mut 512))
            {
                let yes = match op {
                    BinaryOp::Eq(_) if hi < min || max < lo => Some(false),
                    BinaryOp::Ne if hi < min || max < lo => Some(true),
                    BinaryOp::Eq(_) if lo == hi && min == max => Some(lo == min),
                    BinaryOp::Ne if lo == hi && min == max => Some(lo != min),
                    BinaryOp::Inequality(InequalityOp::Le) if hi <= min => Some(true),
                    BinaryOp::Inequality(InequalityOp::Le) if lo > max => Some(false),
                    BinaryOp::Inequality(InequalityOp::Lt) if hi < min => Some(true),
                    BinaryOp::Inequality(InequalityOp::Lt) if lo >= max => Some(false),
                    BinaryOp::Inequality(InequalityOp::Ge) if lo >= max => Some(true),
                    BinaryOp::Inequality(InequalityOp::Ge) if hi < min => Some(false),
                    BinaryOp::Inequality(InequalityOp::Gt) if lo > max => Some(true),
                    BinaryOp::Inequality(InequalityOp::Gt) if hi <= min => Some(false),
                    _ => None,
                };
                if let Some(yes) = yes {
                    return Ok(boolean(span, yes));
                }
            }
            if eligibility::integer(&a.expr.typ) && eligibility::integer(&b.expr.typ) {
                let diff = summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                    a,
                    b,
                )?;
                if let Some(mut linear) = Linear::of(&diff.expr) {
                    if linear.atoms.len() == 1 {
                        let (atom, mut c) = linear.atoms.pop().unwrap();
                        let mut op = op;
                        if c.is_negative() {
                            c = -c;
                            linear.constant = -linear.constant;
                            op = flip(op);
                        }
                        let n = -linear.constant;
                        let q = floor(&n, &c);
                        let divisible = (&n % &c).is_zero();
                        let (op, n) = match op {
                            BinaryOp::Inequality(InequalityOp::Le) => (op, q),
                            BinaryOp::Inequality(InequalityOp::Lt) => {
                                (BinaryOp::Inequality(InequalityOp::Le), floor(&(n - 1), &c))
                            }
                            BinaryOp::Inequality(InequalityOp::Ge) => {
                                (op, if divisible { q } else { q + 1 })
                            }
                            BinaryOp::Inequality(InequalityOp::Gt) => {
                                (BinaryOp::Inequality(InequalityOp::Ge), q + 1)
                            }
                            BinaryOp::Eq(_) | BinaryOp::Ne if !divisible => {
                                return Ok(boolean(span, op == BinaryOp::Ne));
                            }
                            _ => (op, q),
                        };
                        let atom = Term { view: None, size: nodes(&atom), expr: atom };
                        let value = summary::binary(span, op, atom, number(span, n)?)?;
                        if value.size <= raw.size {
                            return Ok(value);
                        }
                    } else if linear.atoms.len() == 2 && linear.constant.is_zero() {
                        let positive = linear.atoms.iter().find(|(_, c)| c == &BigInt::one());
                        let negative = linear.atoms.iter().find(|(_, c)| c == &-BigInt::one());
                        if let (Some((a, _)), Some((b, _))) = (positive, negative) {
                            let value = summary::binary(
                                span,
                                op,
                                Term { view: None, expr: a.clone(), size: nodes(a) },
                                Term { view: None, expr: b.clone(), size: nodes(b) },
                            )?;
                            if value.size < raw.size {
                                return Ok(value);
                            }
                        }
                    } else if linear.atoms.is_empty() {
                        let n = linear.constant;
                        return Ok(boolean(
                            span,
                            match op {
                                BinaryOp::Eq(_) => n.is_zero(),
                                BinaryOp::Ne => !n.is_zero(),
                                BinaryOp::Inequality(InequalityOp::Lt) => n < BigInt::zero(),
                                BinaryOp::Inequality(InequalityOp::Le) => n <= BigInt::zero(),
                                BinaryOp::Inequality(InequalityOp::Gt) => n > BigInt::zero(),
                                _ => n >= BigInt::zero(),
                            },
                        ));
                    }
                }
            }
        }
        Ok(raw)
    }
}

pub(super) fn contract(f: &Function, pre: &Expr, post: &Expr) -> Result<(Expr, Expr), VirErr> {
    let vars =
        f.x.params
            .iter()
            .chain(std::iter::once(&f.x.inner_ret))
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
    let pre = instantiate(pre, &vars)?;
    let pre = Simplifier { remaining: FORMULA_BUDGET }.go(&pre.expr, &[])?.expr;
    // Preserve the serialized canonical scalar range shape for sound summaries.
    let post = if f.x.attrs.infer_allow_approx {
        post.clone()
    } else {
        let checked = instantiate(post, &vars)?;
        if let ExprX::Binary(op @ BinaryOp::Eq(_), ret, output) = &checked.expr.x {
            let output = Simplifier { remaining: FORMULA_BUDGET }.go(output, &[])?;
            summary::binary(&f.span, *op, Term { view: None, expr: ret.clone(), size: 1 }, output)?
                .expr
        } else {
            Simplifier { remaining: FORMULA_BUDGET }.go(&checked.expr, &[])?.expr
        }
    };
    Ok((pre, post))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{Value, evaluate_integer, function, input, span};
    fn n(value: i32) -> Expr {
        SpannedTyped::new(&span(), &int_typ(), ExprX::Const(Constant::Int(value.into())))
    }
    fn math(op: ArithOp, a: Expr, b: Expr) -> Expr {
        SpannedTyped::new(&span(), &int_typ(), ExprX::Binary(BinaryOp::Arith(op), a, b))
    }
    fn cmp(op: BinaryOp, a: Expr, b: Expr) -> Expr {
        SpannedTyped::new(&span(), &bool_typ(), ExprX::Binary(op, a, b))
    }
    #[test]
    fn simplified_predicates_and_values_match_exhaustive_integer_semantics() {
        use InequalityOp::*;
        let add = ArithOp::Add(OverflowBehavior::Allow);
        let sub = ArithOp::Sub(OverflowBehavior::Allow);
        let mul = ArithOp::Mul(OverflowBehavior::Allow);
        let x = input("x");
        let y = input("y");
        let f = function("simplify", x.clone());
        let sum = math(add, x.clone(), y.clone());
        let bound = cmp(BinaryOp::Inequality(Le), sum.clone(), n(255));
        let predicates = [
            SpannedTyped::new(
                &span(),
                &bool_typ(),
                ExprX::Logical(
                    LogicalOp::And,
                    cmp(BinaryOp::Inequality(Le), n(0), x.clone()),
                    SpannedTyped::new(
                        &span(),
                        &bool_typ(),
                        ExprX::Logical(LogicalOp::And, bound.clone(), bound),
                    ),
                ),
            ),
            cmp(BinaryOp::Inequality(Le), math(add, x.clone(), x.clone()), n(255)),
            cmp(BinaryOp::Inequality(Lt), math(add, math(mul, n(3), x.clone()), n(2)), n(128)),
            cmp(BinaryOp::Inequality(Ge), math(sub, math(mul, n(-3), x.clone()), n(2)), n(-128)),
            cmp(BinaryOp::Eq(Mode::Spec), math(mul, n(2), x.clone()), n(1)),
            cmp(BinaryOp::Ne, math(mul, n(2), x.clone()), n(1)),
        ];
        let output = math(add, math(sub, x.clone(), y.clone()), math(add, x, y));
        let ret = SpannedTyped::new(
            &span(),
            &f.x.inner_ret.x.typ,
            ExprX::Var(f.x.inner_ret.x.name.clone()),
        );
        let post = cmp(BinaryOp::Eq(Mode::Spec), ret, output.clone());
        for pre in predicates {
            let (simple, post) = contract(&f, &pre, &post).unwrap();
            let ExprX::Binary(BinaryOp::Eq(_), _, value) = &post.x else { panic!() };
            for x in 0..=255 {
                for y in 0..=255 {
                    assert_eq!(evaluate_integer(&pre, x, y), evaluate_integer(&simple, x, y));
                    assert_eq!(evaluate_integer(&output, x, y), evaluate_integer(value, x, y));
                }
            }
            assert!(nodes(&simple) <= nodes(&pre));
        }
        let (simple, _) = contract(
            &f,
            &cmp(BinaryOp::Inequality(Le), math(add, input("x"), input("x")), n(255)),
            &post,
        )
        .unwrap();
        assert_eq!(format_expr(&simple), "(x <= 127)");
    }
    #[test]
    fn simplification_does_not_remove_prior_executable_safety() {
        let body = SpannedTyped::new(
            &span(),
            &input("x").typ,
            ExprX::Binary(
                BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8)))),
                input("x"),
                input("x"),
            ),
        );
        let original = function("cancel", body);
        let inferred = infer_function(&original).unwrap();
        assert!(Arc::ptr_eq(original.x.body.as_ref().unwrap(), inferred.x.body.as_ref().unwrap()));
        assert_eq!(format_contract(&inferred), "requires true; ensures result == 0");
        let source = SpannedTyped::new(
            &span(),
            &input("x").typ,
            ExprX::Binary(
                BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(IntRange::U(8)))),
                SpannedTyped::new(
                    &span(),
                    &input("x").typ,
                    ExprX::Binary(
                        BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(8)))),
                        input("x"),
                        SpannedTyped::new(
                            &span(),
                            &input("x").typ,
                            ExprX::Const(Constant::Int(1.into())),
                        ),
                    ),
                ),
                input("x"),
            ),
        );
        let inferred = infer_function(&function("prior", source)).unwrap();
        assert_eq!(evaluate_integer(&inferred.x.require[0], 254, 0), Value::Bool(true));
        assert_eq!(evaluate_integer(&inferred.x.require[0], 255, 0), Value::Bool(false));
    }
}
