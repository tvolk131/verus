//! Checked pointwise transport proposals for a returned record containing a
//! checked clone. No invariant or clone relation is assumed by this pass.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor_env};

fn key(e: &Expr) -> Option<(VarIdent, Vec<FieldOpr>)> {
    fn place(p: &Place) -> Option<(VarIdent, Vec<FieldOpr>)> {
        match &p.x {
            PlaceX::Local(n) => Some((n.clone(), vec![])),
            PlaceX::Temporary(e) => key(e),
            PlaceX::Field(op, p) => {
                let (n, mut path) = place(p)?;
                path.push(op.clone());
                Some((n, path))
            }
            _ => None,
        }
    }
    match &e.x {
        ExprX::Var(n) => Some((n.clone(), vec![])),
        ExprX::ReadPlace(p, _) => place(p),
        ExprX::UnaryOpr(UnaryOpr::Field(op), a) => {
            let (n, mut path) = key(a)?;
            path.push(op.clone());
            Some((n, path))
        }
        ExprX::Block(ss, Some(e)) if ss.is_empty() => key(e),
        _ => None,
    }
}
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(e)) if ss.is_empty() => plain(e),
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
fn call(e: &Expr) -> Option<(&Fun, &Typs, &Exprs)> {
    let ExprX::Call { target: CallTarget::Fun(kind, n, ts, ..), args, post_args: None, body: None } =
        &plain(e).x
    else {
        return None;
    };
    Some(match kind {
        CallTargetKind::DynamicResolved { resolved, typs, .. } => (resolved, typs, args),
        _ => (n, ts, args),
    })
}
fn substitute(body: &Expr, f: &Function, types: &Typs, arg: &Expr) -> Result<Expr, VirErr> {
    let ts: HashMap<_, _> = f.x.typ_params.iter().cloned().zip(types.iter().cloned()).collect();
    map_expr_visitor_env(
        body,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(if matches!(&e.x,ExprX::Var(n) if n==&f.x.params[0].x.name) {
                arg.clone()
            } else {
                e.clone()
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(match &**t {
                TypX::TypParam(n) => ts.get(n).cloned().unwrap_or_else(|| t.clone()),
                _ => t.clone(),
            })
        },
        &|_, _, p| {
            Ok(if matches!(&p.x,PlaceX::Local(n) if n==&f.x.params[0].x.name) {
                p.new_x(PlaceX::Temporary(arg.clone()))
            } else {
                p.clone()
            })
        },
        &|_, _, p| Ok(p.clone()),
    )
}
fn expand(e: &Expr, owner: &Option<Path>, all: &HashMap<Fun, Function>) -> Result<Expr, VirErr> {
    let mut out = e.clone();
    let mut budget = 4096usize;
    for _ in 0..8 {
        out = crate::ast_visitor::map_expr_visitor(&out, &|e| {
            if let Some((n, ts, args)) = call(e) {
                if let Some(f) = all.get(n).filter(|f| {
                    f.x.name.path.krate != CrateId::Vstd
                        && is_pure_observation(f)
                        && !f.x.attrs.is_external_body
                        && f.x.proxy.is_none()
                        && f.x.params.len() == 1
                        && ts.len() == f.x.typ_params.len()
                        && args.len() == 1
                        && f.x.body.is_some()
                        && owner.as_ref().is_some_and(|m| {
                            crate::ast_util::is_body_visible_to(&f.x.body_visibility, m)
                        })
                }) {
                    return substitute(f.x.body.as_ref().unwrap(), f, ts, &args[0]);
                }
            }
            if let ExprX::UnaryOpr(UnaryOpr::Field(op), a) = &e.x {
                if let ExprX::Ctor(dt, v, fs, None) = &plain(a).x {
                    if dt == &op.datatype && v == &op.variant {
                        if let Some(f) = fs.iter().find(|f| f.name == op.field) {
                            return Ok(f.a.clone());
                        }
                    }
                }
            }
            Ok(e.clone())
        })?;
        expr_visitor_walk(&out, &mut |_| {
            if budget == 0 {
                return VisitorControlFlow::Stop(());
            }
            budget -= 1;
            VisitorControlFlow::Recurse
        });
        if budget == 0 {
            return Err(unsupported(&e.span, "pointwise definition expansion budget exceeded"));
        }
    }
    Ok(out)
}
fn rewrite(e: &Expr, pairs: &[(VarIdent, Vec<FieldOpr>, Expr)]) -> Result<Expr, VirErr> {
    map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut 4096usize,
        &|budget, _, e| {
            *budget = budget
                .checked_sub(1)
                .ok_or_else(|| unsupported(&e.span, "pointwise transport budget exceeded"))?;
            if let Some((root, path)) = key(e) {
                if let Some((_, _, value)) =
                    pairs.iter().find(|(n, p, _)| n == &root && records::shape::same_path(p, &path))
                {
                    return records::coerce(&e.span, records::atom(value.clone())?, &e.typ)
                        .map(|t| t.expr);
                }
            }
            Ok(e.clone())
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| {
            let e = SpannedTyped::new(
                &p.span,
                &p.typ,
                ExprX::ReadPlace(
                    p.clone(),
                    UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
                ),
            );
            if let Some((root, path)) = key(&e) {
                if let Some((_, _, value)) =
                    pairs.iter().find(|(n, k, _)| n == &root && records::shape::same_path(k, &path))
                {
                    let value =
                        records::coerce(&p.span, records::atom(value.clone())?, &p.typ)?.expr;
                    return Ok(p.new_x(PlaceX::Temporary(value)));
                }
            }
            Ok(p.clone())
        },
        &|_, _, p| Ok(p.clone()),
    )
}
fn proposal(f: &Function, all: &HashMap<Fun, Function>) -> Result<Vec<Expr>, VirErr> {
    if f.x.attrs.infer
        || f.x.mode != Mode::Exec
        || f.x.attrs.is_external_body
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || f.x.attrs.prophecy_dependent
        || f.x.params.len() != 1
        || !trait_getters::shared(&f.x.params[0].x.typ)
            .is_some_and(|t| crate::ast_util::types_equal(t, &f.x.inner_ret.x.typ))
    {
        return Ok(vec![]);
    }
    let Some(body) = &f.x.body else {
        return Ok(vec![]);
    };
    let ExprX::Block(stmts, Some(tail)) = &body.x else {
        return Ok(vec![]);
    };
    let Some((out, _)) = key(tail).filter(|(_, p)| p.is_empty()) else {
        return Ok(vec![]);
    };
    let Some(init) = stmts.iter().find_map(|s| {
        let StmtX::Decl { pattern, init: Some(p), .. } = &s.x else { return None };
        if !matches!(&pattern.x, PatternX::Var(b) if b.name == out) {
            return None;
        }
        let PlaceX::Temporary(e) = &p.x else { return None };
        matches!(&plain(e).x, ExprX::Ctor(_, _, _, None)).then_some(e)
    }) else {
        return Ok(vec![]);
    };
    let ExprX::Ctor(dt, variant, fields, None) = &plain(init).x else { unreachable!() };
    let mut pairs = vec![];
    for field in fields.iter() {
        if !cloning::call(plain(&field.a), all) {
            continue;
        }
        let Some((_, _, args)) = call(&field.a) else {
            continue;
        };
        let Some((root, path)) = key(&args[0])
            .filter(|(n, p)| n == &f.x.params[0].x.name && !p.is_empty() && p.len() <= 4)
        else {
            continue;
        };
        let op = FieldOpr {
            datatype: dt.clone(),
            variant: variant.clone(),
            field: field.name.clone(),
            get_variant: false,
            check: VariantCheck::None,
        };
        let target = SpannedTyped::new(
            &init.span,
            &field.a.typ,
            ExprX::UnaryOpr(
                UnaryOpr::Field(op),
                SpannedTyped::new(&tail.span, &f.x.inner_ret.x.typ, ExprX::Var(out.clone())),
            ),
        );
        pairs.push((root, path, target));
    }
    if pairs.is_empty() || pairs.len() > 4 {
        return Ok(vec![]);
    }
    let mut posts = vec![];
    for e in f.x.ensure.0.iter() {
        growth_goals::clauses(e, &mut posts);
    }
    let mut facts = vec![];
    for post in posts {
        let ExprX::Logical(LogicalOp::Implies, input, output) = &post.x else {
            continue;
        };
        let (Some((a, ts, args)), Some((b, us, outs))) = (call(input), call(output)) else {
            continue;
        };
        if a != b
            || args.len() != 1
            || outs.len() != 1
            || ts.len() != us.len()
            || !ts.iter().zip(us.iter()).all(|(a, b)| crate::ast_util::types_equal(a, b))
            || !key(&args[0]).is_some_and(|(n, p)| n == f.x.params[0].x.name && p.is_empty())
            || !key(&outs[0]).is_some_and(|(n, p)| n == f.x.inner_ret.x.name && p.is_empty())
        {
            continue;
        }
        let expanded = expand(input, &f.x.owning_module, all)?;
        let mut clauses = vec![];
        growth_goals::clauses(plain(&expanded), &mut clauses);
        for q in clauses {
            let ExprX::Quant(kind, vars, body) = &q.x else {
                continue;
            };
            if kind.quant != air::ast::Quant::Forall
                || vars.len() != 1
                || !crate::ast_util::types_equal(&vars[0].a, &int_typ())
            {
                continue;
            }
            let (triggers, inner) = match &body.x {
                ExprX::WithTriggers { triggers, body } => (Some(triggers), body),
                _ => (None, body),
            };
            let ExprX::Logical(LogicalOp::Implies, guard, property) = &inner.x else {
                continue;
            };
            let mapped_guard = rewrite(guard, &pairs)?;
            let mapped_property = rewrite(property, &pairs)?;
            if owned::skeleton(&mapped_property) == owned::skeleton(property) {
                continue;
            }
            let retained = SpannedTyped::new(
                &q.span,
                &bool_typ(),
                ExprX::Logical(LogicalOp::And, guard.clone(), mapped_property),
            );
            let mut mapped = SpannedTyped::new(
                &q.span,
                &bool_typ(),
                ExprX::Logical(LogicalOp::Implies, mapped_guard, retained),
            );
            if let Some(triggers) = triggers {
                mapped = mapped.new_x(ExprX::WithTriggers {
                    triggers: triggers.clone(),
                    body: mapped.clone(),
                });
            }
            let quantified = q.new_x(ExprX::Quant(*kind, vars.clone(), mapped));
            facts.push(SpannedTyped::new(
                &q.span,
                &bool_typ(),
                ExprX::Logical(LogicalOp::Implies, input.clone(), quantified),
            ));
            if facts.len() == 4 {
                return facts.iter().map(growth_goals::current).collect();
            }
        }
    }
    facts.iter().map(growth_goals::current).collect()
}
/// Independently project away only new, pure checked assertions. In particular,
/// neither the executable statements, return expression, nor contracts may change.
fn validate_decoration(
    original: &Function,
    candidate: &Function,
    all: &HashMap<Fun, Function>,
) -> Result<(), VirErr> {
    let bad = || {
        unsupported(
            &original.span,
            "pointwise assistance changed source or added an invalid assertion",
        )
    };
    let Some(body) = &original.x.body else { return Err(bad()) };
    let Some(decorated) = &candidate.x.body else { return Err(bad()) };
    let ExprX::Block(before, _) = &body.x else { return Err(bad()) };
    let ExprX::Block(after, tail) = &decorated.x else { return Err(bad()) };
    let count = after.len().checked_sub(before.len()).ok_or_else(bad)?;
    if count == 0 || count > 4 {
        return Err(bad());
    }
    for statement in &after[before.len()..] {
        let StmtX::Expr(assertion) = &statement.x else { return Err(bad()) };
        let ExprX::AssertAssume { is_assume: false, expr, msg: None } = &assertion.x else {
            return Err(bad());
        };
        if !crate::ast_util::types_equal(&assertion.typ, &crate::ast_util::unit_typ())
            || !crate::ast_util::types_equal(&expr.typ, &bool_typ())
        {
            return Err(bad());
        }
        growth_goals::pure(expr, all)?;
    }
    let mut projected = candidate.x.clone();
    projected.body =
        Some(decorated.new_x(ExprX::Block(Arc::new(after[..before.len()].to_vec()), tail.clone())));
    if serde_json::to_vec(&projected).map_err(|_| bad())?
        != serde_json::to_vec(&original.x).map_err(|_| bad())?
    {
        return Err(bad());
    }
    Ok(())
}

fn install(
    f: &Function,
    facts: &[Expr],
    all: &HashMap<Fun, Function>,
    id: &mut u64,
) -> Result<Function, VirErr> {
    let body = f.x.body.as_ref().unwrap();
    let ExprX::Block(original, tail) = &body.x else { unreachable!() };
    let mut stmts = (**original).clone();
    for fact in facts {
        growth_goals::pure(fact, all)?;
        let fact = growth_goals::fresh(fact, id)?;
        let mut span = fact.span.clone();
        span.id = *id;
        *id -= 1;
        let assertion = SpannedTyped::new(
            &span,
            &crate::ast_util::unit_typ(),
            ExprX::AssertAssume { is_assume: false, expr: fact, msg: None },
        );
        stmts.push(crate::def::Spanned::new(span, StmtX::Expr(assertion)));
    }
    let mut x = f.x.clone();
    x.body = Some(body.new_x(ExprX::Block(Arc::new(stmts), tail.clone())));
    let candidate = f.new_x(x);
    validate_decoration(f, &candidate, all)?;
    Ok(candidate)
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = crate::infer::pipeline::function_map(fs, imported);
    let mut id = u64::MAX / 4;
    fs.iter()
        .map(|f| {
            let facts = proposal(f, &all)?;
            if facts.is_empty() {
                return Ok(f.clone());
            }
            let candidate = install(f, &facts, &all, &mut id)?;
            report.pointwise_facts.push((f.x.name.clone(), facts.len()));
            Ok(candidate)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointwise_projection_accepts_only_pure_checked_assertions_after_source_statements() {
        let original = crate::infer::loops::tests::counted_function();
        let all = HashMap::new();
        let fact = boolean(&original.span, true).expr;
        let candidate = install(&original, &[fact], &all, &mut (u64::MAX / 4)).unwrap();
        let body = candidate.x.body.as_ref().unwrap();
        let ExprX::Block(stmts, tail) = &body.x else { unreachable!() };
        for attack in 0..7 {
            let mut changed = candidate.x.clone();
            let mut ss = (**stmts).clone();
            let StmtX::Expr(assertion) = &ss.last().unwrap().x else { unreachable!() };
            let ExprX::AssertAssume { expr, .. } = &assertion.x else { unreachable!() };
            match attack {
                0 => {
                    *ss.last_mut().unwrap() = crate::def::Spanned::new(
                        assertion.span.clone(),
                        StmtX::Expr(assertion.new_x(ExprX::AssertAssume {
                            is_assume: true,
                            expr: expr.clone(),
                            msg: None,
                        })),
                    );
                }
                1 => {
                    ss.remove(0);
                }
                2 => {
                    ss.swap(0, stmts.len() - 1);
                }
                3 => {
                    changed.require = Arc::new(vec![boolean(&original.span, false).expr]);
                }
                4 => {
                    let effect = SpannedTyped::new(
                        &expr.span,
                        &bool_typ(),
                        ExprX::AssertAssume {
                            is_assume: true,
                            expr: boolean(&expr.span, false).expr,
                            msg: None,
                        },
                    );
                    let fact = expr.new_x(ExprX::WithTriggers {
                        triggers: Arc::new(vec![Arc::new(vec![effect])]),
                        body: expr.clone(),
                    });
                    *ss.last_mut().unwrap() = crate::def::Spanned::new(
                        assertion.span.clone(),
                        StmtX::Expr(assertion.new_x(ExprX::AssertAssume {
                            is_assume: false,
                            expr: fact,
                            msg: None,
                        })),
                    );
                }
                5 => {
                    let mutated = crate::ast_visitor::map_expr_visitor(body, &|e| {
                        if let ExprX::Assign { place, op, typ, resolve, .. } = &e.x {
                            Ok(e.new_x(ExprX::Assign {
                                place: place.clone(),
                                op: *op,
                                typ: typ.clone(),
                                resolve: *resolve,
                                rhs: SpannedTyped::new(
                                    &e.span,
                                    typ,
                                    ExprX::Const(Constant::Int(2.into())),
                                ),
                            }))
                        } else {
                            Ok(e.clone())
                        }
                    })
                    .unwrap();
                    changed.body = Some(mutated);
                }
                _ => {}
            }
            if attack != 5 {
                changed.body = Some(body.new_x(ExprX::Block(
                    Arc::new(ss),
                    if attack == 6 { None } else { tail.clone() },
                )));
            }
            assert!(
                validate_decoration(&original, &candidate.new_x(changed), &all).is_err(),
                "attack {attack}"
            );
        }
    }
}
