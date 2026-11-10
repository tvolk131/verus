# Canonical mutable-buffer proving: active goal

Start from accepted checkpoint `cac21695` and its
[acceptance record](../../../../evidence/infer-cycle-20261109/README.md).
The parent is integration lead. Three isolated workers cover mutable effects,
proof assistance and an independently frozen real transfer. A fresh independent
review follows integration. This goal has not been achieved at this checkpoint.

## Frozen workstream milestones

The integrated canonical assistance proves the unchanged full predicate cold
and warm (6 verified, zero errors), with checked executable `{u64}`, proof `{}`
and inferred-helper `{}` inventories. Candidate 008 retains authored semantic
invariants and exit clauses, with no application-local mechanical proofs or
snapshots. The [parent combined record](parent-combined.json) binds the matching
`676b99bf` engine, 26 canonical replay cases, 23 observation cases, 148 VIR tests
and vstd 2,226/0. Both decorators receive full generated-body well-formedness
and mode checking. The earlier scoped canonical integration received an
[independent acceptance recommendation](../../../../evidence/canonical-parent-review/REVIEW.md).
Subsequent combined reviews found observer eligibility bugs and a stale reporting
test: [initial findings](../../../../evidence/shared-buffer-review/REVIEW.md) and
[alias follow-up](../../../../evidence/shared-buffer-fix-review/FINDINGS.md).
The full 32-target compiler suite passed 611 tests at `ea897a9b`, but the alias
follow-up still requested changes. The declaration/match and constructor-to-loop fixes passed matching validation:
[32 targets / 618 tests, 26 canonical and 23 observer cases](parent-observer-origin.json).
A new independent review closed those findings but found mutable/effectful Zip
key operands. The stable-key restriction (`a93b774a`) passes 31 focused tests,
148 VIR tests, 621 compiler tests across all 32 Infer targets, 148 Python tests
in each mode and both replay suites. Its [independent recheck](parent-stable-key.json)
accepted the scoped fix (`d64897d4`); full native/transfer acceptance remains open. Earlier green receipts describe their historical source
checkpoints; they do not establish acceptance of the current branch.

The independent transfer is fixed by `7fe03428`: bitcoin_hashes 0.14.0's exact
HMAC ipad-XOR loop, with arbitrary initialized `[u8;128]` and arbitrary key length.
Its property includes both the XOR prefix and the unchanged suffix. The complete
property remains unproved under the inherited native IterMut/Zip model. Shared
observation assistance removes two source snapshots and four mechanical
invariant formulas from the prefix-only proof: enabled it verifies 2/0, disabled
it fails 1/1. The full property still fails 1/1. Preserve this
property/domain and the distinction between concrete-model evidence and a law
for the actual native opaque types. [transfer-freeze.json](transfer-freeze.json)
binds the original pretrial source freeze; this is a byte-writer theorem, not an
HMAC/security claim. A [45-case controlled ablation](../shared_observation_ablation/REPORT.md)
finds the observer entirely redundant for full canonical encoding, necessary as
a group for Zip prefix, and its capacity proposal redundant for both. Necessary
shared proof work across the full properties is not confirmed; parser reuse or
running both under one flag does not establish it.

The remaining model task is a checked native ownership/refinement bridge.
Checked reference entry and 18 conditional linear-resource proofs are preserved,
but zero actual native correspondences have been proved; all 101 native effect
obligations remain open. A smaller constructor/drop case has a complete six-instance,
44-operation [source inventory](native-drop-inventory.json), without a frame
proof. The next step carries one ownership interpretation through the actual
constructor operand, pointer operations and drop, avoiding a second live Rust
borrow. The [pinned VerusBelt baseline](foundation-baseline.json) now compiles
and passes recursive kernel checking. The new same-reference physical access,
restoration and byte-array scripts also compile and pass recursive kernel
checking: [18 model theorems and six byte-type obligations](parent-semantic-foundation.json).
The exact frozen model replay has independent acceptance. A subsequent
source-freeze completeness finding was closed. The next review found a
validation-to-snapshot race; the local fix binds frozen identities to retained
guard snapshots. Matching model compilation, all 25 assumption entries and
recursive kernel checking pass with the original four object hashes. The
inspection/control batch also passed. A subsequent source-only review found
three remaining controller issues; their local source-identity, support-inventory
and final-status fixes pass targeted tests. Matching replay and fresh review
of those fixes remain pending.
The [source-entry investigation](../native_source_entry/MISSING_LAYER.md)
identifies the missing actual source-state and typing-context interpretation;
its 42-constant inspection proves no new native theorem. The
[fact-provenance trace](FACT_PROVENANCE.md) records inherited standard-contract
facts used by the observer-disabled canonical proof, without claiming individual
indispensability or a checked common transfer step.
The model results remain distinct from a native Rust interpretation theorem.
The existing `source_loan_release`/`typed_source_loan_release` already derive
future/current equality for a resolved model unique reference. Repeating that
lemma would not connect the opaque native iterator to the same original loan;
the source-state/context and native-body correspondences remain required.
No assumed final/current equation, executable rewrite or new unchecked
dependency law is permitted. Shared automation must remove actual proof work on
both full properties; parser reuse or a conditional model theorem is insufficient.

## Required property and domain

Prove the existing unsigned-varint 0.8.0 canonical encoder property for **every
u64 and arbitrary initialized exclusive ten-byte buffer**. Retain the frozen
original executable body, its normal safety/debug checks and the existing
layout-free macro syntax adapter. Do not substitute a different algorithm,
unroll or rewrite the loop, assume a zero buffer, restrict numeric inputs, or
weaken the canonical property to safety/length.

The canonical predicate specifies returned length, radix-128 payloads,
continuation bits, upper bound and minimality. Its existing definition is in
the frozen layout-free fixtures. Formal untouched-suffix or pointer-identity
claims are not added to that property; Rust lifetime/exclusivity checks remain
enabled, and native correspondence remains supporting evidence.

Authors may supply semantic definitions and inductive semantic invariants.
Inventory these separately from mechanical bounds, iterator-index equalities,
shift lemmas, snapshots, representation bridges and frame proofs. Freeze each
new authored semantic-input candidate before testing it; retain failed attempts
and identify the final accepted inventory. Do not hide removed proof work inside
application spec/proof helpers or rename mechanical facts as semantic inputs.

## Completion criteria

1. The full frozen canonical theorem verifies over the original domain.
2. The accepted application fixture has no local mechanical proof blocks,
   ghost snapshots or theorem calls. Reusable generic library proofs may be
   authored once, checked normally and inventoried explicitly. Semantic
   invariants remain visible inputs. Assistance preserves executable content.
3. A second independently frozen real buffer writer/codec verifies a meaningful
   full-domain property using the same automation. A renamed varint wrapper,
   toy implementation or newly restricted domain does not establish transfer.
4. Full normal checking, source fidelity, explicit helper identity coverage,
   required inference proofs, cold/warm cases, imports, poisoned candidate/cache/
   metadata controls and seeded failed-export removal pass. Distinguish ordinary
   semantic negatives, safety failures, failed auxiliary proposals and intentional
   inference-poison rejections. Resource/frontend errors are not semantic evidence.
5. Fresh independent review and matching combined regression/replay validation
   pass. Record trust, authored inputs, reductions, source/tool hashes and limits.

Ordinary-proof-first assistance is preferred. Exact inference for every function
or for the entire encoder is not a completion requirement. A call allowlist for
`iter_mut` is not a mutable-effect model. Partial length/safety progress or a
generic lemma library alone does not complete this goal.

## Original parallel ownership and current coordination

- **Mutable-effects worker:** inspect the existing prophetic `IterMut` model;
  establish checked reusable observation, current-referent and unchanged-state
  frame laws needed by ordinary proofs. Own new `source/vstd/mutable_buffer.rs`,
  its registration, focused library tests and
  `source/tools/research/mutable_buffer_effects/`. Necessary additions to existing
  slice/iterator files must preserve their specifications and assumptions.
  Propose typed helper signatures and minimal repros early. Do not widen trusted
  external models or implement a broad direct-inference effect engine speculatively.
- **Proof-assistance worker:** first establish a source-faithful ordinary proof
  of the canonical property with an honest annotation inventory, then move
  reusable mechanical steps into bounded checked assistance. Own new
  `source/vir/src/infer/buffer_goals.rs`, its module/pipeline registration,
  `infer_mutable_buffer_goals.rs` and
  `source/tools/research/varint_canonical_goal/`. Keep temporary manual prototypes
  separate from the final annotation-free mechanical fixture. Use the existing
  mutable model while identifying specific missing laws for the effects worker.
- **Independent-transfer worker:** own
  `source/tools/research/buffer_transfer_goal/`. Inspect real pinned source,
  select and freeze its body/domain/property before trials, reproduce the
  accepted-engine baseline, and develop a source-faithful authored proof and
  semantic/safety controls. Do not edit engine/library/shared harness sources.
- **Integration lead:** owns this charter/freeze, global policy/cache/schema
  constants, shared harness utilities, current docs, cross-workstream interface
  reconciliation, integration and final acceptance. Parent routes interface
  proposals and useful commits between isolated workers. Workers may read explicitly
  delivered committed milestones with `git show`. They must not inspect live
  sibling worktrees or edit sibling, parent or application checkouts.

The original three workstreams above have delivered their scoped milestones.
Current ownership and validation state are recorded in the linked parent records;
new foundation/review work keeps separate isolated checkouts and owned paths.

Keep module interfaces small and application independent. Each worker should
deliver an early committed interface/baseline milestone before expanding scope.
Use additive helpers and separate narrowly scoped commits for necessary hooks;
record dependencies on another worker rather than duplicating their module.
Do not spend a long run blocked on a sibling: report the minimal missing
interface/repro and complete independent work. No subdelegation or push/PR.

Completed baseline workers may be followed by focused isolated implementation
lanes while the parent keeps integration ownership:

- `source/tools/research/native_iterator_refinement/`: investigate checked
  correspondence for the actual native resolution bridge. The existing concrete
  model theorem and the unproved native theorem remain distinct.
- `source/tools/research/native_iterator_certificate/`: implement the diagnostic
  native-body extraction/checking stages in
  [NATIVE_CERTIFICATE.md](NATIVE_CERTIFICATE.md). No proof fact may be installed
  merely from an identity/hash match or an unchecked ownership declaration.
- `source/vir/src/infer/native_mutable_for.rs`,
  `source/vir/src/infer/buffer_observation_goals.rs`,
  `infer_buffer_observation_goals.rs` and
  `source/tools/research/buffer_observation_goal/`: develop reusable checked
  iterator observations, lengths and generated view snapshots for plain and
  zipped mutable iteration. Do not edit the numeric worker's `buffer_goals.rs`;
  provide a small shared interface and separate registration hooks for parent
  reconciliation. This lane cannot assume or silently synthesize the missing
  unused-tail frame. A prefix-only diagnostic is not full transfer acceptance.

All generated IDs must be fresh relative to original code and preceding passes.
Each decorator needs an independent full-function projection check restoring
exactly its own additions, with mutation and insertion-count controls. Existing
compiler-generated resolution nodes, original invariants and termination data
must remain intact. A decorator adding body statements or bindings must rerun
full function well-formedness and mode checks, then refresh resolution maps for
all generated locals and expression IDs. Contract-only checking does not cover
new body expressions. Shared automation acceptance requires actual checked proof
steps on both full goals, not parser reuse or unrelated successful probes.

## Build, evidence and inherited boundaries

The original accepted baseline uses Rust 1.99.1, Z3 4.16.0, metadata schema 38,
policy 67 and candidate cache 70. The current integrated engine uses policy 68
and cache 71, preserving schema 38 and requiring rebuilt providers/dependents.
The lead reconciles version changes if needed; workers
must not change global constants or weaken acceptance checks. Cargo commands
run from `source/` with `/home/dev/.cargo/bin` first on PATH and
`CARGO_BUILD_JOBS=2`. Build own matching compiler/macros/vstd for code changes.
An immutable private COPY of the accepted baseline bundle/solver may be used
for initial measurements; record hashes and runtime identity, never symlink or
modify parent artifacts. Preserve normal lifetime/trait/body checking and
`--no-cheating`; no new application assumptions or assumed helper summaries.

Reuse `source/tools/infer_experiment.py` and `infer_eval_support.py` for explicit
checks, logs, artifact stability and failure receipts. Python runtime assertions
are unsuitable as acceptance gates. Require expected helper identities/counts
before checking all named obligations. Known ordinary-only cases explicitly
expect zero helpers. Semantic negatives retain healthy helper checks and one
intended failure kind; intentional poisoned-helper controls are separate.

Raw evidence goes to private persistent `/tmp` directories. Commit compact
source-connected freezes, fixtures, reports, controllers and manifests; preserve
the earlier frozen varint and transfer evidence unchanged. Per-command Git
identity may use Codex/codex@openai.com; do not set global configuration.

Shared clean pinned application inputs, read only:

- `/tmp/verus-infer-cycle-20261109/applications/redb`,
  `c82ab865bb362bc4af067d332adc44978ff64813`.
- `/tmp/verus-infer-cycle-20261109/applications/fedimint`,
  `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.

The unfamiliar writer may use another faithfully pinned upstream source with
explicit provenance. Existing standard-library contracts and declared dependency
specifications remain trust boundaries. Existing no-flag legacy ordinary export
behavior is not an inference-aware proof certificate. Whole-database, consensus,
crypto and arbitrary-Rust coverage are not claimed by this bounded goal.
