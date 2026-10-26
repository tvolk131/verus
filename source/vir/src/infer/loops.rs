//! Untrusted inference for one closed counted-loop pattern. Proof annotations
//! are checked by ordinary Verus; installation separately preserves source code.
use super::*;

pub(super) struct CountedLoop {
    pub initial: Param,
    pub bound: Param,
    pub accumulator: VarIdent,
    pub counter: VarIdent,
    pub typ: Typ,
    pub bits: u32,
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

fn increment(stmt: &Stmt, name: &VarIdent, typ: &Typ, bits: u32) -> bool {
    let StmtX::Expr(e) = &stmt.x else { return false };
    let ExprX::Assign { place, rhs, op, typ: assigned, resolve: false } = &e.x else {
        return false;
    };
    if local(place) != Some(name)
        || !crate::ast_util::types_equal(&place.typ, typ)
        || !crate::ast_util::types_equal(assigned, typ)
    {
        return false;
    }
    let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::U(bits))));
    match (op, &rhs.x) {
        (Some(op), _) => {
            *op == add && constant(rhs, 1) && crate::ast_util::types_equal(&rhs.typ, typ)
        }
        (None, ExprX::Binary(op, a, b)) => {
            *op == add
                && read(a) == Some(name)
                && constant(b, 1)
                && crate::ast_util::types_equal(&a.typ, typ)
                && crate::ast_util::types_equal(&b.typ, typ)
                && crate::ast_util::types_equal(&rhs.typ, typ)
        }
        _ => false,
    }
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
            "loops currently require one counted unsigned accumulator loop with two mutable locals, unit increments, and no handwritten loop proofs",
        )
    };
    let f = &function.x;
    if f.attrs.infer_allow_approx
        || f.attrs.exec_assume_termination
        || f.attrs.exec_allows_no_decreases_clause
        || f.params.len() != 2
    {
        return Err(invalid());
    }
    let typ = f.inner_ret.x.typ.clone();
    let TypX::Int(IntRange::U(bits @ (8 | 16 | 32 | 64 | 128))) = &*typ else {
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
    let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), i, n) = &cond.x else {
        return Err(invalid());
    };
    if !matches!(&*cond.typ, TypX::Bool)
        || !crate::ast_util::types_equal(&i.typ, &typ)
        || !crate::ast_util::types_equal(&n.typ, &typ)
    {
        return Err(invalid());
    }
    let counter = read(i).ok_or_else(invalid)?;
    let bound = f.params.iter().find(|p| Some(&p.x.name) == read(n)).ok_or_else(invalid)?.clone();
    let ((accumulator, init), (counter, zero)) = if &second.0 == counter {
        (first, second)
    } else if &first.0 == counter {
        (second, first)
    } else {
        return Err(invalid());
    };
    let initial =
        f.params.iter().find(|p| Some(&p.x.name) == local(&init)).ok_or_else(invalid)?.clone();
    if initial.x.name == bound.x.name
        || accumulator == counter
        || f.params.iter().any(|p| p.x.name == accumulator || p.x.name == counter)
        || read(tail) != Some(&accumulator)
        || !crate::ast_util::types_equal(&tail.typ, &typ)
        || !matches!(&zero.x, PlaceX::Temporary(e) if constant(e, 0) && crate::ast_util::types_equal(&e.typ, &typ))
    {
        return Err(invalid());
    }
    let ExprX::Block(updates, None) = &loop_body.x else { return Err(invalid()) };
    if updates.len() != 2
        || !increment(&updates[0], &accumulator, &typ, *bits)
        || !increment(&updates[1], &counter, &typ, *bits)
    {
        return Err(invalid());
    }
    Ok(Some(CountedLoop {
        initial,
        bound,
        accumulator,
        counter,
        typ: typ.clone(),
        bits: *bits,
        isolation: *loop_isolation,
    }))
}

impl CountedLoop {
    fn variable(&self, span: &Span, name: &VarIdent) -> Term {
        Term { expr: SpannedTyped::new(span, &self.typ, ExprX::Var(name.clone())), size: 1 }
    }
    pub(super) fn contract(&self, f: &Function) -> Result<Contract, VirErr> {
        let span = &f.span;
        let sum = summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
            self.variable(span, &self.initial.x.name),
            self.variable(span, &self.bound.x.name),
        )?;
        let (_, max) = arithmetic::bounds(IntRange::U(self.bits)).unwrap();
        let max = term(span, &int_typ(), ExprX::Const(Constant::Int(max)), 1)?;
        let pre = summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), sum.clone(), max)?;
        let post = summary::binary(
            span,
            BinaryOp::Eq(Mode::Spec),
            self.variable(span, &f.x.inner_ret.x.name),
            sum,
        )?;
        Contract::new(pre.expr, post.expr)
    }
    fn annotations(&self, span: &Span) -> Result<(LoopInvariants, Exprs), VirErr> {
        let i = self.variable(span, &self.counter);
        let n = self.variable(span, &self.bound.x.name);
        let upper =
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), i.clone(), n.clone())?;
        let sum = summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
            self.variable(span, &self.initial.x.name),
            i.clone(),
        )?;
        let relation = summary::binary(
            span,
            BinaryOp::Eq(Mode::Spec),
            self.variable(span, &self.accumulator),
            sum,
        )?;
        let measure =
            summary::binary(span, BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)), n, i)?;
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
    let variables = [&plan.initial.x.name, &plan.bound.x.name, &plan.counter, &plan.accumulator]
        .into_iter()
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
