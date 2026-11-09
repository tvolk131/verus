# Canonical mutable-buffer proving: active goal

Start from accepted checkpoint `cac21695` and its
[acceptance record](../../../../evidence/infer-cycle-20261109/README.md).
The parent is integration lead. Three isolated workers cover mutable effects,
proof assistance and an independently frozen real transfer. A fresh independent
review follows integration. This goal has not been achieved at this checkpoint.

## Frozen workstream milestones

The canonical worker's `e8ad009b` milestone proves the unchanged full predicate
ordinarily (9 verified, zero errors) using explicit checked diagnostic mechanical
helpers. Parent cold/warm replays with and without goal assistance reproduce
that result and reject a false caller while removing its seeded export.
`49235d23` removes the helper definitions using directed existing library laws
and freezes a proofless fixture. The proposed `a4479a6e`/`3e3b4e4c` engine now
verifies candidate 008 with no application-local mechanical proofs or snapshots.
The [parent proofless replay](parent-proofless-milestone.json) reproduces cold/warm
6/0 runs with checked executable `{u64}`, proof `{}` and inferred-helper `{}`
inventories; a healthy-encoder false caller rejects and removes its seeded export.
Production integration, the complete worker control manifest, shared transfer
and fresh independent review remain pending. These are milestones, not final
automation acceptance.

The independent transfer is fixed by `7fe03428`: bitcoin_hashes 0.14.0's exact
HMAC ipad-XOR loop, with arbitrary initialized `[u8;128]` and arbitrary key length.
Its property includes both the XOR prefix and the unchanged suffix. The complete
property remains unproved under the inherited native IterMut/Zip model, while
manual prefix-only proofs and concrete-model frame laws pass. Preserve this
property/domain and the distinction between concrete-model evidence and a law
for the actual native opaque types. [transfer-freeze.json](transfer-freeze.json)
binds the original pretrial source freeze; this is a byte-writer theorem, not an
HMAC/security claim.

The remaining model task is a checked native-resolution bridge, with no assumed
final/current equation, executable rewrite or new unchecked dependency law.
Source-aware refinement/ghost instrumentation may be investigated separately;
acceptance requires checked derivation and source correspondence within the
stated trust boundary. A concrete-model theorem alone does not supply the
missing native bridge. Shared automation must remove actual proof work on both
accepted applications; parser reuse alone is not a transfer result.

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

## Parallel ownership and coordination

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
  proposals and useful commits between isolated workers. Workers may read other
  worktrees but never edit them or parent/application checkouts.

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

The accepted engine uses Rust 1.99.1, Z3 4.16.0, metadata schema 38, policy 67
and candidate cache 70. The lead reconciles version changes if needed; workers
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
