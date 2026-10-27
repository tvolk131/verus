//! Bounded path refinement for immutable scalar values. This generates candidates;
//! ordinary verification independently proves every resulting range.
use super::*;

pub(super) fn local(e: &Expr) -> Option<&VarIdent> {
    match &e.x {
        ExprX::Var(n) => Some(n),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}
impl Interpreter<'_> {
    fn root(&self, n: &VarIdent) -> VarIdent {
        self.aliases.get(n).unwrap_or(n).clone()
    }
    fn narrow(&mut self, n: &VarIdent, range: Range) -> bool {
        let root = self.root(n);
        for (name, value) in &mut self.locals {
            if self.aliases.get(name).unwrap_or(name) != &root {
                continue;
            }
            match (&*value, &range) {
                (Range::Int(a, b), Range::Int(c, d)) => {
                    let (lo, hi) = (a.max(c).clone(), b.min(d).clone());
                    if lo > hi {
                        return false;
                    }
                    *value = Range::Int(lo, hi);
                }
                (Range::Bool(Some(a)), Range::Bool(Some(b))) if a != b => return false,
                (Range::Bool(_), Range::Bool(Some(b))) => *value = Range::Bool(Some(*b)),
                _ => {}
            }
        }
        true
    }
    fn refine(&mut self, e: &Expr, truth: bool) -> Result<bool, VirErr> {
        self.spend(&e.span)?;
        if let Some(n) = local(e) {
            if matches!(&*e.typ, TypX::Bool) {
                if !self.narrow(n, Range::Bool(Some(truth))) {
                    return Ok(false);
                }
                if let Some(definition) = self.definitions.get(&self.root(n)).cloned() {
                    return self.refine(&definition, truth);
                }
            }
            return Ok(true);
        }
        match &e.x {
            ExprX::Const(Constant::Bool(b)) => return Ok(*b == truth),
            ExprX::Unary(UnaryOp::Not, a) => return self.refine(a, !truth),
            ExprX::Logical(LogicalOp::And, a, b) if truth => {
                return Ok(self.refine(a, true)? && self.refine(b, true)?);
            }
            ExprX::Logical(LogicalOp::Or, a, b) if !truth => {
                return Ok(self.refine(a, false)? && self.refine(b, false)?);
            }
            ExprX::Logical(LogicalOp::Implies, a, b) if !truth => {
                return Ok(self.refine(a, true)? && self.refine(b, false)?);
            }
            _ => {}
        }
        let ExprX::Binary(op, a, b) = &e.x else { return Ok(true) };
        let (Range::Int(lo_a, hi_a), Range::Int(lo_b, hi_b)) = (self.expr(a)?, self.expr(b)?)
        else {
            return Ok(true);
        };
        use InequalityOp::*;
        let op = match (*op, truth) {
            (BinaryOp::Eq(_), false) => BinaryOp::Ne,
            (BinaryOp::Ne, false) => BinaryOp::Eq(Mode::Spec),
            (BinaryOp::Inequality(Lt), false) => BinaryOp::Inequality(Ge),
            (BinaryOp::Inequality(Le), false) => BinaryOp::Inequality(Gt),
            (BinaryOp::Inequality(Gt), false) => BinaryOp::Inequality(Le),
            (BinaryOp::Inequality(Ge), false) => BinaryOp::Inequality(Lt),
            (op, _) => op,
        };
        let (mut a_lo, mut a_hi, mut b_lo, mut b_hi) =
            (lo_a.clone(), hi_a.clone(), lo_b.clone(), hi_b.clone());
        match op {
            BinaryOp::Eq(_) => {
                a_lo = a_lo.max(lo_b.clone());
                a_hi = a_hi.min(hi_b.clone());
                b_lo = b_lo.max(lo_a.clone());
                b_hi = b_hi.min(hi_a.clone());
            }
            BinaryOp::Ne => {
                if lo_b == hi_b {
                    if a_lo == lo_b {
                        a_lo += 1;
                    } else if a_hi == hi_b {
                        a_hi -= 1;
                    }
                }
                if lo_a == hi_a {
                    if b_lo == lo_a {
                        b_lo += 1;
                    } else if b_hi == hi_a {
                        b_hi -= 1;
                    }
                }
            }
            BinaryOp::Inequality(Lt) => {
                a_hi = a_hi.min(&hi_b - 1);
                b_lo = b_lo.max(&lo_a + 1);
            }
            BinaryOp::Inequality(Le) => {
                a_hi = a_hi.min(hi_b);
                b_lo = b_lo.max(lo_a);
            }
            BinaryOp::Inequality(Gt) => {
                a_lo = a_lo.max(&lo_b + 1);
                b_hi = b_hi.min(&hi_a - 1);
            }
            BinaryOp::Inequality(Ge) => {
                a_lo = a_lo.max(lo_b);
                b_hi = b_hi.min(hi_a);
            }
            _ => return Ok(true),
        }
        if a_lo > a_hi || b_lo > b_hi {
            return Ok(false);
        }
        if let Some(n) = local(a) {
            if !self.narrow(n, Range::Int(a_lo, a_hi)) {
                return Ok(false);
            }
        }
        if let Some(n) = local(b) {
            if !self.narrow(n, Range::Int(b_lo, b_hi)) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub(super) fn branch(
        &mut self,
        c: &Expr,
        truth: bool,
        body: &Expr,
    ) -> Result<Option<Range>, VirErr> {
        let saved = self.locals.clone();
        let result = if self.refine(c, truth)? { Some(self.expr(body)?) } else { None };
        self.locals = saved;
        Ok(result)
    }
}
