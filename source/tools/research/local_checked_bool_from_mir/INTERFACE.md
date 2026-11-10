# Local checked foreign body interface (stage 1)

Resolve a direct HIR FnDef call in its caller TypingEnv to the semantic core
From<bool> for u64 Item instance with empty substitutions. Instantiate and inspect
all of its optimized typed MIR. Independently compare the generated full exec
Function (body, types, modes, binders and original local mapping) against that MIR.
Preregister a distinct BodyBackedForeign full function and stub before Query
collects stubs. Redirect only the VIR direct call; keep original HIR erasure and
runtime identity. Never manufacture a FnDef equivalence.

The ordinary no-requires postcondition r == if b {1u64} else {0u64} is a proposal.
Normal WF/modes/traits/resolution/lifetimes and solver body verification must run;
no inference-summary proof claim follows from ordinary contract checking.
Reject all selected exports and delete stale output before pruning/proofs;
reject incomplete checking. Only --infer-goals enables the adapter.

Parent owns policy70/cache73/schema reconciliation: constants remain unchanged
here. No certified publication/import or native theorem. Publication=false,
native=0, goal_complete=false. Fresh independent parent review required.
