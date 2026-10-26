//! Bounded, untrusted safety synthesis from scalar contracts and interval values.
//! No executable, verifier obligation, or acceptance rule is changed here.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

fn incomplete(span: &Span, why: &str) -> VirErr {
    error(span, format!("Verus Infer incomplete: compositional safety {why}"))
}

#[derive(Clone)]
enum Bound {
    // Normalize linear arithmetic so x+x repeatedly does not duplicate formulas.
    Affine(BigInt, BTreeMap<VarIdent, (Typ, BigInt)>),
    Expr(Term),
}
impl Bound {
    fn constant(n: BigInt) -> Self {
        Self::Affine(n, BTreeMap::new())
    }
    fn variable(name: VarIdent, typ: Typ) -> Self {
        Self::Affine(0.into(), BTreeMap::from([(name, (typ, BigInt::one()))]))
    }
    fn value(&self) -> Option<&BigInt> {
        match self {
            Self::Affine(n, vars) if vars.is_empty() => Some(n),
            _ => None,
        }
    }
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Affine(a, xs), Self::Affine(b, ys)) => {
                a == b
                    && xs.len() == ys.len()
                    && xs.iter().zip(ys).all(|((x, (_, a)), (y, (_, b)))| x == y && a == b)
            }
            (Self::Expr(a), Self::Expr(b)) => Arc::ptr_eq(&a.expr, &b.expr),
            _ => false,
        }
    }
    fn term(&self, span: &Span) -> Result<Term, VirErr> {
        let Self::Affine(n, vars) = self else {
            let Self::Expr(t) = self else { unreachable!() };
            return Ok(t.clone());
        };
        let mut out = if n.is_zero() { None } else { Some(literal(span, n.clone())?) };
        for (name, (typ, coefficient)) in vars {
            let v = Term { expr: SpannedTyped::new(span, typ, ExprX::Var(name.clone())), size: 1 };
            let v = if coefficient == &BigInt::one() {
                v
            } else {
                summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Allow)),
                    literal(span, coefficient.clone())?,
                    v,
                )?
            };
            out = Some(match out {
                Some(a) => summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
                    a,
                    v,
                )?,
                None => v,
            });
        }
        Ok(out.unwrap_or(literal(span, 0.into())?))
    }
    fn arithmetic(span: &Span, op: ArithOp, a: &Self, b: &Self) -> Result<Self, VirErr> {
        if let (Self::Affine(x, xs), Self::Affine(y, ys)) = (a, b) {
            let affine = match op {
                ArithOp::Add(_) | ArithOp::Sub(_) => {
                    let sign =
                        if matches!(op, ArithOp::Sub(_)) { -BigInt::one() } else { BigInt::one() };
                    let mut vars = xs.clone();
                    for (name, (typ, c)) in ys {
                        let entry = vars.entry(name.clone()).or_insert((typ.clone(), 0.into()));
                        entry.1 += &sign * c;
                    }
                    Some((x + &sign * y, vars))
                }
                ArithOp::Mul(_) if xs.is_empty() || ys.is_empty() => {
                    let (factor, n, vars) = if xs.is_empty() { (x, y, ys) } else { (y, x, xs) };
                    Some((
                        factor * n,
                        vars.iter()
                            .map(|(v, (t, c))| (v.clone(), (t.clone(), factor * c)))
                            .collect(),
                    ))
                }
                _ => None,
            };
            if let Some((n, mut vars)) = affine {
                vars.retain(|_, (_, c)| !c.is_zero());
                if n.bits() > 4096 || vars.values().any(|(_, c)| c.bits() > 4096) {
                    return Err(incomplete(span, "coefficient budget exceeded"));
                }
                return Ok(Self::Affine(n, vars));
            }
        }
        Ok(Self::Expr(summary::binary(span, BinaryOp::Arith(op), a.term(span)?, b.term(span)?)?))
    }
    fn select(span: &Span, c: Term, a: Self, b: Self) -> Result<Self, VirErr> {
        if a.same(&b) {
            return Ok(a);
        }
        if is_bool(&c, true) {
            return Ok(a);
        }
        if is_bool(&c, false) {
            return Ok(b);
        }
        Ok(Self::Expr(summary::conditional(span, c, a.term(span)?, b.term(span)?)?))
    }
}
fn literal(span: &Span, n: BigInt) -> Result<Term, VirErr> {
    if n.bits() > 4096 {
        return Err(incomplete(span, "integer budget exceeded"));
    }
    term(span, &int_typ(), ExprX::Const(Constant::Int(n)), 1)
}
fn compare(span: &Span, op: BinaryOp, a: &Bound, b: &Bound) -> Result<Term, VirErr> {
    if let (Some(a), Some(b)) = (a.value(), b.value()) {
        let yes = match op {
            BinaryOp::Eq(_) => a == b,
            BinaryOp::Ne => a != b,
            BinaryOp::Inequality(InequalityOp::Lt) => a < b,
            BinaryOp::Inequality(InequalityOp::Le) => a <= b,
            BinaryOp::Inequality(InequalityOp::Gt) => a > b,
            BinaryOp::Inequality(InequalityOp::Ge) => a >= b,
            _ => unreachable!(),
        };
        return Ok(boolean(span, yes));
    }
    if matches!(a, Bound::Affine(..)) && matches!(b, Bound::Affine(..)) {
        let Bound::Affine(n, vars) =
            Bound::arithmetic(span, ArithOp::Sub(OverflowBehavior::Allow), a, b)?
        else {
            unreachable!()
        };
        let (mut lo, mut hi) = (n.clone(), n);
        for (_, (typ, c)) in vars {
            let TypX::Int(r) = &*typ else { unreachable!() };
            // This optional affine simplification uses an envelope valid for
            // both models; architecture-specific arithmetic checks below use
            // the current analysis width.
            let (min, max) = arithmetic::bounds_at(*r, 64)
                .ok_or_else(|| unsupported(span, "unbounded affine input"))?;
            if c >= BigInt::zero() {
                lo += &c * min;
                hi += c * max;
            } else {
                lo += &c * max;
                hi += c * min;
            }
        }
        let known = match op {
            BinaryOp::Eq(_) => {
                if lo > BigInt::zero() || hi < BigInt::zero() {
                    Some(false)
                } else if lo.is_zero() && hi.is_zero() {
                    Some(true)
                } else {
                    None
                }
            }
            BinaryOp::Ne => {
                if lo > BigInt::zero() || hi < BigInt::zero() {
                    Some(true)
                } else if lo.is_zero() && hi.is_zero() {
                    Some(false)
                } else {
                    None
                }
            }
            BinaryOp::Inequality(InequalityOp::Lt) => {
                if hi < BigInt::zero() {
                    Some(true)
                } else if lo >= BigInt::zero() {
                    Some(false)
                } else {
                    None
                }
            }
            BinaryOp::Inequality(InequalityOp::Le) => {
                if hi <= BigInt::zero() {
                    Some(true)
                } else if lo > BigInt::zero() {
                    Some(false)
                } else {
                    None
                }
            }
            BinaryOp::Inequality(InequalityOp::Gt) => {
                if lo > BigInt::zero() {
                    Some(true)
                } else if hi <= BigInt::zero() {
                    Some(false)
                } else {
                    None
                }
            }
            BinaryOp::Inequality(InequalityOp::Ge) => {
                if lo >= BigInt::zero() {
                    Some(true)
                } else if hi < BigInt::zero() {
                    Some(false)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(b) = known {
            return Ok(boolean(span, b));
        }
    }
    if a.same(b) {
        return Ok(boolean(
            span,
            matches!(
                op,
                BinaryOp::Eq(_) | BinaryOp::Inequality(InequalityOp::Le | InequalityOp::Ge)
            ),
        ));
    }
    summary::binary(span, op, a.term(span)?, b.term(span)?)
}
fn extreme(span: &Span, a: Bound, b: Bound, min: bool) -> Result<Bound, VirErr> {
    if is_bool(&compare(span, BinaryOp::Inequality(InequalityOp::Le), &a, &b)?, true) {
        return Ok(if min { a } else { b });
    }
    if is_bool(&compare(span, BinaryOp::Inequality(InequalityOp::Ge), &a, &b)?, true) {
        return Ok(if min { b } else { a });
    }
    let c = compare(
        span,
        BinaryOp::Inequality(if min { InequalityOp::Le } else { InequalityOp::Ge }),
        &a,
        &b,
    )?;
    Bound::select(span, c, a, b)
}

#[derive(Clone)]
struct Predicate {
    yes: Term,
    no: Term,
    exact: Option<Term>,
}
impl Predicate {
    fn exact(span: &Span, t: Term) -> Result<Self, VirErr> {
        Ok(Self { yes: t.clone(), no: exits::not(span, t.clone())?, exact: Some(t) })
    }
    fn known(span: &Span, b: bool) -> Self {
        Self::exact(span, boolean(span, b)).unwrap()
    }
    fn unknown(span: &Span) -> Self {
        Self { yes: boolean(span, false), no: boolean(span, false), exact: None }
    }
    fn not(self) -> Self {
        Self { yes: self.no.clone(), no: self.yes, exact: self.exact.map(|_| self.no) }
    }
    fn logical(span: &Span, op: LogicalOp, a: Self, b: Self) -> Result<Self, VirErr> {
        if op == LogicalOp::Implies {
            return Self::logical(span, LogicalOp::Or, a.not(), b);
        }
        if let (Some(a), Some(b)) = (&a.exact, &b.exact) {
            return Self::exact(span, logical(span, op, a.clone(), b.clone())?);
        }
        let dual = if op == LogicalOp::And { LogicalOp::Or } else { LogicalOp::And };
        Ok(Self {
            yes: logical(span, op, a.yes, b.yes)?,
            no: logical(span, dual, a.no, b.no)?,
            exact: None,
        })
    }
}
#[derive(Clone)]
struct Numeric {
    excluded: BTreeSet<BigInt>,
    origin: Option<Arc<()>>,
    lo: Option<Bound>,
    hi: Option<Bound>,
    point: Option<Bound>,
    range: Range,
}
impl Numeric {
    fn point(p: Bound, range: Range) -> Self {
        Self {
            excluded: BTreeSet::new(),
            origin: None,
            lo: Some(p.clone()),
            hi: Some(p.clone()),
            point: Some(p),
            range,
        }
    }
    fn range(range: Range) -> Self {
        match &range {
            Range::Int(a, b) if a == b => Self::point(Bound::constant(a.clone()), range),
            Range::Int(a, b) => Self {
                excluded: BTreeSet::new(),
                origin: None,
                lo: Some(Bound::constant(a.clone())),
                hi: Some(Bound::constant(b.clone())),
                point: None,
                range,
            },
            _ => Self {
                excluded: BTreeSet::new(),
                origin: None,
                lo: None,
                hi: None,
                point: None,
                range,
            },
        }
    }
    fn ends(&self, span: &Span) -> Result<(Bound, Bound), VirErr> {
        match (&self.lo, &self.hi) {
            (Some(a), Some(b)) => Ok((a.clone(), b.clone())),
            _ => Err(incomplete(span, "cannot express finite operand bounds")),
        }
    }
    fn compare(span: &Span, op: BinaryOp, a: &Self, b: &Self) -> Result<Predicate, VirErr> {
        if let (Range::Int(al, ah), Range::Int(bl, bh)) = (&a.range, &b.range) {
            let excluded =
                (bl == bh && a.excluded.contains(bl)) || (al == ah && b.excluded.contains(al));
            let known = match op {
                BinaryOp::Eq(_) => {
                    if excluded || ah < bl || bh < al {
                        Some(false)
                    } else if al == ah && bl == bh && al == bl {
                        Some(true)
                    } else {
                        None
                    }
                }
                BinaryOp::Ne => {
                    if excluded || ah < bl || bh < al {
                        Some(true)
                    } else if al == ah && bl == bh && al == bl {
                        Some(false)
                    } else {
                        None
                    }
                }
                BinaryOp::Inequality(InequalityOp::Lt) => {
                    if ah < bl {
                        Some(true)
                    } else if al >= bh {
                        Some(false)
                    } else {
                        None
                    }
                }
                BinaryOp::Inequality(InequalityOp::Le) => {
                    if ah <= bl {
                        Some(true)
                    } else if al > bh {
                        Some(false)
                    } else {
                        None
                    }
                }
                BinaryOp::Inequality(InequalityOp::Gt) => {
                    if al > bh {
                        Some(true)
                    } else if ah <= bl {
                        Some(false)
                    } else {
                        None
                    }
                }
                BinaryOp::Inequality(InequalityOp::Ge) => {
                    if al >= bh {
                        Some(true)
                    } else if ah < bl {
                        Some(false)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(b) = known {
                return Ok(Predicate::known(span, b));
            }
        }
        if let (Some(a), Some(b)) = (&a.point, &b.point) {
            return Predicate::exact(span, compare(span, op, a, b)?);
        }
        let (al, ah) = a.ends(span)?;
        let (bl, bh) = b.ends(span)?;
        let cmp = |op, a: &Bound, b: &Bound| compare(span, op, a, b);
        let (yes, no) = match op {
            BinaryOp::Inequality(InequalityOp::Lt) => {
                (cmp(op, &ah, &bl)?, cmp(BinaryOp::Inequality(InequalityOp::Ge), &al, &bh)?)
            }
            BinaryOp::Inequality(InequalityOp::Le) => {
                (cmp(op, &ah, &bl)?, cmp(BinaryOp::Inequality(InequalityOp::Gt), &al, &bh)?)
            }
            BinaryOp::Inequality(InequalityOp::Gt) => {
                return Self::compare(span, BinaryOp::Inequality(InequalityOp::Lt), b, a);
            }
            BinaryOp::Inequality(InequalityOp::Ge) => {
                return Self::compare(span, BinaryOp::Inequality(InequalityOp::Le), b, a);
            }
            BinaryOp::Ne => return Ok(Self::compare(span, BinaryOp::Eq(Mode::Spec), a, b)?.not()),
            BinaryOp::Eq(_) => {
                let yes = logical(
                    span,
                    LogicalOp::And,
                    cmp(op, &al, &ah)?,
                    logical(span, LogicalOp::And, cmp(op, &bl, &bh)?, cmp(op, &al, &bl)?)?,
                )?;
                let no = logical(
                    span,
                    LogicalOp::Or,
                    cmp(BinaryOp::Inequality(InequalityOp::Lt), &ah, &bl)?,
                    cmp(BinaryOp::Inequality(InequalityOp::Lt), &bh, &al)?,
                )?;
                (yes, no)
            }
            _ => return Err(unsupported(span, "non-scalar safety comparison")),
        };
        Ok(Predicate { yes, no, exact: None })
    }
    fn arithmetic(span: &Span, op: ArithOp, a: &Self, b: &Self) -> Result<Self, VirErr> {
        let range = match (&a.range, &b.range) {
            (Range::Int(a, b), Range::Int(c, d)) => match op {
                ArithOp::Add(_) => Range::Int(a + c, b + d),
                ArithOp::Sub(_) => Range::Int(a - d, b - c),
                ArithOp::Mul(_) => {
                    let xs = [a * c, a * d, b * c, b * d];
                    Range::Int(xs.iter().min().unwrap().clone(), xs.iter().max().unwrap().clone())
                }
                ArithOp::TruncatingDiv(..) => divide(a, b, c, d, true),
                ArithOp::TruncatingMod(..) => remainder(a, b, c, d, true),
                ArithOp::EuclideanDiv(_) => divide(a, b, c, d, false),
                ArithOp::EuclideanMod(_) => remainder(a, b, c, d, false),
            },
            _ => Range::IntTop,
        };
        if let (Some(x), Some(y)) = (&a.point, &b.point) {
            return Ok(Self::point(Bound::arithmetic(span, op, x, y)?, range));
        }
        if matches!(
            op,
            ArithOp::EuclideanDiv(_)
                | ArithOp::EuclideanMod(_)
                | ArithOp::TruncatingDiv(..)
                | ArithOp::TruncatingMod(..)
        ) {
            return Ok(Self::range(range));
        }
        let (al, ah) = a.ends(span)?;
        let (bl, bh) = b.ends(span)?;
        let (lo, hi) = match op {
            ArithOp::Add(_) => {
                (Bound::arithmetic(span, op, &al, &bl)?, Bound::arithmetic(span, op, &ah, &bh)?)
            }
            ArithOp::Sub(_) => {
                (Bound::arithmetic(span, op, &al, &bh)?, Bound::arithmetic(span, op, &ah, &bl)?)
            }
            ArithOp::Mul(_) => {
                let xs = [
                    Bound::arithmetic(span, op, &al, &bl)?,
                    Bound::arithmetic(span, op, &al, &bh)?,
                    Bound::arithmetic(span, op, &ah, &bl)?,
                    Bound::arithmetic(span, op, &ah, &bh)?,
                ];
                let mut lo = xs[0].clone();
                let mut hi = lo.clone();
                for x in &xs[1..] {
                    lo = extreme(span, lo, x.clone(), true)?;
                    hi = extreme(span, hi, x.clone(), false)?;
                }
                (lo, hi)
            }
            _ => unreachable!(),
        };
        Ok(Self {
            excluded: BTreeSet::new(),
            origin: None,
            lo: Some(lo),
            hi: Some(hi),
            point: None,
            range,
        })
    }
    fn in_range(&self, span: &Span, r: IntRange, word_bits: u32) -> Result<Term, VirErr> {
        let (min, max) = arithmetic::bounds_at(r, word_bits)
            .ok_or_else(|| unsupported(span, "safety arithmetic range"))?;
        if matches!(&self.range,Range::Int(a,b) if a>=&min && b<=&max) {
            return Ok(boolean(span, true));
        }
        let (lo, hi) = self.ends(span)?;
        logical(
            span,
            LogicalOp::And,
            compare(span, BinaryOp::Inequality(InequalityOp::Le), &Bound::constant(min), &lo)?,
            compare(span, BinaryOp::Inequality(InequalityOp::Le), &hi, &Bound::constant(max))?,
        )
    }
}
#[derive(Clone)]
enum Value {
    Number(Numeric),
    Bool(Predicate),
    Unit,
}
impl Value {
    fn from_range(span: &Span, r: Range) -> Self {
        match r {
            Range::Bool(Some(b)) => Self::Bool(Predicate::known(span, b)),
            Range::Bool(None) => Self::Bool(Predicate::unknown(span)),
            Range::Unit => Self::Unit,
            r => Self::Number(Numeric::range(r)),
        }
    }
    fn number(&self, span: &Span) -> Result<Numeric, VirErr> {
        match self {
            Self::Number(n) => Ok(n.clone()),
            _ => Err(unsupported(span, "non-numeric safety value")),
        }
    }
    fn predicate(&self, span: &Span) -> Result<Predicate, VirErr> {
        match self {
            Self::Bool(p) => Ok(p.clone()),
            _ => Err(unsupported(span, "non-boolean safety value")),
        }
    }
    fn merge(span: &Span, c: &Predicate, a: Self, b: Self) -> Result<Self, VirErr> {
        match (a, b) {
            (Self::Unit, Self::Unit) => Ok(Self::Unit),
            (Self::Number(a), Self::Number(b)) => {
                let r = a.range.clone().join(b.range.clone());
                if let Some(c) = &c.exact {
                    if let (Some(a), Some(b)) = (&a.point, &b.point) {
                        return Ok(Self::Number(Numeric::point(
                            Bound::select(span, c.clone(), a.clone(), b.clone())?,
                            r,
                        )));
                    }
                    let (al, ah) = a.ends(span)?;
                    let (bl, bh) = b.ends(span)?;
                    return Ok(Self::Number(Numeric {
                        excluded: BTreeSet::new(),
                        origin: None,
                        lo: Some(Bound::select(span, c.clone(), al, bl)?),
                        hi: Some(Bound::select(span, c.clone(), ah, bh)?),
                        point: None,
                        range: r,
                    }));
                }
                let (al, ah) = a.ends(span)?;
                let (bl, bh) = b.ends(span)?;
                Ok(Self::Number(Numeric {
                    excluded: BTreeSet::new(),
                    origin: None,
                    lo: Some(extreme(span, al, bl, true)?),
                    hi: Some(extreme(span, ah, bh, false)?),
                    point: None,
                    range: r,
                }))
            }
            (Self::Bool(a), Self::Bool(b)) => {
                if let (Some(c), Some(a), Some(b)) = (&c.exact, &a.exact, &b.exact) {
                    return Ok(Self::Bool(Predicate::exact(
                        span,
                        summary::conditional(span, c.clone(), a.clone(), b.clone())?,
                    )?));
                }
                let guard = |x: Term, y: Term| {
                    logical(
                        span,
                        LogicalOp::And,
                        logical(span, LogicalOp::Or, c.no.clone(), x)?,
                        logical(span, LogicalOp::Or, c.yes.clone(), y)?,
                    )
                };
                Ok(Self::Bool(Predicate {
                    yes: guard(a.yes, b.yes)?,
                    no: guard(a.no, b.no)?,
                    exact: None,
                }))
            }
            _ => Err(unsupported(span, "inconsistent safety branch values")),
        }
    }
}
struct Checked {
    safe: Term,
    value: Value,
}
struct Analyzer<'a> {
    locals: HashMap<VarIdent, Value>,
    callees: &'a HashMap<Fun, Function>,
    remaining: usize,
    word_bits: u32,
}
impl Analyzer<'_> {
    fn refine(&mut self, e: &Expr, truth: bool) {
        match &e.x {
            ExprX::Unary(UnaryOp::Not, a) => {
                self.refine(a, !truth);
                return;
            }
            ExprX::Logical(LogicalOp::And, a, b) if truth => {
                self.refine(a, true);
                self.refine(b, true);
                return;
            }
            ExprX::Logical(LogicalOp::Or, a, b) if !truth => {
                self.refine(a, false);
                self.refine(b, false);
                return;
            }
            _ => {}
        }
        let ExprX::Binary(op, a, b) = &e.x else { return };
        let (mut op, mut a, mut b) = (*op, a, b);
        let name = |e: &Expr| match &e.x {
            ExprX::Var(n) => Some(n.clone()),
            ExprX::ReadPlace(p, _) => match &p.x {
                PlaceX::Local(n) => Some(n.clone()),
                _ => None,
            },
            _ => None,
        };
        let constant = |e: &Expr| match &e.x {
            ExprX::Const(Constant::Int(n)) => Some(n.clone()),
            _ => arithmetic::negative_literal(e),
        };
        if name(a).is_none() {
            std::mem::swap(&mut a, &mut b);
            op = match op {
                BinaryOp::Inequality(InequalityOp::Lt) => BinaryOp::Inequality(InequalityOp::Gt),
                BinaryOp::Inequality(InequalityOp::Le) => BinaryOp::Inequality(InequalityOp::Ge),
                BinaryOp::Inequality(InequalityOp::Gt) => BinaryOp::Inequality(InequalityOp::Lt),
                BinaryOp::Inequality(InequalityOp::Ge) => BinaryOp::Inequality(InequalityOp::Le),
                op => op,
            };
        }
        let (Some(name), Some(n)) = (name(a), constant(b)) else { return };
        let Some(Value::Number(old)) = self.locals.get(&name).cloned() else { return };
        for (local, value) in self.locals.iter_mut() {
            let Value::Number(v) = value else { continue };
            let alias = match (&v.origin, &old.origin) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            };
            if local != &name && !alias {
                continue;
            }
            let Range::Int(mut lo, mut hi) = v.range.clone() else { continue };
            match (op, truth) {
                (BinaryOp::Eq(_), true) | (BinaryOp::Ne, false) => {
                    lo = lo.max(n.clone());
                    hi = hi.min(n.clone());
                }
                (BinaryOp::Ne, true) | (BinaryOp::Eq(_), false) => {
                    v.excluded.insert(n.clone());
                }
                (BinaryOp::Inequality(InequalityOp::Lt), true)
                | (BinaryOp::Inequality(InequalityOp::Ge), false) => hi = hi.min(&n - 1),
                (BinaryOp::Inequality(InequalityOp::Le), true)
                | (BinaryOp::Inequality(InequalityOp::Gt), false) => hi = hi.min(n.clone()),
                (BinaryOp::Inequality(InequalityOp::Gt), true)
                | (BinaryOp::Inequality(InequalityOp::Le), false) => lo = lo.max(&n + 1),
                (BinaryOp::Inequality(InequalityOp::Ge), true)
                | (BinaryOp::Inequality(InequalityOp::Lt), false) => lo = lo.max(n.clone()),
                _ => {}
            }
            while lo <= hi && v.excluded.contains(&lo) {
                lo += 1;
            }
            while lo <= hi && v.excluded.contains(&hi) {
                hi -= 1;
            }
            if lo <= hi {
                v.range = Range::Int(lo.clone(), hi.clone());
                if v.point.is_none() {
                    v.lo = Some(Bound::constant(lo));
                    v.hi = Some(Bound::constant(hi));
                }
            }
        }
    }
    fn place(&mut self, p: &Place, checked: bool) -> Result<Checked, VirErr> {
        match &p.x {
            PlaceX::Local(name) => Ok(Checked {
                safe: boolean(&p.span, true),
                value: self
                    .locals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| unsupported(&p.span, "unknown safety local"))?,
            }),
            PlaceX::Temporary(e) => self.expr(e, checked),
            _ => Err(unsupported(&p.span, "safety place outside scalar subset")),
        }
    }
    fn expr(&mut self, e: &Expr, checked: bool) -> Result<Checked, VirErr> {
        if self.remaining == 0 {
            return Err(incomplete(&e.span, "analysis budget exceeded"));
        }
        self.remaining -= 1;
        let s = &e.span;
        let pure = |value| Ok(Checked { safe: boolean(s, true), value });
        if let Some(n) = arithmetic::negative_literal(e) {
            return pure(Value::Number(Numeric::point(
                Bound::constant(n.clone()),
                Range::Int(n.clone(), n),
            )));
        }
        if let Some(n) = arithmetic::arch_value(e, self.word_bits) {
            return pure(Value::Number(Numeric::point(
                Bound::constant(n.clone()),
                Range::Int(n.clone(), n),
            )));
        }
        match &e.x {
            ExprX::Const(Constant::Int(n)) => pure(Value::Number(Numeric::point(
                Bound::constant(n.clone()),
                Range::Int(n.clone(), n.clone()),
            ))),
            ExprX::Const(Constant::Bool(b)) => pure(Value::Bool(Predicate::known(s, *b))),
            ExprX::Var(name) => pure(
                self.locals
                    .get(name)
                    .cloned()
                    .ok_or_else(|| unsupported(s, "unknown safety variable"))?,
            ),
            ExprX::ReadPlace(p, _) => self.place(p, checked),
            ExprX::NeverToAny(a) => self.expr(a, checked),
            ExprX::Unary(UnaryOp::BitNot(_), a) => {
                let a = self.expr(a, checked)?;
                Ok(Checked {
                    safe: a.safe,
                    value: Value::from_range(s, Range::full(&e.typ, self.word_bits)?),
                })
            }
            ExprX::Binary(BinaryOp::Bitwise(bit, policy), a, b) => {
                let l = self.expr(a, checked)?;
                let r = self.expr(b, checked)?;
                let mut safe = logical(s, LogicalOp::And, l.safe, r.safe)?;
                if checked && matches!(bit, BitwiseOp::Shl(..) | BitwiseOp::Shr) {
                    let BitshiftBehavior::Error(width) = policy else {
                        return Err(unsupported(s, "unchecked source shift"));
                    };
                    let count = match width {
                        IntegerTypeBitwidth::Width(w) => *w,
                        IntegerTypeBitwidth::ArchWordSize => self.word_bits,
                    };
                    let n = r.value.number(s)?;
                    let zero = Numeric::range(Range::Int(0.into(), 0.into()));
                    let limit = Numeric::range(Range::Int(count.into(), count.into()));
                    let valid = logical(
                        s,
                        LogicalOp::And,
                        Numeric::compare(s, BinaryOp::Inequality(InequalityOp::Ge), &n, &zero)?.yes,
                        Numeric::compare(s, BinaryOp::Inequality(InequalityOp::Lt), &n, &limit)?
                            .yes,
                    )?;
                    safe = logical(s, LogicalOp::And, safe, valid)?;
                }
                Ok(Checked {
                    safe,
                    value: Value::from_range(s, Range::full(&e.typ, self.word_bits)?),
                })
            }
            ExprX::Unary(UnaryOp::Not, a) => {
                let a = self.expr(a, checked)?;
                Ok(Checked { safe: a.safe, value: Value::Bool(a.value.predicate(s)?.not()) })
            }
            ExprX::Unary(UnaryOp::Clip { range, .. }, a) => {
                let a = self.expr(a, checked)?;
                let n = a.value.number(s)?;
                let (min, max) = arithmetic::bounds_at(*range, self.word_bits)
                    .ok_or_else(|| unsupported(s, "safety cast range"))?;
                let fits = matches!(&n.range,Range::Int(a,b) if a>=&min&&b<=&max);
                let value = if fits {
                    n
                } else if let Some(p) = &n.point {
                    Numeric::point(
                        Bound::Expr(summary::unary(
                            s,
                            UnaryOp::Clip { range: *range, truncate: true },
                            p.term(s)?,
                        )?),
                        Range::Int(min, max),
                    )
                } else {
                    Numeric::range(Range::Int(min, max))
                };
                Ok(Checked { safe: a.safe, value: Value::Number(value) })
            }
            ExprX::Logical(op, a, b) => {
                let condition = a.clone();
                let a = self.expr(a, checked)?;
                let p = a.value.predicate(s)?;
                let skip = if *op == LogicalOp::Or { p.yes.clone() } else { p.no.clone() };
                if is_bool(&skip, true) {
                    return Ok(Checked {
                        safe: a.safe,
                        value: Value::Bool(Predicate::known(s, *op != LogicalOp::And)),
                    });
                }
                let saved = self.locals.clone();
                self.refine(&condition, *op != LogicalOp::Or);
                let b = self.expr(b, checked)?;
                self.locals = saved;
                Ok(Checked {
                    safe: logical(
                        s,
                        LogicalOp::And,
                        a.safe,
                        logical(s, LogicalOp::Or, skip, b.safe)?,
                    )?,
                    value: Value::Bool(Predicate::logical(s, *op, p, b.value.predicate(s)?)?),
                })
            }
            ExprX::Binary(op, a, b) => {
                let a = self.expr(a, checked)?;
                let b = self.expr(b, checked)?;
                let mut safe = logical(s, LogicalOp::And, a.safe, b.safe)?;
                let value = match op {
                    BinaryOp::Arith(arith) => {
                        let (n, m) = (a.value.number(s)?, b.value.number(s)?);
                        let (pure_op, overflow, div_check) = if checked {
                            eligibility::source_binary(*op)
                                .ok_or_else(|| unsupported(s, "safety arithmetic policy"))?
                        } else {
                            (*op, None, false)
                        };
                        let BinaryOp::Arith(pure_op) = pure_op else { unreachable!() };
                        let mut out = Numeric::arithmetic(s, pure_op, &n, &m)?;
                        if let Some(r) = overflow {
                            safe = logical(
                                s,
                                LogicalOp::And,
                                safe,
                                out.in_range(s, r, self.word_bits)?,
                            )?;
                        }
                        if div_check {
                            let zero = Numeric::range(Range::Int(0.into(), 0.into()));
                            let mut valid = Numeric::compare(s, BinaryOp::Ne, &m, &zero)?.yes;
                            if let ArithOp::TruncatingDiv(
                                _,
                                SignedDivEdgeCaseBehavior::Error(width),
                            )
                            | ArithOp::TruncatingMod(
                                _,
                                SignedDivEdgeCaseBehavior::Error(width),
                            ) = arith
                            {
                                let (min, _) = arithmetic::bounds_at(
                                    arithmetic::signed_range(*width),
                                    self.word_bits,
                                )
                                .unwrap();
                                let min = Numeric::range(Range::Int(min.clone(), min));
                                let minus_one =
                                    Numeric::range(Range::Int((-1).into(), (-1).into()));
                                let edge = Predicate::logical(
                                    s,
                                    LogicalOp::Or,
                                    Numeric::compare(s, BinaryOp::Ne, &n, &min)?,
                                    Numeric::compare(s, BinaryOp::Ne, &m, &minus_one)?,
                                )?;
                                valid = logical(s, LogicalOp::And, valid, edge.yes)?;
                            }
                            safe = logical(s, LogicalOp::And, safe, valid)?;
                        }
                        out.range = out.range.restrict(&e.typ, self.word_bits);
                        Value::Number(out)
                    }
                    BinaryOp::Eq(_) | BinaryOp::Ne | BinaryOp::Inequality(_) => {
                        match (&a.value, &b.value) {
                            (Value::Number(n), Value::Number(m)) => {
                                Value::Bool(Numeric::compare(s, *op, n, m)?)
                            }
                            (Value::Bool(n), Value::Bool(m)) => {
                                let same = Predicate::logical(
                                    s,
                                    LogicalOp::Or,
                                    Predicate::logical(s, LogicalOp::And, n.clone(), m.clone())?,
                                    Predicate::logical(
                                        s,
                                        LogicalOp::And,
                                        n.clone().not(),
                                        m.clone().not(),
                                    )?,
                                )?;
                                Value::Bool(if matches!(op, BinaryOp::Ne) {
                                    same.not()
                                } else {
                                    same
                                })
                            }
                            _ => return Err(unsupported(s, "safety comparison types")),
                        }
                    }
                    BinaryOp::BoolAndNoSC | BinaryOp::BoolOrNoSC | BinaryOp::BoolXor => {
                        let (p, q) = (a.value.predicate(s)?, b.value.predicate(s)?);
                        let p = match op {
                            BinaryOp::BoolAndNoSC => Predicate::logical(s, LogicalOp::And, p, q)?,
                            BinaryOp::BoolOrNoSC => Predicate::logical(s, LogicalOp::Or, p, q)?,
                            _ => Predicate::logical(
                                s,
                                LogicalOp::Or,
                                Predicate::logical(s, LogicalOp::And, p.clone(), q.clone().not())?,
                                Predicate::logical(s, LogicalOp::And, p.not(), q)?,
                            )?,
                        };
                        Value::Bool(p)
                    }
                    _ => return Err(unsupported(s, "safety binary operation")),
                };
                Ok(Checked { safe, value })
            }
            ExprX::If(c, a, b) => {
                let condition = c.clone();
                let c = self.expr(c, checked)?;
                let p = c.value.predicate(s)?;
                if is_bool(&p.yes, true) {
                    let a = self.expr(a, checked)?;
                    return Ok(Checked {
                        safe: logical(s, LogicalOp::And, c.safe, a.safe)?,
                        value: a.value,
                    });
                }
                if is_bool(&p.no, true) {
                    let b = match b {
                        Some(b) => self.expr(b, checked)?,
                        None => Checked { safe: boolean(s, true), value: Value::Unit },
                    };
                    return Ok(Checked {
                        safe: logical(s, LogicalOp::And, c.safe, b.safe)?,
                        value: b.value,
                    });
                }
                let saved = self.locals.clone();
                self.refine(&condition, true);
                let a = self.expr(a, checked)?;
                self.locals = saved.clone();
                self.refine(&condition, false);
                let b = match b {
                    Some(b) => self.expr(b, checked)?,
                    None => Checked { safe: boolean(s, true), value: Value::Unit },
                };
                self.locals = saved;
                let branches = logical(
                    s,
                    LogicalOp::And,
                    logical(s, LogicalOp::Or, p.no.clone(), a.safe)?,
                    logical(s, LogicalOp::Or, p.yes.clone(), b.safe)?,
                )?;
                Ok(Checked {
                    safe: logical(s, LogicalOp::And, c.safe, branches)?,
                    value: Value::merge(s, &p, a.value, b.value)?,
                })
            }
            ExprX::Block(stmts, tail) => {
                let saved = self.locals.clone();
                let mut safe = boolean(s, true);
                for stmt in stmts.iter() {
                    let a = match &stmt.x {
                        StmtX::Expr(a) => self.expr(a, checked)?,
                        StmtX::Decl { pattern, init: Some(p), .. } => {
                            let PatternX::Var(v) = &pattern.x else {
                                return Err(unsupported(s, "safety binding"));
                            };
                            let mut a = self.place(p, checked)?;
                            if let Value::Number(n) = &mut a.value
                                && n.point.is_none()
                                && n.origin.is_none()
                            {
                                n.origin = Some(Arc::new(()));
                            }
                            self.locals.insert(v.name.clone(), a.value.clone());
                            a
                        }
                        _ => return Err(unsupported(s, "safety statement")),
                    };
                    safe = logical(s, LogicalOp::And, safe, a.safe)?;
                }
                let a = match tail {
                    Some(a) => self.expr(a, checked)?,
                    None => Checked { safe: boolean(s, true), value: Value::Unit },
                };
                self.locals = saved;
                Ok(Checked { safe: logical(s, LogicalOp::And, safe, a.safe)?, value: a.value })
            }
            ExprX::Call {
                target: CallTarget::Fun(CallTargetKind::Static, name, ..),
                args,
                post_args: None,
                body: None,
            } => {
                let f = self
                    .callees
                    .get(name)
                    .ok_or_else(|| unsupported(s, "missing safety callee"))?
                    .clone();
                if let Some(model) = integer_models::recognize(&f) {
                    if model.policy == integer_models::Policy::Checked {
                        return Err(unsupported(
                            s,
                            "enum expressions currently require exact inference",
                        ));
                    }
                    if args.len() != f.x.params.len() {
                        return Err(unsupported(s, "integer model argument count"));
                    }
                    let mut safe = boolean(s, true);
                    for arg in args.iter() {
                        let arg = self.expr(arg, checked)?;
                        safe = logical(s, LogicalOp::And, safe, arg.safe)?;
                    }
                    return Ok(Checked {
                        safe,
                        value: Value::from_range(s, Range::full(&e.typ, self.word_bits)?),
                    });
                }
                if !f.x.attrs.infer || args.len() != f.x.params.len() {
                    return Err(unsupported(s, "safety requires an inferred scalar callee"));
                }
                let mut safe = boolean(s, true);
                let mut locals = HashMap::new();
                for (param, arg) in f.x.params.iter().zip(args.iter()) {
                    let a = self.expr(arg, checked)?;
                    safe = logical(s, LogicalOp::And, safe, a.safe)?;
                    locals.insert(param.x.name.clone(), a.value);
                }
                let mut child = Analyzer {
                    locals,
                    callees: self.callees,
                    remaining: self.remaining,
                    word_bits: self.word_bits,
                };
                let pre = child.expr(&f.x.require[0], false)?.value.predicate(s)?.yes;
                safe = logical(s, LogicalOp::And, safe, pre)?;
                let value = if f.x.attrs.infer_allow_approx {
                    Value::from_range(s, range_from_summary(&f, self.word_bits)?)
                } else {
                    let ExprX::Binary(BinaryOp::Eq(_), _, out) = &f.x.ensure.0[0].x else {
                        return Err(unsupported(s, "invalid safety callee summary"));
                    };
                    let mut v = child.expr(out, false)?.value;
                    if let Value::Number(n) = &mut v {
                        n.range = n.range.clone().restrict(&f.x.inner_ret.x.typ, self.word_bits);
                        if n.lo.is_none() {
                            *n = Numeric::range(n.range.clone());
                        }
                    }
                    v
                };
                self.remaining = child.remaining;
                Ok(Checked { safe, value })
            }
            _ => Err(unsupported(s, "expression outside compositional safety subset")),
        }
    }
}

pub(super) fn infer(
    function: &Function,
    callees: &HashMap<Fun, Function>,
    word_bits: u32,
) -> Result<Term, VirErr> {
    let mut locals = HashMap::new();
    for p in function.x.params.iter() {
        let r = Range::full(&p.x.typ, word_bits)?;
        let v = if matches!(&*p.x.typ, TypX::Bool) {
            Value::Bool(Predicate::exact(
                &p.span,
                Term {
                    expr: SpannedTyped::new(&p.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
                    size: 1,
                },
            )?)
        } else {
            Value::Number(Numeric::point(Bound::variable(p.x.name.clone(), p.x.typ.clone()), r))
        };
        locals.insert(p.x.name.clone(), v);
    }
    let body = function
        .x
        .body
        .as_ref()
        .ok_or_else(|| unsupported(&function.span, "missing safety body"))?;
    let pre =
        Analyzer { locals, callees, remaining: FORMULA_BUDGET, word_bits }.expr(body, true)?.safe;
    if is_bool(&pre, false) {
        return Err(incomplete(
            &function.span,
            "cannot establish a usable input domain from the available summaries",
        ));
    }
    Ok(pre)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infer::tests::{input, span};

    #[test]
    fn affine_normalization_is_bounded_and_non_affine_expansion_fails_explicitly() {
        let x = input("x");
        let ExprX::Var(name) = &x.x else { unreachable!() };
        let mut linear = Bound::variable(name.clone(), x.typ.clone());
        for _ in 0..80 {
            linear =
                Bound::arithmetic(&span(), ArithOp::Add(OverflowBehavior::Allow), &linear, &linear)
                    .unwrap();
        }
        assert!(linear.term(&span()).unwrap().size < 10);
        let Bound::Affine(n, vars) = linear else { panic!("linear normalization lost") };
        assert!(n.is_zero());
        assert_eq!(vars[name].1, BigInt::one() << 80);
        let mut square = Bound::variable(name.clone(), x.typ.clone());
        let mut exhausted = false;
        for _ in 0..20 {
            match Bound::arithmetic(
                &span(),
                ArithOp::Mul(OverflowBehavior::Allow),
                &square,
                &square,
            ) {
                Ok(next) => square = next,
                Err(_) => {
                    exhausted = true;
                    break;
                }
            }
        }
        assert!(exhausted, "non-affine expansion escaped its formula budget");
    }
}
