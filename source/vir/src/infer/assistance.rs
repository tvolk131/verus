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

/// Conservative identity reservation shared by annotation preparers.
/// Every serialized integer is reserved, including IDs outside expression nodes.
/// This allocator supplies fresh syntax only; it never authorizes an annotation.
pub(super) struct FreshIds {
    pub(super) used: std::collections::HashSet<u64>,
    next: u64,
    names: std::collections::HashSet<String>,
}

impl FreshIds {
    pub(super) fn new(functions: &[Function], imported: &[Function]) -> Self {
        fn reserve(value: &serde_json::Value, ids: &mut FreshIds) {
            match value {
                serde_json::Value::Number(n) => {
                    if let Some(n) = n.as_u64() {
                        ids.used.insert(n);
                    }
                }
                serde_json::Value::String(name) => {
                    ids.names.insert(name.clone());
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        reserve(value, ids);
                    }
                }
                serde_json::Value::Object(values) => {
                    for value in values.values() {
                        reserve(value, ids);
                    }
                }
                _ => (),
            }
        }
        let mut ids =
            Self { used: Default::default(), next: u64::MAX - 1, names: Default::default() };
        for function in functions.iter().chain(imported) {
            reserve(&serde_json::to_value(function).expect("VIR serialization"), &mut ids);
        }
        ids
    }
    pub(super) fn id(&mut self) -> u64 {
        while !self.used.insert(self.next) {
            self.next = self.next.checked_sub(1).expect("annotation ID exhaustion");
        }
        let id = self.next;
        self.next = self.next.checked_sub(1).expect("annotation ID exhaustion");
        id
    }
    pub(super) fn span(&mut self, span: &Span) -> Span {
        let mut span = span.clone();
        span.id = self.id();
        span
    }
    pub(super) fn named_binding(&mut self, prefix: &str) -> VarIdent {
        loop {
            let id = self.id();
            let name = format!("{prefix}{id}");
            if self.names.insert(name.clone()) {
                return VarIdent(Arc::new(name), VarIdentDisambiguate::VirTemp(id));
            }
        }
    }
    /// Existing buffer/word snapshots require specification reads. Set proposals
    /// retain their separate read-policy visitor and independent validator.
    pub(super) fn spec_stmt(&mut self, statement: &Stmt) -> Result<Stmt, VirErr> {
        use crate::ast_visitor::{VisitorScopeMap, map_stmt_visitor_env};
        Ok(map_stmt_visitor_env(
            statement,
            &mut VisitorScopeMap::new(),
            self,
            &|ids, _, e| {
                let x = match &e.x {
                    ExprX::ReadPlace(p, _) => ExprX::ReadPlace(
                        p.clone(),
                        UnfinalizedReadKind { preliminary_kind: ReadKind::Spec, id: ids.id() },
                    ),
                    _ => e.x.clone(),
                };
                Ok(SpannedTyped::new(&ids.span(&e.span), &e.typ, x))
            },
            &|ids, _, s| {
                let x = match &s.x {
                    StmtX::Expr(e) => StmtX::Expr(e.clone()),
                    StmtX::Decl { pattern, mode, init, els, assert_irrefutable } => StmtX::Decl {
                        pattern: pattern.clone(),
                        mode: *mode,
                        init: init.clone(),
                        els: els.clone(),
                        assert_irrefutable: *assert_irrefutable,
                    },
                };
                Ok(vec![crate::def::Spanned::new(ids.span(&s.span), x)])
            },
            &|_, t| Ok(t.clone()),
            &|ids, _, p| Ok(SpannedTyped::new(&ids.span(&p.span), &p.typ, p.x.clone())),
            &|ids, _, p| Ok(SpannedTyped::new(&ids.span(&p.span), &p.typ, p.x.clone())),
        )?
        .pop()
        .expect("one annotation statement"))
    }
}
