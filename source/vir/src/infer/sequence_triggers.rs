//! Alternate instantiation patterns for pointwise sequence equations.
//! Only trigger metadata changes. The quantified proposition, domain and body
//! remain identical and are checked by the ordinary verifier.
use super::*;
use crate::ast_visitor::{VisitorControlFlow, expr_visitor_walk, map_expr_visitor};

fn is_variable(e: &Expr, v: &VarIdent) -> bool {
    match &e.x {
        ExprX::Var(n) => n == v,
        ExprX::Block(ss, Some(e)) if ss.is_empty() => is_variable(e, v),
        ExprX::ReadPlace(p, _) => match &p.x {
            PlaceX::Local(n) => n == v,
            PlaceX::Temporary(e) => is_variable(e, v),
            _ => false,
        },
        ExprX::Unary(UnaryOp::Clip { .. }, e) => is_variable(e, v),
        _ => false,
    }
}
fn sequence_read(target: &CallTarget) -> bool {
    let CallTarget::Fun(kind, fun, ..) = target else { return false };
    let f =
        if let CallTargetKind::DynamicResolved { resolved, .. } = kind { resolved } else { fun };
    f.path.krate == CrateId::Vstd
        && f.path.segments.len() == 3
        && f.path.segments[0].as_str() == "seq"
        && (f.path.segments[1].as_str() == "Seq" || f.path.segments[1].starts_with("impl"))
        && ["index", "spec_index"].contains(&f.path.last_segment().as_str())
}

pub fn prepare(
    functions: &[Function],
    report: &mut GenerationReport,
) -> Result<Vec<Function>, VirErr> {
    functions
        .iter()
        .map(|f| {
            if f.x.mode != Mode::Spec || f.x.body.is_none() || f.x.attrs.is_external_body {
                return Ok(f.clone());
            }
            let count = std::cell::Cell::new(0);
            let body = map_expr_visitor(f.x.body.as_ref().unwrap(), &|e| {
                let ExprX::Quant(q, bs, body) = &e.x else { return Ok(e.clone()) };
                if q.quant != air::ast::Quant::Forall || bs.len() != 1 {
                    return Ok(e.clone());
                }
                let mut patterns = Vec::new();
                let mut explicit = false;
                expr_visitor_walk(body, &mut |n| {
                    match &n.x {
                        ExprX::WithTriggers { .. } | ExprX::Unary(UnaryOp::Trigger(_), _) => {
                            explicit = true;
                        }
                        ExprX::Quant(..)
                        | ExprX::Closure(..)
                        | ExprX::NonSpecClosure { .. }
                        | ExprX::Choose { .. } => return VisitorControlFlow::<()>::Return,
                        ExprX::Call { target, args, .. }
                            if sequence_read(target)
                                && args.len() == 2
                                && is_variable(&args[1], &bs[0].name) =>
                        {
                            patterns.push(Arc::new(vec![n.clone()]));
                        }
                        _ => {}
                    }
                    VisitorControlFlow::Recurse
                });
                if explicit || patterns.len() < 2 {
                    return Ok(e.clone());
                }
                count.set(count.get() + 1);
                Ok(e.new_x(ExprX::Quant(
                    *q,
                    bs.clone(),
                    body.new_x(ExprX::WithTriggers {
                        triggers: Arc::new(patterns),
                        body: body.clone(),
                    }),
                )))
            })?;
            if count.get() == 0 {
                return Ok(f.clone());
            }
            report.sequence_patterns.push((f.x.name.clone(), count.get()));
            let mut x = f.x.clone();
            x.body = Some(body);
            Ok(f.new_x(x))
        })
        .collect()
}
