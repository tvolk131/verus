# Mutable reference observation: early interface milestone

The unchanged `IterMut` contract is enough to prove a written prefix, including
an early break, over arbitrary initial contents and lengths. The frozen baseline
`candidates/baseline-prefix-v1` verifies 2 queries, zero errors, on accepted
runtime 0c00b6ec (full ordinary checking, --no-cheating). The probe keeps the
counter convention of a loop that breaks before incrementing the final index.
It contains an authored mechanical iterator/index invariant and a semantic
written-prefix invariant, and no local proof blocks or assumptions.

Proposed small application-independent interface in `vstd::mutable_buffer`:

```rust
pub open spec fn current_values<'a, T>(refs: Seq<&'a mut T>) -> Seq<T>;
#[verifier::prophetic]
pub open spec fn final_values<'a, T>(refs: Seq<&'a mut T>) -> Seq<T>;
#[verifier::prophetic]
pub open spec fn written_prefix<'a, T>(
    refs: Seq<&'a mut T>, count: int, property: spec_fn(int, T) -> bool,
) -> bool;
```

Checked observation/index laws expose each value; prefix extension combines a
prior prefix with a final observation of the current referent. A history law
transports final-value properties through equality of referents. All require
ordinary checked hypotheses and add no mutable effect or summary assumptions.
The definitions are generic in T, length, and semantic predicate.

The ordinary baseline can immediately be used by the proof-assistance worker:
use `forall|j: int| 0 <= j < it.index() ==> P(j, *final(it.seq()[j]))` as the
written-prefix invariant. Inside the loop, the current `b` is at the **pre-next**
index; the wrapper has advanced its internal index/history by one. A break must
prove its loop `ensures` explicitly at that position. Natural exhaustion gives
the full iterator length. The initial/final slice correspondence from iter_mut
then transfers these facts to the returned original buffer. These are existing
contracts, not a requested new inference allowlist.

## Exact untouched-state boundary

`candidates/baseline-unused-frame-v1` is a minimal failure: merely create
`buf.iter_mut()` and let it go out of scope, then ask `final(buf)@ == old(buf)@`.
It fails exactly its postcondition (0 verified, 1 error), retaining full checking.
The inherited contract relates initial and eventual referents to the original
slice but does not require an unyielded referent's future value to equal its
initial value. `next` only removes a head. `will_return_none` gives termination,
not a frame, and the standard iterator has no drop contract resolving unused
reference prophecies. Hence checked sequence laws alone cannot supply that
missing equation. This is a limitation of this abstraction, not evidence that
Rust mutates the untouched suffix. No trusted contract change is proposed.
The canonical goal does not include an untouched-suffix property, so this
boundary should not block its consumed-prefix proof.

## Authored-input inventory

- Goal meaning for this toy baseline: the requested prefix consists of `value`.
- Domain: arbitrary initialized exclusive slice, arbitrary u8, `1 <= count <= len`.
- Semantic invariant: prior consumed references eventually equal `value`.
- Mechanical inputs: counter/index equality, length bounds, explicit early-break
  final prefix assertion. No snapshots/theorem calls/proof blocks in this probe.
- New library proofs are manual reusable proof work, not inferred semantics.
- Existing trusted external specifications: slice iter_mut and its external type;
  Iterator::next; array/slice indexing and views; Verus mutable-reference prophecy
  semantics; existing Seq axioms. Rust lifetime/trait checking remains enabled.
- `examples/iterators/slice_iter_mut.rs` provides a checked model implementation;
  its custom prophecy implementation does not automatically strengthen the
  standard library contract. Neither it nor the external contract is edited.

## Replays

`probe.py` creates a new immutable candidate directory (fails if the ID exists),
freezes its source before running, guards the tool bundle and any extra inputs,
and writes raw commands/logs/receipts under the supplied private output. Every
ordinary trial explicitly requires zero inferred helpers and checks all named
required obligations for any reported helper; frontend/resource failures cannot
stand in for semantic negatives. It uses shared infer_experiment/eval support
with explicit runtime gates that also work under Python -O.

```sh
python3 -B -O source/tools/research/mutable_buffer_effects/probe.py \
  --source source/tools/research/mutable_buffer_effects/candidates/baseline-prefix-v1/probe.rs \
  --id fresh-prefix-replay --verus /private/bundle/verus --solver /private/bundle/z3 \
  --output /private/fresh-prefix-replay
```

Raw evidence and the immutable copied accepted bundle live under
`/tmp/mutable-buffer-effects-20261109/`. `baseline.json` binds artifact hashes
and both first receipts. This milestone is partial mutable-effects progress;
it is not completion of the canonical or transfer goal.
