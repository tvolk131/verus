//! Observations of candidate generation, never proof evidence or cache inputs.
use super::*;

#[derive(Clone)]
pub struct GeneratedCandidate {
    pub function: Function,
    pub cache_hit: bool,
    pub elapsed_micros: u128,
    pub requires_nodes: usize,
    pub ensures_nodes: usize,
    pub referenced_functions: Vec<Fun>,
}

#[derive(Clone, Default)]
pub struct GenerationReport {
    pub requested: usize,
    pub candidates: Vec<GeneratedCandidate>,
    pub elapsed_micros: u128,
    pub failure: Option<String>,
}

impl GeneratedCandidate {
    pub(super) fn new(function: &Function, cache_hit: bool, elapsed_micros: u128) -> Self {
        fn nodes(expressions: &[Expr]) -> usize {
            let mut count = 0;
            for expression in expressions {
                crate::ast_visitor::expr_visitor_walk(expression, &mut |_| {
                    count += 1;
                    crate::ast_visitor::VisitorControlFlow::<()>::Recurse
                });
            }
            count
        }
        let mut referenced = std::collections::BTreeSet::new();
        if let Some(body) = &function.x.body {
            crate::ast_visitor::expr_visitor_walk(body, &mut |e| {
                if let ExprX::Call { target: CallTarget::Fun(_, name, ..), .. } = &e.x {
                    referenced.insert(name.clone());
                }
                crate::ast_visitor::VisitorControlFlow::<()>::Recurse
            });
        }
        Self {
            function: function.clone(),
            cache_hit,
            elapsed_micros,
            requires_nodes: nodes(&function.x.require),
            ensures_nodes: nodes(&function.x.ensure.0),
            referenced_functions: referenced.into_iter().collect(),
        }
    }
}
