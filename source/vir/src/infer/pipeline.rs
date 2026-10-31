//! Ordered preparation of checked proof assistance on the original functions.
//!
//! This phase only proposes annotations. The driver's subsequent well-formedness,
//! mode, ordinary-body and precision checks remain the acceptance boundary.
use super::*;

/// Immutable inputs shared by preparation passes. Function definitions are not
/// cached here: a later pass must observe the annotations installed before it.
pub struct GoalContext<'a> {
    imported: &'a [Function],
    datatypes: &'a [Datatype],
}

impl<'a> GoalContext<'a> {
    pub fn new(imported: &'a [Function], datatypes: &'a [Datatype]) -> Self {
        Self { imported, datatypes }
    }

    pub fn prepare(
        &self,
        functions: &[Function],
        report: &mut GenerationReport,
    ) -> Result<Vec<Function>, VirErr> {
        // Preserve the dependency order: callback/getter models precede their
        // consumers; structural loop/selector preparation precedes lemma and
        // trigger assistance. Each pass receives the preceding pass's output.
        let mut functions = type_invariant_goals::prepare(functions, self.datatypes, report);
        functions = owned::array_callbacks::prepare(&functions, self.imported, report)?;
        functions = trait_getters::prepare(&functions, self.imported);
        functions = iterators::prepare(&functions, self.imported);
        functions =
            owned::factory_callbacks::prepare(&functions, self.imported, self.datatypes, report)?;
        functions = owned::error_callbacks::prepare(&functions, self.imported, report)?;
        functions = pointwise_goals::prepare(&functions, self.imported, report)?;
        functions = local_fold_goals::prepare(&functions, self.imported, self.datatypes, report)?;
        functions = selector_goals::prepare(&functions, self.imported, self.datatypes, report)?;
        functions = growth_goals::prepare(&functions, self.imported, report)?;
        functions = range_goals::prepare(&functions, self.imported, report)?;
        functions = counting_goals::prepare(&functions, self.imported, report);
        functions = view_bounds_goals::prepare(&functions, self.imported, report);
        functions = fold_goals::prepare(&functions, self.imported, report);
        sequence_triggers::prepare(&functions, report)
    }
}

/// Build a fresh lookup for one pass. Keep the existing local/import ordering;
/// retaining this map across passes would hide newly generated definitions.
pub(super) fn function_map(
    functions: &[Function],
    imported: &[Function],
) -> HashMap<Fun, Function> {
    functions.iter().chain(imported).map(|f| (f.x.name.clone(), f.clone())).collect()
}
