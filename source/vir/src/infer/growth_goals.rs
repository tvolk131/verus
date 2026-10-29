//! Checked loop-proof proposals for ordinary append-only transition goals.
//! The supplied application contract remains the goal, never an assumption.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

pub(super) fn clauses(e: &Expr, out: &mut Vec<Expr>) {
    if let ExprX::Logical(LogicalOp::And, a, b) = &e.x {
        clauses(a, out);
        clauses(b, out);
    } else {
        out.push(e.clone());
    }
}
pub(super) fn future(e: &Expr) -> bool {
    let mut yes = false;
    expr_visitor_walk(e, &mut |e| {
        yes |= matches!(&e.x, ExprX::Unary(UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_), _));
        VisitorControlFlow::<()>::Recurse
    });
    yes
}
pub(super) fn seq_len(e: &Expr, all: &HashMap<Fun, Function>) -> bool {
    match &e.x {
        ExprX::Unary(UnaryOp::Clip { range: IntRange::Int, .. }, a) => seq_len(a, all),
        ExprX::Call { target: CallTarget::Fun(_, n, ..), .. } => {
            all.get(n).and_then(collections::model) == Some(collections::Model::SeqLen)
        }
        _ => false,
    }
}
pub(super) fn current(e: &Expr) -> Result<Expr, VirErr> {
    map_expr_visitor(e, &|e| {
        Ok(match &e.x {
            ExprX::Old(a) => {
                let n = match &a.x {
                    ExprX::Var(n) => n.clone(),
                    ExprX::ReadPlace(p, _) => match &p.x {
                        PlaceX::Local(n) => n.clone(),
                        _ => return Err(unsupported(&e.span, "growth entry snapshot")),
                    },
                    _ => return Err(unsupported(&e.span, "growth entry snapshot")),
                };
                SpannedTyped::new(&e.span, &e.typ, ExprX::VarAt(n, VarAt::Pre))
            }
            ExprX::Unary(UnaryOp::MutRefFuture(_) | UnaryOp::MutRefFinal(_), a) => {
                e.new_x(ExprX::Unary(UnaryOp::MutRefCurrent, a.clone()))
            }
            ExprX::ReadPlace(p, _) => e.new_x(ExprX::ReadPlace(
                p.clone(),
                UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: u64::MAX },
            )),
            _ => e.clone(),
        })
    })
}
pub(super) fn pure(e: &Expr, all: &HashMap<Fun, Function>) -> Result<(), VirErr> {
    let mut budget = FORMULA_BUDGET;
    crate::ast_visitor::expr_visitor_check(e, &mut |_, e| {
        budget = budget
            .checked_sub(1)
            .ok_or_else(|| unsupported(&e.span, "growth annotation budget"))?;
        let yes = match &e.x {
            ExprX::Const(_)
            | ExprX::Var(_)
            | ExprX::VarAt(_, VarAt::Pre)
            | ExprX::Unary(_, _)
            | ExprX::UnaryOpr(_, _)
            | ExprX::Binary(_, _, _)
            | ExprX::Logical(_, _, _)
            | ExprX::Multi(_, _)
            | ExprX::If(_, _, Some(_))
            | ExprX::Ctor(_, _, _, None)
            | ExprX::Quant(_, _, _)
            | ExprX::Closure(_, _)
            | ExprX::WithTriggers { .. } => true,
            ExprX::Block(ss, Some(_)) => ss.is_empty(),
            ExprX::ReadPlace(p, k) => matches!(k.preliminary_kind, ReadKind::Spec) && place_pure(p),
            ExprX::Call {
                target: CallTarget::Fun(kind, n, ..),
                post_args: None,
                body: None,
                ..
            } => {
                let n = if let CallTargetKind::DynamicResolved { resolved, .. } = kind {
                    resolved
                } else {
                    n
                };
                all.get(n).is_some_and(|f| f.x.mode == Mode::Spec)
            }
            _ => false,
        };
        if yes {
            Ok(())
        } else {
            Err(unsupported(&e.span, "effect or assumption in growth annotation"))
        }
    })
}
fn place_pure(p: &Place) -> bool {
    match &p.x {
        PlaceX::Local(_) => true,
        PlaceX::Field(_, p) | PlaceX::DerefMut(p) => place_pure(p),
        PlaceX::Temporary(_) => true,
        _ => false,
    }
}
pub(super) fn fresh(e: &Expr, id: &mut u64) -> Result<Expr, VirErr> {
    crate::ast_visitor::map_expr_visitor_env(
        e,
        &mut crate::ast_visitor::VisitorScopeMap::new(),
        id,
        &|id, _, e| {
            let mut s = e.span.clone();
            s.id = *id;
            *id -= 1;
            Ok(SpannedTyped::new(&s, &e.typ, e.x.clone()))
        },
        &|_, _, s| Ok(vec![s.clone()]),
        &|_, t| Ok(t.clone()),
        &|id, _, p| {
            let mut s = p.span.clone();
            s.id = *id;
            *id -= 1;
            Ok(SpannedTyped::new(&s, &p.typ, p.x.clone()))
        },
        &|_, _, p| Ok(p.clone()),
    )
}
/// An annotation-only installer; ordinary verification checks every proposal.
fn install(
    f: &Function,
    source: &Expr,
    invs: Vec<Expr>,
    measure: Expr,
    all: &HashMap<Fun, Function>,
) -> Result<Function, VirErr> {
    for e in invs.iter().chain(std::iter::once(&measure)) {
        pure(e, all)?;
    }
    let mut id = u64::MAX - 1;
    let invs = Arc::new(
        invs.iter()
            .map(|e| {
                Ok(LoopInvariant {
                    kind: LoopInvariantKind::InvariantAndEnsures,
                    inv: fresh(e, &mut id)?,
                })
            })
            .collect::<Result<Vec<_>, VirErr>>()?,
    );
    let measure = fresh(&measure, &mut id)?;
    let update = |e: &Expr, decorated: bool| -> Result<Expr, VirErr> {
        if e.span.id != source.span.id {
            return Ok(e.clone());
        }
        let ExprX::Loop {
            loop_isolation,
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            atomic_call,
            ..
        } = &e.x
        else {
            return Err(unsupported(&e.span, "growth loop identity"));
        };
        Ok(e.new_x(ExprX::Loop {
            loop_isolation: if decorated {
                false
            } else {
                match &source.x {
                    ExprX::Loop { loop_isolation, .. } => *loop_isolation,
                    _ => *loop_isolation,
                }
            },
            allow_complex_invariants: *allow_complex_invariants,
            is_for_loop: *is_for_loop,
            assume_termination: *assume_termination,
            label: label.clone(),
            cond: cond.clone(),
            body: body.clone(),
            atomic_call: *atomic_call,
            invs: if decorated { invs.clone() } else { Arc::new(vec![]) },
            decrease: if decorated { Arc::new(vec![measure.clone()]) } else { Arc::new(vec![]) },
        }))
    };
    let original = f.x.body.as_ref().unwrap();
    let body = map_expr_visitor(original, &|e| update(e, true))?;
    validate_decoration(f, source, &body, all)?;
    let mut x = f.x.clone();
    x.body = Some(body);
    Ok(f.new_x(x))
}

fn validate_decoration(
    f: &Function,
    source: &Expr,
    body: &Expr,
    all: &HashMap<Fun, Function>,
) -> Result<(), VirErr> {
    let bad = || {
        unsupported(
            &f.span,
            "growth proof decoration changed executable code or contains invalid annotations",
        )
    };
    let ExprX::Loop { loop_isolation: original_isolation, .. } = &source.x else {
        return Err(bad());
    };
    let found = std::cell::Cell::new(0usize);
    let projected = map_expr_visitor(body, &|e| {
        if e.span.id != source.span.id {
            return Ok(e.clone());
        }
        let ExprX::Loop {
            loop_isolation,
            allow_complex_invariants,
            is_for_loop,
            assume_termination,
            label,
            cond,
            body,
            atomic_call,
            invs,
            decrease,
        } = &e.x
        else {
            return Err(bad());
        };
        if *loop_isolation || invs.is_empty() || invs.len() > 14 || decrease.len() != 1 {
            return Err(bad());
        }
        for inv in invs.iter() {
            if inv.kind != LoopInvariantKind::InvariantAndEnsures
                || !crate::ast_util::types_equal(&inv.inv.typ, &bool_typ())
            {
                return Err(bad());
            }
            pure(&inv.inv, all)?;
        }
        if !eligibility::integer(&decrease[0].typ) {
            return Err(bad());
        }
        pure(&decrease[0], all)?;
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
            invs: Arc::new(vec![]),
            decrease: Arc::new(vec![]),
        }))
    })?;
    let serialize = |e: &Expr| serde_json::to_vec(e).map_err(|_| bad());
    if found.get() != 1 || serialize(&projected)? != serialize(f.x.body.as_ref().unwrap())? {
        return Err(bad());
    }
    Ok(())
}

pub fn prepare(
    fs: &[Function],
    imported: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let all =
        fs.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect::<HashMap<_, _>>();
    fs.iter().map(|f| prepare_one(f, &all, report)).collect()
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
        || f.x.params.iter().skip(1).any(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
        || !f.x.params.first().is_some_and(|p| matches!(&*p.x.typ, TypX::MutRef(_)))
    {
        return Ok(f.clone());
    }
    let mut loops = vec![];
    let mut budget = 8192usize;
    expr_visitor_walk(f.x.body.as_ref().unwrap(), &mut |e| {
        if budget == 0 {
            return VisitorControlFlow::Stop(());
        }
        budget -= 1;
        if matches!(&e.x, ExprX::Loop { .. }) {
            loops.push(e.clone());
        }
        VisitorControlFlow::Recurse
    });
    if budget == 0 || loops.len() != 1 {
        return Ok(f.clone());
    }
    let source = &loops[0];
    let ExprX::Loop {
        cond: Some(_),
        body,
        invs,
        decrease,
        is_for_loop: false,
        assume_termination: false,
        atomic_call: false,
        ..
    } = &source.x
    else {
        return Ok(f.clone());
    };
    if !invs.is_empty() || !decrease.is_empty() {
        return Ok(f.clone());
    }
    let mut pushes = 0;
    expr_visitor_walk(body, &mut |e| {
        if owned::model_call(e, all) == Some(owned::Model::Push) {
            pushes += 1;
        }
        VisitorControlFlow::<()>::Recurse
    });
    if pushes != 1 {
        return Ok(f.clone());
    }
    let mut posts = vec![];
    for e in f.x.ensure.0.iter() {
        clauses(e, &mut posts);
    }
    let mut bounds = vec![];
    for p in &posts {
        if let ExprX::Binary(BinaryOp::Eq(_), a, b) = &p.x {
            for (a, b) in [(a, b), (b, a)] {
                if seq_len(a, all) && future(a) && !future(b) {
                    bounds.push((a.clone(), b.clone()));
                }
            }
        }
    }
    if bounds.len() != 1 {
        return Ok(f.clone());
    }
    let (len, limit) = &bounds[0];
    let mut invs = vec![];
    for p in posts {
        if !future(&p) {
            continue;
        }
        let is_bound =
            matches!(&p.x,ExprX::Binary(BinaryOp::Eq(_),a,b) if seq_len(a,all)||seq_len(b,all));
        if !is_bound {
            invs.push(current(&p)?);
        }
    }
    if invs.len() > 12 {
        return Ok(f.clone());
    }
    let len = current(len)?;
    let limit = current(limit)?;
    let receiver = &f.x.params[0].x.name;
    let initial = map_expr_visitor(&len, &|e| {
        Ok(match &e.x {
            ExprX::Var(n) if n == receiver => e.new_x(ExprX::VarAt(n.clone(), VarAt::Pre)),
            ExprX::ReadPlace(p, _) if matches!(&p.x,PlaceX::Local(n) if n==receiver) => {
                e.new_x(ExprX::VarAt(receiver.clone(), VarAt::Pre))
            }
            _ => e.clone(),
        })
    })?;
    invs.push(SpannedTyped::new(
        &source.span,
        &bool_typ(),
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), initial, len.clone()),
    ));

    invs.push(SpannedTyped::new(
        &source.span,
        &bool_typ(),
        ExprX::Binary(BinaryOp::Inequality(InequalityOp::Le), len.clone(), limit.clone()),
    ));
    let measure = SpannedTyped::new(
        &source.span,
        &int_typ(),
        ExprX::Binary(BinaryOp::Arith(ArithOp::Sub(OverflowBehavior::Allow)), limit, len),
    );
    let result = install(f, source, invs, measure, all)?;
    report.growth_loops.push((f.x.name.clone(), source.span.clone()));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn growth_decoration_rejects_executable_changes_and_assumptions() {
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
        let measure =
            SpannedTyped::new(&source.span, &int_typ(), ExprX::Const(Constant::Int(0.into())));
        let decorated = install(
            &original,
            &source,
            vec![boolean(&source.span, true).expr],
            measure.clone(),
            &all,
        )
        .unwrap();
        let changed = map_expr_visitor(decorated.x.body.as_ref().unwrap(), &|e| {
            Ok(match &e.x {
                ExprX::Assign { place, op, typ, resolve, .. } => e.new_x(ExprX::Assign {
                    place: place.clone(),
                    rhs: SpannedTyped::new(&e.span, typ, ExprX::Const(Constant::Int(2.into()))),
                    op: *op,
                    typ: typ.clone(),
                    resolve: *resolve,
                }),
                _ => e.clone(),
            })
        })
        .unwrap();
        assert!(validate_decoration(&original, &source, &changed, &all).is_err());
        let assume = SpannedTyped::new(
            &source.span,
            &bool_typ(),
            ExprX::AssertAssume {
                is_assume: true,
                expr: boolean(&source.span, false).expr,
                msg: None,
            },
        );
        assert!(install(&original, &source, vec![assume], measure, &all).is_err());
    }
}
