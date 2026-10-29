//! Compose an exact complete sequence-view update into an owned accumulator.
use super::*;
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::ReadPlace(p, _) => {
            if let PlaceX::Temporary(e) = &p.x {
                plain(e)
            } else {
                e
            }
        }
        _ => e,
    }
}
fn receiver(e: &Expr, param: &VarIdent, future: bool) -> bool {
    match &plain(e).x {
        ExprX::Unary(UnaryOp::MutRefFuture(_), a) if future => {
            matches!(&plain(a).x,ExprX::Var(n) if n==param)
        }
        ExprX::Unary(UnaryOp::MutRefCurrent, a) if !future => {
            matches!(&plain(a).x,ExprX::Old(a) if matches!(&plain(a).x,ExprX::Var(n) if n==param))
        }
        _ => false,
    }
}
impl Engine<'_> {
    pub(super) fn mutable_sequence_helper(
        &mut self,
        s: &mut State,
        e: &Expr,
        f: &Function,
        types: &Typs,
        args: &Exprs,
    ) -> Result<Val, VirErr> {
        let bad = || {
            unsupported(
                &e.span,
                "owned mutation requires one complete checked sequence-view equation and a local receiver",
            )
        };
        if !enums::unit(&f.x.inner_ret.x.typ) || f.x.params.is_empty() || args.is_empty() {
            return Err(bad());
        }
        let (ExprX::BorrowMut(place) | ExprX::TwoPhaseBorrowMut(place)) = &args[0].x else {
            return Err(bad());
        };
        let PlaceX::Local(dest) = &place.x else {
            return Err(bad());
        };
        let before = s.locals.get(dest).ok_or_else(bad)?.clone();
        let Node::Seq(sequence) = &before.node else {
            return Err(bad());
        };
        if sequence_views::find(&before.typ, self.all, self.ds).is_none() {
            return Err(bad());
        }
        let param = &f.x.params[0].x.name;
        let ts =
            f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect::<HashMap<_, _>>();
        let mut values = HashMap::new();
        for (p, a) in f.x.params.iter().zip(args.iter()).skip(1) {
            values.insert(p.x.name.clone(), self.expr(s, a)?.scalar(&e.span)?);
        }
        let is_view = |e: &Expr, future| {
            if let ExprX::Call {
                target: CallTarget::Fun(kind, n, ..),
                args,
                post_args: None,
                body: None,
            } = &plain(e).x
            {
                let n = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                    resolved
                } else {
                    n
                };
                args.len() == 1
                    && self.all.get(n).is_some_and(|f| {
                        f.x.attrs.infer_sequence_view
                            || collections::model(f) == Some(collections::Model::View)
                    })
                    && receiver(&args[0], param, future)
            } else {
                false
            }
        };
        let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &f.x.ensure.0[0].x else {
            return Err(bad());
        };
        if !is_view(lhs, true) {
            return Err(bad());
        }
        let substitute = |e: &Expr| -> Result<Expr, VirErr> {
            let mapped = crate::ast_visitor::map_expr_visitor_env(
                e,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut (),
                &|_, _, e| {
                    if is_view(e, false) {
                        Ok(sequence.expr.clone())
                    } else if let ExprX::Var(n) = &e.x {
                        Ok(values
                            .get(n)
                            .map(|v: &Term| v.expr.clone())
                            .unwrap_or_else(|| e.clone()))
                    } else {
                        Ok(e.clone())
                    }
                },
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| {
                    Ok(if let TypX::TypParam(n) = &**t {
                        ts.get(n).cloned().unwrap_or_else(|| t.clone())
                    } else {
                        t.clone()
                    })
                },
                &|_, _, p| Ok(p.clone()),
                &|_, _, p| Ok(p.clone()),
            )?;
            crate::ast_visitor::expr_visitor_check(&mapped, &mut |_, e| {
                if matches!(&e.x,ExprX::Var(n) if n==param) { Err(bad()) } else { Ok(()) }
            })?;
            Ok(mapped)
        };
        let pre = substitute(&f.x.require[0])?;
        let post = substitute(rhs)?;
        let pre = self.formula(s, &pre)?;
        self.require(s, pre)?;
        let after = self.formula(s, &post)?;
        if !same(&after.expr.typ, &sequence.expr.typ) {
            return Err(bad());
        }
        s.locals.insert(dest.clone(), Val { identity: None, typ: before.typ, node: Node::Seq(after) });
        Ok(Val::atom(exits::unit_value(&e.span)?))
    }
}
