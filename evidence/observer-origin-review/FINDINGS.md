# Prioritized findings

## R1 — P2: generated Zip observations follow a reassigned key local

Locations: `source/vir/src/infer/native_mutable_for.rs:420` and
`source/vir/src/infer/buffer_observation_goals.rs:380` (key expression reused),
`source/vir/src/infer/buffer_observation_goals.rs:686` (whole loop exempted).

Frozen repro: [zip-key-local-reassigned.rs](repros/zip-key-local-reassigned.rs:5).

```rust
let mut k = key;
for (b, h) in buf.iter_mut().zip(k) { k = other; *b = 1; }
```

The actual iterator retains the original slice value. The generated observation
instead repeatedly evaluates `k` in the loop invariant. Assignment changes that
binding without changing the actual iterator. This ordinary full-domain program
verifies **2/0 with assistance off**, but selects `client` and fails **1/1 with
assistance on**, solely `invariant not satisfied at end of loop body`.
No precondition, authored invariant, application proof/helper or assumption is
present. Both modes retain `--no-cheating` and normal body/lifetime/trait checks;
exact executable inventory `{client}`, proof/helper inventories `{}`.

The independently frozen
[same-lifetime parameter variant](repros/zip-key-parameter-same-lifetime.rs:5)
also verifies **2/0 off** and fails **1/1 on** with the same sole generated
invariant diagnostic. Its explicit shared lifetime keeps both modes Rust-valid.
The earlier parameter probe without matching lifetimes fails Rust lifetime
checking in both modes; those two frontend failures are preserved and excluded.

Suggested scoped fix: conservatively skip key expressions whose referenced locals
can change during iteration, or capture the actual evaluated constructor operand
in a fresh, checked binding with whole-function projection and body rechecks.
Do not install an assumed iterator-to-key equation. Retain readonly stable-key
prefix assistance and arbitrary key lengths.

## R2 — P2: nested executable key producer aborts valid ordinary verification

Locations: `source/vir/src/infer/native_mutable_for.rs:411` (accepts any shared
byte-slice expression), `source/vir/src/infer/buffer_observation_goals.rs:380`
(copies expression into annotation), and `:618` (`pure` error propagates).

Frozen repro: [zip-helper-key.rs](repros/zip-helper-key.rs:5). `make_key` simply
returns its input slice with a normally checked equality postcondition:

```rust
for (b, h) in buf.iter_mut().zip(make_key(key)) { *b = 1; }
```

Off verifies **3/0**, checking exactly `{make_key, client}` with no proof or
inference helper bodies. On aborts before proof with
`Verus Infer unsupported: effect or assumption in growth annotation` at
`make_key(key)`. The descriptor accepts the native Zip constructor but copies an
Exec call into a proof annotation, where the purity guard rejects it. This is an
unsupported frontend outcome, **not a semantic negative**, and not evidence of
unsound acceptance. Unsupported annotation synthesis should skip assistance and
preserve ordinary verification rather than reject an otherwise valid program.
Alternatively, support the evaluated operand with a checked, projected scaffold.

## Disposition of requested fixes

The original direct-write/declaration/match-pattern F1 repros and the confirmed
unrelated-constructor rejection are closed in the fresh probes. The new constructor
identity checks retain the actual init/IntoIterator/wrapper/match binding chain.
R1/R2 expose additional eligibility gaps on the immutable Zip operand; they do not
show that the new mutable-constructor identity checks are incorrect. The relevant
key-expression reuse predates both fixes; this review makes no fresh older-engine
runtime claim. F2 remains accepted: fresh unfiltered `infer` passes **299/299**, and the complete
32-target compiler suite passes **618/618**; see REVIEW.md. No production edit was made.

## Reproduction

From this checkout root after the own matching build and immutable bundle copy:

```sh
export VERUS_Z3_PATH=/tmp/observer-origin-review-d35c31fa/bundle/z3
/tmp/observer-origin-review-d35c31fa/bundle/verus evidence/observer-origin-review/repros/zip-key-local-reassigned.rs --crate-type=lib --no-cheating --output-json
/tmp/observer-origin-review-d35c31fa/bundle/verus evidence/observer-origin-review/repros/zip-key-local-reassigned.rs --crate-type=lib --no-cheating --output-json --infer-goals
# Repeat the two commands with zip-helper-key.rs.
```

The recorded runs compile independent private copies, not the committed paths;
`probe-freeze.json` binds both copies' bytes. `manifest.json` binds exact argv,
solver/runtime/source hashes, selected inventories and raw stdout/stderr hashes.
Raw repro outputs: `/tmp/observer-origin-review-d35c31fa/probes/`.
