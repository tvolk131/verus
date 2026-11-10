# Reference carrier versus referent: source cut

The additive trial004 queries the **actual parameter** BackendRepr, separately
from its pointee layout. On the pinned64-bit target, canonical's `&mut [u8;10]`
and HMAC's `&mut [u8;128]` are Scalar,8 bytes, alignment8. HMAC's `&[u8]` is
ScalarPair,16 bytes, alignment8; its pointee remains unsized. These are compiler
facts about these exact function inputs, not physical reference observations.

The pinned rustc `interpret/place.rs` source has the following ordered cuts:

1. `copy_op_allow_transmute` invokes `copy_op_inner(..., true)`.
2. `copy_op_inner` first runs `copy_op_no_validate`, then destination validation.
   When source/destination types differ it validates at both layouts. The
   same-type case avoids the first redundant validation, but actual caller
   layout/type compatibility still needs its own source justification.
3. `copy_op_no_validate` declares Scalar layouts padding-free. It also declares
   reference/raw-pointer ScalarPair layouts padding-free. This selects
   `read_immediate_raw` rather than the padding-preserving operand path; it does
   not prove that the carrier read succeeds or which source state it returns.
4. A returned immediate is written by `write_immediate_no_validate` or its
   memory-place variant. The direct-local branch stores the immediate **before**
   the fallible `after_local_write` hook. Memory-place writes may invoke bounds,
   permission/race/allocation checks, and the wide-pointer branch writes its
   components in order. None is an atomic pure reference assignment theorem.
5. `pass_argument` subsequently protects an in-place source carrier. Miri's
   `protect_in_place_function_argument` retags that place, then `write_uninit`s
   it. This is a real carrier mutation, distinct from retagging its referent.
   `write_uninit` itself may call a fallible local-write hook after changing it.
   Return-place protection and remaining entry hooks also remain to simulate.

Source identities: the existing native_source_reborrow pins bind
`interpret/place.rs`, and native_miri_policy_pin binds Miri machine.rs; the
entry review preserves the exact call.rs source identity in immutable Git.
These source observations are not a translation correctness proof.

The missing frame obligation must relate the actual source/carrier destination
and source in-place spans to the SAME original array, its initialized bytes,
incoming provenance and suspended ancestor/protector context. Required separation
must be derived from the actual valid source context, not appended as an
application precondition. Different allocation IDs cannot simply be assumed:
lawful same-allocation disjoint interiors must remain covered. Errors after
partial mutation and normal/unwind/cleanup paths cannot be silently erased.

The compiler-layout result justifies a type-directed branch classification. It
does not establish that whole copy operation, runtime separation, current/future
bindings, source safety or any native bridge. Native=0;44+3/101 remain open;
full HMAC1/1 remains unproved;publication=false;goal_complete=false.
