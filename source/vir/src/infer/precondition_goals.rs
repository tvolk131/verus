//! Proof assistance for calls to inferred functions with large preconditions.
//!
//! A large inferred precondition is installed as a call to its named view
//! (`f_requires`). When an ordinary function calls such an `f` with arguments
//! that are its own immutable parameters, this pass proposes, just before the
//! call, `assert(f_requires(args)) by(nonlinear_arith) requires <caller's
//! requires>;`. It is a proposal like any other goal assistance: ordinary
//! verification must prove it, and nothing is assumed.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk};
use std::cell::RefCell;
use std::collections::HashSet;

pub(super) const PROFILE: &str = "inferred_precondition_nonlinear";

/// Inferred functions whose installed precondition is a call to their view.
fn viewed_callees(functions: &[Function]) -> HashMap<Fun, Fun> {
    let names: HashSet<&Fun> =
        functions.iter().filter(|f| views::is_view(f)).map(|f| &f.x.name).collect();
    functions
        .iter()
        .filter(|f| f.x.attrs.infer)
        .filter_map(|f| match &f.x.require[..] {
            [req] => match &req.x {
                ExprX::Call { target: CallTarget::Fun(_, view, ..), .. }
                    if names.contains(view) =>
                {
                    Some((f.x.name.clone(), view.clone()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// A spec reading of an argument that is one of `params`, or None.
fn parameter_argument(arg: &Expr, params: &HashSet<VarIdent>) -> Option<Expr> {
    match &arg.x {
        ExprX::Var(n) if params.contains(n) => Some(arg.clone()),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(n) if params.contains(n) => Some(crate::ast_util::place_to_spec_expr(p)),
            _ => None,
        },
        _ => None,
    }
}

fn proposal(
    call: &Expr,
    view: &Fun,
    typs: &Typs,
    args: Vec<Expr>,
    requires: &[Expr],
    id: &mut u64,
) -> Result<Expr, VirErr> {
    let span = &call.span;
    let unit = crate::ast_util::unit_typ();
    let goal = SpannedTyped::new(
        span,
        &Arc::new(TypX::Bool),
        ExprX::Call {
            target: CallTarget::Fun(
                CallTargetKind::Static,
                view.clone(),
                typs.clone(),
                Arc::new(vec![]),
                CallTargetAttrs {
                    autospec: AutospecUsage::Final,
                    const_var: false,
                    assume_external_allowed: false,
                },
            ),
            args: Arc::new(args),
            post_args: None,
            body: None,
        },
    );
    let requires =
        requires.iter().map(|r| assistance::fresh(r, id)).collect::<Result<Vec<_>, VirErr>>()?;
    let empty = SpannedTyped::new(span, &unit, ExprX::Block(Arc::new(vec![]), None));
    let query = SpannedTyped::new(
        span,
        &unit,
        ExprX::AssertQuery {
            requires: Arc::new(requires),
            ensures: Arc::new(vec![assistance::fresh(&goal, id)?]),
            proof: empty,
            mode: AssertQueryMode::NonLinear,
        },
    );
    let ghost = SpannedTyped::new(
        span,
        &unit,
        ExprX::Ghost {
            alloc_wrapper: false,
            tracked: false,
            expr: SpannedTyped::new(
                span,
                &unit,
                ExprX::Block(
                    Arc::new(vec![crate::def::Spanned::new(span.clone(), StmtX::Expr(query))]),
                    None,
                ),
            ),
        },
    );
    Ok(SpannedTyped::new(
        span,
        &call.typ,
        ExprX::Block(
            Arc::new(vec![crate::def::Spanned::new(span.clone(), StmtX::Expr(ghost))]),
            Some(call.clone()),
        ),
    ))
}

pub(super) fn prepare(
    functions: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    let viewed = viewed_callees(functions);
    if viewed.is_empty() {
        return Ok(functions.to_vec());
    }
    // Reserved id range for these proposals, disjoint from other passes.
    let id = RefCell::new(u64::MAX - (1 << 41));
    functions
        .iter()
        .map(|g| {
            let Some(body) = &g.x.body else { return Ok(g.clone()) };
            if g.x.attrs.infer
                || views::is_view(g)
                || g.x.require.is_empty()
                || !matches!(g.x.mode, Mode::Exec | Mode::Proof)
            {
                return Ok(g.clone());
            }
            let params: HashSet<VarIdent> =
                g.x.params
                    .iter()
                    .filter(|p| !p.x.user_mut && !matches!(&*p.x.typ, TypX::MutRef(_)))
                    .map(|p| p.x.name.clone())
                    .collect();
            // Only bodies that never assign to those parameters.
            let mut assigned = false;
            expr_visitor_walk(body, &mut |e| {
                if let ExprX::Assign { place: lhs, .. } = &e.x {
                    if let Some(local) = crate::ast_util::place_get_local(lhs) {
                        if matches!(&local.x, PlaceX::Local(n) if params.contains(n)) {
                            assigned = true;
                        }
                    }
                }
                VisitorControlFlow::<()>::Recurse
            });
            if assigned {
                return Ok(g.clone());
            }
            let sites = RefCell::new(Vec::new());
            let new_body = crate::ast_visitor::map_expr_visitor(body, &|e| {
                let ExprX::Call {
                    target: CallTarget::Fun(CallTargetKind::Static, callee, typs, _, _),
                    args,
                    post_args: None,
                    body: None,
                } = &e.x
                else {
                    return Ok(e.clone());
                };
                let Some(view) = viewed.get(callee) else { return Ok(e.clone()) };
                let Some(args) =
                    args.iter().map(|a| parameter_argument(a, &params)).collect::<Option<Vec<_>>>()
                else {
                    return Ok(e.clone());
                };
                sites.borrow_mut().push(e.span.clone());
                proposal(e, view, typs, args, &g.x.require, &mut id.borrow_mut())
            })?;
            let sites = sites.into_inner();
            if sites.is_empty() {
                return Ok(g.clone());
            }
            report.body_rechecks.insert(g.x.name.clone());
            for source in sites {
                report.assistance_decisions.push(report::AssistanceDecision {
                    function: g.x.name.clone(),
                    source,
                    profile: PROFILE,
                    outcome: report::AssistanceOutcome::Applied,
                });
            }
            let mut x = g.x.clone();
            x.body = Some(new_body);
            Ok(g.new_x(x))
        })
        .collect()
}
