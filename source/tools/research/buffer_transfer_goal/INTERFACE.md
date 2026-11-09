# Independent transfer: first interface milestone

Frozen before all proof trials at `7fe03428`. Target: exact bitcoin_hashes
0.14.0 HMAC short-key ipad loop, extracted as a standalone fixed-array writer.
This is a source slice, not a proof of HmacEngine::new or HMAC. No dependency
helper/body has been assumed. The domain covers arbitrary `[u8;128]` and an
arbitrary immutable key slice of any length; the original call context is a
subset. Zip truncates naturally. All 128 output bytes and the untouched suffix
are specified. Immutable-key preservation comes from Rust's borrow discipline.

Accepted bundle baseline: four full-domain failures (1 verified, 1 error),
ordinary/`--infer-goals`, cold/warm. Each has only a postcondition diagnostic.
Safety-only result is 2 verified, zero errors. Helpers: explicitly zero in every
case. There is no helper cache entry, cache-hit claim, or inferred writer summary.
Semantic wrong-frame control fails only an assertion; `+=` mutant fails only
an overflow safety obligation. These are deliberately distinct.

## Typed dependency proposal (not assumed or proved)

A generic checked library should expose observation of a finite mutable-reference
sequence and preserve its frame. Suitable vocabulary is:

```rust
#[verifier::prophetic]
spec fn observed<T>(refs: Seq<&mut T>) -> Seq<T>; // pointwise initial dereference
#[verifier::prophetic]
spec fn eventual<T>(refs: Seq<&mut T>) -> Seq<T>; // pointwise final dereference
```

The target already uses the existing standard-library prophetic IterMut contract,
which relates initial/final dereferences to the enclosing array/slice. It needs
checked transport through `Zip<IterMut<u8>, Iter<u8>>`, preserving a prefix XOR
invariant while proving the unconsumed suffix unchanged. Do not assume
`eventual(refs)[i] == observed(refs)[i]` for arbitrary references: that claim needs
an actual untouched/dropped-referent observation from the mutable iterator model.
A theorem merely about unrelated sequences does not establish this bridge.

The proposed first application candidate is `candidate-01.rs`, separately frozen
before trial: a single semantic processed-prefix invariant, no mechanical inputs.
A future ordinary manual prototype may add explicitly inventoried snapshots,
bridges and theorem calls, but is never the final automation fixture.

## Replay

```sh
python3 -B -O source/tools/research/buffer_transfer_goal/replay.py \
  --verus /tmp/verus-buffer-transfer-20261109/accepted/verus \
  --solver /tmp/verus-buffer-transfer-20261109/accepted/z3 \
  --output /tmp/verus-buffer-transfer-20261109/runs
```

This checks expected baseline failures, not transfer success. Normal
`--no-cheating`, lifetime/trait and whole-crate checks remain enabled. Bundle
identity is recorded in `bundle-identity.json`; the accepted files were copied,
never linked, to private immutable files, and are guarded/rehashed during replay.
Two harness-development failures (loop indentation validator, and classifying
an overflow diagnostic as an assertion) are retained as failed receipts and
excluded from accepted outcomes. An initial unrecorded `--version` shell probe
without Cargo on PATH failed setup; the guarded replay fixes PATH explicitly.
