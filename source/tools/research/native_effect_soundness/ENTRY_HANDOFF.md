# Interface reconciliation with native_borrow_entry 98f428fa

Read-only inputs: its INTERFACE.md, REPORT.md, tracked-slice-v2/probe.rs,
heap-foundation-v5/probe.rs, simultaneous-owner-v2/probe.rs, and the
same-allocation caller. entry-evidence.json binds these committed blobs.

The checked entry API is:

```rust
fn enter<'a>(buf: &'a mut [u8;128]) -> Tracked<&'a mut [u8]>;
proof fn borrow_slot<'a>(tracked buf: &'a mut [u8], i:int) -> tracked &'a mut u8;
```

Its contracts connect all current/future array bytes and one selected future
slice update. This is useful executable-location authority. It exposes no
numeric pointer provenance, cannot write executable bytes from proof mode, and
cannot coexist with the original iter_mut borrow (E0499). It does NOT establish
`full_owner(Map<int,PointsTo<u8>>, Entry)` in this lane. No conversion is added.
The executable split carrier's 3/0 and pure heap calculus's 10/0 remain distinct.

The checked module here uses actual **non-Copy inherited PointsTo** values in
tracked maps. It moves, borrows, uses, returns and merges those values. In
particular World owns every outstanding Loan.token; pending ghost receipts do
not count as ownership. The duplicate-token control fails ordinary linear-use
checking. This adds conditional resource checking beyond the duplicable heap
calculus, while keeping the entire physical input carrier conditional.

Proposed native-transition boundary (requirements, NOT assumed declarations):

1. **One handoff at the actual constructor operand.** Connect the existing
   executable source loan/reborrow/unsize chain to the *same* linear owner
   transferred into native IterMut. Preserve the source array/loan identity and
   current/future values; keep no second live Rust reference. If using this
   module's token representation, supply existing tracked pad tokens with
   `full_owner` and a shared key token map with `key_owner`. A checked conversion
   is required; the tracked-slice entry result is not silently coerced to tokens.
   An alternative compiler representation may retain tracked executable slices
   with the single transferred loan, but needs its own checked native lowering.
2. **Native next.** Given the handoff relation, align actual Zip.index/len and
   both fixed physical inner records with Carrier.physical. The mutable getter
   must discharge its actual bb0:s6 Offset and bb0:s8 raw-to-reference borrow
   using exactly the token at the old Zip.index. Immutable getter analogously
   borrows the supplied read token. Pointer provenance/stride/bounds, retag and
   lifetime semantics must justify the correspondence to ptr_mut_ref/ptr_ref.
   The conditional `issue`, `projection_step`, `world_issue` laws are available;
   none proves that the MIR has that effect. Only Zip.index advances.
3. **None, exceptions and drop.** Prove all actual native exits preserve retained
   slots. The identity `none` proof and empty-drop MIR observations are separate
   evidence. An empty physical drop body does not resolve ghost/native loans.
   Every call/helper/cleanup/panic/intrinsic leaf needs its semantics or a checked
   unreachable-path proof. Unavailable panic MIR is an open obligation.
4. **Return to the same executable source loan.** Every issued reference must
   return its permission with its actual final value. `resolve_all` and
   `close_world` reconstruct a map, including its generic unchanged unissued
   suffix. Connect this exact map back to `final(buf)` of the original source
   borrow. Map reconstruction is not that final native relation.

Entry.valid compares byte addresses for pad/key separation and retains pointer
provenance; it never requires distinct allocations or distinct provenance.
Disjoint fields of one allocation are permitted. Only min(128,key.len()) read
tokens are needed; complete key length/end metadata remains untruncated.
Its scalar address conditions do NOT prove allocation validity or native pointer
arithmetic. Those follow only after interpreting the existing Rust types and
borrow facts. No requires clause is added to the original application.

All these links remain conditional/unproved. The existing ordinary frontend
checks local wrapper bodies but does not interpret foreign native MIR into
these tracked fields. No checked source lowering rule or native fact is added;
independent soundness review would be required before accepting such a rule.
