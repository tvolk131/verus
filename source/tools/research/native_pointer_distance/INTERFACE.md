# Pointer-distance / size-hint conditional interface

Owned scope: this new directory only. Baseline 40a8e281. Native=0,
44+3 minimal/phase and 101 full effects open; publication=false; goal incomplete.
This lane proves arithmetic and explicit operational fragments, never source
validity or lawful-history erasure. No application requires are introduced.

## Minimal input needed from the source-entry / constructor integration

Use NativeMirLrust.fragment's checked `window`, `pointer`, `slice_valid`,
`shifted`, `pointer_loc`, `pointer_address`, `offset_valid` and numeric bounds.
Supply a relation from the ACTUAL typed iterator field operands at size_hint
entry to either (a) `slice_valid p n`, end=`shifted p n`, same allocation,
or (b) n=0, start=end, a nonnull aligned 64-bit address with no allocation or
readable-cell premise. Case (b) includes dangling empty slices and q=p.
The relation must be derived from lawful entry, not required from clients.
It must justify transparent NonNull/raw casts, source validity and provenance
through the actual locals. This lane does not define that missing source law.

For pad use n=128 at arbitrary cursor, including strict containing-allocation
interior; for key use every valid n, including 0 and >128. Pad and key may
share a containing allocation, subject to original lawful references.
No pointer-distance operation reads bytes or grants physical authority.

## Output and precise remaining dependency

Planned normal Coq results: finite 64-bit/isize bounds; legal one-past Offset
formation via existing laws; end>=start; unsigned modular subtraction equals n;
equal dangling endpoints yield 0; branch exclusions for u8 CTFE, optional
UbChecks/address-order check and sizeof assertions; exact tuple and .0 size
projection, and min(128,n) with unchanged inner pointer fields. Operational
lemmas use existing lrust scalar reductions; unbounded Offset alone is never
identified with native finite-width pointer arithmetic.

The intrinsic's compiler/core meaning is to be documented against pinned source.
If no existing checked native compiler preservation theorem is available, the
source-to-fragment correspondence remains explicitly rejected. A new intrinsic
axiom, assumed no-unwind bit, future-defined-run restriction or identity allowlist
cannot fill it. Cleanup is retained in the structured inventory; only a proven
concrete failure-branch exclusion can remove a reachable panic obligation.

Root integration needs: (1) actual lawful source operands -> above window/equal
endpoint relation, (2) intrinsic/cast/source-machine interpretation -> proved
finite arithmetic fragment, (3) whole-call simulation with retained cleanup and
source observations. These are proof obligations, not exported assumptions.
