# Checked native Option across inference profiles

This report extends the frozen dirty parent baseline, not clean HEAD. The
worktree stays on its assigned branch with HEAD
`55bd67ad980e90e92c77efe4453a927f097c3386`; no commit, branch, PR, subdelegate,
parent edit or application-clone edit is made. HEAD and `origin/verus-infer`
match the pin. The local `verus-infer` ref instead names
`bad5b8896274d6df1be70e547eb81b004ab62ba1`; that mismatch was reported before
implementation and no ref was moved to hide it.

## Baseline and replay identity

The first restoration attempt used the supplied `restore.apply_patch` directly.
Its numbered unified-diff hunk headers are incompatible with this environment's
`apply_patch` parser. A temporary copy changes only those headers to `@@`; the
restoration itself still runs through `apply_patch`. All 60 restored file hashes
and all nine independent tool hashes match `baseline/manifest.json` before any
implementation edit. A separate copy of the complete baseline bundle, initial
status and HEAD-relative dirty diff is saved first.

- Baseline manifest SHA-256:
  `093188a64998c13166fa0c575432afbeea8df4a311ddbaabb95aa30a2f34c581`.
- Original restore patch SHA-256:
  `03205039c64769315600df825a8f6ff2654200042f3b36ef1fb0ac40ae24097e`.
- Saved baseline and independent artifacts:
  `/private/tmp/verus-option-unification-2b97c3ad`.
- The handoff patch is relative to that restored dirty baseline. Applying it to
  clean HEAD is not the experiment and does not reconstruct the implementation.

No applicable repository `AGENTS.md` exists in the worktree's ancestors or
source/evidence subtrees. The inference direction, architecture, contributor
guidance and live feature guide were read. The read-only application clone's
agent instructions and codebase/development/coding-style skills were also read;
the referenced linked-specs skill is unavailable, and no governed application
source is edited.

## Implementation and proof boundaries

Canonical `then_some`, `map_or`, `is_some` and direct immutable callbacks now use
one bounded readonly value interpreter for scalar values, scalar-newtype
projections and records. The scalar path retains its existing signature
eligibility and exact-callee restrictions. Its interpreter has an empty record
field set, not invented field observations. Native calls retain the existing
identity/type/proxy checks and checked `vstd::native_option` bridge bodies.

Newtype callback formulas are lifted to original source field projections.
Callback decorations are restored onto original scalar, newtype and record
bodies, with the existing binder/body correspondence validation. The exact-only
callback predicate is shared by AST simplification and SST lowering: inverse
callback-domain facts and independent precision packets cannot disagree about
which profiles receive them. Sound-only functions do not acquire those facts.
The original callback body must independently prove its safe-domain coverage,
equation and complete output exactness, without the caller's inferred domain.

Receiver evaluation, ordinary arguments and defaults remain eager and ordered.
Only callback execution requirements are guarded by `Some`. Immutable captures
and shadowing retain their source identities. The existing owned callback
purity gate is reused; effects, mutable calls, assignments, nested callbacks,
returns, authored callback contracts and nontrivial callback construction remain
rejected. Pure array literals are admitted to generated decoration formulas so
the existing byte models compose inside callbacks; this is literal data, not a
new executable or mutable model.

Both the integer-unwrap and Option-map maximality model gates remain intact.
Core Option/Result values remain excluded from the by-value record-to-array
receiver profile. No native Option specification, axiom, unchecked callback
summary, assumed precondition or mutable/stateful profile is added. See
`TRUST.md` for the remaining frontend/library/backend trust boundary.

Candidate-cache version **66**, inference policy **60**, metadata schema **38**
identify this candidate. Parent combined version **67 / 61** remains reserved.
Native value/record candidates are regenerated on warm runs. Ordinary scalar
forwarders can hit the formula cache; their required proofs still run fresh.
Poisoned cached domains in both directions must fail the appropriate proof and
remove stale exports.

## Validation

The exact motivating `Wrapper { value: u8 }` query is rejected by the saved
baseline tools and accepted by the independently built tools, with all three
required proofs. Regression coverage includes scalar/newtype/record cold/warm
providers and imported consumers, captured/shadowed values, by-value wrappers,
eager false/default/receiver safety negatives, Some-only callbacks, direct
`is_some`, zero-argument functions, byte arrays, checked conversions/unwrap,
early returns, stale metadata and corrupted cached domains. Core Option and
Result receivers returning arrays are positive controls for the receiver
exclusion.

Final validation passes **486 Rust tests** (393 integration tests and 93 VIR
inference units), **41 Python test executions**, and a fresh checked vstd build
with **2,221 verified, 0 errors**. The machine-readable manifest records source/tool
hashes, command receipts and fresh vstd verification. Commands run offline with
locked release builds from `source/`, Rust 1.99.1, two Cargo jobs, matching private
target/build directories and the requested stack/solver environment. Independent
target storage is an APFS copy, not a symlink or shared mutable target. The
baseline's independently copied tools are used only for negative comparisons.

## Bounded Fedimint follow-up

Application pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`. The actual seven-field
request item, both helper bodies and `NO_INCOMING_CIRCUIT` are extracted from the
read-only clone. Bitcoin **0.32.8**, bitcoin_hashes **0.14.0**, and all 21 selected
registry package versions/checksums match the application lockfile. A standalone
native compilation uses the real `bitcoin::hashes::sha256::Hash`; no local hash
substitute is introduced. This does not establish complete-crate or full-feature
dependency equivalence.

Both baseline and final frontends reject the full request at its
`payment_hash: sha256::Hash` field with:

> `bitcoin_hashes::sha256::Hash` is not supported

The diagnostic requests an `external_type_specification`. Dropping the field
only in a diagnostic subset makes the other **six native fields**, including
unread amount fields and `short_channel_id: Option<u64>`, verify. No opaque type
specification or generic unused-external-field frontend bypass is added here:
establishing that abstraction's representation, identity, invariant and import
boundary is a separate extension. The full seven-field request is therefore
**not formally verified by this work**.

The available caller-composition milestone is explicit and narrower: the
six-field diagnostic subset keeps both original helper items and the tuple
constant byte-preserved, retains the original `u32`/`u64`/Option domain and the
authored deadline policy, checks both helpers exactly on cold/warm runs, and
checks the authored policy through an imported consumer. Removing exactly the
original forward guard still gives exact helper inference, but fails the
postcondition and deletes the previously published interface. An unsupported
frontend case is not counted as that semantic negative.

No whole-gateway, stateful funding, amount-authenticity or fund-safety claim is
made. Historical transfer reports and runners remain unchanged. Raw artifacts,
dependency comparison, application receipt, `handoff.json`, patch and SHA-256
are retained in the independent scratch directory.
