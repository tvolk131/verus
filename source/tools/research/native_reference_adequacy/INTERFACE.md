# Original-reference adequacy boundary (early interface)

Only this new directory is owned. Native publication is false. No new API,
axiom, production hook, runtime replacement, or application requires is installed.
The original native-drop and frozen full writer remain the target diagnostics.

The compiler already lowers `&mut T` to a typed value with current/future
projections. Reborrow phase 1 equates current with the source place; phase 2
updates that place to the new reference's future. Resolution equates current
and future of the **resolved reference value**. It does not follow arbitrary
spec observers or raw pointers. Array-to-slice coercion has an inherited axiom
preserving both views. All of these are existing TCB, not new native facts.

Smallest probes will distinguish:
1. Local reference reborrow/coercion and original-array final observation.
2. The unchanged native constructor/drop postcondition.
3. Obtaining the native constructor's raw pointer from that same reference.
4. Resolution of opaque IterMut versus its prophetic remaining references.

Needed extension, NOT a declaration usable as a proof:
- Interpret the single source reborrow operand in the existing reference
  encoding; carry original place/current/future identity through native calls.
- Derive from valid input reference the live containing allocation and subobject
  bounds, initialized bytes, provenance and the one exclusive loan. No second
  live reference, supplied token, ghost map, or spec choice supplies this.
- Check each actual native instruction/call/branch/return/drop/cleanup against
  that interpretation. The earlier 44-op model trace is a read-only candidate;
  its bytes/custodian are not source authority.
- Show that releasing that loan identifies actual final(original array) with
  the resulting bytes. No-write in a chosen semantics alone does not do this.

The full writer must retain all key lengths and same-allocation disjoint ranges.
Do not propagate a frame from this diagnostic until native simulation and output
adequacy are proved and independently reviewed. If the primitive link cannot be
derived, retain executable failing probes and propose the exact core obligation,
with publication false. No conditional interpreter is planned here.
