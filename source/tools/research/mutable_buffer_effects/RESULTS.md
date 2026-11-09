# Mutable-effects boundary: consumed prefix succeeds; unused tail is opaque

This workstream supplies reproducible ordinary probes and a checked concrete-model
law. It does **not** complete the canonical/transfer goal, add a mutable-effect
summary, or change any production specification. After integration reported that
the canonical proof already succeeds with existing `*final(it.seq()[j])`, the
unused proposed `vstd::mutable_buffer` module was withdrawn before any trial.
Its untested draft is retained as `candidates/library-v1`, not registered or
claimed as checked/shared-law use. Production vstd is unchanged from bb1e46b3.

The concrete transfer client is the separately frozen bitcoin_hashes 0.14.0
HMAC ipad loop at integration commits 7fe03428/d99ab886. Its arbitrary initialized
128-byte buffer and arbitrary key length, including >128, require XOR on the
truncated prefix **and unchanged suffix**. This report does not weaken that
property or change its body/domain. No claim is made that either accepted
application uses the laws/probes below.

## Exact outcomes and model boundary

Each source was frozen before its first trial. Successful trials use full
ordinary checking, normal lifetime/trait checks and `--no-cheating`. No helpers
are inferred: every case explicitly requires the exact empty helper identity set
and then checks all named soundness/maximality/exactness obligations for any
reported helper. No inferred helper/cache warmth is claimed.

| Frozen probe | Verified/errors | Meaning |
| --- | --- | --- |
| baseline-prefix-v1 | 2/0 | Arbitrary u8 value, arbitrary slice/valid nonempty prefix, break before counter increment |
| baseline-xor-v1 | 4/0 | Read/modify/write prefix and natural exhaustion, arbitrary contents/mask/length, including empty exhaustion |
| baseline-return-v1 | 2/0 | Original exclusive u16 slice returned as the written prefix, arbitrary values/lengths |
| iterator-resolution-v1 | 2/0 | `has_resolved(it)` holds for unused native IterMut and native Zip |
| baseline-unused-frame-v1 | 0/1 postcondition | Even unused native IterMut cannot prove unchanged original slice |
| empty-zip-v1 | 1/1 postcondition | Empty-key native Zip writes nothing but cannot prove unchanged original slice |
| resolved-frame-v1 | 0/1 postcondition | Typed native IterMut resolution-to-remaining law is missing |
| zip-resolution-v1 | 0/1 postcondition | Typed native Zip resolution-to-inner law is missing |
| concrete-model-v3 | 11/0 | Original checked example implementation, byte-for-byte unchanged, plus generic remaining-frame law |
| direct-frame-v1 | 2/0 | Direct reborrow and one arbitrary indexed write prove full untouched-state frames |
| wrong-index-v1 | 2/1 assertion | A good prefix writer does not justify claiming the next slot was written |
| wrong-referent-v1 | 2/1 assertion | A good writer does not justify a claim about another immutable slice |
| wrong-untouched-v1 | 0/1 postcondition | An arbitrary indexed write does not justify whole-slice unchanged state |
| zip-prefix-v1 | 1/1 postcondition | Bare processed-prefix invariant lacks representation transport; rejected proposal |
| zip-prefix-v2 | 2/0 | Native Zip consumed-prefix XOR, arbitrary slice/key lengths, with explicit mechanical snapshots/representation invariants |

The model-gap probes are true Rust properties that this abstraction does not
prove. They are **not** false-property semantic controls. The wrong-* cases are
ordinary semantic negatives, retain the same healthy zero-helper checks, and
exclude mixed postcondition/precondition/assertion failure kinds. Missing-law
attempts and the bare-prefix bridge proposal are separate rejected auxiliary
proofs. No poisoned-helper control is needed or claimed for this zero-helper
workstream; integration owns inference acceptance/cache controls.

`results.json` records each initial receipt, excluded failures, and the matching
independent compiler/vstd replay with and without `--infer-goals`. Raw logs,
commands, tool guards and complete function reports remain in private persistent
`/tmp/mutable-buffer-effects-20261109/`.

## Minimal typed statements

These **fail** with the existing native contracts; they are never assumptions
or callable checked summaries in accepted code:

```rust
proof fn remaining_resolved<'a,T>(it: core::slice::IterMut<'a,T>)
    requires has_resolved(it),
    ensures forall|j:int| 0<=j<it.remaining().len() ==>
        *final(it.remaining()[j]) == *it.remaining()[j];

proof fn zip_inner_resolved<'a,T>(
    it: core::iter::Zip<core::slice::IterMut<'a,T>,core::slice::Iter<'a,T>>,
)
    requires has_resolved(it),
    ensures has_resolved(zip_iter_fst(it));
```

The checked theorem in `concrete-model-v3` has the first conclusion for the
**concrete** `SliceIterMut<'a,T>` example, with premises `has_resolved(it)` and
its existing `it.slice_iter_mut_wf()` type invariant. It proves resolution of
`it.slice`, uses that slice's current/future equality, instantiates the existing
constrained-prophecy predicate, and transports through the existing invariant.
The copied implementation is unchanged and uses no added axioms or admissions.
This demonstrates the mathematical frame on the concrete model. There is no
checked refinement bridge mapping that theorem to opaque std IterMut.

For native IterMut, `iter_mut` exposes only original length plus pointwise
initial/current and final/eventual correspondence. `next` removes a head and
preserves the reference-identity tail. Thus consumed final values can be written
into an inductive prefix invariant and transferred back to the original buffer.
Unyielded eventual values remain unconstrained: a one-element initial slice 0,
future slice 1, and a not-yet-resolved remaining reference with current 0/future 1
satisfy the exposed correspondence equations. Resolving the **opaque iterator**
does not resolve the reference in those equations. This is an API-model
counterinterpretation, not an execution of Rust.

The same issue occurs independently at native Zip's hidden-inner boundary.
`zip_iter_fst`/`zip_iter_snd` are uninterpreted accessors; the constructor relates
them to its inputs and exposes only a truncated remaining sequence. Iterator
termination/will-return-none and exhausted Zip do not assert exhaustion of the
longer input. Even a future IterMut remaining-frame rule alone would still need
checked Zip/state resolution transport for the real target. Neither failed typed
statement is asserted sufficient to finish that target on its own.

`source/vir/src/resolve_axioms.rs::resolve_datatype_axiom` explicitly emits field
resolution rules only for visible transparent, non-destructor datatypes, skipping
spec-mode fields. `source/vir/src/resolution_types.rs` conservatively marks
opaque types potentially unresolved. Thus this is not a delayed lifetime close:
iterator-resolution-v1 already proves object resolution. New checked sequence
lemmas cannot reveal the hidden native mutable slice or Zip inner field. A native
refinement/resolution contract would need separate justified work and is outside
the instruction to leave trusted external specs unchanged.

## Authored work and inherited trust audit

- Baseline prefix inputs: semantic value/XOR predicates, processed-prefix
  invariant, iterator-index equality, explicit bounds and early-break ensures.
  Returned-prefix probe adds the final counter equation. No new application
  assumption, zero-buffer condition or numeric restriction is used.
- Zip passing prefix diagnostic: two ghost snapshots (`initial = buf@`,
  `eventual = final(buf)@`), min-length equality, and pointwise initial/key/final
  representation invariants. These are manual mechanical work, visibly present
  in the source; they are not semantic inputs or automation removal. The bare
  proposal was frozen and retained before these were added.
- Concrete-model law: manual proof instantiations, explicit arithmetic bounds
  and predicate equalities over the example's existing model. Its wf premise is
  the existing type invariant, not a new application precondition.
- Inherited contracts: `std_specs/slice.rs` external IterMut type and iter_mut;
  `std_specs/iter.rs` Iterator::next/zip, native Zip type/accessors/postcondition;
  slice/array indexing/views and existing mutable-reference/Seq/prophecy axioms.
  The checked VerusForLoopWrapper keeps stable snapshot/reference history and
  proves its own next implementation. The user-visible named `it` inside a
  for-body is the **pre-next ghost snapshot**, so its index identifies the
  current reference; the internal wrapper has already advanced one position.
- No existing slice/iterator module or external specification was modified. No
  engine, registry, allowlist, kernel, cache/schema/policy or sibling edits.
  No assumed summaries, disabled-check certificates, custom drop axioms or
  blanket final==current bridges.
- The independent compiler/macros/vstd build uses Rust 1.99.1, two build jobs,
  private copied Z3 4.16.0, and the normal cargo-verus build: 2,226/0 vstd.
  Forwarding `--no-cheating` into vstd's **own source build** first rejected 83
  inherited admissions; that failed build is separately recorded, not accepted.
  The normal project vstd build retains those existing library trust boundaries.
  Every new proof/application probe still passes or rejects with `--no-cheating`.

## Retained failures and next dependency

`concrete-model-v1` was a frontend mode error from passing a spec parameter to
`use_type_invariant`; excluded from semantic evidence. `concrete-model-v2` checked
10 other queries but its new assertion failed until explicit existing-model
instantiations were supplied in v3. Neither is accepted as a checked law. The
initial expected-success `zip-prefix-v1` failed its postcondition; it is replayed
as a deliberately retained failed proposal, never a semantic mutant. The unused
library-v1 draft was never tested or integrated.

First remaining transfer blocker: a justified native mutable-iterator unused-tail
frame and its transport through Zip, unavailable in the unchanged external model.
The parent must reconcile this with the frozen unchanged-model requirement;
this workstream supplies the minimal failures rather than silently changing the
model or property. No sibling dependency prevented the independent probes and
checked concrete-model result. Canonical success reported by the parent does
not depend on this missing suffix frame.

## Replay

```sh
python3 -B -O source/tools/research/mutable_buffer_effects/replay.py \
  --verus source/target-verus/release/verus \
  --solver /tmp/mutable-buffer-effects-20261109/baseline/z3 \
  --output /private/fresh-mutable-effects
```

This replays 30 exact expected outcomes (15 cases, ordinary/goals) with source
hash checks, artifact guards, explicit helper coverage, named required proofs,
full-crate checks and isolated semantic failure kinds. Output must be fresh.
To author a new probe, use `probe.py` with a fresh candidate ID: its source and
expected domain/property inputs are frozen before the compiler is invoked.
