//! Pure callback candidates. Generated contracts are checked against the
//! original executable closure, and decoration cannot change its body or binds.
use super::*;

#[derive(Clone)]
pub(super) struct Plan {
    pub span: u64,
    pub requires: Exprs,
    pub ensures: Exprs,
}

impl Engine<'_> {
    pub(super) fn predicate(
        &mut self,
        s: &State,
        e: &Expr,
        sequence: Term,
        short_circuit: bool,
    ) -> Result<(Term, bool), VirErr> {
        let (param, pre, value) = self.callback(s, e)?;
        let mut item_dependent = false;
        crate::ast_visitor::expr_visitor_walk(&pre.expr, &mut |e| {
            item_dependent |= matches!(&e.x,ExprX::Var(n) if n==&param.name);
            crate::ast_visitor::VisitorControlFlow::<()>::Recurse
        });
        if short_circuit && item_dependent {
            return Err(unsupported(
                &e.span,
                "native short-circuit search requires item-independent callback safety",
            ));
        }
        let item = iterators::sequence(&sequence.expr.typ)
            .ok_or_else(|| unsupported(&e.span, "predicate sequence type"))?
            .clone();
        let name = VarIdent(
            Arc::new(format!("__verus_infer_native_item_{}", e.span.id)),
            VarIdentDisambiguate::AirLocal,
        );
        let input = atom(SpannedTyped::new(&e.span, &item, ExprX::Var(name.clone())))?;
        let mut env = s.clone();
        env.locals.insert(param.name.clone(), Val::atom(coerce(&e.span, input, &param.a)?));
        let pred = self.formula(&mut env, &value.expr)?;
        if !is_bool(&pre, true) && !item_dependent {
            let len = self.standard(
                &e.span,
                collections::Model::SeqLen,
                vec![item.clone()],
                vec![sequence.clone()],
            )?;
            let zero = term(&e.span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
            let nonempty =
                summary::binary(&e.span, BinaryOp::Inequality(InequalityOp::Lt), zero, len)?;
            self.require(s, logical(&e.span, LogicalOp::Implies, nonempty, pre.clone())?)?;
        } else if !is_bool(&pre, true) {
            let index_name = VarIdent(
                Arc::new(format!("__verus_infer_native_index_{}", e.span.id)),
                VarIdentDisambiguate::AirLocal,
            );
            let index =
                atom(SpannedTyped::new(&e.span, &int_typ(), ExprX::Var(index_name.clone())))?;
            let entry = self.standard(
                &e.span,
                collections::Model::SeqIndex,
                vec![item.clone()],
                vec![sequence.clone(), index.clone()],
            )?;
            env.locals.insert(param.name.clone(), Val::atom(coerce(&e.span, entry, &param.a)?));
            let needed = self.formula(&mut env, &pre.expr)?;
            let len = self.standard(
                &e.span,
                collections::Model::SeqLen,
                vec![item.clone()],
                vec![sequence.clone()],
            )?;
            let zero = term(&e.span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?;
            let bounds = logical(
                &e.span,
                LogicalOp::And,
                summary::binary(
                    &e.span,
                    BinaryOp::Inequality(InequalityOp::Le),
                    zero,
                    index.clone(),
                )?,
                summary::binary(&e.span, BinaryOp::Inequality(InequalityOp::Lt), index, len)?,
            )?;
            let condition = logical(&e.span, LogicalOp::Implies, bounds, needed)?;
            let requirement = term(
                &e.span,
                &bool_typ(),
                ExprX::Quant(
                    Quant { quant: air::ast::Quant::Forall },
                    Arc::new(vec![Arc::new(VarBinderX { name: index_name, a: int_typ() })]),
                    condition.expr,
                ),
                condition.size + 1,
            )?;
            self.require(s, requirement)?;
        }
        let lambda = term(
            &e.span,
            &Arc::new(TypX::SpecFn(Arc::new(vec![item.clone()]), bool_typ())),
            ExprX::Closure(
                Arc::new(vec![Arc::new(VarBinderX { name, a: item.clone() })]),
                pred.expr,
            ),
            pred.size + 1,
        )?;
        Ok((
            self.lib(&e.span, Model::SeqFilter, vec![item], vec![sequence, lambda])?,
            !is_bool(&pre, true),
        ))
    }
    pub(super) fn callback(
        &mut self,
        s: &State,
        e: &Expr,
    ) -> Result<(VarBinder<Typ>, Term, Term), VirErr> {
        let (param, pre, value) = self.callback_value(s, e, true)?;
        Ok((param, pre, value.scalar(&e.span)?))
    }
    pub(super) fn callback_value(
        &mut self,
        s: &State,
        e: &Expr,
        boolean_only: bool,
    ) -> Result<(VarBinder<Typ>, Term, Val), VirErr> {
        let mut e = e;
        loop {
            match &e.x {
                ExprX::ReadPlace(p, _) => {
                    if let PlaceX::Temporary(inner) = &p.x {
                        e = inner;
                        continue;
                    }
                }
                ExprX::Block(stmts, Some(inner)) if stmts.is_empty() => {
                    e = inner;
                    continue;
                }
                _ => {}
            }
            break;
        }
        let ExprX::NonSpecClosure {
            params,
            proof_fn_modes: None,
            body,
            requires,
            ensures,
            ret,
            external_spec: None,
        } = &e.x
        else {
            return Err(unsupported(
                &e.span,
                "inferred callbacks require a direct executable closure",
            ));
        };
        if params.len() != 1
            || boolean_only && !matches!(&*ret.a, TypX::Bool)
            || !requires.is_empty()
            || !ensures.is_empty()
            || self.callbacks.iter().any(|p| p.span == e.span.id)
        {
            return Err(unsupported(
                &e.span,
                "inferred callbacks require one immutable value and no handwritten contracts",
            ));
        }
        validate_body(body, &ret.a, self.all, boolean_only)?;
        // Callback propagation/returns belong to this closure, not its enclosing
        // executable function. Keep all candidate path state and exits local.
        let mut fx = self.f.x.clone();
        let mut return_param = fx.inner_ret.x.clone();
        return_param.name = ret.name.clone();
        return_param.typ = ret.a.clone();
        fx.inner_ret = fx.inner_ret.new_x(return_param);
        fx.outer_ret = fx.inner_ret.clone();
        let callback_function = self.f.new_x(fx);
        let mut engine = Engine {
            f: &callback_function,
            all: self.all,
            ds: self.ds,
            requirements: vec![],
            exits: vec![],
            plans: vec![],
            callbacks: vec![],
            remaining: self.remaining,
        };
        let mut local = s.clone();
        local.active = boolean(&e.span, true);
        let p = &params[0];
        local.locals.insert(
            p.name.clone(),
            engine.layout(SpannedTyped::new(&e.span, &p.a, ExprX::Var(p.name.clone())), 8)?,
        );
        let mut value = engine.expr(&mut local, body)?;
        for (condition, exit) in engine.exits.iter().rev() {
            value = choose(&e.span, condition.clone(), exit.clone(), value)?;
        }
        self.remaining = engine.remaining;
        let reqs = engine.requirements;
        let mut pre = boolean(&e.span, true);
        for req in reqs {
            pre = logical(&e.span, LogicalOp::And, pre, req)?;
        }
        let post =
            self.post(&value, &SpannedTyped::new(&e.span, &ret.a, ExprX::Var(ret.name.clone())))?;
        pure_contract(&pre.expr)?;
        pure_contract(&post.expr)?;
        self.callbacks.push(Plan {
            span: e.span.id,
            requires: Arc::new(vec![pre.expr.clone()]),
            ensures: Arc::new(vec![post.expr]),
        });
        Ok((p.clone(), pre, value))
    }
}

pub(super) fn annotate(f: &Function, body: &Expr, plans: &[Plan]) -> Result<Expr, VirErr> {
    let out = crate::ast_visitor::map_expr_visitor(body, &|e| {
        if let ExprX::NonSpecClosure {
            params,
            proof_fn_modes,
            body,
            requires,
            ensures,
            ret,
            external_spec,
        } = &e.x
        {
            if let Some(p) = plans.iter().find(|p| p.span == e.span.id) {
                if !requires.is_empty()
                    || !ensures.is_empty()
                    || proof_fn_modes.is_some()
                    || external_spec.is_some()
                {
                    return Err(unsupported(&e.span, "callback decoration source"));
                }
                return Ok(e.new_x(ExprX::NonSpecClosure {
                    params: params.clone(),
                    proof_fn_modes: None,
                    body: body.clone(),
                    requires: p.requires.clone(),
                    ensures: p.ensures.clone(),
                    ret: ret.clone(),
                    external_spec: None,
                }));
            }
        }
        Ok(e.clone())
    })?;
    validate(f, &out)?;
    Ok(out)
}

pub(super) fn strip(body: &Expr) -> Result<Expr, VirErr> {
    crate::ast_visitor::map_expr_visitor(body, &|e| {
        if let ExprX::NonSpecClosure {
            params,
            proof_fn_modes,
            body,
            requires,
            ensures,
            ret,
            external_spec,
        } = &e.x
        {
            if !requires.is_empty() || !ensures.is_empty() {
                if requires.len() != 1
                    || ensures.len() != 1
                    || proof_fn_modes.is_some()
                    || external_spec.is_some()
                {
                    return Err(unsupported(&e.span, "callback decoration shape"));
                }
                pure_contract(&requires[0])?;
                pure_contract(&ensures[0])?;
                return Ok(e.new_x(ExprX::NonSpecClosure {
                    params: params.clone(),
                    proof_fn_modes: None,
                    body: body.clone(),
                    requires: Arc::new(vec![]),
                    ensures: Arc::new(vec![]),
                    ret: ret.clone(),
                    external_spec: None,
                }));
            }
        }
        Ok(e.clone())
    })
}

fn pure_contract(e: &Expr) -> Result<(), VirErr> {
    loops::pure(e)?;
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        if matches!(&e.x, ExprX::Quant(..) | ExprX::Closure(..)) {
            Err(unsupported(&e.span, "callback contracts require pure expressions without binders"))
        } else {
            Ok(())
        }
    })
}
pub(super) fn validate(f: &Function, body: &Expr) -> Result<(), VirErr> {
    loops::validate_decoration(f, &strip(body)?)
}
pub(super) fn plans(body: &Expr) -> Vec<Plan> {
    let mut plans = vec![];
    crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
        if let ExprX::NonSpecClosure { requires, ensures, .. } = &e.x {
            if !requires.is_empty() || !ensures.is_empty() {
                plans.push(Plan {
                    span: e.span.id,
                    requires: requires.clone(),
                    ensures: ensures.clone(),
                });
            }
        }
        crate::ast_visitor::VisitorControlFlow::<()>::Recurse
    });
    plans
}

fn validate_body(
    body: &Expr,
    ret: &Typ,
    all: &HashMap<Fun, Function>,
    boolean_only: bool,
) -> Result<(), VirErr> {
    use crate::ast_visitor::{VisitorControlFlow, VisitorScopeMap, expr_visitor_dfs};
    let walked = expr_visitor_dfs(body, &mut VisitorScopeMap::new(), &mut |_, e| {
        match propagation::recognize(e, all, ret) {
            Ok(Some(p)) => {
                return match validate_body(&p.operand, ret, all, boolean_only) {
                    Ok(()) => VisitorControlFlow::Return,
                    Err(err) => VisitorControlFlow::Stop(err),
                };
            }
            Err(err) => return VisitorControlFlow::Stop(err),
            Ok(None) => {}
        }
        if matches!(
            e.x,
            ExprX::Assign { .. }
                | ExprX::BorrowMut(_)
                | ExprX::TwoPhaseBorrowMut(_)
                | ExprX::Loop { .. }
                | ExprX::Return(_)
                | ExprX::NonSpecClosure { .. }
                | ExprX::Ghost { .. }
                | ExprX::AssertAssume { .. }
                | ExprX::AssertBy { .. }
        ) {
            return VisitorControlFlow::Stop(unsupported(
                &e.span,
                "inferred callbacks must be immutable expressions",
            ));
        }
        if let ExprX::Call { target: CallTarget::Fun(_, n, ..), .. } = &e.x {
            if boolean_only
                || all.get(n).is_some_and(|f| {
                    f.x.params.iter().any(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
                })
            {
                return VisitorControlFlow::Stop(unsupported(
                    &e.span,
                    "mapped callbacks require immutable calls",
                ));
            }
        }
        VisitorControlFlow::Recurse
    });
    match walked {
        VisitorControlFlow::Stop(err) => Err(err),
        _ => Ok(()),
    }
}
