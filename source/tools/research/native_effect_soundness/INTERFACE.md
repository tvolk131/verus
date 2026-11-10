# Proposed conditional carrier interface (frozen before trials)

This proposal supplies no native fact. `native_bridge_proved=false` permanently
in this lane. Entry/exit relation to the actual executable borrow belongs to
native_borrow_entry; it must be proved, never inferred from observations.

`Entry` is a ghost descriptor: arbitrary 128 initial bytes, arbitrary key bytes
(full length, not truncated), per-slot provenance-bearing pointers, and actual
slice base/extent metadata. Its validity predicate explicitly includes byte
stride, bounds, provenance and mutable/key address separation. All are
conditional input obligations, not new application preconditions.

Proposed entry signature:

```
proof fn accept_owner(e: Entry,
    tracked pad: Map<int, PointsTo<u8>>) -> (tracked c: Carrier)
requires e.valid(), full_owner(pad,e)
ensures c.wf(), c.entry == e, c.physical.index == 0
```

The sibling should supply the *existing* pad tokens plus shared access to
`Map<int,PointsTo<u8>>` for indices `0..min(128,key.len())` with values equal to
`e.key`. Full key metadata remains in `Entry`. No token constructor is proposed.
It must additionally connect the tokens and their eventual return to the SAME
executable array/slice borrows. Returning a map is not that native exit theorem.

`Carrier` owns retained slots, returned slots, and pending loan receipts. The
physical descriptor contains fixed inner origins/extents, native Zip index and
min extent. Each successful abstract transition removes exactly index from
retained, returns its linear `Loan`, records pending, and increments only index.
Returned loans restore permissions; pending receipts never count as ownership.
Logical projection uses index; unchanged physical inner pointers are NOT the
remaining logical sequences.

Minimal first proof candidate: accept_owner; issue one slot preserving every
other retained token; None is identity at min extent (including empty key).
Then: resolve arbitrary issued loans; reconstruct all 128 slot permissions;
derive unchanged tail from retained values; conditional key reads and mutable
loan use through inherited ptr_ref/ptr_mut_ref. No required XOR postcondition is
an input invariant. No unchecked creation, assume, external body or new axiom.

Native source gate: consume the existing schema-2 typed evidence and source
binding from native_iterator_certificate; inventory EACH statement, terminator,
callee, drop query and exceptional exit. Every correspondence obligation stays
unproved unless a checked lowering argument exists. Source/hash/shape matches
are diagnostic, never a resource theorem. Unknown or unavailable effects reject.
Current minimal missing source interface: an ordinary-checkable interpretation
of actual foreign MIR pointer/retag/call/unwind operations into this carrier.
The ordinary frontend does not accept foreign MIR as local checked Rust bodies.
