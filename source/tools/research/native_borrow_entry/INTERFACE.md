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
