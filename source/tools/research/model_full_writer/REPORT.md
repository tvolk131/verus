# Initialized SC model writer

`writer.initialized_shared_xor_writer` proves the concrete model writer's final
pad, key and outside-frame property. Its premises are byte ranges, initialized
pad/key lookups and disjointness of the accessed intervals. Ownership, reader
tokens, guards and proof credits are constructed internally. No physical
transformation WP or final-memory observation remains a client premise.

The program is the existing `zip_transform byte_xor` for
`min(padlen,keylen)` iterations, inside the checked child/original-parent borrow
protocol and `pair_setup`. All finite lengths are covered, including a 128-byte
pad and empty or longer keys. Disjoint intervals may share an allocation; an
empty key imposes no accessed-cell separation requirement.

The physical callback's input explicitly contains Reader, Cap, linear R and
loop credits. It returns R. G, its original closing rule, outside fragments and
key-observation credits remain outside mutation. The checked callback changes
the child bytes, closes the same child and recovers/releases the original pad
parent. Joining R/G recovers the key context before final observation.

The postcondition is an observer closure constructed from actual resources.
It accepts the final heap context and derives the exact XOR prefix, untouched
suffix, unchanged complete key and outside frame in that heap. Auxiliary
non-atomic/lock bookkeeping instances are universally quantified. The checked
`heap_ctx_for_observation` retains the actual map/freeability authority and
discards only bookkeeping, observing `(sigma.1,true)` with the identical map.
The returned pure property concerns the original sigma, not an assumed state.
Shared key read counts may be nonzero.

The sufficient internally supplied budget is `3+2*min(padlen,keylen)+keylen`:
two setup credits, one payload-opening credit, two per SC iteration and one per
key byte observed. This is neither a minimality nor a source-runtime bound.

## Validation

Trial 008 passed normal compilation of both modules, individual assumptions
reports for all 12 source declarations and full recursive `coqchk`. Ten
declarations are closed; the two composed execution theorems inherit only
dependent functional extensionality and `Eq_rect_eq`. The checked generated
assumptions module contains the marker proofs. No new constructor, program
obligation, axiom, unchecked primitive or unsafe checking mode was introduced.

The accepted [freeze](trial008-freeze.json) binds all candidate/controller and
shared support sources. [Results](results.json) bind the durable parent and
proof receipts, source/object identities and individual assumptions. The replay
authenticates 13 completed model libraries, the 523-source/284-object foundation,
both readiness and its successful build receipt, compiler/checker/runner and
the pinned GMP manifest; input and product guards remain active through success.
Raw logs and objects remain private.

Trials 001–007 are retained proof-development failures, not semantic negative
evidence. They stopped on type annotations, Hoare continuation/resource framing
or the final observer's bookkeeping-instance mismatch. Trial 008 preserves the
runtime and semantic post from the initial prototype; it generalizes the ghost
observer to accept the actual final heap context.

```sh
python3 -B -O source/tools/research/model_full_writer/check.py \
  --freeze source/tools/research/model_full_writer/trial008-freeze.json \
  --output /tmp/model-full-writer-replay-UNUSED
```

The output directory must be unused. Independent acceptance of this exact SC
composition remains pending. The separate ordinary-memory composition is in
[model_ordinary_full_writer](../model_ordinary_full_writer/REPORT.md).

## Boundary

This is MODEL partial-correctness adequacy with explicit setup/borrow markers.
It does not identify actual incoming Rust references, provenance, ancestor
loans, native iterator operations, retags, cleanup phases or original C/F
observations. SC operations are not silently native ordinary operations. The
separate caller-context law, not this fresh initialization, preserves an
already existing model caller. No total termination or SC/Na concurrent
equivalence is claimed. Native correspondences remain zero; 44 minimal plus
three phase simulations and all 101 writer effects remain open. Actual full
HMAC remains 1/1 unproved; publication and goal completion remain false.
