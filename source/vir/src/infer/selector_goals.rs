//! Untrusted ordinary contracts for a checked preparation call followed by a
//! direct mutable vector-element borrow. No body or input contract is replaced.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor_env};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Current,
    Entry,
    Future,
}
fn plain(e: &Expr) -> &Expr {
    match &e.x {
        ExprX::Block(ss, Some(t)) if ss.is_empty() => plain(t),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, e) => plain(e),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Temporary(e) => plain(e),
            _ => e,
        },
        _ => e,
    }
}
fn projection(e: &Expr) -> Option<(VarIdent, Phase, Vec<FieldOpr>)> {
    fn place(p: &Place) -> Option<(VarIdent, Phase, Vec<FieldOpr>)> {
        match &p.x {
            PlaceX::Local(n) => Some((n.clone(), Phase::Current, vec![])),
            PlaceX::Temporary(e) => projection(e),
            PlaceX::DerefMut(p) => place(p),
            PlaceX::Field(op, p) => {
                let (n, q, mut path) = place(p)?;
                path.push(op.clone());
                Some((n, q, path))
            }
            _ => None,
        }
    }
    match &e.x {
        ExprX::Var(n) => Some((n.clone(), Phase::Current, vec![])),
        ExprX::VarAt(n, VarAt::Pre) => Some((n.clone(), Phase::Entry, vec![])),
        ExprX::ReadPlace(p, _) | ExprX::BorrowMut(p) | ExprX::TwoPhaseBorrowMut(p) => place(p),
        ExprX::Unary(UnaryOp::MutRefCurrent, e) => projection(e),
        ExprX::Old(e) => {
            let (n, _, p) = projection(e)?;
            Some((n, Phase::Entry, p))
        }
        ExprX::Unary(UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_), e) => {
            let (n, _, p) = projection(e)?;
            Some((n, Phase::Future, p))
        }
        ExprX::UnaryOpr(UnaryOpr::Field(op), e) => {
            let (n, q, mut p) = projection(e)?;
            p.push(op.clone());
            Some((n, q, p))
        }
        ExprX::Block(ss, Some(t)) if ss.is_empty() => projection(t),
        ExprX::EvalAndResolve(e, _) => projection(e),
        _ => None,
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
fn spec(e: &Expr) -> Result<Expr, VirErr> {
    crate::ast_visitor::map_expr_visitor(e, &|e| {
        Ok(match &e.x {
            ExprX::ReadPlace(p, _) => e.new_x(ExprX::ReadPlace(
                p.clone(),
                UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
            )),
            _ => e.clone(),
        })
    })
}
fn pure_contract(e: &Expr, f: &Function, all: &HashMap<Fun, Function>) -> Result<(), VirErr> {
    // Contracts retain ordinary Old nodes. Loop-only VarAt::Pre snapshots do
    // not have a binding in a function's global postcondition axiom.
    let allowed: std::collections::HashSet<_> =
        f.x.params
            .iter()
            .map(|p| p.x.name.clone())
            .chain(std::iter::once(f.x.inner_ret.x.name.clone()))
            .collect();
    let bound = |scope: &crate::ast_visitor::VisitorScopeMap, n: &VarIdent| {
        allowed.contains(n) || scope.get(n).is_some()
    };
    let checked = map_expr_visitor_env(
        &spec(e)?,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, scope, e| {
            if matches!(&e.x,ExprX::Var(n)|ExprX::VarAt(n,_) if !bound(scope,n)) {
                return Err(unsupported(&e.span, "undeclared variable in selector contract"));
            }
            if let ExprX::Old(a) = &e.x {
                if !projection(a).is_some_and(|(n, q, path)| {
                    q == Phase::Current
                        && path.is_empty()
                        && f.x
                            .params
                            .iter()
                            .any(|p| p.x.name == n && matches!(&*p.x.typ, TypX::MutRef(_)))
                }) {
                    return Err(unsupported(&e.span, "selector contract entry snapshot"));
                }
                // Only for the purity traversal: keep and inspect the entire
                // operand. The installed/published formula is never rewritten.
                Ok(e.new_x(ExprX::Block(Arc::new(vec![]), Some(a.clone()))))
            } else {
                Ok(e.clone())
            }
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, scope, p| {
            if matches!(&p.x,PlaceX::Local(n) if !bound(scope,n)) {
                Err(unsupported(&p.span, "undeclared place in selector contract"))
            } else {
                Ok(p.clone())
            }
        },
        &|_, _, p| Ok(p.clone()),
    )?;
    growth_goals::pure(&checked, all)
}
fn header(f: &Function) -> bool {
    let x = &f.x;
    !x.attrs.infer
        && x.mode == Mode::Exec
        && !x.attrs.is_external_body
        && !x.attrs.is_unsafe
        && !x.attrs.is_async
        && !x.attrs.prophecy_dependent
        && !x.attrs.exec_assume_termination
        && !x.attrs.exec_allows_no_decreases_clause
        && x.proxy.is_none()
        && matches!(x.kind, FunctionKind::Static)
        && matches!(x.item_kind, ItemKind::Function)
        && (1..=4).contains(&x.params.len())
        && x.typ_params.len() <= 4
        && x.typ_bounds.len() <= 32
        && !x.params.iter().any(|p| p.x.user_mut)
        && x.params.iter().all(|p| p.x.mode == Mode::Exec)
        && x.inner_ret.x.mode == Mode::Exec
        && x.outer_ret.x.mode == Mode::Exec
        && matches!(&*x.params[0].x.typ,TypX::MutRef(t) if matches!(&**t,TypX::Datatype(..)))
        && x.params.iter().skip(1).all(|p| eligibility::scalar(&p.x.typ))
        && matches!(&*x.inner_ret.x.typ, TypX::MutRef(_))
        && crate::ast_util::types_equal(&x.inner_ret.x.typ, &x.outer_ret.x.typ)
        && x.ensure.1.is_empty()
        && x.returns.is_none()
}
fn instantiate(e: &Expr, callee: &Function, ts: &Typs, args: &Exprs) -> Result<Expr, VirErr> {
    let mut names = HashMap::new();
    for (p, a) in callee.x.params.iter().zip(args.iter()) {
        let (n, phase, path) =
            projection(a).ok_or_else(|| unsupported(&a.span, "selector preparation argument"))?;
        if phase != Phase::Current || !path.is_empty() {
            return Err(unsupported(&a.span, "selector preparation argument projection"));
        }
        names.insert(p.x.name.clone(), n);
    }
    let types: HashMap<_, _> =
        callee.x.typ_params.iter().cloned().zip(ts.iter().cloned()).collect();
    let mapped = map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(match &e.x {
                ExprX::Var(n) if names.contains_key(n) => e.new_x(ExprX::Var(names[n].clone())),
                _ => e.clone(),
            })
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| {
            Ok(match &**t {
                TypX::TypParam(n) => types.get(n).cloned().unwrap_or_else(|| t.clone()),
                _ => t.clone(),
            })
        },
        &|_, _, p| {
            Ok(match &p.x {
                PlaceX::Local(n) if names.contains_key(n) => {
                    p.new_x(PlaceX::Local(names[n].clone()))
                }
                _ => p.clone(),
            })
        },
        &|_, _, p| Ok(p.clone()),
    )?;
    spec(&mapped)
}
fn state(f: &Function, phase: Phase) -> Expr {
    let p = &f.x.params[0];
    let TypX::MutRef(t) = &*p.x.typ else { unreachable!() };
    let mut v = SpannedTyped::new(&f.span, &p.x.typ, ExprX::Var(p.x.name.clone()));
    if phase == Phase::Entry {
        v = v.new_x(ExprX::Old(v.clone()));
    }
    SpannedTyped::new(
        &f.span,
        t,
        ExprX::Unary(
            if phase == Phase::Future {
                UnaryOp::MutRefFuture(MutRefFutureSourceName::Final)
            } else {
                UnaryOp::MutRefCurrent
            },
            v,
        ),
    )
}
fn result(f: &Function, future: bool) -> Expr {
    let p = &f.x.inner_ret;
    let TypX::MutRef(t) = &*p.x.typ else { unreachable!() };
    SpannedTyped::new(
        &f.span,
        t,
        ExprX::Unary(
            if future {
                UnaryOp::MutRefFuture(MutRefFutureSourceName::Final)
            } else {
                UnaryOp::MutRefCurrent
            },
            SpannedTyped::new(&f.span, &p.x.typ, ExprX::Var(p.x.name.clone())),
        ),
    )
}
fn field(base: Expr, op: &FieldOpr, typ: &Typ) -> Expr {
    SpannedTyped::new(&base.span, typ, ExprX::UnaryOpr(UnaryOpr::Field(op.clone()), base.clone()))
}
fn relation(span: &Span, op: BinaryOp, a: Expr, b: Expr) -> Expr {
    SpannedTyped::new(span, &bool_typ(), ExprX::Binary(op, a, b))
}
fn logic(span: &Span, op: LogicalOp, a: Expr, b: Expr) -> Expr {
    SpannedTyped::new(span, &bool_typ(), ExprX::Logical(op, a, b))
}
fn sequence_model(
    kind: collections::Model,
    sequence: Expr,
    extra: Vec<Expr>,
    all: &HashMap<Fun, Function>,
) -> Result<Expr, VirErr> {
    let item = iterators::sequence(&sequence.typ)
        .ok_or_else(|| unsupported(&sequence.span, "selector sequence type"))?
        .clone();
    let f = all
        .values()
        .find(|f| collections::model(f) == Some(kind))
        .ok_or_else(|| unsupported(&sequence.span, "selector sequence vocabulary"))?;
    let args =
        std::iter::once(sequence).chain(extra).map(records::atom).collect::<Result<Vec<_>, _>>()?;
    Ok(trait_getters::instantiate(f, &Arc::new(vec![item]), args, &Arc::new(vec![]))?.expr)
}
fn view(vector: Expr, all: &HashMap<Fun, Function>) -> Result<Expr, VirErr> {
    let ts = collections::vec_args(&vector.typ)
        .ok_or_else(|| unsupported(&vector.span, "selector vector type"))?
        .clone();
    let f = all
        .values()
        .find(|f| collections::model(f) == Some(collections::Model::View))
        .ok_or_else(|| unsupported(&vector.span, "selector vector view"))?;
    let term = trait_getters::instantiate(f, &ts, vec![records::atom(vector)?], &Arc::new(vec![]))?;
    let FunctionKind::TraitMethodImpl { method, impl_path, trait_typ_args, .. } = &f.x.kind else {
        return Err(unsupported(&term.expr.span, "selector vector-view implementation"));
    };
    let substitutions = f.x.typ_params.iter().cloned().zip(ts.iter().cloned()).collect();
    let ExprX::Call { args, .. } = &term.expr.x else { unreachable!() };
    Ok(term.expr.new_x(ExprX::Call {
        target: CallTarget::Fun(
            CallTargetKind::DynamicResolved {
                resolved: f.x.name.clone(),
                typs: ts,
                impl_paths: Arc::new(vec![]),
                is_trait_default: false,
            },
            method.clone(),
            Arc::new(
                trait_typ_args
                    .iter()
                    .map(|t| crate::sst_util::subst_typ(&substitutions, t))
                    .collect(),
            ),
            Arc::new(vec![ImplPath::TraitImplPath(impl_path.clone())]),
            CallTargetAttrs {
                autospec: AutospecUsage::Final,
                const_var: false,
                assume_external_allowed: false,
            },
        ),
        args: args.clone(),
        post_args: None,
        body: None,
    }))
}
fn is_sequence(
    e: &Expr,
    f: &Function,
    op: &FieldOpr,
    phase: Phase,
    all: &HashMap<Fun, Function>,
) -> bool {
    let Some((n, _, args)) = call(e) else { return false };
    all.get(n).and_then(collections::model) == Some(collections::Model::View)
        && args.len() == 1
        && projection(&args[0]).is_some_and(|(n, q, path)| {
            n == f.x.params[0].x.name
                && q == phase
                && records::shape::same_path(&path, &[op.clone()])
        })
}
fn stable(e: &Expr, f: &Function, op: &FieldOpr, all: &HashMap<Fun, Function>) -> bool {
    let mut valid = true;
    expr_visitor_walk(e, &mut |e| {
        if let Some((n, q, path)) = projection(e) {
            if n == f.x.params[0].x.name && q == Phase::Future {
                if !path.is_empty() && path[0].field != op.field {
                    return VisitorControlFlow::<()>::Return;
                }
                valid = false;
                return VisitorControlFlow::Stop(());
            }
        }
        if let Some((n, _, args)) = call(e) {
            if all.get(n).and_then(collections::model) == Some(collections::Model::SeqLen)
                && args.len() == 1
                && is_sequence(&args[0], f, op, Phase::Future, all)
            {
                return VisitorControlFlow::Return;
            }
        }
        VisitorControlFlow::Recurse
    });
    valid
}
fn mentions(e: &Expr, n: &VarIdent) -> bool {
    let mut yes = false;
    expr_visitor_walk(e, &mut |e| {
        yes |= projection(e).is_some_and(|(x, _, _)| &x == n);
        VisitorControlFlow::<()>::Recurse
    });
    yes
}
fn future_result(e: &Expr, f: &Function) -> Result<Expr, VirErr> {
    let replacement = result(f, true);
    let name = &f.x.inner_ret.x.name;
    map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        &mut (),
        &|_, _, e| {
            Ok(
                if projection(e).is_some_and(|(n, q, path)| {
                    &n == name && q == Phase::Current && path.is_empty()
                }) && !matches!(&*e.typ, TypX::MutRef(_))
                {
                    records::coerce(&e.span, records::atom(replacement.clone())?, &e.typ)?.expr
                } else {
                    e.clone()
                },
            )
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|_, _, p| {
            Ok(
                if matches!(&p.x,PlaceX::DerefMut(base) if matches!(&base.x,PlaceX::Local(n) if n==name))
                {
                    p.new_x(PlaceX::Temporary(replacement.clone()))
                } else {
                    p.clone()
                },
            )
        },
        &|_, _, p| Ok(p.clone()),
    )
}
fn proposal(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
    inline_posts: Option<&[Expr]>,
) -> Result<Vec<Expr>, VirErr> {
    if !header(f) {
        return Ok(vec![]);
    }
    let Some(body) = &f.x.body else { return Ok(vec![]) };
    let ExprX::Block(ss, Some(tail)) = &body.x else { return Ok(vec![]) };
    if ss.len() != 1 {
        return Ok(vec![]);
    }
    let StmtX::Expr(preparation) = &ss[0].x else { return Ok(vec![]) };
    let Some((borrow, _, borrow_args)) = call(tail) else { return Ok(vec![]) };
    let preparation_call = call(preparation);
    let callee = preparation_call.and_then(|(callee, types, args)| {
        all.get(callee).filter(|c| {
            c.x.mode == Mode::Exec
                && !c.x.attrs.infer
                && !c.x.attrs.is_external_body
                && !c.x.attrs.is_unsafe
                && !c.x.attrs.is_async
                && !c.x.attrs.prophecy_dependent
                && !c.x.attrs.exec_assume_termination
                && !c.x.attrs.exec_allows_no_decreases_clause
                && c.x.proxy.is_none()
                && enums::unit(&c.x.inner_ret.x.typ)
                && c.x.params.len() == args.len()
                && c.x.typ_params.len() == types.len()
        })
    });
    if inline_posts.is_none() && callee.is_none() {
        return Ok(vec![]);
    }
    if !all.get(borrow).is_some_and(|c| collections::model(c) == Some(collections::Model::IndexMut))
        || borrow_args.len() != 2
        || preparation_call.is_some_and(|(_, _, args)| {
            args.is_empty()
                || !projection(&args[0]).is_some_and(|(n, q, path)| {
                    n == f.x.params[0].x.name && q == Phase::Current && path.is_empty()
                })
        })
    {
        return Ok(vec![]);
    }
    let Some((root, phase, path)) = projection(&borrow_args[0]) else { return Ok(vec![]) };
    if root != f.x.params[0].x.name || phase != Phase::Current || path.len() != 1 {
        return Ok(vec![]);
    }
    let op = &path[0];
    let TypX::MutRef(record) = &*f.x.params[0].x.typ else { unreachable!() };
    let Some(fields) = records::shape::fields(record, ds) else { return Ok(vec![]) };
    let Some((_, vector)) =
        fields.iter().find(|(p, _)| records::shape::same_path(&[p.clone()], &path))
    else {
        return Ok(vec![]);
    };
    let mut remaining = FORMULA_BUDGET;
    for e in callee
        .into_iter()
        .flat_map(|c| c.x.ensure.0.iter())
        .chain(f.x.ensure.0.iter())
        .chain(f.x.require.iter())
        .chain(std::iter::once(body))
    {
        expr_visitor_walk(e, &mut |_| {
            if remaining == 0 {
                return VisitorControlFlow::Stop(());
            }
            remaining -= 1;
            VisitorControlFlow::Recurse
        });
        if remaining == 0 {
            return Err(unsupported(&f.span, "selector formula budget exceeded"));
        }
    }
    let Some(vector_types) = collections::vec_args(vector) else { return Ok(vec![]) };
    let TypX::MutRef(element) = &*f.x.inner_ret.x.typ else { unreachable!() };
    if !crate::ast_util::types_equal(&vector_types[0], element) {
        return Ok(vec![]);
    }
    let index = SpannedTyped::new(
        &tail.span,
        &int_typ(),
        ExprX::Unary(
            UnaryOp::Clip { range: IntRange::Int, truncate: false },
            spec(&borrow_args[1])?,
        ),
    );
    let mut index_inputs = true;
    let names: std::collections::HashSet<_> =
        f.x.params.iter().skip(1).map(|p| p.x.name.clone()).collect();
    expr_visitor_walk(&index, &mut |e| {
        if let Some((n, _, _)) = projection(e) {
            index_inputs &= names.contains(&n);
        }
        VisitorControlFlow::<()>::Recurse
    });
    if !index_inputs {
        return Ok(vec![]);
    }
    growth_goals::pure(&index, all)?;
    let final_sequence = view(field(state(f, Phase::Future), op, vector), all)?;
    let old_sequence = view(field(state(f, Phase::Entry), op, vector), all)?;
    let mut facts = vec![relation(
        &f.span,
        BinaryOp::Eq(Mode::Spec),
        sequence_model(
            collections::Model::SeqIndex,
            final_sequence.clone(),
            vec![index.clone()],
            all,
        )?,
        result(f, true),
    )];
    let mut posts = vec![];
    if let Some(inline) = inline_posts {
        for e in inline {
            growth_goals::clauses(e, &mut posts);
        }
    } else if let (Some(callee), Some((_, types, args))) = (callee, preparation_call) {
        for e in callee.x.ensure.0.iter() {
            if mentions(e, &callee.x.inner_ret.x.name) {
                continue;
            }
            growth_goals::clauses(&instantiate(e, callee, types, args)?, &mut posts);
        }
    }
    let mut original_posts = vec![];
    for e in f.x.ensure.0.iter() {
        growth_goals::clauses(e, &mut original_posts);
    }
    let mut supplied = vec![];
    for e in original_posts.iter().filter(|e| mentions(e, &f.x.inner_ret.x.name)) {
        supplied.push(future_result(&spec(e)?, f)?);
    }
    let mut requirements = vec![];
    for e in f.x.require.iter() {
        growth_goals::clauses(e, &mut requirements);
    }
    for post in posts {
        if stable(&post, f, op, all) {
            facts.push(post);
            continue;
        }
        if let ExprX::Binary(BinaryOp::Eq(Mode::Spec), a, b) = &post.x {
            if let Some((n, _, args)) = call(a) {
                if all.get(n).and_then(owned::model) == Some(owned::Model::Take)
                    && args.len() == 2
                    && is_sequence(&args[0], f, op, Phase::Future, all)
                    && is_sequence(b, f, op, Phase::Entry, all)
                {
                    let Some((len, _, len_args)) = call(&args[1]) else { continue };
                    if all.get(len).and_then(collections::model) != Some(collections::Model::SeqLen)
                        || len_args.len() != 1
                        || !is_sequence(&len_args[0], f, op, Phase::Entry, all)
                    {
                        continue;
                    }
                    let name = VarIdent(
                        Arc::new("__verus_infer_selector_index".into()),
                        VarIdentDisambiguate::AirLocal,
                    );
                    let i = SpannedTyped::new(&f.span, &int_typ(), ExprX::Var(name.clone()));
                    let low = relation(
                        &f.span,
                        BinaryOp::Inequality(InequalityOp::Le),
                        SpannedTyped::new(
                            &f.span,
                            &int_typ(),
                            ExprX::Const(Constant::Int(0.into())),
                        ),
                        i.clone(),
                    );
                    let high = relation(
                        &f.span,
                        BinaryOp::Inequality(InequalityOp::Lt),
                        i.clone(),
                        sequence_model(
                            collections::Model::SeqLen,
                            old_sequence.clone(),
                            vec![],
                            all,
                        )?,
                    );
                    let different = relation(&f.span, BinaryOp::Ne, i.clone(), index.clone());
                    let guard = logic(
                        &f.span,
                        LogicalOp::And,
                        logic(&f.span, LogicalOp::And, low, high),
                        different,
                    );
                    let after = sequence_model(
                        collections::Model::SeqIndex,
                        final_sequence.clone(),
                        vec![i.clone()],
                        all,
                    )?;
                    let before = sequence_model(
                        collections::Model::SeqIndex,
                        old_sequence.clone(),
                        vec![i],
                        all,
                    )?;
                    let eq = relation(&f.span, BinaryOp::Eq(Mode::Spec), after.clone(), before);
                    let property = logic(&f.span, LogicalOp::Implies, guard, eq);
                    let triggered = property.new_x(ExprX::WithTriggers {
                        triggers: Arc::new(vec![Arc::new(vec![after])]),
                        body: property.clone(),
                    });
                    facts.push(SpannedTyped::new(
                        &f.span,
                        &bool_typ(),
                        ExprX::Quant(
                            Quant { quant: air::ast::Quant::Forall },
                            Arc::new(vec![Arc::new(VarBinderX { name, a: int_typ() })]),
                            triggered,
                        ),
                    ));
                    // The initially returned element also retains its old
                    // complete value. A caller that leaves the borrow unchanged
                    // can recover preservation of the entire old prefix,
                    // including the selected position.
                    let bound = logic(
                        &f.span,
                        LogicalOp::And,
                        relation(
                            &f.span,
                            BinaryOp::Inequality(InequalityOp::Le),
                            SpannedTyped::new(
                                &f.span,
                                &int_typ(),
                                ExprX::Const(Constant::Int(0.into())),
                            ),
                            index.clone(),
                        ),
                        relation(
                            &f.span,
                            BinaryOp::Inequality(InequalityOp::Lt),
                            index.clone(),
                            sequence_model(
                                collections::Model::SeqLen,
                                old_sequence.clone(),
                                vec![],
                                all,
                            )?,
                        ),
                    );
                    facts.push(logic(
                        &f.span,
                        LogicalOp::Implies,
                        bound,
                        relation(
                            &f.span,
                            BinaryOp::Eq(Mode::Spec),
                            result(f, false),
                            sequence_model(
                                collections::Model::SeqIndex,
                                old_sequence.clone(),
                                vec![index.clone()],
                                all,
                            )?,
                        ),
                    ));
                }
            }
        }
        if supplied.is_empty() {
            continue;
        }
        let Some((predicate, ts, outputs)) = call(&post) else { continue };
        if outputs.len() != 1
            || !projection(&outputs[0]).is_some_and(|(n, q, path)| {
                n == f.x.params[0].x.name && q == Phase::Future && path.is_empty()
            })
        {
            continue;
        }
        let matched = requirements.iter().any(|e| {
            call(e).is_some_and(|(n, us, inputs)| {
                n == predicate
                    && us.len() == ts.len()
                    && us.iter().zip(ts.iter()).all(|(a, b)| crate::ast_util::types_equal(a, b))
                    && inputs.len() == 1
                    && projection(&inputs[0]).is_some_and(|(n, q, path)| {
                        n == f.x.params[0].x.name && q == Phase::Entry && path.is_empty()
                    })
            })
        });
        if matched {
            let guard = supplied
                .iter()
                .cloned()
                .reduce(|a, b| logic(&f.span, LogicalOp::And, a, b))
                .unwrap();
            facts.push(logic(&f.span, LogicalOp::Implies, guard, post));
        }
    }
    if facts.len() > 12 {
        return Err(unsupported(&f.span, "selector contract budget exceeded"));
    }
    facts.iter().map(spec).collect()
}

/// Propose append-only preparation facts for an inline growth loop. As with an
/// explicit preparation API, these are untrusted claims: the original selector
/// body must prove them and every loop invariant. No helper is added or called.
fn inline_growth(
    f: &Function,
    all: &HashMap<Fun, Function>,
    ds: &[Datatype],
    report: &mut GenerationReport,
) -> Result<Option<(Function, Vec<Expr>)>, VirErr> {
    if !header(f) {
        return Ok(None);
    }
    let Some(body) = &f.x.body else { return Ok(None) };
    let ExprX::Block(ss, Some(tail)) = &body.x else { return Ok(None) };
    if ss.len() != 1 {
        return Ok(None);
    }
    let StmtX::Expr(source) = &ss[0].x else { return Ok(None) };
    let mut loops = vec![];
    expr_visitor_walk(source, &mut |e| {
        if matches!(&e.x, ExprX::Loop { .. }) {
            loops.push(e.clone());
        }
        VisitorControlFlow::<()>::Recurse
    });
    if loops.len() != 1
        || !matches!(&loops[0].x, ExprX::Loop { cond: Some(_), is_for_loop: false, .. })
    {
        return Ok(None);
    }
    let Some((borrow, _, args)) = call(tail) else { return Ok(None) };
    if args.len() != 2
        || !all
            .get(borrow)
            .is_some_and(|c| collections::model(c) == Some(collections::Model::IndexMut))
    {
        return Ok(None);
    }
    let Some((root, Phase::Current, path)) = projection(&args[0]) else { return Ok(None) };
    if root != f.x.params[0].x.name || path.len() != 1 {
        return Ok(None);
    }
    let TypX::MutRef(record) = &*f.x.params[0].x.typ else { return Ok(None) };
    let Some(fields) = records::shape::fields(record, ds) else { return Ok(None) };
    let Some((op, vector)) =
        fields.iter().find(|(op, _)| records::shape::same_path(&[op.clone()], &path))
    else {
        return Ok(None);
    };
    let Some(vector_types) = collections::vec_args(vector) else { return Ok(None) };
    let old = view(field(state(f, Phase::Entry), op, vector), all)?;
    let after = view(field(state(f, Phase::Future), op, vector), all)?;
    let as_int = |e: Expr| {
        SpannedTyped::new(
            &f.span,
            &int_typ(),
            ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, e),
        )
    };
    let old_len = as_int(sequence_model(collections::Model::SeqLen, old.clone(), vec![], all)?);
    let new_len = as_int(sequence_model(collections::Model::SeqLen, after.clone(), vec![], all)?);
    let index = SpannedTyped::new(
        &tail.span,
        &int_typ(),
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, truncate: false }, spec(&args[1])?),
    );
    let target = SpannedTyped::new(
        &tail.span,
        &int_typ(),
        ExprX::Binary(
            BinaryOp::Arith(ArithOp::Add(OverflowBehavior::Allow)),
            index,
            SpannedTyped::new(&tail.span, &int_typ(), ExprX::Const(Constant::Int(1.into()))),
        ),
    );
    let bound = SpannedTyped::new(
        &tail.span,
        &int_typ(),
        ExprX::If(
            relation(
                &tail.span,
                BinaryOp::Inequality(InequalityOp::Lt),
                target.clone(),
                old_len.clone(),
            ),
            old_len.clone(),
            Some(target),
        ),
    );
    let mut posts = vec![relation(&f.span, BinaryOp::Eq(Mode::Spec), new_len, bound)];
    let take = all
        .values()
        .find(|c| owned::model(c) == Some(owned::Model::Take))
        .ok_or_else(|| unsupported(&f.span, "inline growth requires checked prefix vocabulary"))?;
    let prefix = trait_getters::instantiate(
        take,
        &Arc::new(vec![vector_types[0].clone()]),
        vec![records::atom(after)?, records::atom(old_len)?],
        &Arc::new(vec![]),
    )?
    .expr;
    posts.push(relation(&f.span, BinaryOp::Eq(Mode::Spec), prefix, old));
    for (other, typ) in &fields {
        if other.field != op.field {
            posts.push(relation(
                &f.span,
                BinaryOp::Eq(Mode::Spec),
                field(state(f, Phase::Future), other, typ),
                field(state(f, Phase::Entry), other, typ),
            ));
        }
    }
    let mut requirements = vec![];
    for p in f.x.require.iter() {
        growth_goals::clauses(p, &mut requirements);
    }
    for p in requirements {
        let Some((name, types, args)) = call(&p) else { continue };
        if args.len() != 1
            || !all.get(name).is_some_and(|c| c.x.mode == Mode::Spec)
            || !projection(&args[0])
                .is_some_and(|(n, q, path)| n == root && q == Phase::Entry && path.is_empty())
        {
            continue;
        }
        posts.push(
            trait_getters::instantiate_source(
                &all[name],
                types,
                vec![records::atom(state(f, Phase::Future))?],
                &f.x.typ_bounds,
                &p,
            )?
            .expr,
        );
    }
    let mut projected = f.x.clone();
    let mut ret = projected.inner_ret.x.clone();
    ret.typ = crate::ast_util::unit_typ();
    projected.inner_ret = projected.inner_ret.new_x(ret.clone());
    projected.outer_ret = projected.outer_ret.new_x(ret);
    projected.ensure = (Arc::new(posts.clone()), Arc::new(vec![]));
    let analysis = f.new_x(projected);
    let annotated = growth_goals::prepare_one(&analysis, all, report)?;
    if Arc::ptr_eq(&analysis, &annotated) {
        return Ok(None);
    }
    let mut out = f.x.clone();
    out.body = annotated.x.body.clone();
    Ok(Some((f.new_x(out), posts)))
}

/// Only append pure postconditions and an internal publication marker. Preserve
/// all source code, input/output goals, modes, bounds and termination metadata.
fn validate_decoration(
    original: &Function,
    candidate: &Function,
    all: &HashMap<Fun, Function>,
) -> Result<(), VirErr> {
    let bad = || {
        unsupported(
            &original.span,
            "selector assistance changed source or added an invalid contract",
        )
    };
    let added =
        candidate.x.ensure.0.len().checked_sub(original.x.ensure.0.len()).ok_or_else(bad)?;
    if added == 0 || added > 12 || !candidate.x.attrs.infer_mutable_selector {
        return Err(bad());
    }
    for p in &candidate.x.ensure.0[original.x.ensure.0.len()..] {
        if !crate::ast_util::types_equal(&p.typ, &bool_typ()) {
            return Err(bad());
        }
        pure_contract(p, original, all)?;
    }
    let mut projected = candidate.x.clone();
    projected.ensure.0 = Arc::new(candidate.x.ensure.0[..original.x.ensure.0.len()].to_vec());
    Arc::make_mut(&mut projected.attrs).infer_mutable_selector =
        original.x.attrs.infer_mutable_selector;
    if serde_json::to_vec(&projected).map_err(|_| bad())?
        != serde_json::to_vec(&original.x).map_err(|_| bad())?
    {
        return Err(bad());
    }
    Ok(())
}
pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    ds: &[Datatype],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all = fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect();
    let mut id = u64::MAX / 8;
    fs.iter()
        .map(|f| {
            let inline = inline_growth(f, &all, ds, report)?;
            let prepared = inline.as_ref().map(|(f, _)| f).unwrap_or(f);
            let facts = proposal(prepared, &all, ds, inline.as_ref().map(|(_, p)| p.as_slice()))?;
            if facts.is_empty() {
                return Ok(prepared.clone());
            }
            let mut x = prepared.x.clone();
            for p in &facts {
                pure_contract(p, f, &all)?;
                Arc::make_mut(&mut x.ensure.0).push(growth_goals::fresh(p, &mut id)?);
            }
            Arc::make_mut(&mut x.attrs).infer_mutable_selector = true;
            let candidate = f.new_x(x);
            validate_decoration(prepared, &candidate, &all)?;
            report.mutable_selectors.push((f.x.name.clone(), facts.len()));
            Ok(candidate)
        })
        .collect()
}
pub(super) fn validate_imports(fs: &[Function]) -> Result<(), VirErr> {
    let all = fs.iter().map(|f| (f.x.name.clone(), f.clone())).collect();
    for f in fs.iter().filter(|f| f.x.attrs.infer_mutable_selector) {
        if !header(f) || f.x.ensure.0.is_empty() || f.x.ensure.0.len() > 32 {
            return Err(unsupported(&f.span, "invalid checked mutable-selector interface"));
        }
        for p in f.x.ensure.0.iter() {
            pure_contract(p, f, &all)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_projection_preserves_code_input_contracts_and_original_output_goals() {
        let base = crate::infer::loops::tests::counted_function();
        let mut x = base.x.clone();
        x.ensure.0 = Arc::new(vec![boolean(&base.span, true).expr]);
        let original = base.new_x(x);
        let mut x = original.x.clone();
        Arc::make_mut(&mut x.ensure.0).push(boolean(&base.span, true).expr);
        Arc::make_mut(&mut x.attrs).infer_mutable_selector = true;
        let candidate = base.new_x(x);
        let all = HashMap::new();
        validate_decoration(&original, &candidate, &all).unwrap();
        for attack in 0..7 {
            let mut x = candidate.x.clone();
            match attack {
                0 => x.require = Arc::new(vec![boolean(&base.span, false).expr]),
                1 => x.body = None,
                2 => Arc::make_mut(&mut x.ensure.0)[0] = boolean(&base.span, false).expr,
                3 => Arc::make_mut(&mut x.attrs).infer_mutable_selector = false,
                4 | 5 => {
                    let effect = SpannedTyped::new(
                        &base.span,
                        &bool_typ(),
                        ExprX::AssertAssume {
                            is_assume: true,
                            expr: boolean(&base.span, false).expr,
                            msg: None,
                        },
                    );
                    Arc::make_mut(&mut x.ensure.0)[1] = if attack == 4 {
                        effect
                    } else {
                        SpannedTyped::new(
                            &base.span,
                            &bool_typ(),
                            ExprX::WithTriggers {
                                triggers: Arc::new(vec![Arc::new(vec![effect])]),
                                body: boolean(&base.span, true).expr,
                            },
                        )
                    };
                }
                6 => {
                    Arc::make_mut(&mut x.ensure.0)[1] = SpannedTyped::new(
                        &base.span,
                        &bool_typ(),
                        ExprX::Var(VarIdent(
                            Arc::new("undeclared".into()),
                            VarIdentDisambiguate::AirLocal,
                        )),
                    )
                }
                _ => unreachable!(),
            }
            assert!(
                validate_decoration(&original, &candidate.new_x(x), &all).is_err(),
                "attack {attack}"
            );
        }
        // A forged internal marker alone cannot make an arbitrary imported
        // signature a selector interface, even if its postconditions are pure.
        assert!(validate_imports(&[candidate]).is_err());
    }
}
