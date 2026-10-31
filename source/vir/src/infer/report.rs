//! Observations of candidate generation, never proof evidence or cache inputs.
use super::*;

#[derive(Clone)]
pub struct GeneratedCandidate {
    pub function: Function,
    pub cache_hit: bool,
    /// Reporting only: regenerated record/query templates, never a proof input.
    pub record_template: bool,
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
    /// Scoped lemma selections are observations, never proof receipts.
    pub goal_lemmas: Vec<(Fun, Vec<Fun>)>,
    pub view_bound_lemmas: Vec<(Fun, Vec<Fun>)>,
    pub fold_lemmas: Vec<(Fun, Vec<Fun>)>,
    pub factory_callbacks: Vec<(Fun, usize)>,
    pub error_callbacks: Vec<(Fun, usize)>,
    pub array_callbacks: Vec<(Fun, usize)>,
    pub type_invariant_entries: Vec<(Fun, usize)>,
    pub growth_loops: Vec<(Fun, Span)>,
    pub local_folds: Vec<(Fun, Span)>,
    pub range_loops: Vec<(Fun, Span)>,
    pub pointwise_facts: Vec<(Fun, usize)>,
    pub mutable_selectors: Vec<(Fun, usize)>,
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
            record_template: false,
            elapsed_micros,
            requires_nodes: nodes(&function.x.require),
            ensures_nodes: nodes(&function.x.ensure.0),
            referenced_functions: referenced.into_iter().collect(),
        }
    }
}
