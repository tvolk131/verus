//! Checked proof decoration for the closed fill/copy loop families.
use super::*;
#[derive(Clone)]
pub(super) struct Plan {
    pub span: u64,
    pub invs: LoopInvariants,
    pub decrease: Exprs,
}
fn local(e: &Expr) -> Option<&VarIdent> {
    match &e.x {
        ExprX::Var(n) => Some(n),
        ExprX::ReadPlace(p, _) => place_local(p),
        _ => None,
    }
}
fn place_local(p: &Place) -> Option<&VarIdent> {
    match &p.x {
        PlaceX::Local(n) => Some(n),
        PlaceX::Temporary(e) => local(e),
        _ => None,
    }
}
fn borrow_local(e: &Expr) -> Option<&VarIdent> {
    match &e.x {
        ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => place_local(p),
        _ => None,
    }
}
fn unwrap(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::ReadPlace(p, _) => {
            if let PlaceX::Temporary(e) = &p.x {
                unwrap(e)
            } else {
                e
            }
        }
        ExprX::Block(ss, Some(t)) if ss.is_empty() => unwrap(t),
        _ => e,
    }
}
fn zero(v: &Val) -> bool {
    matches!(&v.node,Node::Atom(t) if matches!(&t.expr.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::from(0)))
}
impl Engine<'_> {
    pub(super) fn loop_expr(&mut self, s: &mut State, e: &Expr) -> Result<Val, VirErr> {
        let bad = || {
            unsupported(&e.span, "owned loops require one empty-vector fill or prefix-copy pattern")
        };
        if !self.plans.is_empty() {
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
        let ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), lhs, rhs) = &cond.x else {
            return Err(bad());
        };
        let ExprX::Block(stmts, tail) = &body.x else { return Err(bad()) };
        if tail.is_some() || stmts.is_empty() || stmts.len() > 2 {
            return Err(bad());
        }
        let StmtX::Expr(push) = &stmts[0].x else { return Err(bad()) };
        let push = unwrap(push);
        if model_call(push, self.all) != Some(Model::Push) {
            return Err(bad());
        }
        let ExprX::Call { args, .. } = &push.x else { unreachable!() };
        if args.len() != 2 {
            return Err(bad());
        }
        let dest = borrow_local(&args[0]).ok_or_else(bad)?.clone();
        let initial = s.locals.get(&dest).ok_or_else(bad)?.clone();
        let Node::Seq(empty) = &initial.node else { return Err(bad()) };
        if !matches!(&empty.expr.x,ExprX::Call{target:CallTarget::Fun(_,n,..),..} if self.all.get(n).and_then(model)==Some(Model::Empty))
        {
            return Err(bad());
        }
        let span = &e.span;
        let elem = element(&initial.typ, self.all, self.ds)?;
        let actual =
            self.layout(SpannedTyped::new(span, &initial.typ, ExprX::Var(dest.clone())), 5)?;
        let Node::Seq(current) = actual.node else { return Err(bad()) };
        let current_len = self.standard(
            span,
            collections::Model::SeqLen,
            vec![elem.clone()],
            vec![current.clone()],
        )?;
        let mut annotations = vec![];
        let (result, measure) = if stmts.len() == 1 {
            let ExprX::Call { target: CallTarget::Fun(_, name, ..), args: len_args, .. } =
                &unwrap(lhs).x
            else {
                return Err(bad());
            };
            if self.all.get(name).and_then(collections::model) != Some(collections::Model::Len)
                || len_args.len() != 1
                || local(&len_args[0]) != Some(&dest)
            {
                return Err(bad());
            }
            let n = self.expr(s, rhs)?.scalar(span)?;
            let mut refs = false;
            crate::ast_visitor::expr_visitor_walk(&args[1], &mut |x| {
                if local(x) == Some(&dest) {
                    refs = true;
                }
                crate::ast_visitor::VisitorControlFlow::Recurse
            });
            if refs {
                return Err(bad());
            }
            let mut branch = s.clone();
            branch.active = logical(
                span,
                LogicalOp::And,
                s.active.clone(),
                summary::binary(
                    span,
                    BinaryOp::Inequality(InequalityOp::Lt),
                    term(span, &int_typ(), ExprX::Const(Constant::Int(0.into())), 1)?,
                    n.clone(),
                )?,
            )?;
            let obligations = self.requirements.len();
            let value = self.expr(&mut branch, &args[1])?.scalar(span)?;
            // The current backend checks necessity at loop exits. It cannot recover
            // the first-iteration safety fact from arbitrary repeated expressions.
            // Fail closed until that independent proof transform is supported.
            if self.requirements[obligations..].iter().any(|p| !is_bool(p, true)) {
                return Err(unsupported(
                    span,
                    "fill value must be total; compute checked arithmetic before the loop",
                ));
            }
            annotations.push(summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                current_len.clone(),
                n.clone(),
            )?);
            annotations.push(records::equal(
                span,
                current.clone(),
                self.repeat(span, current_len.clone(), value.clone())?,
            )?);
            (
                self.repeat(span, n.clone(), value)?,
                summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                    n,
                    current_len,
                )?,
            )
        } else {
            let counter = local(lhs).ok_or_else(bad)?.clone();
            if !s.locals.get(&counter).is_some_and(zero) {
                return Err(bad());
            }
            let StmtX::Expr(update) = &stmts[1].x else { return Err(bad()) };
            let ExprX::Assign { place, rhs: step, op, resolve: false, .. } = &update.x else {
                return Err(bad());
            };
            let add = BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Error(IntRange::USize)));
            let step = if *op == Some(add) {
                step
            } else if op.is_none() {
                if let ExprX::Binary(o, a, b) = &step.x {
                    if *o == add && local(a) == Some(&counter) {
                        b
                    } else {
                        return Err(bad());
                    }
                } else {
                    return Err(bad());
                }
            } else {
                return Err(bad());
            };
            if place_local(place) != Some(&counter)
                || !matches!(&step.x,ExprX::Const(Constant::Int(n)) if n==&BigInt::one())
            {
                return Err(bad());
            }
            let ExprX::Call { target: CallTarget::Fun(_, name, ..), args: len_args, .. } =
                &unwrap(rhs).x
            else {
                return Err(bad());
            };
            if self.all.get(name).and_then(collections::model) != Some(collections::Model::Len)
                || len_args.len() != 1
            {
                return Err(bad());
            }
            let source_expr = &len_args[0];
            let source = self.expr(s, source_expr)?;
            let Node::Seq(sequence) = source.node else { return Err(bad()) };
            let index = unwrap(&args[1]);
            let ExprX::Call { target: CallTarget::Fun(_, name, ..), args: index_args, .. } =
                &index.x
            else {
                return Err(bad());
            };
            if self.all.get(name).and_then(collections::model) != Some(collections::Model::Index)
                || index_args.len() != 2
                || local(&index_args[1]) != Some(&counter)
                || skeleton(&index_args[0]) != skeleton(source_expr)
            {
                return Err(bad());
            }
            let i = atom(SpannedTyped::new(
                span,
                &Arc::new(TypX::Int(IntRange::USize)),
                ExprX::Var(counter.clone()),
            ))?;
            let len = self.standard(
                span,
                collections::Model::SeqLen,
                vec![elem.clone()],
                vec![sequence.clone()],
            )?;
            annotations.push(summary::binary(
                span,
                BinaryOp::Inequality(InequalityOp::Le),
                i.clone(),
                len.clone(),
            )?);
            annotations.push(records::equal(
                span,
                current,
                self.lib(span, Model::Take, vec![elem.clone()], vec![sequence.clone(), i.clone()])?,
            )?);
            s.locals.insert(counter, Val::atom(len.clone()));
            (
                sequence,
                summary::binary(
                    span,
                    BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)),
                    len,
                    i,
                )?,
            )
        };
        let invs = Arc::new(
            annotations
                .into_iter()
                .map(|v| LoopInvariant {
                    kind: LoopInvariantKind::InvariantAndEnsures,
                    inv: v.expr,
                })
                .collect(),
        );
        self.plans.push(Plan { span: e.span.id, invs, decrease: Arc::new(vec![measure.expr]) });
        s.locals.insert(dest, Val { typ: initial.typ, node: Node::Seq(result) });
        Ok(Val::atom(exits::unit_value(span)?))
    }
}
pub(super) fn annotate(f: &Function, plans: &[Plan]) -> Result<Expr, VirErr> {
    let body = f.x.body.as_ref().unwrap();
    let out = crate::ast_visitor::map_expr_visitor(body, &|e| {
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
            let plan = plans
                .iter()
                .find(|p| p.span == e.span.id)
                .ok_or_else(|| unsupported(&e.span, "missing owned loop plan"))?;
            Ok(e.new_x(ExprX::Loop {
                loop_isolation: false,
                allow_complex_invariants: *allow_complex_invariants,
                is_for_loop: *is_for_loop,
                assume_termination: *assume_termination,
                label: label.clone(),
                cond: cond.clone(),
                body: body.clone(),
                atomic_call: *atomic_call,
                invs: plan.invs.clone(),
                decrease: plan.decrease.clone(),
            }))
        } else {
            Ok(e.clone())
        }
    })?;
    validate_decoration(f, &out)?;
    Ok(out)
}
pub(crate) fn validate_decoration(f: &Function, body: &Expr) -> Result<(), VirErr> {
    let mut flags = HashMap::new();
    let native=native_for::sources(f).into_iter().map(|s|(s.loop_expr.span.id,s)).collect::<HashMap<_,_>>();
    crate::ast_visitor::expr_visitor_check(f.x.body.as_ref().unwrap(), &mut |_, e| {
        if let ExprX::Loop {
            loop_isolation,
            invs,
            decrease,
            assume_termination,
            is_for_loop,
            atomic_call,
            ..
        } = &e.x
        {
            if (!native.contains_key(&e.span.id) && (!invs.is_empty() || !decrease.is_empty() || *is_for_loop))
                || *assume_termination || *atomic_call
            {
                return Err(unsupported(
                    &e.span,
                    "owned loops require no handwritten invariants or termination bypass",
                ));
            }
            flags.insert(e.span.id, *loop_isolation);
        }
        Ok(())
    })?;
    let stripped = crate::ast_visitor::map_expr_visitor(body, &|e| {
        if let ExprX::Loop {
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            atomic_call,
            invs,
            decrease,
            ..
        } = &e.x
        {
            let native_source=native.get(&e.span.id);
            let expected=native_source.map(|s|if let ExprX::Loop{invs,..}=&s.loop_expr.x{invs.len()+3}else{0}).unwrap_or(2);
            if invs.len() != expected || decrease.len() != 1 {
                return Err(unsupported(&e.span, "owned loop decoration shape"));
            }
            if native_source.is_some() {
                for inv in invs.iter().take(3){native_for::pure(&inv.inv)?;}
                let ExprX::Loop{invs:original,decrease:original_decrease,..}=&native_source.unwrap().loop_expr.x else{unreachable!()};
                if serde_json::to_vec(&invs[3..]).ok()!=serde_json::to_vec(original).ok() || serde_json::to_vec(decrease).ok()!=serde_json::to_vec(original_decrease).ok(){
                    return Err(unsupported(&e.span,"native fold changed original iterator proof annotations"));
                }
                return Ok(e.new_x(ExprX::Loop{
                    loop_isolation:*flags.get(&e.span.id).unwrap(),allow_complex_invariants:*allow_complex_invariants,is_for_loop:*is_for_loop,
                    assume_termination:*assume_termination,label:label.clone(),cond:cond.clone(),body:body.clone(),atomic_call:*atomic_call,
                    invs:original.clone(),decrease:original_decrease.clone(),
                }));
            }
            for inv in invs.iter() {
                if inv.kind != LoopInvariantKind::InvariantAndEnsures
                    || !same(&inv.inv.typ, &bool_typ())
                {
                    return Err(unsupported(&e.span, "owned invariant type"));
                }
                pure(&inv.inv)?;
            }
            if !eligibility::integer(&decrease[0].typ) {
                return Err(unsupported(&e.span, "owned loop measure"));
            }
            pure(&decrease[0])?;
            Ok(e.new_x(ExprX::Loop {
                loop_isolation: *flags
                    .get(&e.span.id)
                    .ok_or_else(|| unsupported(&e.span, "inserted owned loop"))?,
                allow_complex_invariants: *allow_complex_invariants,
                is_for_loop: *is_for_loop,
                assume_termination: *assume_termination,
                label: label.clone(),
                cond: cond.clone(),
                body: body.clone(),
                atomic_call: *atomic_call,
                invs: Arc::new(vec![]),
                decrease: Arc::new(vec![]),
            }))
        } else {
            Ok(e.clone())
        }
    })?;
    if serde_json::to_vec(&stripped).ok() != serde_json::to_vec(f.x.body.as_ref().unwrap()).ok() {
        return Err(unsupported(&f.span, "owned decoration changed executable code"));
    }
    Ok(())
}
pub(super) fn pure(e: &Expr) -> Result<(), VirErr> {
    let mut budget = FORMULA_BUDGET;
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        budget =
            budget.checked_sub(1).ok_or_else(|| unsupported(&e.span, "owned decoration budget"))?;
        let allowed = match &e.x {
            ExprX::Const(_)
            | ExprX::Var(_)
            | ExprX::Unary(_, _)
            | ExprX::UnaryOpr(_, _)
            | ExprX::Binary(_, _, _)
            | ExprX::Logical(_, _, _)
            | ExprX::Ctor(_, _, _, None)
            | ExprX::Closure(_, _)
            | ExprX::If(_, _, Some(_)) => true,
            ExprX::ReadPlace(p, _) => pure_place(p),
            ExprX::Call {
                target: CallTarget::Fun(_, name, ..),
                post_args: None,
                body: None,
                ..
            } => {
                [
                    vec!["view", "View", "view"],
                    vec!["std_specs", "vec", "spec_vec_len"],
                    vec!["seq", "Seq", "new"],
                    vec!["seq", "Seq", "empty"],
                    vec!["seq", "Seq", "len"],
                    vec!["seq", "Seq", "index"],
                    vec!["seq", "Seq", "take"],
                ]
                .iter()
                .any(|p| models::path_is(&name.path, CrateId::Vstd, p))
                    || borrowed::is_value_name(name)
                    || name.path.krate == CrateId::Vstd
                        && name.path.segments.len() == 3
                        && name.path.segments[0].as_str() == "seq"
                        && name.path.segments[1].starts_with("impl")
                        && name.path.segments[2].as_str() == "take"
            }
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(unsupported(&e.span, "executable or assumption in owned loop decoration"))
        }
    })
}
fn pure_place(p: &Place) -> bool {
    match &p.x {
        PlaceX::Local(_) => true,
        PlaceX::Temporary(e) => pure(e).is_ok(),
        PlaceX::Field(_, p) => pure_place(p),
        _ => false,
    }
}
