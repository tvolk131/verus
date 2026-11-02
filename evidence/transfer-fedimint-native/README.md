# Byte-preserved native Fedimint deadline transfer

This is a new transfer, not a revision of the frozen gateway, early-return,
redb or integration evidence. The starting engine is
`55bd67ad980e90e92c77efe4453a927f097c3386` **plus the integrated uncommitted
snapshot**. All 22 snapshot file hashes matched before this work. No branch,
commit, PR, application edit, subdelegate or shared mutable build was created.

## Source fidelity and intent

The public application pin is
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`. The runner checks the complete
lightning/client source hashes, the literal tuple constant, the complete impl
hash and each complete method item hash. The original `incoming_circuit` and
`lnv2_blocks_to_claim_deadline` items, including their signatures and bodies,
are byte-preserved. The `(0, 0)` tuple constant is unchanged. Infer attributes
are inserted outside those items. There are **no application desugarings** of
`then_some`, `map_or`, `is_some` or tuple comparisons.

The request is projected to its three read fields; unrelated fields, serde
derives, surrounding modules and dependencies are omitted. Items are wrapped
in `verus!`. This is an extracted native-helper proof, not a full-crate build.

The ordinary authored `deadline_policy` has the same three postconditions as
the earlier transfer, and **no precondition**. Its domain is all u64 channel
identifiers, all u64 HTLC identifiers, all u32 expiries, and None or every u32
current height. It proves intercepted-forward exclusion, unknown-height/known
height safety, and the two-block coverage/non-vacuity implication. The margin
2 is checked against the pinned client source. The implementation relations
are inferred; the intended policy is not inferred.

## Reusable implementation support

`source/rust_verify/src/fn_call_to_vir.rs` recognizes only the canonical core
inherent bool `then_some` and Option `map_or` identities, including their
inherent self types and argument/type-argument arities. It maps them to
diagnostic-registered, identity-checked `vstd::native_option` definitions.
These definitions are ordinary checked functions, not assumed specifications:
`then_some` constructs an Option; `map_or` composes the existing specified
Option `map` and `unwrap_or`. Original Rust HIR calls remain the erased native
execution. Their all-exec argument/return modes come from the matching bridge.

The readonly-record interpreter evaluates receiver and ordinary arguments
eagerly, in order. A literal `map_or` callback is interpreted independently
over a symbolic payload and immutable captures, and its requirements are
applied only on the Some path. Generated callback requirements/equations use
the existing owned-callback decorator and independent SST precision packet.
The original closure body and binders cannot change. Soundness checks the
original callback normally; maximality independently proves its complete safe
domain and its equation's independent soundness/exactness. The wrapper-model
lifting boundary restores this checked decoration instead of dropping it.
Only exact readonly candidates receive the existing two-sided closure-domain
fact. The necessary-domain checker recognizes only the closed canonical
`map_or` bridge for its callback requirement; arbitrary callee requirements
are not assumed. Adversarial partial callbacks exercise these gates, and
sound-only record candidates cannot receive these exact-domain facts.

The existing Option `is_some` specification is recognized only with canonical
core identity, exact vstd proxy, modes, arity and signature. Constant support
is restricted to visible literal scalar/tuple bodies with matching types;
opaque constants, calls and other initializer forms fail closed. Source and
formula budgets remain bounded. Handwritten callback contracts, nontrivial
callback-construction blocks, closure returns/nesting, effects and unsupported
value types are rejected rather than silently approximated.
This addition belongs to the existing readonly-record profile. Transparent
single-scalar wrappers projected into the separate scalar profile still reject
these idioms; a regression locks that fail-closed boundary. It is not a claim
of general Rust-combinator support in every inference profile.

Cache generation is v62 and interface evidence policy v56. The canonical models
and matching vstd artifact remain certificate dependencies. All three exact
checks and ordinary crate verification are required before publication;
negative controls seed a previously valid export and require its deletion.
Cold/warm regenerated record summaries honestly remain candidate-cache misses;
no proof-cache hit is claimed.

## Bounded follow-up

The same unchanged helpers also prove an ordinary zero-at-expiry boundary
property and height monotonicity. Neither property adds a precondition or
trust. Focused regressions distinguish eager `then_some(false, unsafe_value)`
and eager `map_or` defaults from lazy callbacks; a None callback does not impose
its arithmetic domain. Native Rust execution independently checks evaluation
order and overflow witnesses. Imported summaries retain their exact safety
domains and cannot publish after a failing caller obligation.

## Trust and limitations

No new axiom, `assume_specification`, external-body model or unchecked
application contract is introduced. The ordinary Verus backend, existing vstd
library specifications, Rust typing/erasure and the new narrowly bounded
canonical-API frontend correspondence are trusted. Checked bridge bodies do
not constitute a formal proof of libcore's implementation; the independent
native oracle is finite testing, not that missing universal correspondence
proof. This trust is explicit, not hidden in an application-specific summary.

The full gateway, callsite wiring, asynchronous operations, database atomicity,
cryptography, concurrency, recovery and stateful fund safety remain out of
scope. The next step is parent integration through the shared transfer front
door and review of this bounded frontend correspondence. A later stateful
obligation needs its own authored policy and frozen baseline; these helpers do
not establish whole-flow fund safety.

## Observed outcomes

The fresh replay has native cold/warm **8 verified, 0 errors**, with both
helpers exactly inferred and both inferred entry requirements true. Each run
performs fresh soundness, maximality and exactness checks, publishes a fresh
interface, and reports two honest regenerated-state cache misses. The boundary
and monotonicity follow-up has **10 verified, 0 errors** with those same two
helper bodies and six fresh required exact checks.

Deleting only the intercepted-forward early-return guard has **7 verified,
1 authored policy error**. Both mutant helpers still pass all exact checks;
there is no precondition/frontend/timeout error. Its seeded valid interface is
removed and no interface is published. The separately authored false goal has
the same 7/1 semantic outcome. The malformed goal is a frontend/type rejection,
not bug detection, and also removes its seeded stale export. Across the five
non-frontend cases there are **30 fresh required helper checks**.

The independent native oracle checks **2,112** boundary combinations, the
two-block non-vacuity witness at the u32 maximum, and the concrete guard-deletion
witness (fixed budget 0, mutant budget 10). Formal verification, not this finite
oracle, establishes the full scalar-domain policy.

Validation also passes **530 Rust tests**: 121 VIR units, 113 focused integration
tests (including nine native-Option cases), and 296 root inference tests. All
**10 Python tests** pass (six native runner tests and four unchanged earlier
transfer tests). The checked vstd build reports **2,221 verified, 0 errors**,
including the two new bridge functions. Subsequent incremental builds that
replay diagnostics without a verification count are not counted again.

## Replay

The new runner needs no historical worktree, baseline report or old receipt.
It prints its fresh output directory as the final stdout line, including on a
failed run; that directory contains `receipt.json`. The receipt separates
controller completion from application status through `transfer.status`.

```sh
python3 source/tools/check_fedimint_native.py \
  --application /path/to/clean/pinned/fedimint \
  --verus /path/to/matching/release/verus \
  --solver /path/to/z3 --output /private/tmp/native-replays --timeout 120
PYTHONPATH=source/tools python3 -m unittest source/tools/test_fedimint_native.py
```

Build/test commands, actual verification counts, fresh receipt location,
artifact/source hashes and retained development attempts are in `manifest.json`.
All compilation uses Rust 1.99.1, offline/locked Cargo, two build jobs and
isolated target **and build** directories. The seed is an APFS copy-on-write
copy of the integrated target, never a symlink to the parent's mutable target.
