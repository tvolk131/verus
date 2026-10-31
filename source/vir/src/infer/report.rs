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
    /// Preparation observations are never consumed by checking or publication.
    pub assistance_preparation_started: bool,
    pub assistance_preparation_completed: bool,
    pub assistance_decisions: Vec<AssistanceDecision>,
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
    pub sequence_patterns: Vec<(Fun, usize)>,
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

/// A decision made by an instrumented preparation profile, not a proof result.
#[derive(Clone)]
pub struct AssistanceDecision {
    pub function: Fun,
    pub source: Span,
    pub profile: &'static str,
    pub outcome: AssistanceOutcome,
}

#[derive(Clone)]
pub enum AssistanceOutcome {
    Applied,
    Skipped(AssistanceSkip),
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssistanceSkip {
    InferredFunction,
    NonExecutable,
    ExternalBody,
    UnsafeOrAsync,
    TerminationSuppressed,
    NonUnitReturn,
    MissingBody,
    MutableParameterBinding,
    MissingMutableReceiver,
    AdditionalMutableInput,
    SourceBudget,
    ExpectedSingleLoop,
    UnsupportedWhileLoop,
    AuthoredLoopProof,
    ExpectedSinglePush,
    MissingLengthGoal,
    AmbiguousLengthGoal,
    GoalBudget,
    ExpectedSingleNativeFor,
    IteratorProofShape,
    UnsupportedRange,
    UnsupportedIndex,
    MissingFinalStateGoal,
}

impl AssistanceSkip {
    /// Stable reason keys and profile requirements for presentation only.
    pub fn diagnostic(self) -> (&'static str, &'static str) {
        match self {
            Self::InferredFunction => (
                "inference_candidate",
                "This assistance profile targets ordinary executable functions; inferred functions use their own contract-generation path.",
            ),
            Self::NonExecutable => {
                ("non_executable_function", "This assistance profile targets executable functions.")
            }
            Self::ExternalBody => (
                "external_body",
                "An external body is outside this original-body assistance profile.",
            ),
            Self::UnsafeOrAsync => {
                ("unsafe_or_async_function", "This profile supports safe, synchronous functions.")
            }
            Self::TerminationSuppressed => (
                "termination_suppressed",
                "This profile requires the original termination checks to remain enabled.",
            ),
            Self::NonUnitReturn => (
                "non_unit_return",
                "This loop profile requires a unit return; a larger result may use a separate selector or local-fold profile.",
            ),
            Self::MissingBody => ("missing_body", "This profile requires a local executable body."),
            Self::MutableParameterBinding => (
                "mutable_parameter_binding",
                "This profile requires immutable parameter bindings; the receiver may be a mutable reference.",
            ),
            Self::MissingMutableReceiver => (
                "missing_mutable_receiver",
                "This profile requires a mutable reference as its first parameter.",
            ),
            Self::AdditionalMutableInput => (
                "additional_mutable_input",
                "Growth-loop assistance supports only the first parameter as a mutable reference.",
            ),
            Self::SourceBudget => (
                "source_budget_exhausted",
                "Loop discovery reached its existing 8192-expression budget; this profile proposed no annotations.",
            ),
            Self::ExpectedSingleLoop => (
                "expected_single_loop",
                "This profile requires exactly one loop in the function body.",
            ),
            Self::UnsupportedWhileLoop => (
                "unsupported_while_loop",
                "Growth assistance requires a conditional while loop without atomic execution or assumed termination.",
            ),
            Self::AuthoredLoopProof => (
                "authored_loop_proof",
                "The loop already has invariants or a decreasing measure; growth assistance preserves them.",
            ),
            Self::ExpectedSinglePush => (
                "expected_single_push",
                "Growth assistance requires exactly one recognized vector push in the loop body.",
            ),
            Self::MissingLengthGoal => (
                "missing_final_length_goal",
                "Growth assistance requires an authored equality between the final sequence length and a bound without final-state observations.",
            ),
            Self::AmbiguousLengthGoal => (
                "ambiguous_final_length_goal",
                "Growth assistance found multiple matching final-length equalities; this profile requires exactly one.",
            ),
            Self::GoalBudget => (
                "goal_budget_exhausted",
                "The postconditions exceed this profile's existing 12-goal limit; this profile proposed no annotations.",
            ),
            Self::ExpectedSingleNativeFor => (
                "expected_single_native_for",
                "Range assistance requires exactly one recognized native for loop.",
            ),
            Self::IteratorProofShape => (
                "unsupported_iterator_proof_shape",
                "Range assistance requires the original iterator proof scaffolding without additional loop clauses.",
            ),
            Self::UnsupportedRange => (
                "unsupported_range",
                "Range assistance requires a native unsigned Range or RangeInclusive iterator.",
            ),
            Self::UnsupportedIndex => (
                "unsupported_range_index",
                "Range assistance could not identify the original iterator's supported index observation.",
            ),
            Self::MissingFinalStateGoal => (
                "missing_final_state_goal",
                "Range assistance found no authored postcondition referring to the final mutable state.",
            ),
        }
    }

    pub fn budget_exhausted(self) -> bool {
        matches!(self, Self::SourceBudget | Self::GoalBudget)
    }
}

impl GenerationReport {
    /// Observe each actual attempt in source order, including a failing attempt.
    /// A failure still propagates unchanged and later functions are not attempted.
    pub(super) fn observe_assistance(
        &mut self,
        functions: &[Function],
        profile: &'static str,
        mut prepare: impl FnMut(&Function, &mut Self) -> Result<(Function, AssistanceOutcome), VirErr>,
    ) -> Result<Vec<Function>, VirErr> {
        let mut prepared = Vec::with_capacity(functions.len());
        for f in functions {
            let result = prepare(f, self);
            let outcome = match &result {
                Ok((_, outcome)) => outcome.clone(),
                Err(error) => AssistanceOutcome::Error(error.note.clone()),
            };
            self.assistance_decisions.push(AssistanceDecision {
                function: f.x.name.clone(),
                source: f.span.clone(),
                profile,
                outcome,
            });
            prepared.push(result?.0);
        }
        Ok(prepared)
    }
}
