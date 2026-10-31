//! Proposed ordinary error-callback contracts, composed from existing call
//! postconditions. The original closure and every call still verify normally.
use super::*;

fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(tail)) if ss.is_empty() => plain(tail),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Temporary(inner) => plain(inner),
            _ => e,
        },
        _ => e,
    }
}

// Closure argument patterns lower to immutable local declarations. Recover only
// their pure projections for candidate formulas; retain all original statements.
fn return_expression(body: &Expr) -> Option<Expr> {
    fn substitute(e: &Expr, vars: &HashMap<VarIdent, Expr>) -> Option<Expr> {
        crate::ast_visitor::map_expr_visitor_env(
            e,
            &mut crate::ast_visitor::VisitorScopeMap::new(),
            &mut (),
            &|_, _, e| {
                Ok(if let ExprX::Var(n) = &e.x {
                    vars.get(n).cloned().unwrap_or_else(|| e.clone())
                } else {
                    e.clone()
                })
            },
            &|_, _, s| Ok(vec![s.clone()]),
            &|_, t| Ok(t.clone()),
            &|_, _, p| {
                Ok(if let PlaceX::Local(n) = &p.x {
                    vars.get(n)
                        .map(|v| p.new_x(PlaceX::Temporary(v.clone())))
                        .unwrap_or_else(|| p.clone())
                } else {
                    p.clone()
                })
            },
            &|_, _, p| Ok(p.clone()),
        )
        .ok()
    }
    fn bind(p: &Pattern, v: Expr, vars: &mut HashMap<VarIdent, Expr>, depth: usize) -> Option<()> {
        if depth == 0 {
            return None;
        }
        match &p.x {
            PatternX::Var(b) if b.user_mut != Some(true) && matches!(b.by_ref, ByRef::No) => {
                vars.insert(b.name.clone(), v);
                Some(())
            }
            PatternX::Constructor(dt @ Dt::Tuple(_), variant, fields) => {
                if fields.len() > 4 {
                    return None;
                }
                for field in fields.iter() {
                    let op = FieldOpr {
                        datatype: dt.clone(),
                        variant: variant.clone(),
                        field: field.name.clone(),
                        get_variant: false,
                        check: VariantCheck::None,
                    };
                    bind(
                        &field.a,
                        SpannedTyped::new(
                            &v.span,
                            &field.a.typ,
                            ExprX::UnaryOpr(UnaryOpr::Field(op), v.clone()),
                        ),
                        vars,
                        depth - 1,
                    )?;
                }
                Some(())
            }
            PatternX::Wildcard => Some(()),
            _ => None,
        }
    }
    let mut vars = HashMap::new();
    let mut body = body.clone();
    for _ in 0..4 {
        if let ExprX::Block(stmts, Some(tail)) = &plain(&body).x {
            if stmts.len() > 8 {
                return None;
            }
            for s in stmts.iter() {
                let StmtX::Decl {
                    pattern,
                    mode: None | Some((Mode::Exec, DeclProph::Default)),
                    init: Some(init),
                    els: None,
                    assert_irrefutable: false,
                } = &s.x
                else {
                    return None;
                };
                let value = substitute(&crate::ast_util::place_to_spec_expr(init), &vars)?;
                bind(pattern, observations::logical_value(&value), &mut vars, 4)?;
            }
            body = substitute(tail, &vars)?;
        } else {
            return substitute(&body, &vars);
        }
    }
    None
}

fn value(e: &Expr) -> Option<Expr> {
    let e = observations::logical_value(plain(e));
    match &e.x {
        ExprX::Var(_) | ExprX::Const(_) => Some(e.clone()),
        ExprX::UnaryOpr(UnaryOpr::Field(op), parent)
            if !op.get_variant && op.check == VariantCheck::None =>
        {
            value(parent)?;
            Some(e.clone())
        }
        _ => None,
    }
}

fn pure(e: &Expr, all: &HashMap<Fun, Function>) -> bool {
    let mut remaining = FORMULA_BUDGET;
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        remaining = remaining
            .checked_sub(1)
            .ok_or_else(|| unsupported(&e.span, "error callback formula budget"))?;
        let allowed = match &e.x {
            ExprX::Const(_)
            | ExprX::Var(_)
            | ExprX::VarAt(_, VarAt::Pre)
            | ExprX::Unary(..)
            | ExprX::UnaryOpr(..)
            | ExprX::Binary(..)
            | ExprX::Logical(..)
            | ExprX::Ctor(_, _, _, None)
            | ExprX::If(_, _, Some(_)) => true,
            ExprX::Old(value) => matches!(value.x, ExprX::Var(_)),
            ExprX::ReadPlace(place, _) => loops::pure_place(place),
            ExprX::Block(stmts, Some(_)) if stmts.is_empty() => true,
            ExprX::Call {
                target: CallTarget::Fun(kind, name, ..),
                post_args: None,
                body: None,
                ..
            } => {
                let name = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                    resolved
                } else {
                    name
                };
                all.get(name).is_some_and(is_pure_observation)
            }
            _ => false,
        };
        if allowed {
            Ok(())
        } else {
            Err(unsupported(&e.span, "error callback requires pure call postconditions"))
        }
    })
    .is_ok()
}

fn call_post(source: &Expr, output: &Expr, all: &HashMap<Fun, Function>) -> Option<Vec<Expr>> {
    let ExprX::Call {
        target: CallTarget::Fun(kind, name, types, ..),
        args,
        post_args: None,
        body: None,
    } = &plain(source).x
    else {
        return None;
    };
    let (name, types) = match kind {
        CallTargetKind::DynamicResolved { resolved, typs, .. } => (resolved, typs),
        CallTargetKind::Static => (name, types),
        _ => return None,
    };
    let f = all.get(name)?;
    if f.x.mode != Mode::Exec
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || f.x.attrs.is_external_body
        || f.x.attrs.exec_assume_termination
        || f.x.attrs.exec_allows_no_decreases_clause
        || f.x.params.len() != args.len()
        || f.x.typ_params.len() != types.len()
        || !f.x.require.is_empty()
        || f.x.ensure.0.is_empty() && f.x.returns.is_none()
        || f.x.ensure.0.len() > 4
        || !f.x.ensure.1.is_empty()
        || f.x
            .params
            .iter()
            .any(|p| p.x.mode != Mode::Exec || p.x.user_mut || matches!(&*p.x.typ, TypX::MutRef(_)))
    {
        return None;
    }
    let types: HashMap<_, _> = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    if !same(&crate::sst_util::subst_typ(&types, &f.x.inner_ret.x.typ), &output.typ) {
        return None;
    }
    let mut vars: HashMap<_, _> =
        f.x.params
            .iter()
            .zip(args.iter())
            .map(|(p, arg)| Some((p.x.name.clone(), value(arg)?)))
            .collect::<Option<_>>()?;
    vars.insert(f.x.inner_ret.x.name.clone(), output.clone());
    let mut posts = (*f.x.ensure.0).clone();
    if let Some(value) = &f.x.returns {
        let result = SpannedTyped::new(
            &f.span,
            &f.x.inner_ret.x.typ,
            ExprX::Var(f.x.inner_ret.x.name.clone()),
        );
        posts.push(SpannedTyped::new(
            &f.span,
            &bool_typ(),
            ExprX::Binary(BinaryOp::Eq(Mode::Spec), result, value.clone()),
        ));
    }
    posts
        .iter()
        .map(|post| {
            let mapped = crate::ast_visitor::map_expr_visitor_env(
                post,
                &mut crate::ast_visitor::VisitorScopeMap::new(),
                &mut (),
                &|_, _, e| {
                    Ok(match &e.x {
                        ExprX::Var(n) => vars.get(n).cloned().unwrap_or_else(|| e.clone()),
                        _ => e.clone(),
                    })
                },
                &|_, _, s| Ok(vec![s.clone()]),
                &|_, t| Ok(crate::sst_util::subst_typ(&types, t)),
                &|_, _, p| {
                    Ok(match &p.x {
                        PlaceX::Local(n) if vars.contains_key(n) => {
                            p.new_x(PlaceX::Temporary(vars[n].clone()))
                        }
                        _ => p.clone(),
                    })
                },
                &|_, _, p| Ok(p.clone()),
            )
            .ok()?;
            if !pure(&mapped, all) {
                return None;
            }
            Some(mapped)
        })
        .collect()
}

fn relation(
    source: &Expr,
    output: &Expr,
    all: &HashMap<Fun, Function>,
    depth: usize,
) -> Option<Vec<Expr>> {
    if depth == 0 {
        return None;
    }
    let source = plain(source);
    if !same(&source.typ, &output.typ) {
        return None;
    }
    if let ExprX::If(condition, left, Some(right)) = &source.x {
        let condition = value(condition)?;
        if !matches!(&*condition.typ, TypX::Bool) {
            return None;
        }
        let left = relation(left, output, all, depth - 1)?;
        let right = relation(right, output, all, depth - 1)?;
        let not = SpannedTyped::new(
            &condition.span,
            &bool_typ(),
            ExprX::Unary(UnaryOp::Not, condition.clone()),
        );
        return Some(
            left.into_iter()
                .map(|p| {
                    SpannedTyped::new(
                        &p.span,
                        &bool_typ(),
                        ExprX::Logical(LogicalOp::Implies, condition.clone(), p.clone()),
                    )
                })
                .chain(right.into_iter().map(|p| {
                    SpannedTyped::new(
                        &p.span,
                        &bool_typ(),
                        ExprX::Logical(LogicalOp::Implies, not.clone(), p.clone()),
                    )
                }))
                .collect(),
        );
    }
    if let ExprX::Ctor(dt, variant, fields, None) = &source.x {
        if fields.len() > 8 {
            return None;
        }
        let mut clauses = vec![SpannedTyped::new(
            &source.span,
            &bool_typ(),
            ExprX::UnaryOpr(
                UnaryOpr::IsVariant { datatype: dt.clone(), variant: variant.clone() },
                output.clone(),
            ),
        )];
        for field in fields.iter() {
            let op = FieldOpr {
                datatype: dt.clone(),
                variant: variant.clone(),
                field: field.name.clone(),
                get_variant: false,
                check: VariantCheck::None,
            };
            let selected = SpannedTyped::new(
                &source.span,
                &field.a.typ,
                ExprX::UnaryOpr(UnaryOpr::Field(op), output.clone()),
            );
            clauses.extend(relation(&field.a, &selected, all, depth - 1)?);
        }
        Some(clauses)
    } else if let Some(value) = value(source) {
        Some(vec![
            records::equal(&source.span, atom(output.clone()).ok()?, atom(value).ok()?).ok()?.expr,
        ])
    } else {
        call_post(source, output, all)
    }
}

pub(super) fn decorate(
    f: &Function,
    body: &Expr,
    plans: &[callbacks::Plan],
) -> Result<Expr, VirErr> {
    fn transform(body: &Expr, plans: &[callbacks::Plan], strip: bool) -> Result<Expr, VirErr> {
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
                if let Some(plan) = plans.iter().find(|p| p.span == e.span.id) {
                    if proof_fn_modes.is_some()
                        || external_spec.is_some()
                        || !strip && (!requires.is_empty() || !ensures.is_empty())
                    {
                        return Err(unsupported(&e.span, "error callback source changed"));
                    }
                    return Ok(e.new_x(ExprX::NonSpecClosure {
                        params: params.clone(),
                        proof_fn_modes: None,
                        body: body.clone(),
                        requires: if strip { Arc::new(vec![]) } else { plan.requires.clone() },
                        ensures: if strip { Arc::new(vec![]) } else { plan.ensures.clone() },
                        ret: ret.clone(),
                        external_spec: None,
                    }));
                }
            }
            Ok(e.clone())
        })
    }
    let decorated = transform(body, plans, false)?;
    let original = transform(&decorated, plans, true)?;
    let (Ok(original), Ok(source)) = (serde_json::to_vec(&original), serde_json::to_vec(body))
    else {
        return Err(unsupported(&f.span, "error callback source comparison failed"));
    };
    if original != source {
        return Err(unsupported(&f.span, "error callback decoration changed executable code"));
    }
    Ok(decorated)
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    fs.iter()
        .map(|f| {
            let Some(body) = &f.x.body else { return Ok(f.clone()) };
            if f.x.mode != Mode::Exec
                || f.x.attrs.infer
                || f.x.attrs.is_external_body
                || f.x.attrs.is_unsafe
                || f.x.attrs.is_async
                || f.x.ensure.0.is_empty()
            {
                return Ok(f.clone());
            }
            let mut authored = false;
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if let ExprX::NonSpecClosure { requires, ensures, .. } = &e.x {
                    authored |= !requires.is_empty() || !ensures.is_empty();
                }
                authored |= matches!(e.x, ExprX::Loop { .. });
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
            if authored {
                return Ok(f.clone());
            }
            let mut plans = vec![];
            let mut remaining = FORMULA_BUDGET;
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if remaining == 0 {
                    return crate::ast_visitor::VisitorControlFlow::<()>::Return;
                }
                remaining -= 1;
                if let Some(args) = result_map::call(e, &all) {
                    let closure = plain(&args[1]);
                    if let ExprX::NonSpecClosure {
                        params,
                        proof_fn_modes: None,
                        body,
                        requires,
                        ensures,
                        ret,
                        external_spec: None,
                    } = &closure.x
                    {
                        if params.len() == 1
                            && requires.is_empty()
                            && ensures.is_empty()
                            && !plans.iter().any(|p: &callbacks::Plan| p.span == closure.span.id)
                        {
                            let output = SpannedTyped::new(
                                &closure.span,
                                &ret.a,
                                ExprX::Var(ret.name.clone()),
                            );
                            if let Some(clauses) = return_expression(body)
                                .filter(|e| matches!(&plain(e).x, ExprX::Ctor(..)))
                                .and_then(|body| relation(&body, &output, &all, 6))
                            {
                                if clauses.iter().all(|p| pure(p, &all)) {
                                    let post = clauses.into_iter().try_fold(
                                        boolean(&closure.span, true),
                                        |post, clause| {
                                            logical(
                                                &closure.span,
                                                LogicalOp::And,
                                                post,
                                                atom(clause).ok()?,
                                            )
                                            .ok()
                                        },
                                    );
                                    if let Some(post) = post {
                                        plans.push(callbacks::Plan {
                                            span: closure.span.id,
                                            requires: Arc::new(vec![
                                                boolean(&closure.span, true).expr,
                                            ]),
                                            ensures: Arc::new(vec![post.expr]),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
            if plans.is_empty() || plans.len() > 4 || remaining == 0 {
                return Ok(f.clone());
            }
            let mut x = f.x.clone();
            x.body = Some(decorate(f, body, &plans)?);
            report.error_callbacks.push((f.x.name.clone(), plans.len()));
            Ok(f.new_x(x))
        })
        .collect()
}
