# Review findings (terminal status in REPORT.md / RESULTS.json)

## Medium — delivered Tree Borrows nested signature inventory is not expanded

Location: source/tools/research/tree_borrows_checked_foundation/inventory.py:22–24,
summarize.py:93–96; REPORT.md opening inventory claim.

The controller queries `Print Module` only for the 507 library roots. This
command prints immediate fields but represents nested modules/module types by
name. `Set Printing All` and a large depth do not recursively print their
signatures. The delivered 53-byte `Corelib.BinNums.IntDef` output is exactly
`Module Corelib.BinNums.IntDef := Struct Module Z End`. The delivered
`iris.base_logic.lib.iprop` output similarly contains `Module Type
iProp_solution_sig Module iProp_solution`, without the sealed interface fields.
The checker walk inventories nested module/constant names but does not list
inductive constructors in those abbreviated module signatures. Therefore the
claim of complete nested compiled signatures is stronger than the delivered
query outputs. This does not refute the successful recursive kernel check,
the exact 2,399 individual internal constant queries, or the recorded global
axiom context. Suggested controller correction: query every checker-discovered
nested module/type and recursively expand named children; retain all output
hashes. The review supplements this independently in nested.py without editing
the delivered controller.

## Informational — mathematical safety premises are not original-source proofs

Stacked Borrows: lang/steps_progress.v:805–809 requires BAR (a live call), LOC
(valid_block), and WF; lang/steps_retag.v:805–807 requires positive size and an
already successful retag; sim/global_adequacy.v:50–57 consumes never_stuck,
source/target WF and simulation. Its classical alternative at :110–118 consumes
stuck_decidable_1, simulation and WF. sim/program.v:8–15 consumes has_main and
local-function simulation. The bounds TODO, zero-sized no-op and Untagged raw
behavior are explicit in lang/bor_semantics.v:298–340. The inherited Equations
Type/Subterm.v:38–45 proof admission belongs to the full installed kernel
context, not all individual stbor exports. Iris iprop.v:118–156 deliberately
seals a checked implementation; its exposed abstract fields must be reported
separately from ordinary assumptions and from new proof admissions.

Tree Borrows: simulation/language.v:639 defines model pool_safe;
simulation/slsls.v:63–67 consumes it; simulation/lifting.v:76–95 defines
SafeImplies and consumes safety in pool_safe_implies; simulation/adequacy.v:207–208
transfers safety from a safe model source. tree_borrows/bor_semantics.v:944–965
requires a live call, existing parent, freshness, child-creation effect and
retag read (or the separately specified no-op case). No original Rust source
safety or native correspondence follows from these model results. Both delivered
reports correctly retain this boundary. No native/application runs are in scope.

## Final disposition

Both exact mathematical checkpoints are ACCEPTED relative to their complete
reported assumptions after terminal independent normal builds and recursive
kernels were reauthenticated. The medium Tree Borrows finding remains a
**delivery inventory/controller documentation limitation**, not a mathematical
defect or kernel failure. No delivered implementation was edited.

The independent typed structural walk expands56,837 Stacked Borrows and61,887
Tree Borrows fields, including lexical functor contexts, nested modules/types,
constructors and sealed interfaces. All model global bindings are separately
queried:1,262 SB /2,753 TB. The exact sets, individual assumption classes,
query/source/plugin/loader hashes and full private manifests are bound by
RESULTS.json and MANIFESTS.json. The inspection plugin does not replace kernel
checking. Parent can integrate the compact supplemental coverage and correct
its delivered nested-signature claim without changing mathematical source.

## Informational — explicit individual index supplements

Location: source/tools/research/stbor_checked_foundation/inventory.py:46–61,
REPORT.md inventory section; Tree inventory.py:10–28.

The1,247 SB query entries do not independently name two generated nested graph
inductives and13 constructors. All15 names already occur in its43 full block
prints, so this is not evidence of missing compiled proofs. Independent
supplemental `Check`/`Print Assumptions`/`Print` queries for all15 passed and are
closed. The complete explicit SB binding index has1,262 entries (1,200 closed,
62 JMeq-only); the original1,247-entry assumption counts remain correct.

TB's2,399 ordinary constant queries and529 root constant queries are correct.
They do not constitute an individual index for all inductives/constructors.
The independent complete index adds82 inductives and272 constructors:329 from
the first independent discovery and25 further constructors from the typed
structural walk. All354 additions are closed. The total2,753-binding index is
2,722 closed plus the same31 nonclosed constants in the original five classes.
Neither supplement changes the broader dependency context or native status.
