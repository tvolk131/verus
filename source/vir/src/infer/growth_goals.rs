//! Checked loop-proof proposals for ordinary append-only transition goals.
//! The supplied application contract remains the goal, never an assumption.
use super::assistance::{clauses, current, fresh, future, pure, seq_len};
use super::eligibility::loop_assistance;
use super::report::AssistanceSkip;
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

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
    let all = crate::infer::pipeline::function_map(fs, imported);
    report.observe_assistance(fs, "growth_loop", |f, report| prepare_one_explained(f, &all, report))
}

pub(super) fn prepare_one(
    f: &Function,
    all: &HashMap<Fun, Function>,
    report: &mut GenerationReport,
) -> Result<Function, VirErr> {
    // Selectors use a temporary analysis projection. Only top-level preparation
    // records decisions; the existing applied-annotation observation is retained.
    prepare_one_explained(f, all, report).map(|(function, _)| function)
}

fn prepare_one_explained(
    f: &Function,
    all: &HashMap<Fun, Function>,
    report: &mut GenerationReport,
) -> Result<(Function, AssistanceOutcome), VirErr> {
    let skip = |reason| Ok((f.clone(), AssistanceOutcome::Skipped(reason)));
    if let Some(reason) = loop_assistance(f) {
        return skip(reason);
    }
    if f.x.params.iter().skip(1).any(|p| matches!(&*p.x.typ, TypX::MutRef(_))) {
        return skip(AssistanceSkip::AdditionalMutableInput);
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
    if budget == 0 {
        return skip(AssistanceSkip::SourceBudget);
    }
    if loops.len() != 1 {
        return skip(AssistanceSkip::ExpectedSingleLoop);
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
        return skip(AssistanceSkip::UnsupportedWhileLoop);
    };
    if !invs.is_empty() || !decrease.is_empty() {
        return skip(AssistanceSkip::AuthoredLoopProof);
    }
    let mut pushes = 0;
    expr_visitor_walk(body, &mut |e| {
        if owned::model_call(e, all) == Some(owned::Model::Push) {
            pushes += 1;
        }
        VisitorControlFlow::<()>::Recurse
    });
    if pushes != 1 {
        return skip(AssistanceSkip::ExpectedSinglePush);
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
    if bounds.is_empty() {
        return skip(AssistanceSkip::MissingLengthGoal);
    }
    if bounds.len() != 1 {
        return skip(AssistanceSkip::AmbiguousLengthGoal);
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
        return skip(AssistanceSkip::GoalBudget);
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
    Ok((result, AssistanceOutcome::Applied))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_budget_is_reported_without_decorating_the_function() {
        let original = crate::infer::tests::function("large_body", crate::infer::tests::input("x"));
        let unit = crate::ast_util::unit_typ();
        let mut x = original.x.clone();
        Arc::make_mut(&mut x.attrs).infer = false;
        let mut receiver = x.params[0].x.clone();
        receiver.typ = Arc::new(TypX::MutRef(receiver.typ.clone()));
        x.params = Arc::new(vec![x.params[0].new_x(receiver)]);
        let mut ret = x.inner_ret.x.clone();
        ret.typ = unit.clone();
        x.inner_ret = x.inner_ret.new_x(ret);
        x.outer_ret = x.inner_ret.clone();
        let statement = crate::def::Spanned::new(
            original.span.clone(),
            StmtX::Expr(boolean(&original.span, true).expr),
        );
        x.body = Some(SpannedTyped::new(
            &original.span,
            &unit,
            ExprX::Block(Arc::new(vec![statement; 9000]), None),
        ));
        let original = original.new_x(x);
        let mut report = GenerationReport::default();
        let prepared = prepare(std::slice::from_ref(&original), &[], &mut report).unwrap();
        assert!(Arc::ptr_eq(&prepared[0], &original));
        assert!(report.growth_loops.is_empty());
        assert_eq!(report.assistance_decisions.len(), 1);
        assert!(matches!(
            report.assistance_decisions[0].outcome,
            AssistanceOutcome::Skipped(AssistanceSkip::SourceBudget)
        ));
    }

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
