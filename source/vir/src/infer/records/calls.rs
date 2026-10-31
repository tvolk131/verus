//! Instantiate checked candidate relations, without inlining executable helpers.
use super::*;

fn strip(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::ReadPlace(p, _) => {
            if let PlaceX::Temporary(e) = &p.x {
                strip(e)
            } else {
                e
            }
        }
        _ => e,
    }
}
fn projection(e: &Expr, f: &Function) -> Option<(Vec<FieldOpr>, bool)> {
    let mut e = strip(e);
    let mut path = vec![];
    while let ExprX::UnaryOpr(UnaryOpr::Field(op), a) = &e.x {
        path.push(op.clone());
        e = strip(a);
    }
    let mut future = false;
    if mutable(f) {
        e = match &e.x {
            ExprX::Unary(UnaryOp::MutRefFuture(_), a) => {
                future = true;
                strip(a)
            }
            ExprX::Unary(UnaryOp::MutRefCurrent, a) => match &strip(a).x {
                ExprX::Old(a) => strip(a),
                _ => return None,
            },
            _ => return None,
        };
    }
    if !matches!(&e.x,ExprX::Var(n) if n==&f.x.params[0].x.name) {
        return None;
    }
    path.reverse();
    Some((path, future))
}
fn clauses(e: &Expr, out: &mut Vec<Expr>) {
    if let ExprX::Logical(LogicalOp::And, a, b) = &e.x {
        clauses(a, out);
        clauses(b, out)
    } else {
        out.push(e.clone());
    }
}

impl Interpreter<'_> {
    fn instantiate_call(
        &self,
        e: &Expr,
        callee: &Function,
        shape: &Shape,
        indices: &[usize],
        before: &State,
        vars: &HashMap<VarIdent, Term>,
        types: &HashMap<Ident, Typ>,
    ) -> Result<Term, VirErr> {
        let expr = crate::ast_visitor::map_expr_visitor_env(
            e,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut (),
            &|_, _, e| {
                if let ExprX::Var(n) = &e.x {
                    if let Some(t) = vars.get(n) {
                        return Ok(t.expr.clone());
                    }
                }
                let field = |value: &Expr| -> Option<usize> {
                    let (path, future) = projection(value, callee)?;
                    if future {
                        return None;
                    }
                    shape.fields.iter().position(|f| {
                        shape::same_path(
                            &path,
                            &f.path.iter().map(|p| p.op.clone()).collect::<Vec<_>>(),
                        )
                    })
                };
                if let ExprX::Call {
                    target: CallTarget::Fun(kind, name, ..),
                    args,
                    post_args: None,
                    body: None,
                } = &e.x
                {
                    let name = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                        resolved
                    } else {
                        name
                    };
                    if args.len() == 1 {
                        if let Some(i) = field(&args[0]) {
                            if self.all.get(name).is_some_and(|f| f.x.attrs.infer_sequence_view) {
                                return Ok(before.fields[indices[i]].expr.clone());
                            }
                            match self.all.get(name).and_then(collections::model) {
                                Some(Model::View) => {
                                    return Ok(before.fields[indices[i]].expr.clone());
                                }
                                Some(Model::SpecLen) => return Ok(self.len(indices[i])?.expr),
                                _ => {}
                            }
                        }
                    }
                }
                if let Some(i) = field(e) {
                    if shape.fields[i].element.is_none() {
                        return Ok(before.fields[indices[i]].expr.clone());
                    }
                }
                Ok(e.clone())
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| {
                Ok(match &**t {
                    // This visitor already traverses type children. Substitute a
                    // parameter once, so callee T -> (caller T,u16) does not expand
                    // that caller T again at each enclosing Option/reference node.
                    TypX::TypParam(n) => types.get(n).cloned().unwrap_or_else(|| t.clone()),
                    _ => t.clone(),
                })
            },
            &|_, _, p| Ok(p.clone()),
            &|_, _, p| Ok(p.clone()),
        )?;
        atom(expr)
    }
    pub(super) fn helper_call(
        &mut self,
        s: &mut State,
        e: &Expr,
        callee: &Function,
        types: &Typs,
        args: &Exprs,
    ) -> Result<Term, VirErr> {
        let bad = || {
            unsupported(
                &e.span,
                "record helper requires a checked inferred record receiver, value arguments and a complete relation",
            )
        };
        crate::infer_proof::CandidateId::from_function(callee)?;
        if !self.function.x.attrs.infer_allow_approx && callee.x.attrs.infer_allow_approx {
            return Err(unsupported(&e.span, "exact record inference requires exact-mode callees"));
        }
        if !callee.x.attrs.infer
            || callee.x.require.len() != 1
            || callee.x.ensure.0.len() != 1
            || !callee.x.ensure.1.is_empty()
            || types.len() != callee.x.typ_params.len()
            || args.len() != callee.x.params.len()
            || !self.function.x.attrs.infer_allow_approx && callee.x.attrs.infer_allow_approx
        {
            return Err(bad());
        }
        let prefix = match &args[0].x {
            ExprX::ReadPlace(p, _) | ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => {
                if matches!(&p.x,PlaceX::Local(n) if n==&self.function.x.params[0].x.name) {
                    Some(vec![])
                } else {
                    self.path(p)
                }
            }
            _ => None,
        }
        .ok_or_else(bad)?;
        if mutable(callee) && !mutable(self.function) {
            return Err(bad());
        }
        let types: HashMap<_, _> =
            callee.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
        // The caller's immutable receiver is still its original complete
        // value. Compose the checked relation without opening private fields.
        if !mutable(callee) && !mutable(self.function) {
            let substitute = |expr: &Expr| {
                crate::ast_visitor::map_expr_visitor_env(
                    expr,
                    &mut crate::ast_visitor::VisitorScopeMap::new(),
                    &mut (),
                    &|_, _, e| Ok(e.clone()),
                    &|_, _, s| Ok(vec![s.clone()]),
                    &|_, t| {
                        Ok(match &**t {
                            TypX::TypParam(n) => types.get(n).cloned().unwrap_or_else(|| t.clone()),
                            _ => t.clone(),
                        })
                    },
                    &|_, _, p| Ok(p.clone()),
                    &|_, _, p| Ok(p.clone()),
                )
            };
            let pre = substitute(&callee.x.require[0])?;
            let post = substitute(&callee.x.ensure.0[0])?;
            let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &post.x else {
                return Err(bad());
            };
            if !matches!(&lhs.x,ExprX::Var(n) if n==&callee.x.inner_ret.x.name) {
                return Err(bad());
            }
            let receiver = coerce(
                &e.span,
                atom(observations::logical_value(&args[0]))?,
                &crate::sst_util::subst_typ(&types, &callee.x.params[0].x.typ),
            )?;
            let mut vars = HashMap::from([(callee.x.params[0].x.name.clone(), receiver)]);
            for (p, arg) in callee.x.params.iter().skip(1).zip(args.iter().skip(1)) {
                vars.insert(p.x.name.clone(), self.readonly(s, arg)?);
            }
            self.require(s, &e.span, substitute_value(&pre, &vars)?)?;
            return substitute_value(rhs, &vars);
        }
        let mut instantiated = callee.x.clone();
        instantiated.typ_params = self.function.x.typ_params.clone();
        instantiated.typ_bounds = self.function.x.typ_bounds.clone();
        instantiated.owning_module = self.function.x.owning_module.clone();
        instantiated.params = Arc::new(
            callee
                .x
                .params
                .iter()
                .map(|p| {
                    let mut x = p.x.clone();
                    x.typ = crate::sst_util::subst_typ(&types, &x.typ);
                    p.new_x(x)
                })
                .collect(),
        );
        instantiated.inner_ret = {
            let mut x = callee.x.inner_ret.x.clone();
            x.typ = crate::sst_util::subst_typ(&types, &x.typ);
            callee.x.inner_ret.new_x(x)
        };
        let signature = callee.new_x(instantiated);
        let shape = shape(&signature, self.datatypes, self.all)?;
        let mut indices = vec![];
        for field in &shape.fields {
            let mut path = prefix.clone();
            path.extend(field.path.iter().map(|p| p.op.clone()));
            let i = self
                .shape
                .fields
                .iter()
                .position(|f| {
                    shape::same_path(
                        &path,
                        &f.path.iter().map(|p| p.op.clone()).collect::<Vec<_>>(),
                    )
                })
                .ok_or_else(bad)?;
            if !same(&field.typ, &self.shape.fields[i].typ) {
                return Err(bad());
            }
            indices.push(i);
        }
        let mut vars = HashMap::new();
        for (p, arg) in callee.x.params.iter().skip(1).zip(args.iter().skip(1)) {
            vars.insert(p.x.name.clone(), self.readonly(s, arg)?);
        }
        let before = s.clone();
        let pre = self.instantiate_call(
            &callee.x.require[0],
            callee,
            &shape,
            &indices,
            &before,
            &vars,
            &types,
        )?;
        self.require(s, &e.span, pre)?;
        if mutable(callee) {
            let mut posts = vec![];
            clauses(&callee.x.ensure.0[0], &mut posts);
            if posts.len() != shape.fields.len() {
                return Err(bad());
            }
            let mut updates = vec![];
            for (clause, field) in posts.iter().zip(&shape.fields) {
                let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &clause.x else {
                    return Err(bad());
                };
                let (path, future) = projection(lhs, callee)
                    .or_else(|| {
                        if let ExprX::Call { args, .. } = &lhs.x {
                            args.first().and_then(|a| projection(a, callee))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(bad)?;
                if !future
                    || !shape::same_path(
                        &path,
                        &field.path.iter().map(|p| p.op.clone()).collect::<Vec<_>>(),
                    )
                {
                    return Err(bad());
                }
                updates.push(
                    self.instantiate_call(rhs, callee, &shape, &indices, &before, &vars, &types)?,
                );
            }
            for (i, value) in indices.into_iter().zip(updates) {
                s.fields[i] = conditional(&e.span, s.active.clone(), value, s.fields[i].clone())?;
            }
            exits::unit_value(&e.span)
        } else {
            let ExprX::Binary(BinaryOp::Eq(Mode::Spec), lhs, rhs) = &callee.x.ensure.0[0].x else {
                return Err(bad());
            };
            if !matches!(&lhs.x,ExprX::Var(n) if n==&callee.x.inner_ret.x.name) {
                return Err(bad());
            }
            self.instantiate_call(rhs, callee, &shape, &indices, &before, &vars, &types)
        }
    }
}
