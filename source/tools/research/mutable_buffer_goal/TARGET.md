# Canonical mutable-buffer proving: active goal

Start from accepted checkpoint `cac21695` and its
[acceptance record](../../../../evidence/infer-cycle-20261109/README.md).
The parent is integration lead. Three isolated workers handle assigned implementation and review lanes. A fresh independent
review follows integration. This goal has not been achieved at this checkpoint.

## Current evidence and remaining work

The goal remains incomplete. Production checkpoint `a93b774a` retains schema38,
policy68/cache71 and normal body, lifetime, trait, mode and well-formedness checks.
Its [matching validation and independent recheck](parent-stable-key.json) cover
148 VIR tests, 621 compiler tests across32 targets, 148 Python tests in each mode,
vstd2226/0 and both canonical/observer replay suites. Subsequent work below is
MODEL research; it installs no new production fact.

The unchanged canonical fixture proves the full property cold/warm **6/0**, with
exec `{u64}`, proof `{}` and inferred-helper `{}`. It retains authored semantic
invariants/exit clauses and no application-local mechanical proofs or snapshots.
The [canonical evidence](../varint_canonical_goal/RESULTS.md) and
[independent review](../../../../evidence/canonical-parent-review/REVIEW.md)
record its source, domain and validation limits.

The second frozen target is bitcoin_hashes0.14.0's exact HMAC ipad-XOR loop:
arbitrary initialized `[u8;128]` and every key length, with XOR prefix **and
unchanged suffix**. The full theorem remains **1/1 unproved**. Its
[pretrial freeze](transfer-freeze.json) is unchanged. Prefix-only assistance
passes2/0 enabled versus1/1 disabled, removing two snapshots and four mechanical
invariant formulas. This is a byte-writer property, not HMAC/security correctness.

The [45-case observer ablation](../shared_observation_ablation/REPORT.md) finds
that observer redundant for canonical, necessary as a group for Zip prefix, and
its capacity proposal redundant for both. Necessary shared proof work across
both full properties is **not confirmed**. Parser reuse or a common flag does
not establish transfer. The [fact-provenance trace](FACT_PROVENANCE.md) records
the inherited native-contract facts used by the observer-disabled canonical proof.

Checked model prerequisites are recorded separately from native correctness:

| Component | Evidence and scope |
| --- | --- |
| Physical source-loan model and byte representation | [Foundation record](parent-semantic-foundation.json); original model-only assumptions and object identities retained. |
| Finite heap, typed input and resource-valued setup | [29-constant initializer](parent-heap-initialization.json), [26-constant typed context](parent-model-input-context.json), [three-law setup](parent-resource-valued-setup.json). Permissions and credits are derived internally in the model. |
| Actual final model heap observation | [20-law observer](parent-model-state-observation.json); exact interval/frame in the same heap, without requiring all forked threads to terminate. |
| Nested/changing borrow and payload recovery | [24-law nested model](parent-nested-borrow.json); [independently accepted seven-law changing-buffer observation](parent-model-transformation-observation.json). |
| Concrete model XOR and selected instruction fragments | [Full-byte XOR model](parent-byte-xor.json), [primitive fragments](parent-native-primitive-refinement.json), [register segment](parent-native-register-segment.json), [word/cell XOR](parent-native-byte-xor.json). Source selection is not a native execution proof. |
| Same-heap mutable pad and shared key | [18-law pair setup](parent-model-pair-input.json) and [19-law shared reader](parent-model-shared-byte-read.json), with matching replays and independent acceptance. [Fixed-budget reader16](../model_pair_reader/results.json) and [shared XOR loop12](../model_shared_xor_loop/EVIDENCE.json) are now kernel checked; combined review and writer integration remain open. |

A physical transformation WP remains an explicit proof obligation of the generic
model wrapper. Full initialization, shared-key loop, original-parent recovery
and actual final pad/key/frame observation must still be composed. Ordinary
memory accesses also need their checked correspondence; SC model steps are not
silently native ordinary accesses.

**Native correspondences remain zero:** all44 minimal operations plus3 phase
obligations and all101 writer effects remain open. The
[source inventory](native-drop-inventory.json),
[source-entry missing layer](../native_source_entry/MISSING_LAYER.md) and
[native certificate boundary](NATIVE_CERTIFICATE.md) identify the remaining
actual Rust state/typing-context, provenance/ancestor-loan, operation/phase and
original C/F obligations. Fresh model heaps/lifetimes and the existing checked
`source_loan_release` do not establish those correspondences. No assumed opaque
iterator frame, final/current bridge, executable rewrite or unchecked dependency
law is permitted. Publication and whole-goal completion remain false.

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

## Coordination

The original mutable-effects, canonical-assistance and independent-transfer
workers delivered their scoped milestones; their reports preserve the historical
ownership and failed trials. Three isolated implementation/review lanes continue
under the integration lead. Each new task owns only its explicitly assigned paths.

The parent owns this charter, global policy/cache/schema constants, shared tools,
current documentation, interface reconciliation, integration and final acceptance.
Workers may read explicitly delivered committed milestones with `git show` and
named completed read-only artifacts. They must not inspect live sibling/parent
checkouts or edit other scopes. An old checkout path inside a receipt is a label,
not permission to read that checkout. No subdelegation, push or PR.

Keep interfaces small and application independent. Deliver an early committed
interface/baseline before expanding scope. Use separate additive hook commits;
route dependencies through the parent instead of duplicating a sibling module.
When waiting on an interface, report the minimal typed need/repro and continue
independent work. Each implementation is integrated before fresh independent
review and matching acceptance.

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
