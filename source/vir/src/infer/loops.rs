//! Untrusted inference for closed unsigned counted-loop families. Proof annotations
//! are checked by ordinary Verus; installation separately preserves source code.
use super::*;

pub(super) struct CountedLoop {
    pub initial: Param,
    pub bound: Param,
    pub accumulator: VarIdent,
    pub counter: VarIdent,
    pub typ: Typ,
    pub range: IntRange,
    step: Term,
    subtract: bool,
    countdown: bool,
    pub nonlinear: bool,
    isolation: bool,
}

fn read(expr: &Expr) -> Option<&VarIdent> {
    match &expr.x {
        ExprX::Var(v) => Some(v),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(v) => Some(v),
            _ => None,
        },
        _ => None,
    }
}
fn local(place: &Place) -> Option<&VarIdent> {
    if let PlaceX::Local(v) = &place.x { Some(v) } else { None }
}
fn constant(expr: &Expr, n: u32) -> bool {
    matches!(&expr.x, ExprX::Const(Constant::Int(value)) if value == &BigInt::from(n))
}

fn declaration(stmt: &Stmt, typ: &Typ) -> Option<(VarIdent, Place)> {
    let StmtX::Decl {
        pattern,
        mode: None | Some((Mode::Exec, DeclProph::Default)),
        init: Some(init),
        els: None,
        assert_irrefutable: false,
    } = &stmt.x
    else {
        return None;
    };
    let PatternX::Var(binding) = &pattern.x else { return None };
    (binding.user_mut == Some(true)
        && matches!(binding.by_ref, ByRef::No)
        && binding.copy
        && crate::ast_util::types_equal(&binding.typ, typ)
        && crate::ast_util::types_equal(&pattern.typ, typ)
        && crate::ast_util::types_equal(&init.typ, typ))
    .then(|| (binding.name.clone(), init.clone()))
}

/// Return the immutable operand and direction of a checked add/sub assignment.
fn update(stmt: &Stmt, name: &VarIdent, typ: &Typ, range: IntRange) -> Option<(bool, Expr)> {
    let StmtX::Expr(e) = &stmt.x else { return None };
    let ExprX::Assign { place, rhs, op, typ: assigned, resolve: false } = &e.x else { return None };
    if local(place) != Some(name)
        || !crate::ast_util::types_equal(&place.typ, typ)
        || !crate::ast_util::types_equal(assigned, typ)
        || !crate::ast_util::types_equal(&rhs.typ, typ)
    {
        return None;
    }
    let (op, operand) = if let Some(op) = op {
        (*op, rhs.clone())
    } else {
        let ExprX::Binary(op, a, b) = &rhs.x else { return None };
        if read(a) != Some(name)
            || !crate::ast_util::types_equal(&a.typ, typ)
            || !crate::ast_util::types_equal(&b.typ, typ)
        {
            return None;
        }
        (*op, b.clone())
    };
    let subtract = match op {
        BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(r))) if r == range => false,
        BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Error(r))) if r == range => true,
        _ => return None,
    };
    Some((subtract, operand))
}

pub(super) fn recognize(function: &Function) -> Result<Option<CountedLoop>, VirErr> {
    let Some(body) = &function.x.body else { return Ok(None) };
    let mut found = false;
    let mut budget = FORMULA_BUDGET;
    crate::ast_visitor::expr_visitor_check(body, &mut |_, e| {
        if budget == 0 {
            return Err(error(
                &e.span,
                "Verus Infer incomplete: loop source traversal budget exceeded",
            ));
        }
        budget -= 1;
        found |= matches!(e.x, ExprX::Loop { .. });
        Ok::<_, VirErr>(())
    })?;
    if !found {
        return Ok(None);
    }
    let invalid = || {
        unsupported(
            &function.span,
            "loops currently require one counted unsigned accumulator loop with two mutable locals, an immutable step, a unit ascending/countdown counter, and no handwritten loop proofs",
        )
    };
    let f = &function.x;
    if f.attrs.infer_allow_approx
        || f.attrs.exec_assume_termination
        || f.attrs.exec_allows_no_decreases_clause
        || !(2..=3).contains(&f.params.len())
    {
        return Err(invalid());
    }
    let typ = f.inner_ret.x.typ.clone();
    let TypX::Int(range @ (IntRange::USize | IntRange::U(8 | 16 | 32 | 64 | 128))) = &*typ else {
        return Err(invalid());
    };
    if f.params.iter().any(|p| !crate::ast_util::types_equal(&p.x.typ, &typ)) {
        return Err(invalid());
    }
    let ExprX::Block(stmts, Some(tail)) = &body.x else { return Err(invalid()) };
    if stmts.len() != 3 {
        return Err(invalid());
    }
    let first = declaration(&stmts[0], &typ).ok_or_else(invalid)?;
    let second = declaration(&stmts[1], &typ).ok_or_else(invalid)?;
    let StmtX::Expr(loop_expr) = &stmts[2].x else { return Err(invalid()) };
    let ExprX::Loop {
        loop_isolation,
        allow_complex_invariants: false,
        is_for_loop: false,
        assume_termination: false,
        label,
        cond: Some(cond),
        body: loop_body,
        invs,
        decrease,
        atomic_call: false,
    } = &loop_expr.x
    else {
        return Err(invalid());
    };
    if !invs.is_empty() || !decrease.is_empty() || label.name.is_some() {
        return Err(invalid());
    }
    let (counter, bound_expr, countdown) = match &cond.x {
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), i, n)
            if crate::ast_util::types_equal(&i.typ, &typ) =>
        {
            (read(i).ok_or_else(invalid)?, Some(n), false)
        }
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Gt), i, zero)
            if constant(zero, 0)
                && crate::ast_util::types_equal(&zero.typ, &typ)
                && crate::ast_util::types_equal(&i.typ, &typ) =>
        {
            (read(i).ok_or_else(invalid)?, None, true)
        }
        _ => return Err(invalid()),
    };
    if !matches!(&*cond.typ, TypX::Bool) {
        return Err(invalid());
    }
    let ((accumulator, init), (counter, counter_init)) = if &second.0 == counter {
        (first, second)
    } else if &first.0 == counter {
        (second, first)
    } else {
        return Err(invalid());
    };
    let bound = if countdown {
        f.params.iter().find(|p| Some(&p.x.name) == local(&counter_init))
    } else {
        if !matches!(&counter_init.x,PlaceX::Temporary(e) if constant(e,0)
            && crate::ast_util::types_equal(&e.typ,&typ))
        {
            return Err(invalid());
        }
        let n = bound_expr.unwrap();
        if !crate::ast_util::types_equal(&n.typ, &typ) {
            return Err(invalid());
        }
        f.params.iter().find(|p| Some(&p.x.name) == read(n))
    }
    .ok_or_else(invalid)?
    .clone();
    let initial =
        f.params.iter().find(|p| Some(&p.x.name) == local(&init)).ok_or_else(invalid)?.clone();
    if initial.x.name == bound.x.name
        || accumulator == counter
        || f.params.iter().any(|p| p.x.name == accumulator || p.x.name == counter)
        || read(tail) != Some(&accumulator)
        || !crate::ast_util::types_equal(&tail.typ, &typ)
    {
        return Err(invalid());
    }
    let ExprX::Block(updates, None) = &loop_body.x else { return Err(invalid()) };
    if updates.len() != 2 {
        return Err(invalid());
    }
    let (subtract, operand) =
        update(&updates[0], &accumulator, &typ, *range).ok_or_else(invalid)?;
    let (counter_sub, counter_step) =
        update(&updates[1], &counter, &typ, *range).ok_or_else(invalid)?;
    if counter_sub != countdown || !constant(&counter_step, 1) {
        return Err(invalid());
    }
    let mut used = std::collections::HashSet::from([initial.x.name.clone(), bound.x.name.clone()]);
    let (step, nonlinear) =
        if let Some(param) = f.params.iter().find(|p| Some(&p.x.name) == read(&operand)) {
            used.insert(param.x.name.clone());
            (
                Term {
                    view: None,
                    expr: SpannedTyped::new(&operand.span, &typ, ExprX::Var(param.x.name.clone())),
                    size: 1,
                },
                true,
            )
        } else if let ExprX::Const(Constant::Int(n)) = &operand.x {
            if n < &BigInt::from(0) || n.bits() > 128 {
                return Err(invalid());
            }
            (term(&operand.span, &int_typ(), ExprX::Const(Constant::Int(n.clone())), 1)?, false)
        } else if *range == IntRange::USize {
            let Some(value) = arithmetic::arch_constant(&operand) else { return Err(invalid()) };
            (value, true)
        } else {
            return Err(invalid());
        };
    if used.len() != f.params.len() {
        return Err(invalid());
    }
    Ok(Some(CountedLoop {
        initial,
        bound,
        accumulator,
        counter,
        typ: typ.clone(),
        range: *range,
        step,
        subtract,
        countdown,
        nonlinear,
        isolation: *loop_isolation,
    }))
}

impl CountedLoop {
    fn variable(&self, span: &Span, name: &VarIdent) -> Term {
        Term {
            view: None,
            expr: SpannedTyped::new(span, &self.typ, ExprX::Var(name.clone())),
            size: 1,
        }
    }
    fn product(&self, span: &Span, count: Term) -> Result<Term, VirErr> {
        if matches!(&self.step.expr.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::from(1)) {
            return Ok(count);
        }
        if matches!(&self.step.expr.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::from(0)) {
            return Ok(self.step.clone());
        }
        summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Mul(OverflowBehavior::Allow)),
            count,
            self.step.clone(),
        )
    }
    fn result(&self, span: &Span, amount: Term) -> Result<Term, VirErr> {
        summary::binary(
            span,
            BinaryOp::Arith(if self.subtract {
                ArithOp::Sub(OverflowBehavior::Allow)
            } else {
                ArithOp::Add(OverflowBehavior::Allow)
            }),
            self.variable(span, &self.initial.x.name),
            amount,
        )
    }
    pub(super) fn contract(&self, f: &Function) -> Result<Contract, VirErr> {
        let span = &f.span;
        let amount = self.product(span, self.variable(span, &self.bound.x.name))?;
        let result = self.result(span, amount.clone())?;
        let pre = if self.subtract {
            summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                amount,
                self.variable(span, &self.initial.x.name),
            )?
        } else {
            summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                result.clone(),
                arithmetic::maximum(span, self.range)?,
            )?
        };
        let post = summary::binary(
            span,
            BinaryOp::Eq(Mode::Spec),
            self.variable(span, &f.x.inner_ret.x.name),
            result,
        )?;
        Contract::new(pre.expr, post.expr)
    }
    fn annotations(&self, span: &Span) -> Result<(LoopInvariants, Exprs), VirErr> {
        let i = self.variable(span, &self.counter);
        let n = self.variable(span, &self.bound.x.name);
        let upper =
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), i.clone(), n.clone())?;
        let difference = summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
            n,
            i.clone(),
        )?;
        let (done, measure) = if self.countdown { (difference, i) } else { (i, difference) };
        let result = self.result(span, self.product(span, done)?)?;
        let relation = summary::binary(
            span,
            BinaryOp::Eq(Mode::Spec),
            self.variable(span, &self.accumulator),
            result,
        )?;
        Ok((
            Arc::new(
                vec![upper, relation]
                    .into_iter()
                    .map(|v| LoopInvariant {
                        kind: LoopInvariantKind::InvariantAndEnsures,
                        inv: v.expr,
                    })
                    .collect(),
            ),
            Arc::new(vec![measure.expr]),
        ))
    }
}

/// This boundary can change proof annotations only. The source projection is
/// checked in production, independently of the synthesizer's proposed formulas.
pub(super) fn annotate(original: &Function) -> Result<Option<Expr>, VirErr> {
    let Some(plan) = recognize(original)? else { return Ok(original.x.body.clone()) };
    let body = original.x.body.as_ref().unwrap();
    let annotated = crate::ast_visitor::map_expr_visitor(body, &|e| {
        if let ExprX::Loop {
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            atomic_call,
            ..
        } = &e.x
        {
            let (invs, decrease) = plan.annotations(&e.span)?;
            Ok(e.new_x(ExprX::Loop {
                loop_isolation: false,
                allow_complex_invariants: *allow_complex_invariants,
                is_for_loop: *is_for_loop,
                assume_termination: *assume_termination,
                label: label.clone(),
                cond: cond.clone(),
                body: body.clone(),
                invs,
                decrease,
                atomic_call: *atomic_call,
            }))
        } else {
            Ok(e.clone())
        }
    })?;
    validate_decoration(original, &annotated, &plan)?;
    Ok(Some(annotated))
}

fn validate_decoration(
    original: &Function,
    annotated: &Expr,
    plan: &CountedLoop,
) -> Result<(), VirErr> {
    let invalid = || {
        error(
            &original.span,
            "Verus Infer: loop proof decoration changed executable code or contains invalid annotations",
        )
    };
    let variables = original
        .x
        .params
        .iter()
        .map(|p| &p.x.name)
        .chain([&plan.counter, &plan.accumulator])
        .map(|v| (v.clone(), plan.variable(&original.span, v)))
        .collect();
    let projected = crate::ast_visitor::map_expr_visitor(annotated, &|e| {
        if let ExprX::Loop {
            loop_isolation: false,
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            invs,
            decrease,
            atomic_call,
        } = &e.x
        {
            if invs.len() != 2 || decrease.len() != 1 {
                return Err(invalid());
            }
            for inv in invs.iter() {
                if inv.kind != LoopInvariantKind::InvariantAndEnsures
                    || !matches!(&*instantiate(&inv.inv, &variables)?.expr.typ, TypX::Bool)
                {
                    return Err(invalid());
                }
            }
            if !eligibility::integer(&instantiate(&decrease[0], &variables)?.expr.typ) {
                return Err(invalid());
            }
            Ok(e.new_x(ExprX::Loop {
                loop_isolation: plan.isolation,
                allow_complex_invariants: *allow_complex_invariants,
                is_for_loop: *is_for_loop,
                assume_termination: *assume_termination,
                label: label.clone(),
                cond: cond.clone(),
                body: body.clone(),
                invs: Arc::new(vec![]),
                decrease: Arc::new(vec![]),
                atomic_call: *atomic_call,
            }))
        } else {
            Ok(e.clone())
        }
    })?;
    let encode = |e: &Expr| serde_json::to_vec(e).map_err(|_| invalid());
    if encode(&projected)? != encode(original.x.body.as_ref().unwrap())? {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod tests;
