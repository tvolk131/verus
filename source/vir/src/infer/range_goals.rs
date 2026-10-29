//! Untrusted prefix induction for ordinary native integer-range mutation goals.
//! Retains the complete source loop and frontend iterator proof scaffolding.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};
use growth_goals::{clauses, current, fresh, future, pure, seq_len};

fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::ReadPlace(p, _) => {
            if let PlaceX::Temporary(e) = &p.x {
                plain(e)
            } else {
                e
            }
        }
        ExprX::Block(ss, Some(e)) if ss.is_empty() => plain(e),
        _ => e,
    }
}
fn endpoints(e: &Expr, all: &HashMap<Fun, Function>) -> Option<(Expr, Expr)> {
    let TypX::Datatype(Dt::Path(p), ts, _) = &*e.typ else { return None };
    if ts.len() != 1 || !matches!(&*ts[0], TypX::Int(IntRange::U(_) | IntRange::USize)) {
        return None;
    }
    if models::path_is(p, CrateId::Core, &["ops", "range", "RangeInclusive"]) {
        let ExprX::Call {
            target: CallTarget::Fun(CallTargetKind::Static, n, ..),
            args,
            post_args: None,
            body: None,
        } = &plain(e).x
        else {
            return None;
        };
        let f = all.get(n)?;
        if args.len() != 2
            || !f
                .x
                .proxy
                .as_ref()
                .is_some_and(|p| models::spec_proxy(&p.x, &["std_specs", "range"], "new"))
        {
            return None;
        }
        return Some((args[0].clone(), args[1].clone()));
    }
    if models::path_is(p, CrateId::Core, &["ops", "range", "Range"]) {
        let ExprX::Ctor(_, _, fields, None) = &plain(e).x else { return None };
        return Some((
            fields.iter().find(|f| f.name.as_str() == "start")?.a.clone(),
            fields.iter().find(|f| f.name.as_str() == "end")?.a.clone(),
        ));
    }
    None
}
fn has_entry(e: &Expr) -> bool {
    let mut yes = false;
    expr_visitor_walk(e, &mut |e| {
        yes |= matches!(&e.x, ExprX::Old(_) | ExprX::VarAt(_, VarAt::Pre));
        VisitorControlFlow::<()>::Recurse
    });
    yes
}
fn prefix_trigger(
    body: &Expr,
    binder: &VarBinder<Typ>,
    all: &HashMap<Fun, Function>,
) -> Option<Expr> {
    let mut sequences = std::collections::BTreeMap::new();
    let mut budget = 1024usize;
    expr_visitor_walk(body, &mut |e| {
        if budget == 0 {
            return VisitorControlFlow::Stop(());
        }
        budget -= 1;
        if let ExprX::Call {
            target: CallTarget::Fun(_, n, ..),
            args,
            post_args: None,
            body: None,
        } = &e.x
        {
            if all.get(n).and_then(collections::model) == Some(collections::Model::SeqLen)
                && args.len() == 1
                && future(&args[0])
            {
                let mut dependent = false;
                expr_visitor_walk(&args[0], &mut |e| {
                    dependent |= matches!(&e.x,ExprX::Var(n) if n==&binder.name);
                    VisitorControlFlow::<()>::Recurse
                });
                if !dependent {
                    sequences.insert(owned::skeleton(&args[0]), args[0].clone());
                }
            }
        }
        VisitorControlFlow::Recurse
    });
    if budget == 0 || sequences.len() != 1 {
        return None;
    }
    let sequence = sequences.into_values().next()?;
    let item = iterators::sequence(&sequence.typ)?.clone();
    let model =
        all.values().find(|f| collections::model(f) == Some(collections::Model::SeqIndex))?;
    trait_getters::instantiate(
        model,
        &Arc::new(vec![item]),
        vec![
            records::atom(sequence).ok()?,
            records::atom(SpannedTyped::new(
                &body.span,
                &binder.a,
                ExprX::Var(binder.name.clone()),
            ))
            .ok()?,
        ],
        &Arc::new(vec![]),
    )
    .ok()
    .map(|t| t.expr)
}
fn prefix(e: &Expr, end: &Expr, all: &HashMap<Fun, Function>) -> Result<Expr, VirErr> {
    let ExprX::Quant(q, bs, body) = &e.x else { return Ok(e.clone()) };
    if q.quant != air::ast::Quant::Forall
        || bs.len() != 1
        || !crate::ast_util::types_equal(&bs[0].a, &int_typ())
    {
        return Ok(e.clone());
    }
    let var = SpannedTyped::new(&e.span, &int_typ(), ExprX::Var(bs[0].name.clone()));
    let guard = SpannedTyped::new(
        &e.span,
        &bool_typ(),
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Lt), var, end.clone()),
    );
    let wrap = |b: &Expr| {
        SpannedTyped::new(
            &b.span,
            &bool_typ(),
            ExprX::Logical(LogicalOp::Implies, guard.clone(), b.clone()),
        )
    };
    let body = match &body.x {
        ExprX::WithTriggers { triggers, body: b } => {
            body.new_x(ExprX::WithTriggers { triggers: triggers.clone(), body: wrap(b) })
        }
        _ => {
            if let Some(trigger) = prefix_trigger(body, &bs[0], all) {
                // A hidden pointwise predicate may not expose the array access to
                // the default trigger heuristic. The loop's bound supplies it.
                let wrapped = wrap(body);
                wrapped.new_x(ExprX::WithTriggers {
                    triggers: Arc::new(vec![Arc::new(vec![trigger])]),
                    body: wrapped.clone(),
                })
            } else {
                wrap(body)
            }
        }
    };
    Ok(e.new_x(ExprX::Quant(*q, bs.clone(), body)))
}
fn index(source: &owned::native_for::Source, all: &HashMap<Fun, Function>) -> Option<Expr> {
    let f = all.values().find(|f| {
        f.x.mode == Mode::Spec
            && f.x.name.path.krate == CrateId::Vstd
            && f.x.name.path.segments.len() == 4
            && f.x.name.path.segments[0].as_str() == "std_specs"
            && f.x.name.path.segments[1].as_str() == "iter"
            && f.x.name.path.segments[2].starts_with("impl")
            && f.x.name.path.last_segment().as_str() == "index"
    })?;
    let TypX::Datatype(_, types, _) = &*source.wrapper.typ else { return None };
    Some(SpannedTyped::new(
        &source.loop_expr.span,
        &int_typ(),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                f.x.name.clone(),
                types.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(vec![SpannedTyped::new(
                &source.loop_expr.span,
                &source.wrapper.typ,
                ExprX::Var(source.wrapper.name.clone()),
            )]),
            post_args: None,
            body: None,
        },
    ))
}
/// Add only pure invariants; the native iterator's termination argument stays intact.
fn install(
    f: &Function,
    source: &Expr,
    invs: Vec<Expr>,
    all: &HashMap<Fun, Function>,
) -> Result<Function, VirErr> {
    let mut id = u64::MAX - 1;
    let added = invs
        .iter()
        .map(|e| {
            Ok(LoopInvariant {
                kind: LoopInvariantKind::InvariantAndEnsures,
                inv: fresh(e, &mut id)?,
            })
        })
        .collect::<Result<Vec<_>, VirErr>>()?;
    let body = map_expr_visitor(f.x.body.as_ref().unwrap(), &|e| {
        if e.span.id != source.span.id {
            return Ok(e.clone());
        }
        let ExprX::Loop {
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
        else {
            return Err(unsupported(&e.span, "range loop identity"));
        };
        Ok(e.new_x(ExprX::Loop {
            loop_isolation: false,
            allow_complex_invariants: *allow_complex_invariants,
            is_for_loop: *is_for_loop,
            assume_termination: *assume_termination,
            label: label.clone(),
            cond: cond.clone(),
            body: body.clone(),
            atomic_call: *atomic_call,
            invs: Arc::new(added.iter().chain(invs.iter()).cloned().collect()),
            decrease: decrease.clone(),
        }))
    })?;
    validate_decoration(f, source, &body, all)?;
    let mut x = f.x.clone();
    x.body = Some(body);
    Ok(f.new_x(x))
}

fn validate_decoration(
    f: &Function,
    source: &Expr,
    candidate: &Expr,
    all: &HashMap<Fun, Function>,
) -> Result<(), VirErr> {
    let bad = || {
        unsupported(
            &f.span,
            "range proof decoration changed code or iterator scaffolding, or contains invalid annotations",
        )
    };
    let ExprX::Loop {
        invs: original,
        decrease: original_measure,
        loop_isolation: original_isolation,
        ..
    } = &source.x
    else {
        return Err(bad());
    };
    let found = std::cell::Cell::new(0);
    let projected = map_expr_visitor(candidate, &|e| {
        if e.span.id != source.span.id {
            return Ok(e.clone());
        }
        let ExprX::Loop {
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            atomic_call,
            invs,
            decrease,
            loop_isolation,
        } = &e.x
        else {
            return Err(bad());
        };
        let added = invs.len().checked_sub(original.len()).ok_or_else(bad)?;
        if *loop_isolation
            || added == 0
            || added > 14
            || serde_json::to_vec(&invs[added..]).map_err(|_| bad())?
                != serde_json::to_vec(original).map_err(|_| bad())?
            || serde_json::to_vec(decrease).map_err(|_| bad())?
                != serde_json::to_vec(original_measure).map_err(|_| bad())?
        {
            return Err(bad());
        }
        for inv in &invs[..added] {
            if inv.kind != LoopInvariantKind::InvariantAndEnsures
                || !crate::ast_util::types_equal(&inv.inv.typ, &bool_typ())
            {
                return Err(bad());
            }
            pure(&inv.inv, all)?;
        }
        found.set(found.get() + 1);
        Ok(e.new_x(ExprX::Loop {
            loop_isolation: *original_isolation,
            allow_complex_invariants: *allow_complex_invariants,
            is_for_loop: *is_for_loop,
            assume_termination: *assume_termination,
            label: label.clone(),
            cond: cond.clone(),
            body: body.clone(),
            atomic_call: *atomic_call,
            invs: original.clone(),
            decrease: decrease.clone(),
        }))
    })?;
    if found.get() != 1
        || serde_json::to_vec(&projected).map_err(|_| bad())?
            != serde_json::to_vec(f.x.body.as_ref().unwrap()).map_err(|_| bad())?
    {
        return Err(bad());
    }
    Ok(())
}
fn prepare_one(
    f: &Function,
    all: &HashMap<Fun, Function>,
    report: &mut GenerationReport,
) -> Result<Function, VirErr> {
    if f.x.attrs.infer
        || f.x.mode != Mode::Exec
        || f.x.attrs.is_external_body
        || f.x.attrs.is_unsafe
        || f.x.attrs.is_async
        || f.x.attrs.exec_assume_termination
        || f.x.attrs.exec_allows_no_decreases_clause
        || !enums::unit(&f.x.inner_ret.x.typ)
        || f.x.body.is_none()
        || f.x.params.iter().any(|p| p.x.user_mut)
        || !f.x.params.first().is_some_and(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
    {
        return Ok(f.clone());
    }
    let sources = owned::native_for::sources(f);
    if sources.len() != 1 {
        return Ok(f.clone());
    }
    let source = &sources[0];
    let mut count = 0;
    let mut budget = 8192usize;
    expr_visitor_walk(f.x.body.as_ref().unwrap(), &mut |e| {
        if budget == 0 {
            return VisitorControlFlow::Stop(());
        }
        budget -= 1;
        if matches!(&e.x, ExprX::Loop { .. }) {
            count += 1;
        }
        VisitorControlFlow::Recurse
    });
    if count != 1 || budget == 0 {
        return Ok(f.clone());
    }
    let ExprX::Loop { invs, decrease, .. } = &source.loop_expr.x else { unreachable!() };
    if invs.len() != 7 || decrease.len() != 1 {
        return Ok(f.clone());
    }
    let Some((start, _)) = endpoints(&source.iterator, all) else {
        return Ok(f.clone());
    };
    let Some(index) = index(source, all) else {
        return Ok(f.clone());
    };
    let start = SpannedTyped::new(
        &start.span,
        &int_typ(),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, current(&start)?),
    );
    let end = SpannedTyped::new(
        &source.loop_expr.span,
        &int_typ(),
        ExprX::Binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
            start,
            current(&index)?,
        ),
    );
    let mut posts = vec![];
    for p in f.x.ensure.0.iter() {
        clauses(p, &mut posts);
    }
    if posts.len() > 12 {
        return Ok(f.clone());
    }
    let mut proposed = vec![];
    for p in posts {
        if !future(&p) {
            continue;
        }
        if let ExprX::Binary(BinaryOp::Eq(_), a, b) = &p.x {
            if seq_len(a, all) && future(a) && !future(b) && !seq_len(b, all) {
                let len = current(a)?;
                let receiver = &f.x.params[0].x.name;
                let initial = map_expr_visitor(&len, &|e| {
                    Ok(match &e.x {
                        ExprX::Var(n) if n == receiver => {
                            e.new_x(ExprX::VarAt(n.clone(), VarAt::Pre))
                        }
                        ExprX::ReadPlace(p, _) if matches!(&p.x, PlaceX::Local(n) if n == receiver) => {
                            e.new_x(ExprX::VarAt(receiver.clone(), VarAt::Pre))
                        }
                        _ => e.clone(),
                    })
                })?;
                proposed.push(SpannedTyped::new(
                    &p.span,
                    &bool_typ(),
                    ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), initial, len.clone()),
                ));
                let nonempty = SpannedTyped::new(
                    &p.span,
                    &bool_typ(),
                    ExprX::Binary(
                        BinaryOp::Inequality(InequalityOp::Gt),
                        current(&index)?,
                        SpannedTyped::new(
                            &p.span,
                            &int_typ(),
                            ExprX::Const(Constant::Int(0.into())),
                        ),
                    ),
                );
                let covered = SpannedTyped::new(
                    &p.span,
                    &bool_typ(),
                    ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), end.clone(), len.clone()),
                );
                proposed.push(SpannedTyped::new(
                    &p.span,
                    &bool_typ(),
                    ExprX::Logical(LogicalOp::Implies, nonempty, covered),
                ));
                proposed.push(SpannedTyped::new(
                    &p.span,
                    &bool_typ(),
                    ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), current(a)?, current(b)?),
                ));
                continue;
            }
        }
        let p = if !has_entry(&p) { prefix(&p, &end, all)? } else { p };
        proposed.push(current(&p)?);
    }
    if proposed.is_empty() {
        return Ok(f.clone());
    }
    let result = install(f, &source.loop_expr, proposed, all)?;
    report.range_loops.push((f.x.name.clone(), source.loop_expr.span.clone()));
    Ok(result)
}
pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    fs.iter().map(|f| prepare_one(f, &all, report)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn range_projection_preserves_source_code_invariants_and_termination() {
        let original = crate::infer::loops::tests::counted_function();
        let mut x = original.x.clone();
        x.body = Some(fresh(x.body.as_ref().unwrap(), &mut (u64::MAX / 2)).unwrap());
        let original = original.new_x(x);
        let mut source = None;
        expr_visitor_walk(original.x.body.as_ref().unwrap(), &mut |e| {
            if matches!(&e.x, ExprX::Loop { .. }) {
                source = Some(e.clone());
            }
            VisitorControlFlow::<()>::Recurse
        });
        let source = source.unwrap();
        let all = HashMap::new();
        let decorated =
            install(&original, &source, vec![boolean(&source.span, true).expr], &all).unwrap();
        let candidate = decorated.x.body.as_ref().unwrap();
        for attack in 0..5 {
            let changed = map_expr_visitor(candidate, &|e| {
                if attack == 0 {
                    if let ExprX::Assign { place, op, typ, resolve, .. } = &e.x {
                        return Ok(e.new_x(ExprX::Assign {
                            place: place.clone(),
                            rhs: SpannedTyped::new(
                                &e.span,
                                typ,
                                ExprX::Const(Constant::Int(2.into())),
                            ),
                            op: *op,
                            typ: typ.clone(),
                            resolve: *resolve,
                        }));
                    }
                }
                if e.span.id != source.span.id {
                    return Ok(e.clone());
                }
                let ExprX::Loop {
                    allow_complex_invariants,
                    is_for_loop,
                    assume_termination,
                    label,
                    cond,
                    body,
                    atomic_call,
                    invs,
                    decrease,
                    loop_isolation,
                } = &e.x
                else {
                    unreachable!()
                };
                Ok(e.new_x(ExprX::Loop {
                    allow_complex_invariants: *allow_complex_invariants,
                    is_for_loop: *is_for_loop,
                    assume_termination: if attack == 4 { true } else { *assume_termination },
                    label: label.clone(),
                    cond: cond.clone(),
                    body: body.clone(),
                    atomic_call: *atomic_call,
                    loop_isolation: *loop_isolation,
                    invs: if attack == 1 {
                        Arc::new(vec![])
                    } else if attack == 3 {
                        Arc::new(vec![LoopInvariant {
                            kind: LoopInvariantKind::InvariantAndEnsures,
                            inv: SpannedTyped::new(
                                &e.span,
                                &bool_typ(),
                                ExprX::AssertAssume {
                                    is_assume: true,
                                    expr: boolean(&e.span, false).expr,
                                    msg: None,
                                },
                            ),
                        }])
                    } else {
                        invs.clone()
                    },
                    decrease: if attack == 2 {
                        Arc::new(vec![SpannedTyped::new(
                            &e.span,
                            &int_typ(),
                            ExprX::Const(Constant::Int(0.into())),
                        )])
                    } else {
                        decrease.clone()
                    },
                }))
            })
            .unwrap();
            assert!(
                validate_decoration(&original, &source, &changed, &all).is_err(),
                "attack {attack}"
            );
        }
    }
}
