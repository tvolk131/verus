# Entry interface, first ordinary milestone

`exec-carrier-v2` checks 2/0 with ordinary `--no-cheating`, full body/mode/lifetime
checking, zero inferred helpers and no input permission. Its executable
`ArrayBorrow<'a> { array: &'a mut [u8;128] }` retains the original reference.
`enter` proves equality of the current array and of the future array, and
`release` returns that very stored reference with the same equations. This is
real executable borrowing, not a tracked struct declared to be a permission.
It does not transfer ownership into native `IterMut`, split raw-pointer slots,
or justify any native frame law. It does show that an entry carrier need not
use `PointsTo` if its executable reference remains present.

`choose-permission-v1` fails before SMT: spec-mode choice cannot initialize a
tracked `PointsTo`. Its only input is `buf: &mut [u8;128]`. No permission input is
hidden in the signature. This is a frontend failure, not a semantic negative.
`exec-carrier-v1` is an excluded annotation-development failure (ambiguous
prestate field reference), retained with its original pretrial freeze.

Proposed narrow foundation (new research files only):

- `BorrowId { allocation, array, loan }`, `Slot { origin, index }`, exact initial
  and future bytes, and a typed validity predicate over an explicit heap and
  exclusive ownership set; address equality alone never identifies an origin.
- An entry transition transfers the *existing* source ownership set to a
  retained inventory; a split transfers one slot to an outstanding loan;
  resolve returns its exact value; release requires every issued loan resolved
  and reconstructs the same array identity. These will be conditional checked
  semantics theorems, not runtime-created resources.
- A separate rustc-private consumer will extract the actual writer's typed
  array reborrow, unsizing, and constructor operand together with borrowck loan
  records. That record must distinguish compiler-checked typing/loans from the
  still-unproved heap-validity/source-to-resource refinement premises.

No compiler lowering is installed. No research receipt is a permission witness.
`native_bridge_proved=false`, publication disabled. Native indexed yields,
retags/provenance, unissued tail, None/drop/unwind, and full frozen writer remain
unproved until actual source correspondence is checked.

Initial replay:

```sh
python3 -B -O source/tools/research/native_borrow_entry/replay.py \
 --verus /tmp/native-borrow-entry-61f21823/bundle/verus \
 --solver /tmp/native-borrow-entry-61f21823/bundle/z3 \
 --output /tmp/native-borrow-entry-new-run \
 --case choose-permission-v1 --case exec-carrier-v2
```

Private trial receipts: `/tmp/native-borrow-entry-61f21823/entry-v1/receipt.json`
(the first run excludes the annotation failure) and `entry-v2/receipt.json`.
The immutable copied bundle is canonical 112a33, before shared observations;
this is not a matching build of later compiler changes. Bundle hashes and
separate inodes are recorded in `bundle-identity.json`.

## Routed interface update

`candidates/tracked-slice-v2/probe.rs` checks **2/0** ordinarily:

```rust
fn enter<'a>(buf: &'a mut [u8;128]) -> (out: Tracked<&'a mut [u8]>)
    ensures (out@)@ == old(buf)@, final(out@)@ == final(buf)@;
proof fn borrow_slot<'a>(tracked buf: &'a mut [u8], i: int)
    -> (tracked out: &'a mut u8)
    requires 0 <= i < buf.len(),
    ensures *out == old(buf)@[i],
            final(buf)@ == old(buf)@.update(i,*final(out));
```

The implementation first performs the **executable** array-to-slice coercion,
then takes a tracked reborrow of that slice. The slot helper uses the inherited
`slice::tracked_borrow_mut` axiom, not a new axiom. This refines the initial API
search: an existing tracked reference *can* be obtained from the executable
input; a spec sequence or PointsTo need not be invented. It is still a reference
to an exec-mode location. It cannot authorize proof-mode byte writes, and it
has not been paired with the actual native iterator constructor/getters. The
mutable source-to-slot future equation is checked within inherited borrowing
semantics; numeric addresses/provenance are not exposed by this API.

The effect-conservation lane may use the **conditional** `Borrow`, `Heap`,
`State`, `valid_input`, `wf`, `enter`, `issue`, `can_resolve`, `resolve`, `next`,
`ready`, `release` signatures in `candidates/heap-foundation-v5/probe.rs`.
All ten proof bodies check normally. These spec datatypes are a small explicit
footprint calculus, **not** tracked physical permissions. `valid_input` remains
a hypothesis and freely describable spec heaps cannot prove physical ownership.
The heap retains all 128 slots. Its origin contains allocation identity, array
identity and base; it retains the source-loan identity, initial values, current
values, per-index retained/outstanding/resolved states and Zip's logical index.
Physical base remains fixed. `key_len: nat` is unrestricted. No immutable-key
heap semantics or native pointer/retag/unwind theorem is claimed by this calculus.

The new `src/main.rs` obtains actual local borrowck facts before rustc steals
MIR, then calls the **original** `mir_borrowck` provider and completes normal
analysis. `entry_check.py` binds the typed local chain:

```
arg1: &mut [u8;128] -> reborrow local7 (exclusive loan0, region4)
  -> executable unsize local6: &mut [u8] -> native iter_mut -> local5
arg2: &[u8] remains the immutable key input
```

It checks array layout 128 bytes/alignment 1 using rustc queries, the actual
loan reserve location/borrowed place/assigned place and a matching loan-issued
fact. Names/hashes identify the constructor; no constructor body semantics is
proved here. A conservative local-store/call guard rejects extra source writes
and calls; the sibling native MIR checker remains responsible for real effects.
The original source and five independently compiled, honestly rehashed source
mutations plus an invalid-overlap lifetime control are replayed. Twelve forged
witness/status/publication controls also reject. These are eligibility controls,
not semantic/native proofs.

The Rust signature carries an initialized exclusive array reference and a
valid immutable slice. Reading each immutable key element, length/provenance
validity and absence of overlapping mutable/shared referent access must follow
from the interpretation of those **existing Rust types and borrow checking**.
No new application precondition is imposed. What is checked now is the concrete
input types, source reborrow chain and the local borrowck facts. What remains a
hypothesis is their interpretation as `valid_input` and a shared key-read heap,
including allocation provenance, validity, alias separation and future-array
reconstruction. Rust compiler success alone is not that interpretation proof.

All source/native publication remains unconditionally disabled, even for forged
all-proofs-complete inputs. The tracked entry interface does not close actual
native correspondence or the unchanged HMAC suffix.
