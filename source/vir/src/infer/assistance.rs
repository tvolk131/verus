//! Shared pure-annotation plumbing for ordinary checked proof proposals.
//!
//! Profile recognition, executable-preservation checks and precision checking
//! stay separate. These helpers neither verify nor authorize any proposal.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

// Preserve existing diagnostics while moving the common implementation.
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
