# Narrow typed next dependency; not an implemented certificate

Decision: leave production unchanged. The request/profile gates are not the
only obstacle. Removing them would still leave the real carrier Ord call without
a numeric-order contract. Existing PartialOrdSpecImpl does not establish Ord,
and an Eq marker does not supply an ordering law.

## Exact earliest remaining sequent

For an arbitrary genuine `n: NonZero<u64>`, establish that the **actual loaded,
instantiated get MIR fragment** returns `n@`, under the existing ordinary get
contract boundary. The loaded optimized carrier cmp body contains two inlined
get fragments with NonZero-to-u64 transmutes. Their result-to-view correspondence
is not established by a layout match, a source span, an inlining label, or the
existing `NonZero::get` specification alone. No new law `transmute(n) == n@` has
been assumed or proved here. This is the earliest proposed MIR/ordinary-body
correspondence obligation, not a failed SMT query we have secretly discharged.

The actual ordinary query attempted is `carrier_ord::total`: with two arbitrary
borrowed genuine NonZero<u64> inputs and no explicit precondition, the result
of actual `a.cmp(b)` must equal numeric order on `a@` and `b@`. It fails 0/1.
The five direct partial-order carrier queries prove, so this is a specific Ord
boundary rather than an unsupported Rust expression or invalid carrier type.

MIR.json binds the exact resolved instances, full typed Body fingerprints and
private debug-body hashes. The closure is seven instances: two ordinary clients,
two original derived wrapper methods, instantiated carrier Ord, instantiated
get, and primitive u64 Ord. Every instance has available MIR; all call targets
resolve. Actual carrier cmp has two blocks/nine locals; actual get one block;
primitive cmp one block/five locals with a typed Cmp operation returning Ordering.
The sidecar only reads compiler MIR; it is neither a Verus compiler build nor a
certificate provider. Its fingerprints are diagnostic evidence, not proof tokens.

## Required bounded frontend work before enabling ordering

1. Resolve the real core Ord trait item and its NonZero<u64> Item instance by
   trait/type identity. Require exactly `(&NonZero<u64>, &NonZero<u64>) -> Ordering`,
   safe Rust ABI, no unresolved parameters. Check all typed locals, full CFG,
   cleanup/unwind paths, statements, returns, drops, constants and body properties,
   including unreachable blocks. The instantiated body still carries
   `is_polymorphic=true`; blindly admitting that flag is not a type proof.
2. Resolve actual NonZero<u64>::get and authenticate its full body. Prove a
   narrow inlined-subgraph correspondence to the **existing modeled get call**,
   including copies, shared reads, storage scopes and all exits. An ordinary
   translated consumer can then invoke that existing contract. Do not introduce
   a general transmute rule or an arbitrary view equivalence. If such factoring
   cannot be checked without expanding the TCB, stop for explicit design review.
3. Resolve primitive u64 Ord separately. Check its entire typed MIR and lower
   its exact Cmp operation to an ordinary typed numeric Ordering expression;
   prove its full-domain postcondition normally. Signedness, 64-bit width,
   Ordering variant identity and unsigned comparisons must be checked; no
   numeric enum-discriminant or signed subtraction shortcut.
4. Independently compare the resulting actual ordinary VIR bodies with the
   inspected MIR, not just with a recognizer descriptor or candidate contract.
   Check the carrier body under its total domain and existing get boundary.
   Keep ordinary WF/mode/lifetime/trait/termination/erasure checks. No trusted
   leaf, external-body shortcut or new assumed standard specification.

The present checked_foreign.rs is deliberately narrower: one bool input, one
u64 output, one block/two locals, one IntToInt cast. semantic_instance and
current_identity explicitly resolve core From<bool>u64. It has no rule for
NonZero transmutes, borrowed input observations, Cmp, Ordering, nested foreign
calls or the two-block carrier body. Generalizing a descriptor or relaxing those
checks does not implement steps 1–4. No such generalization was made.

## Certificate/import requirements for a future reviewed implementation

Use an explicitly typed separate ordering evidence kind, with no reinterpretation
of an old bool certificate. Bind all executed semantic instances and full MIR,
actual checked Functions, existing get/model dependencies, loaded core artifacts,
compiler/toolchain/executing compiler bytes and target settings. Both required
primitive and carrier ordinary proofs must complete, together with every auxiliary
query and erased Rust compilation, before export. A nonempty required dependency
set must survive private-helper pruning and re-export. Ordinary goals-off imports
must freshly resolve and inspect each required instance; root/worker .vir files
from any changed compiler are invalid positives. Leave policy/schema/cache epochs
to parent reconciliation; this worker changed none.

After that boundary is proved, extend only typed one-field nongeneric genuine
NonZero<u64> derives. Keep the seven original derives, private field and generated
runtime bodies. Ordinary derived cmp must establish numeric order on field views;
original partial_cmp must establish Some of that order. No inference theorem is
needed merely to recognize an enum return: current scalar ordering uses ordinary
body checks. Any inferred bool operator summary must still prove soundness,
full inherited-domain maximality and exactness freshly on cold and warm runs.
Candidate u64 projection remains candidate-only; no 61-bit restriction.

The intended numeric relation is explicitly authored: Less iff x<y, Equal iff
x==y, Greater otherwise, on u64 observations. It is not inferred from a type or
application name. Closed `value(self) = self.0@` stays closed. A future public
ordering/value compatibility lemma must be checked and explicitly counted, like
the existing ONE Eq/value broadcast lemma plus its client import. Do not export
representation bodies or insert a hidden premise to bypass closed views.

Required acceptance matrix, **not executed or claimed here**: genuine BorTag and
unrelated equivalent wrapper; actual < <= > >= /partial_cmp/cmp semantic clients;
1/equal/2^61/high-bit/MAX observations over the full symbolic NonZero domain;
false/swapped semantic goals rejected while helper obligations remain proved;
conservative unsupported widths/derives/shapes; opaque-field/closed-view ordinary
imports on the matching runtime; true miss/hit and fresh exact proofs; ordinary
candidate-poison/metadata/stale-output controls; failed/partial selected-export
removal; auxiliary-only failure with failed_functions=[]; late erased Rust
failure with no publication. Keep frontend/safety failures separate from semantic
negatives. No native test or conditional model can replace these proof gates.

No original full lookup theorem, native refinement, speedup, HMAC result or global
goal closure follows from this proposal. Independent parent review comes first.
