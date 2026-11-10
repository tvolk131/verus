# Proposed native-body adequacy judgment (not implemented or admitted)

No rule here is installed. In particular there is no new `assume` next to
ast_to_sst's FnMut preservation rule. No certificate made only of JSON status,
operation hashes or source names can discharge this proposal.

## Exact existing source sequent

Fresh `native-drop` SST/AIR shows (A is the reborrow of original B0, S the
array-to-slice result, I the actual native constructor result):

```
C(A) = C(B0)
B1 = update_current(B0, F(A))
has_resolved(B1)
ens_array_unsize(A, S)
ens_iter_mut(S, I)
has_resolved(I)
-------------------------------- required but unproved
F(B0) = C(B0)
```

Existing update-current and owned-reference resolution imply `F(B0)=F(A)`.
Inherited unsizing gives `C(S)@=C(A)@` and `F(S)@=F(A)@`. The abstract native
constructor contract maps each `remaining(I)[i]` current/future to S's
current/future. The missing native fact for THIS nonmutated constructor result
is that the actual pending referents undergo no later change when I is abandoned.
`has_resolved(I)` is already emitted; inserting it again cannot help. A law
resolving arbitrary observers from their owner is invalid (`owner-not-referent`).
A new native resolution axiom would merely assume the requested effect.

`constructor-summary` checks the total abstract-summary connection for every
original `&mut [u8;128]`, without a permission argument or second reference.
`expose-exit-v2` checks that equality of pending references' current/future
values would suffice for the original-array observation. Both rely on the
inherited assumed native constructor summary. Neither verifies its native body.

## New proof obligations required before lowering the native body

An implementation must supply an adequate *source execution judgment*, beyond
existing value-only VIR. Proposed internal boundary:

```
ValidSourceRef(runtime_state, operand, rust_type, source_place, source_loan)
  => exists containing_allocation, subobject, authority.
       RefInterpretation(runtime_state, operand, vir_ref,
                         containing_allocation, subobject, source_loan, authority)

RefInterpretation(s0, actual_operand, r, allocation, subobject, loan, authority)
  and CheckedNativeBody(actual_typed_MIR, RustPrimitiveSemantics, simulation)
  and SourceExec(actual_typed_MIR, s0, s1, result, exit_kind)
  => exists authority1. RefInterpretationAfter(s1, result, r,
                                               allocation, subobject, loan, authority1)

RefInterpretationAfter(...) and CheckedOwnedValueRelease(result, authority1)
  => Observation(s1, original_subobject) = F(r)
```

These are semantic judgments to prove, NOT new uninterpreted predicates usable
with `requires`. The universal input quantification is every valid original
Rust reference, including a strict subobject of a larger allocation. Input Rust
validity is already a source-language obligation. Turning it into a new user
precondition, assumed PointsTo map, spec choice or Ghost capability is forbidden.

The first judgment must be tied to the actual frontend reborrow node/borrowck
place/loan and its existing C/F equations. It must not construct a second
executable or tracked `&mut` alongside the consumed native operand. Its linear
interpretation belongs to the proof of the compiler translation and cannot be
copied by copying spec observations. Borrowck `loan_issued_at` authenticates an
operand's static loan, not a live allocation, byte heap, prophecy or permission.

The second judgment must give actual primitive semantics for the constructor's
`RawPtr`, metadata, pointer unsize/thin casts, transparent NonNull/transmute,
constant non-ZST branch, in-allocation byte Offset to one-past, aggregate, move,
retag and return, and each native call/drop/exception path. Prove preservation
of provenance/validity and permission, not only of numeric offsets. Preserve
transparent valid-range constraints, not merely layout size/offset equality.
No one-past pointer dereference is eligible. CTFE-unreachable paths still need
checked exclusion; cleanup may reject the fragment but cannot disappear.

The last judgment must connect the final bytes to F(r) of the SAME incoming
reference. Owned-value resolution alone does not imply referent resolution.
Empty drop glue checks a runtime effect; it does not create the loan release or
prophecy equation. Return/escape, call return, normal return versus panic and
logical release must compose without resolving returned or outstanding loans.

## Eligibility and integration sites for review

- Frontend: attach an internal original-place/loan witness to the actual
  `ExprX::BorrowMut` and unsizing call operand, with a checked source judgment.
  Existing rust_to_vir translates HIR, not foreign MIR. A native body importer
  needs an explicit typed MIR-to-VIR simulation theorem; Python matching is not
  it. Unknown raw effects and unavailable helper bodies reject.
- Backend: preserve `borrow_mut_to_sst`'s existing phase-1/phase-2 equations.
  Native effects must produce ordinarily checked obligations on those actual
  locals, not assert a new final/current equality after recognizing a name.
- Resolution: justify a native projection only with proved reachability of
  owned native resources and release. Structural resolution currently follows
  visible non-spec fields of destructor-free datatypes, not opaque observers.
- Memory: the current MutRef C/F encoding contains no allocation/provenance.
  `vstd::mut_ref::MutRef::unpack(r).ptr` is not a runtime pointer bridge:
  `pack-null-views-v2` proves a spec-only null representation with the same C/F
  observations. No physical reference is created by that proof.

For full Zip, carry all 128 retained bytes and issued loans, the immutable key's
actual extent and read authority and specialized Zip.index. Permit disjoint
pad/key byte ranges within one allocation and every key length (0 and >128).
Prove getter Offset/raw-reference retags, callback XOR store frame, None,
drop/cleanup, pointer-distance intrinsic and each helper before claiming the
frozen writer. Constructor/drop adequacy alone cannot discharge these effects.

## What is blocked now

Existing trusted reference semantics supplies the abstract C/F sequent above.
It supplies no expressible adequate source memory judgment or reference-to-raw
permission conversion. Native `NonNull::from_mut`, raw borrow, reference-to-raw
cast and dereference probes reject before SMT. The concrete failed-positive
`unpack-nonnull` and positive `pack-null-views-v2` show why changing representation
names cannot fill that semantic gap. The existing 115/0 authored interpreter
proves no source adequacy metatheorem for these primitives.

Therefore no implementation of a native publication hook is justified by this
lane. A reviewed mechanized source/ref/primitive simulation is necessary; adding
an unproved relation as a trusted predicate would enlarge the TCB and violate
the goal. Publication remains false, native-drop 0/1, full writer 1/1.

## Concrete borrowck phase evidence inspected

The inherited guarded six-instance extraction's actual pre-erasure local facts
identify exclusive loan 0 / region 3, source `(*_1)` -> reborrow `_4` at
`bb0[3]`, activation at `bb0[4]`, move-unsize `_4` -> `_3` at `bb0[4]`, then
the native constructor consumes `move _3` into `_2`. `loan_issued_at` is
`[3,0,7]`. The ordinary original provider completes analysis. This is one
source loan, not two owners. Static region IDs do not identify runtime
allocation/provenance or give F(r) a byte-heap interpretation.

The borrowck-phase body also contains FakeRead and an unwind cleanup block
ending in unwind_resume. These differ from the six-instance monomorphized
operation ledger. The lowering must justify its MIR phase correspondence and
normal-return/non-unwind proof rather than silently applying the selected
49-step authored trace to that earlier body. No such adequacy is inferred
from the equality of source hashes or from the existing 115/0 result.
