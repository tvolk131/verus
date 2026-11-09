# Two-project mechanical automation goal

Started November 3, 2026. Status: both targets frozen; cycle 1 implementation.
Implementation/review cycles started: **1 of 3**. Feasibility is not a cycle.

## Finish line

Deliver two source-faithful composed proofs, one redb and one Fedimint, whose
low-level mechanics were unsupported or required authored proof plumbing at the
accepted cycle80 baseline. Make those mechanics automatic through reusable Infer
support. The author still chooses the property, abstraction relation and semantic
invariants. Both frozen targets, not merely their scalar helper controls, must
meet the conditions below.

- Preserve the selected native executable item bytes and original input domains.
  Verification annotations and explicitly labeled scaffolding are allowed.
- Freeze exact items, properties, domains, baseline failures and trust boundaries
  before implementation. Do not substitute easier targets or narrow domains
  without an explicit reassessment recorded here.
- Eliminate authored low-level byte/cast/overflow/algebraic proof plumbing in the
  selected scope, rather than moving it into an application-specific assumption.
- Show composition at a real source-connected caller boundary. Label complete
  native callers, native blocks and authored adapters distinctly.
- Retain false-property and invalid-input rejection controls. Unsupported frontend
  declines, arithmetic-safety failures and semantic failures are separate results.
- Independently review the changes and proof evidence, run existing regressions,
  and integrate a review-ready change set with an honest before/after account.

Stop at this finish line. Reassess after three implementation/review cycles;
exhausting that allowance is not success. No whole-application, recovery, storage,
crash/torn-write, durability, consensus, async/backend or funds-safety claim follows.

## Accepted baseline

- HEAD: `3b05c1912b9d7314ad00c832c4065f4362562e02`, with the accepted dirty
  cycle80 snapshot retained.
- Identity: cache79 / policy73 / schema38.
- Snapshot: `/private/tmp/verus-cycle80-parent.rwZwT3/final-package`.
- Combined patch SHA-256:
  `d56bdf8c96af6ca9d5606e8f56c9d892a09f381b30138fae8029382d547866cc`.
- That combined patch applies once to clean pinned HEAD, not to this already
  restored dirty baseline. Its 2,174-file reconstruction, runtime and native
  inventories were rechecked at goal start with zero mismatches.
- Runtime: `/private/tmp/verus-borrowed-vec80._x31ts_v/delivery/runtime79`.
- redb pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
- Fedimint pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.

No native application edits, commits, pushes, manual branches/refs or PR actions
are authorized by this goal. Preserve the dirty baseline and sealed inputs.

## Target selection

### redb — frozen before implementation

The exact bounded byte-ID-to-slot-selection target is frozen in `REDB_SCOPE.md`.
This deliberately selects native whole leaf/selection methods plus three exact
native contiguous blocks, composed through explicitly authored adapters. Both
whole decoders are outside this bounded target, not weakened into convenient
full-buffer preconditions or replaced by checksum oracles. The parent accepts
the explicitly conditional native-block/caller correspondence boundary.

### Fedimint — frozen before implementation

The exact native fee-validation target and the accepted caller-correspondence
boundary are frozen in `FEDIMINT_SCOPE.md`. This includes native
`PaymentFee::checked_add` / `is_within`, nested `Amount::checked_add`, actual
body-checked built-in derived Amount comparisons, and composition with existing
full-domain fee arithmetic. Both targets were frozen before beginning cycle 1.

Parent baseline probes in
`/private/tmp/verus-two-project-goal.AZF2UH/named-comparison-baseline2/receipt.json`
establish two current boundaries: named single-field comparison derives decline,
and a readonly query with a second borrowed transparent record declines. The
tuple-layout control is not the native Amount representation. These are baseline
diagnostics, not completed target proofs or a final scope freeze.

## Cycle ledger

Both feasibility reports, their exact native inputs and raw baseline controls
have been inspected. Both complete package seals verify. No independent proof
acceptance is implied by the target freeze.

Cycle 1: parallel redb and Fedimint implementation against the exact accepted
cycle80 baseline, followed by separate new independent reviews and parent
integration. Reserve cache80/policy74 for the redb candidate, cache81/policy75
for Fedimint, and cache82/policy76 for the integrated candidate, schema38
unchanged. Parent owns shared readers/controllers/CI/guide reconciliation.
Neither track may silently drop a frozen property or reduce its domain. A
substantial blocker must be delivered with its reproducer for explicit
reassessment, not relabeled success. This cycle is in progress, not accepted.

### Cycle 1 progress — Fedimint candidate installed, not yet accepted

The sealed Fedimint candidate (cache81/policy75/schema38) was installed exactly:
all 2,181 candidate file hashes match, with the extra parent tests/goal documents
preserved. The application audit guards 1,410 additional inputs and confirms
unchanged own HEAD/index/branch. Its patch SHA-256 is
`2b666981a9dcb98b1303de2d91b29f2ffcae8ee81ab2be5253a983a699beda05`.
The candidate reports the full frozen proof milestone, but NEW independent
review is pending; this is not integrated or final goal acceptance.

Parent generic fee-composition probes now freshly verify for both u8 and u64:
18 verified / 0 errors each, four exact inferred helpers each, mandatory checks
proved and nonempty compiled/exported artifacts. All three parent
`infer_fee_validation_composition.rs` test methods also pass against the sealed
candidate runtime, including actual imported composition and failed-client
export removal. These establish reusable interaction coverage, not a substitute
for the native-target review. Twelve new acceptance methods pass in each Python
mode. The byte-decoder and mixed-feature methods still await redb integration.

Evidence is under `/private/tmp/verus-two-project-goal.AZF2UH`:
`fedimint-delivery-audit/application.json`,
`composition-fees81-v3/receipt.json`, and
`parent-fee-tests81-v3/receipt.json`. The private offline/locked Rust harness
copies the existing sysroot-routing adaptation; original shared test helpers,
native application trees and sealed runtimes are not edited. Earlier private
launcher/manifest/bootstrap setup failures are retained separately and are not
counted as failing application proofs or successful regression runs.

redb implementation, the Fedimint independent review, final shared-reader
qualification at82/76, integrated build/regressions and independent integration
acceptance remain pending. The frozen scopes and completion conditions are
unchanged; cycle 1 has not finished.

### Cycle 1 broader regression qualification

Parent-side testing now covers all 36 currently applicable Infer Rust suites:
619 test methods pass across corrected fresh runs on sealed81/75. The first
broad run retained one setup failure: a nested native-buffer test requested the
named installed Rust toolchain, absent from the private rustup directory, and
tried fetching it. Linking the already-installed toolchain into that private
directory resolves the issue offline. The targeted test and a complete fresh
296-test rerun of the main Infer suite pass. The other 323 methods passed in
the initial broad run. This is not described as a single green broad invocation;
the original failing receipt and exact runner version are retained. No new
production regression was established. The seven byte/mixed parent methods
remain explicitly pending redb support.

Python checks freshly execute 477 methods in each mode: 474 pass, two expected
identity-consistency tests fail because shared readers still describe79/73,
and one receipt-backed class skips without a fresh native-observation receipt.
Those failures/skips are NOT passing evidence. Final82/76 reconciliation and
fresh receipt-backed/full regressions remain required.

Raw receipts and explicit aggregation are
`/private/tmp/verus-two-project-goal.AZF2UH/broad-infer81-v2/receipt.json`,
`broad-infer81-main-recheck/receipt.json`, `broad-infer81-summary.json`, and
`python81/receipt.json` under the same directory. Guarded source/runtime inputs
match across corrected Rust runs; Python input guards also match. The private
final-build dependency copy is prepared from 10,987 hash-matching cached inputs,
with the original donor unchanged; preparation earns zero build or proof credit.
`AUTHOR_BOUNDARIES.md` records the remaining author work without weakening either
frozen target.

### Cycle 1 combined source installation

The redb delivery is now reconstructed and seal-verified: 2,184 candidate files,
19 delta files and nine runtime files match. A new independent review covers its
production mechanics, original safe domains, exact native-block correspondence,
complete composed theorem and acceptance gates. The Fedimint review found two
acceptance-gate defects, not a production semantic defect. Both are repaired:
complete native-body sets/classifications/counts are checked, and positive reports
must explicitly declare both verifier error flags false. The repaired 46-case
replays and 19 reader methods pass in each Python mode. A new independent recheck
is pending; neither parent validation nor package integrity is independent approval.

The exact redb delta is merged with the repaired Fedimint candidate. Only their
cache/policy identity declarations overlapped. The combined identity is82/76/38;
the 2,191-path source ledger and all additional parent files match their intended
contents, and the Git index is unchanged. The first combined compiler/macros
build passes offline/locked in a new private target, with all 1,098 guarded inputs
unchanged. Evidence is `combined82-source-installation/receipt.json` and
`integrated-build/logs-build/receipt.json` under the parent scratch root.

Fresh matched vstd, final producer/model qualification, both complete application
replays, all ten parent generic methods, broad regressions, independent review
closure and the requirement-by-requirement final audit remain required. Cycle 1
is not accepted or complete; the frozen target and finish line are unchanged.

### Cycle 1 combined replay and review closure

The new independent Fedimint81 recheck accepts both repaired acceptance gates
and the supplementary report-schema guard. Fresh in each Python mode, the
reviewer ran all 46 cases, 19 focused tests, 1,033 intended malformed-report
rejections and 19 isolated guard-removal checks. This accepts the repaired
candidate81 scope only, not the combined82 goal. The sealed report is
`/private/tmp/verus-fedimint81-independent-recheck.UmaDAH8E/REVIEW.md`.

Fresh matching combined82 vstd verification passes 2,221 obligations with zero
errors. Both complete frozen native matrices then pass normally and under
`-O`: Fedimint has 46 cases, provider 55/0 and imported consumers 4/0, with all
41 seeded failed exports removed; redb has 81 cases, frozen composition 47/0
in each debug mode, 1,215 mandatory exact-helper checks and all 76 seeded failed
exports removed. These receipts are under `native-targets82` in the parent
scratch directory. Historical candidate receipts are not substituted for them.

The ten generic cross-feature methods exposed remaining composition failures:
fee methods pass, but a concrete longer-slice byte result and mixed byte/record
callers do not all verify. An implementation follow-up must fix the reusable
mechanics without weakening these tests or the frozen application domains.
The failing receipts remain preserved. The integrated runtime is therefore
interim, not the final accepted build, despite the two successful native targets.

Producer/model qualification also requires explicit reconciliation. All 7,440
exported vstd function definitions, including the current 382 standard-model
definitions, match the accepted baseline. One existing guarded Core
slice-to-array correspondence becomes newly eligible; there is no new library
specification family or application axiom. Actual model fingerprints change
with the cache-version salt and must be pinned from the matching fresh artifact,
not claimed equal to historical fingerprints or rederived from raw exported
AST. The successful definition-consistency audit and retained failed raw-AST
fingerprint attempt are recorded separately under `model-definition-audit82`.

Final generic repair, matched rebuild/regressions, redb independent review,
shared qualification, independent integrated acceptance and the complete
requirement audit are still required. Neither frozen scope is reduced.

### Cycle 1 redb acceptance and combined compatibility regressions

The new redb-only independent review accepts the complete frozen R1 candidate
at80/74/38. Fresh in each Python mode it executes all 81 cases, 1,215 exact
checks, 76 seeded-export removals, nine reader methods, 26 isolated guard
removals and 25 focused probes. Its seal and all 1,809 payloads are verified.
The report is
`/private/tmp/verus-redb-r1-independent-20261103-ced01f98/sealed/REPORT.md`.
This does not accept combined82. Its supplementary MAX/high-bit literal
corollaries remain recorded automation limitations; they do not exclude those
values from the accepted universal little-endian theorem.

The common six-scenario suite freshly passes on interim combined82 in both
Python modes. All twelve scenario outcomes are individually re-read as proved;
the matching-runtime receipts are under `interim-regressions82-six`. The broader
Rust check also passes 625 methods across 38 suites, with zero failures or
ignored methods and unchanged guarded source/runtime inputs. This deliberately
excludes the byte-decoder and mixed-feature suites: their seven methods are
still pending, not silently counted as green. The fee-composition suite is
included. A preceding harness compilation failure was a missing copied native
fixture, corrected only in the private harness; its failed receipt remains
under `broad-integrated82-interim`. The corrected run and its exact executed
runner are under `broad-integrated82-interim-recheck`.

Both frozen scope documents and the complete requirement checklist still match
their pre-installation hashes. This is additional regression and preservation
evidence, not final goal acceptance. The generic repair, final matched runtime,
full forty-suite check, final reader/model qualification and new independent
integrated review remain required.

### Cycle 1 generic composition follow-up

The five-file generic follow-up is seal-verified and installed without unrelated
parent changes. It retains borrowed literal definitions through canonical boxed
array/slice coercions and uses backend-checked, closed bitwise equalities when
composing callee contracts. It adds no trusted model, application axiom or scope
restriction. Identity remains82/76/38. The three positive generic tests explicitly
opt into `--infer-goals`; one negative's exec-call placement is corrected without
changing its false overflow assertion or expected rejection.

Child evidence reports all ten parent methods and both supplementary redb
MAX/high-bit numeric corollaries passing. These are implementation results, not
parent replay or independent integration acceptance. The old failures remain
preserved. A new final compiler/vstd runtime, full target matrices, all forty Rust
suites, shared model/producer qualification, Python/receipt regressions and new
independent review are still required before completing this goal. Frozen scope
and completion-matrix hashes remain unchanged.

The final offline/locked compiler build and matching vstd now pass, guarding all
1,100 Rust/build/test inputs; vstd verifies2,221/0. Fresh shared qualification on
this final runtime proves the unchanged legacy provider23/0 with its exact43-row
scope. It pins the actual producer and382 models, with381 retained identities and
one newly eligible existing slice-to-array correspondence. All7,440 exported
library definitions are identical to the accepted baseline. This is definition
consistency plus actual-artifact fingerprint binding, not a successful raw-AST
fingerprint derivation. Final target/regression evidence and independent final
integration acceptance remain pending. Historical fixtures and prior runtimes
are preserved rather than relabeled.

### Cycle 1 integration regression and bounded repair

The first final runtime passes both full application matrices in both Python
modes and all ten generic composition methods, but it is NOT an accepted final
runtime. The common-suite page-relation witness and all three existing Rust
page-relation methods fail. New closed-constant scans redundantly process exact
inferred summaries and can consume the eight-request budget before the existing
requirement/return tactics. All original failing source, runtime, receipts and
logs are retained under `final-regressions82-six` and `broad-integrated82-final`.

The bounded repair leaves already-inferred summaries to their existing request
path; optional closed-constant discovery is used for ordinary callee contracts.
It does not raise the budget, relax a proof obligation or change any application
body or frozen domain. A new unit regression checks both the inferred-summary
skip and ordinary-callee discovery, including unchanged request/work/visit budgets
for the skip. New follow-up build/replay directories preserve every first-final
artifact. The repair still needs fresh regressions and independent review; this
failure is not relabeled as a harness issue or passing evidence.

On the follow-up matched compiler/vstd, all three previously failing page-relation
methods now pass without changing their sources or assertions. Both identical
supplementary numeric corollaries also verify. Their count drops53 to51 because
redundant assistance queries are no longer emitted; all five inferred helpers
still pass their mandatory checks. The first follow-up wrapper expected the old53
count and rejected an actually successful51/0 proof; its unchanged stdout/stderr
are retained separately from the correctly counted fresh replay. Final matrices,
shared regressions and independent acceptance are still required.
