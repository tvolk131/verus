//! Candidate-only pointwise prefix traversals of a borrowed record collection.
//! The original body, safety checks, loop induction and termination are checked.
use super::*;

fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Temporary(e) => plain(e),
            _ => e,
        },
        ExprX::Block(ss, Some(e)) if ss.is_empty() => plain(e),
        ExprX::UnaryOpr(UnaryOpr::Field(op), base)
            if op.check == VariantCheck::None && !op.get_variant =>
        {
            if let ExprX::Ctor(dt, variant, fields, None) = &plain(base).x {
                if dt == &op.datatype && variant == &op.variant {
                    if let Some(field) = fields.iter().find(|f| f.name == op.field) {
                        return plain(&field.a);
                    }
                }
            }
            e
        }
        _ => e,
    }
}
fn local(e: &Expr) -> Option<&VarIdent> {
    match &plain(e).x {
        ExprX::Var(n) => Some(n),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}
fn replace(e: &Expr, vars: &HashMap<VarIdent, Term>) -> Result<Term, VirErr> {
    let value = substitute_value(e, vars)?;
    atom(crate::ast_visitor::map_expr_visitor(&value.expr, &|e| Ok(plain(e).clone()))?)
}
fn entry(e: &Expr) -> Result<Expr, VirErr> {
    crate::ast_visitor::map_expr_visitor(e, &|x| {
        if let ExprX::Old(a) = &x.x {
            if let ExprX::Var(n) = &a.x {
                return Ok(SpannedTyped::new(&x.span, &x.typ, ExprX::VarAt(n.clone(), VarAt::Pre)));
            }
            return Err(unsupported(&x.span, "pointwise snapshot requires a parameter"));
        }
        Ok(x.clone())
    })
}
fn integer(span: &Span, n: u8) -> Result<Term, VirErr> {
    term(span, &int_typ(), ExprX::Const(Constant::Int(n.into())), 1)
}
fn quantify(span: &Span, name: &VarIdent, body: Term, trigger: Term) -> Result<Term, VirErr> {
    let triggered = SpannedTyped::new(
        span,
        &bool_typ(),
        ExprX::WithTriggers {
            triggers: Arc::new(vec![Arc::new(vec![trigger.expr])]),
            body: body.expr,
        },
    );
    term(
        span,
        &bool_typ(),
        ExprX::Quant(
            Quant { quant: air::ast::Quant::Forall },
            Arc::new(vec![Arc::new(VarBinderX { name: name.clone(), a: int_typ() })]),
            triggered,
        ),
        body.size + trigger.size + 2,
    )
}

impl Interpreter<'_> {
    fn new_sequence(
        &self,
        span: &Span,
        element: &Typ,
        len: Term,
        name: &VarIdent,
        value: Term,
    ) -> Result<Term, VirErr> {
        let mut found = self.all.values().filter(|f| owned::model(f) == Some(owned::Model::SeqNew));
        let f = found
            .next()
            .ok_or_else(|| unsupported(span, "pointwise traversal requires Seq::new"))?;
        if found.next().is_some() {
            return Err(unsupported(span, "ambiguous Seq::new model"));
        }
        let lambda = term(
            span,
            &Arc::new(TypX::SpecFn(Arc::new(vec![int_typ()]), element.clone())),
            ExprX::Closure(
                Arc::new(vec![Arc::new(VarBinderX { name: name.clone(), a: int_typ() })]),
                value.expr,
            ),
            value.size + 1,
        )?;
        let substitutions = HashMap::from([(f.x.typ_params[0].clone(), element.clone())]);
        term(
            span,
            &crate::sst_util::subst_typ(&substitutions, &f.x.inner_ret.x.typ),
            ExprX::Call {
                target: CallTarget::Fun(
                    CallTargetKind::Static,
                    f.x.name.clone(),
                    Arc::new(vec![element.clone()]),
                    Arc::new(vec![]),
                    CallTargetAttrs {
                        autospec: AutospecUsage::Final,
                        const_var: false,
                        assume_external_allowed: false,
                    },
                ),
                args: Arc::new(vec![len.expr, lambda.expr]),
                post_args: None,
                body: None,
            },
            len.size + lambda.size + 1,
        )
    }

    pub(super) fn traversal(&mut self, s: &mut State, e: &Expr) -> Result<Term, VirErr> {
        let bad = || {
            unsupported(
                &e.span,
                "pointwise traversal requires one zero-based usize prefix loop, one same-index collection update, and a unit counter step",
            )
        };
        if !mutable(self.function)
            || self.function.x.attrs.infer_allow_approx
            || !self.plans.is_empty()
        {
            return Err(bad());
        }
        let ExprX::Loop {
            cond: Some(cond),
            body,
            invs,
            decrease,
            is_for_loop: false,
            assume_termination: false,
            atomic_call: false,
            ..
        } = &e.x
        else {
            return Err(bad());
        };
        if !invs.is_empty() || !decrease.is_empty() {
            return Err(bad());
        }
        let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), lhs, rhs) = &plain(cond).x else {
            return Err(bad());
        };
        let counter = local(lhs).ok_or_else(bad)?.clone();
        if !matches!(&*lhs.typ, TypX::Int(IntRange::USize))
            || !s.locals.get(&counter).is_some_and(
                |t| matches!(&t.expr.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::from(0)),
            )
        {
            return Err(bad());
        }
        crate::ast_visitor::expr_visitor_check(rhs, &mut |_, x| {
            if local(x) == Some(&counter) { Err(bad()) } else { Ok(()) }
        })?;
        let limit = self.readonly(s, rhs)?;
        let ExprX::Block(stmts, None) = &body.x else { return Err(bad()) };
        if stmts.len() != 2 {
            return Err(bad());
        }
        let (StmtX::Expr(update), StmtX::Expr(increment)) = (&stmts[0].x, &stmts[1].x) else {
            return Err(bad());
        };
        let ExprX::Assign { place, rhs: amount, op, resolve: false, .. } = &plain(increment).x
        else {
            return Err(bad());
        };
        if !matches!(&place.x,PlaceX::Local(n) if n==&counter) {
            return Err(bad());
        }
        let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::USize)));
        let amount = if *op == Some(add) {
            amount
        } else if op.is_none() {
            let ExprX::Binary(o, a, b) = &plain(amount).x else { return Err(bad()) };
            if *o != add || local(a) != Some(&counter) {
                return Err(bad());
            }
            b
        } else {
            return Err(bad());
        };
        if !matches!(&plain(amount).x,ExprX::Const(Constant::Int(n)) if n==&BigInt::one()) {
            return Err(bad());
        }

        let mut step = s.clone();
        step.active = boolean(&e.span, true);
        step.locals.insert(
            counter.clone(),
            atom(SpannedTyped::new(&e.span, &lhs.typ, ExprX::Var(counter.clone())))?,
        );
        let req_start = self.requirements.len();
        self.expr(&mut step, update)?;
        let requirements = self.requirements.drain(req_start..).collect::<Vec<_>>();
        if !is_bool(&step.active, true) {
            return Err(bad());
        }
        let changed = step
            .fields
            .iter()
            .zip(&s.fields)
            .enumerate()
            .filter(|(_, (a, b))| owned::skeleton(&a.expr) != owned::skeleton(&b.expr))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if changed.len() != 1 {
            return Err(bad());
        }
        let field = changed[0];
        let element = self.shape.fields[field].element.clone().ok_or_else(bad)?;
        let ExprX::Call { target: CallTarget::Fun(_, name, ..), args, post_args: None, body: None } =
            &plain(&step.fields[field].expr).x
        else {
            return Err(bad());
        };
        if self.all.get(name).and_then(collections::model) != Some(Model::SeqUpdate)
            || args.len() != 3
            || owned::skeleton(&args[0]) != owned::skeleton(&s.fields[field].expr)
            || local(&args[1]) != Some(&counter)
        {
            return Err(bad());
        }
        // Reads of the changed collection must use this index. Framed fields
        // may be read freely; dependencies between visited elements need
        // a different invariant family.
        for root in std::iter::once(&args[2]).chain(requirements.iter().map(|r| &r.expr)) {
            crate::ast_visitor::expr_visitor_check(root, &mut |_, x| {
                if let ExprX::Call { target: CallTarget::Fun(_, n, ..), args: a, .. } = &x.x {
                    if self.all.get(n).and_then(collections::model) == Some(Model::SeqIndex)
                        && a.len() == 2
                        && owned::skeleton(&a[0]) == owned::skeleton(&s.fields[field].expr)
                        && local(&a[1]) != Some(&counter)
                    {
                        return Err(bad());
                    }
                }
                Ok(())
            })?;
        }
        let span = &e.span;
        let binder = VarIdent(
            Arc::new("__verus_infer_pointwise_index".into()),
            VarIdentDisambiguate::AirLocal,
        );
        let j = atom(SpannedTyped::new(span, &int_typ(), ExprX::Var(binder.clone())))?;
        // Mathematical loop indices are used in the summary. The quantified
        // range and original usize arithmetic checks establish their bounds.
        let vars = HashMap::from([(counter.clone(), j.clone())]);
        let value = replace(&args[2], &vars)?;
        let len = self.len(field)?;
        let index_bound =
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), j.clone(), len.clone())?;
        let index_bound = replace(&index_bound.expr, &HashMap::new())?;
        let mut req = boolean(span, true);
        fn clauses(e: &Expr, out: &mut Vec<Expr>) {
            if let ExprX::Logical(LogicalOp::And, a, b) = &e.x {
                clauses(a, out);
                clauses(b, out);
            } else {
                out.push(e.clone());
            }
        }
        for r in requirements {
            let r = replace(&r.expr, &vars)?;
            let mut parts = vec![];
            clauses(&r.expr, &mut parts);
            for p in parts {
                if owned::skeleton(&p) != owned::skeleton(&index_bound.expr) {
                    req = logical(span, LogicalOp::And, req, atom(p)?)?;
                }
            }
        }
        let original = self.model_call(
            span,
            Model::SeqIndex,
            vec![element.clone()],
            vec![s.fields[field].clone(), j.clone()],
        )?;
        let prefix_req = |end: Term| -> Result<Term, VirErr> {
            if is_bool(&req, true) {
                return Ok(req.clone());
            }
            let mut mentions_index = false;
            crate::ast_visitor::expr_visitor_walk(&req.expr, &mut |x| {
                mentions_index |= matches!(&x.x,ExprX::Var(n) if n==&binder);
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
            if !mentions_index {
                return logical(
                    span,
                    LogicalOp::Implies,
                    summary::binary(
                        span,
                        BinaryOp::Inequality(InequalityOp::Lt),
                        integer(span, 0)?,
                        end,
                    )?,
                    req.clone(),
                );
            }
            let bounds = logical(
                span,
                LogicalOp::And,
                summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Le),
                    integer(span, 0)?,
                    j.clone(),
                )?,
                summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), j.clone(), end)?,
            )?;
            quantify(
                span,
                &binder,
                logical(span, LogicalOp::Implies, bounds, req.clone())?,
                original.clone(),
            )
        };
        self.require(
            s,
            span,
            summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                limit.clone(),
                len.clone(),
            )?,
        )?;
        self.require(s, span, prefix_req(limit.clone())?)?;
        let mapped = |engine: &Self, end: Term| -> Result<Term, VirErr> {
            let value = conditional(
                span,
                summary::binary(span, BinaryOp::Inequality(InequalityOp::Lt), j.clone(), end)?,
                value.clone(),
                original.clone(),
            )?;
            engine.new_sequence(span, &element, len.clone(), &binder, value)
        };
        let i = atom(SpannedTyped::new(span, &lhs.typ, ExprX::Var(counter.clone())))?;
        let current = |engine: &Self, field: usize| -> Result<Term, VirErr> {
            let value = if engine.shape.fields[field].element.is_some() {
                engine.view(field, false)?
            } else {
                field_value(engine.function, &engine.shape, &engine.shape.fields[field], false)?
            };
            atom(crate::ast_visitor::map_expr_visitor(&value.expr, &|x| {
                Ok(match &x.x {
                    ExprX::Old(a) => a.clone(),
                    _ => x.clone(),
                })
            })?)
        };
        let mut bounds = logical(
            span,
            LogicalOp::And,
            summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                i.clone(),
                limit.clone(),
            )?,
            summary::binary(span, BinaryOp::Inequality(InequalityOp::Le), i.clone(), len.clone())?,
        )?;
        for other in 0..self.shape.fields.len() {
            if other != field {
                bounds = logical(
                    span,
                    LogicalOp::And,
                    bounds,
                    equal(span, current(self, other)?, s.fields[other].clone())?,
                )?;
            }
        }
        let contents = logical(
            span,
            LogicalOp::And,
            equal(span, current(self, field)?, mapped(self, i.clone())?)?,
            prefix_req(i.clone())?,
        )?;
        let mut fresh_id = u64::MAX - 1;
        let mut fresh = vec![];
        for inv in [bounds, contents] {
            // `Old` alone is bookkeeping; installed body annotations must
            // explicitly select the function-entry snapshot before loop havoc.
            let inv = entry(&inv.expr)?;
            let inv = crate::ast_visitor::map_expr_visitor_env(
                &inv,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut fresh_id,
                &|id, _, x| {
                    let mut span = x.span.clone();
                    span.id = *id;
                    *id -= 1;
                    Ok(SpannedTyped::new(&span, &x.typ, x.x.clone()))
                },
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| Ok(t.clone()),
                &|id, _, p| {
                    let mut span = p.span.clone();
                    span.id = *id;
                    *id -= 1;
                    Ok(SpannedTyped::new(&span, &p.typ, p.x.clone()))
                },
                &|_, _, p| Ok(p.clone()),
            )?;
            fresh.push(LoopInvariant { kind: LoopInvariantKind::InvariantAndEnsures, inv });
        }
        let measure = summary::binary(
            span,
            BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
            limit.clone(),
            i,
        )?;
        let measure = entry(&measure.expr)?;
        let measure = crate::ast_visitor::map_expr_visitor_env(
            &measure,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut fresh_id,
            &|id, _, x| {
                let mut span = x.span.clone();
                span.id = *id;
                *id -= 1;
                Ok(SpannedTyped::new(&span, &x.typ, x.x.clone()))
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| Ok(t.clone()),
            &|id, _, p| {
                let mut span = p.span.clone();
                span.id = *id;
                *id -= 1;
                Ok(SpannedTyped::new(&span, &p.typ, p.x.clone()))
            },
            &|_, _, p| Ok(p.clone()),
        )?;
        self.plans.push(owned::loops::Plan {
            span: e.span.id,
            invs: Arc::new(fresh),
            decrease: Arc::new(vec![measure]),
        });
        s.fields[field] = mapped(self, limit.clone())?;
        s.locals.insert(counter, limit);
        exits::unit_value(span)
    }
}
