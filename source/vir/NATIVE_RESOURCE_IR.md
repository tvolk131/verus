# Experimental affine IR transport

`experimental-native-resource` is a default-off VIR crate feature. It exposes
the compiler-internal `native_resource::affine::instrument` constructor, not a
Rust/Verus source attribute or a native ownership admission policy.

The constructor replaces only an empty internal exec/unit fixture body. An
explicit hypothetical entry contract gives one abstract token to `Input` and
leaves `Output` empty. Its immutable descriptor has ten symbolic integer fields:
domain, activation, owner, owner initialization generation, allocation,
allocation generation, permission kind, interval start, interval end, and
authority epoch. These are identity components, not established native heap
facts or validated interval bounds. Slot identities use compiler-only variable
disambiguators; source spelling cannot create or assign the private live flags.

Transfer checks live source, all ten descriptor equalities, distinct slots and
empty destination before updating either flag. Use checks authority without
consuming it; consume clears a live flag. None of these operations denotes a
native read/write, Arc operation, Rust Drop, deallocation, or loan end. The
single descriptor is unchanged for the whole channel; transfer changes only
its location. An independent symbolic scalar is snapshotted and framed, not an
actual heap or byte frame. Every normal and early return checks the declared
token location (or explicitly empty result) and the scalar frame.

Events generate ordinary VIR declarations, assignments, assertions, conditionals
and returns. Normal simplification, SST elaboration, AIR lowering and SMT
submission handle those bodies. There is no parallel host resource oracle and
no success assumption or external-body resource axiom. The program vocabulary
cannot express calls, loops, borrows, allocation, cleanup or unwinding; arbitrary
nonempty source bodies are refused rather than certified by this constructor.

The compiler marker uses the reserved `experimental-native-resource-ir:`
namespace in the existing internal `infer_candidate_id` field. This avoids
changing the ordinary function-attribute serialization layout; ordinary Infer
candidate IDs are not resource markers. The marker survives lowering and
body removal and is also checked after serialization round-trip in a control.
Production Rust admission and VIR artifact import/export refuse any marked
crate before ordinary scope filtering/private-body removal. Thus even an
enabled compiler cannot publish a native theorem or ordinary value contract
from this experimental channel. Existing Infer barriers stay in force.

Targeted tests run with `cargo test --offline -p vir --features
experimental-native-resource --lib native_resource -- --nocapture`. They
construct internal VIR fixtures, not checked original Rust bodies. Positive
controls quantify over symbolic inputs and branch conditions; backend negatives
cover stale/duplicated uses, every descriptor component, self-transfer, path
joins and normal/early exit shape. The branch negatives check the exact failing
event span. With only one incoming token, a distinct live destination is
unreachable; the empty-destination guard is emitted but has no independent
reachable occupied-destination negative in this fragment.

This is a bounded conditional E1 transport fragment. E1 as a whole, E2–E6,
native Arc/Weak/glue/NLL correspondence, primitive-law adoption and all eighteen
redb database obligations remain open. No old replay allowance is replenished.

Independent review found no blocking soundness defect in supported
constructor-generated programs, with important evidence limits. The guard
controls exercise shared predicates, not actual bincode import/export or
filtered/no-verify/no-lifetime driver entrypoints. The lowering hook checks
body verification; it is not a universal firewall for arbitrary bodyless or
skipped internal declarations. Production marked-crate refusal is independent
of that hook. Only the main exec proof supplies the checked resource evidence;
recommends checks can lower assertions to assumptions and cannot substitute.
Localized backend rejection does not separately retain raw SMT SAT status.

Parent-authored follow-up controls add distinct-name equal/disequal descriptor
binding for Transfer/Use/Consume, including an ordinary input named `affine`,
and invalid mode/frame/condition/expected operands, private-name misuse and
excessive depth. These preserve the reviewed production semantics. They do not
establish two-body/activation independence or replace the missing real driver
boundary controls. The independent report predates these test-only additions.

## Later artifact-boundary hardening

The separately authorized consolidation adds real bincode import/export API
tests and an actual compiler-driver marked-import matrix. See
[`VERUS_INFER_STATUS.md`](../../VERUS_INFER_STATUS.md) for the replay command and
[`infer-consolidation95`](../../evidence/infer-consolidation95/README.md) for
results and qualifications. Import rejection becomes the span-free diagnostic
required by the driver instead of using serialized foreign spans. These tests
are not a marked-local-source producer, a universal internal-VIR firewall, a
full Cargo driver test or a native resource theorem. Historical independent
reports retain their original evidence limits.

A separately authorized [test-only follow-up96](../../evidence/infer-boundary-followup96/README.md)
adds paired driver positives, a leading unmarked sibling and inference-aware
malformed-trailer controls. Its separate independent review gives qualified
support with no blocking defect. Pairing covers consumer/options, not identical
marker-only artifact structure; stronger log-only observations and runner limits
remain documented. Its frozen inputs and review are retained separately from
hardening95. No production resource semantics or admission changes are part of
that test delta; full F1/native correspondence remains open.
