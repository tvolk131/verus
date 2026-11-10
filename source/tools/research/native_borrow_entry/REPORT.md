# Checked borrow carriers; actual native ownership entry remains open

This work does **not** prove entry into a native iterator's all-slot ownership
inventory, actual native ownership/value refinement, or the full HMAC writer.
`native_bridge_proved=false`; publication is unconditionally disabled. No engine,
vstd, original application, shared harness, global schema/policy/cache, sibling,
or other-worktree file was changed. No axiom, external contract, failed-proof
export, push or PR was added. All changes are in `native_borrow_entry/`.

`f83f0a0f` is the early frozen entry/interface milestone. `33976bac` adds checked
carriers, the conditional heap calculus and actual local borrowck evidence.
The final follow-up commits the precise counts, key-alias controls and replay.

## Actual executable input and checked reference carriers

Every entry probe's client input is the ordinary executable initialized exclusive
`&mut [u8;128]`; it has no permission argument or value restriction. The native
source fixture is the unchanged independently frozen loop:

```rust
pub fn xor_key(ipad: &mut [u8;128], key: &[u8]) {
    for (b_i,b_h) in ipad.iter_mut().zip(key) { *b_i ^= *b_h; }
}
```

The extractor's fixture preserves the actual frozen formatting. The controller
checks the original pretrial manifest hash, the upstream selected-source hash
and normalized exact loop inclusion. No key length is restricted. Native source
mutations and diagnostic no-write probes are separate inputs, never replacements
for the frozen writer or its XOR-prefix **and unchanged-suffix** property.

The inherited mechanisms support more than an uninhabited permission API:

- `exec-carrier-v2`: **2/0**, executable `ArrayBorrow { array: &mut [u8;128] }`
  entry/release with exact current and future array equations.
- `exec-carrier-v3`: **3/0**, additionally splits into two executable mutable
  slices at every `0 <= mid <=128`, with both initial subranges and the future
  concatenation equal to the original array. The split uses the inherited
  native `split_at_mut` specification; its unsafe core body is not newly proved.
- `tracked-reborrow-v1`: **1/0**, returns a tracked reference to the actual
  executable array with exact current/future values.
- `tracked-slice-v2`: **2/0**, first performs the array-to-slice coercion in
  executable mode, then returns a tracked reborrow with all 128 current/future
  values. `borrow_slot` calls the inherited `slice::tracked_borrow_mut` axiom,
  recording the selected initial byte and exact future slice update. Its input
  tracked slice is supplied by the checked entry helper, not assumed at the client.

These are checked bodies relative to inherited vstd/builtin reference, array,
coercion and slice contracts. No new axioms are introduced. The tracked reference
points to an **exec-mode location**; it is neither `PointsTo` nor a capability to
mutate executable bytes in proof code. It exposes no numeric pointer provenance.
The executable carrier retains a real reference field, not an invented tracked
struct whose mere existence is treated as a permission.

## Smallest concrete obstruction

`simultaneous-owner-v2` tries to keep the tracked slice owner live while calling
actual `slice.iter_mut()`. Ordinary checking rejects it with **E0499**: the same
source slice would have two mutable borrows. Thus the working tracked entry
cannot simply be placed next to the unchanged executable native constructor.
A disappearing ghost snapshot would avoid this lifetime conflict only by losing
the live ownership relation that the certificate needs.

`native-drop-v1` merely constructs and drops `buf.iter_mut()`, without any write.
The unchanged-array postcondition remains an ordinary **0 verified / 1 error**
query. This is a failed positive diagnostic under inherited contracts, not a Rust
counterexample, nor proof that the true native property is false.

`choose-permission-v1` cannot turn a spec choice into tracked `PointsTo` (mode
rejection). `assume-new-permission-v1` tries the existing unchecked constructor;
ordinary translation rejects `Tracked::assume_new` as unsupported. The suggested
new `assume_specification` is deliberately **not** installed. These probes and
the lifetime conflict reject ghost-only/duplicated physical authority; they are
frontend/lifetime evidence, not SMT semantic counterexamples.

The first missing primitive is now precise: a checked ownership interpretation
of the **same executable reborrow/unsize/constructor transfer**, preserving the
original array identity and future reconstruction, without a second live Rust
borrow and without minting `PointsTo` or an abstract tracked token. A compiler
lowering could carry one logical owner with the actual executable transfer, but
this requires a source-to-heap/loan interpretation theorem. No such compiler rule
is implemented or assumed here. The supplied-owner effect-conservation lane
must continue to state its input owner explicitly and conditionally.

## Checked conditional footprint semantics

`heap-foundation-v5` proves **10/0** under ordinary `--no-cheating`. Its explicit
spec `Borrow`, `Heap`, `State` and `Holder` encode all 128 initialized bytes,
allocation/array/base identity, the original source-loan identity, fixed physical
base, Zip's logical index, and retained/outstanding/resolved status at every
index. `key_len: nat` is unrestricted, including 0 and values greater than 128.

The checked bodies establish input-premise inhabitation, entry preservation,
within-footprint stride/nonalias arithmetic, indexed issue conservation, one
outstanding loan's resolution and write frame, rejection of a second write
through a resolved loan, None identity, complete release/reconstruction, the
unchanged unissued suffix, and the empty-key frame. No new axioms or unchecked
proof bodies occur in this file. The release theorem requires all issued loans
resolved. Future output is the exact reconstructed heap bytes with the original
source identity; relating this output to Rust's actual future array remains open.

**This is a conditional single-footprint calculus, not physical permission.**
`valid_input(b,h)` explicitly assumes the existing source-owned initialized heap.
Its pure spec records/sequences are duplicable and constructible. Consequently
the inhabitation theorem is a mathematical consistency check, not allocation or
permission acquisition. `issue`, `resolve` and `release` are semantic functions;
their legal-use premises must be checked for each native transition. A caller
cannot obtain native authority merely by calling those pure functions.

This foundation does not interpret native typed MIR instructions, raw pointer
retags, actual pointer validity, a shared key heap, arbitrary external writes,
cleanup, panic/intrinsic leaves or native drop. It does not discharge any of the
inherited 101 native effect obligations. In particular logical `None` and
`release` here do not prove actual native None/drop semantics. No conditional
math theorem is promoted to actual native correspondence.

Eight frozen ordinary false claims cover duplicate/lost slots, same-address
wrong allocation, wrong array identity, unauthorized tail writes, later writes
through resolved loans, wrong initial bytes and wrong future reconstruction.
Every case checks **10 healthy proofs / 1 intended postcondition error**, with
the identical source-frozen foundation dependency. The healthy 10/0 run is
separate. No resource, frontend, safety or poisoned-helper error counts as one
of these semantic controls. The inferred-helper identity set is explicitly empty;
all three required helper statuses are still checked by the shared harness.

## Actual typed source and key/alias connection

The standalone Rust 1.99.1 rustc-private consumer captures the real local writer's
borrowck MIR and Polonius input facts before rustc steals that body. It then runs
the **original** `mir_borrowck` provider and completes normal compiler analysis.
It does not replace lifetime/trait/body checking with its own result.

`entry_check.py` checks a finite closed local prefix: exclusive array reborrow,
array-to-slice unsizing, then the actual native `iter_mut` call operand. It
rejects unknown/additional entry operations. Actual evidence connects argument
1 to reborrow local 7, exclusive loan 0/region 4/reserve bb0[5], unsized local 6,
and native constructor result local 5. The borrowed/assigned places and a real
loan-issued fact must agree. rustc supplies the array layout (128 bytes, align1)
and actual typed `&mut [u8;128]` / `&[u8]` inputs. This is typed eligibility and
source binding, not a checked semantics for the operation or constructor.

A conservative complete-local-writer call/store inventory additionally rejects
new local effects. It is a diagnostic guard, not a loop or callee proof. The
borrowck MIR still contains explicitly unsupported non-entry operations; this
consumer does not silently treat them as checked. The readonly native iterator
certificate extractor remains the source of actual specialized native MIR.

Final extraction accepts the original entry and a caller borrowing the pad/key
as disjoint fields of **one allocation**. Five independently compiled, honestly
rehashed mutations reject for wrong extent, wrong source array, a pre-loop tail
write, a later tail write, or shared input. Overlapping exclusive borrows reject
with E0499; a caller forming an overlapping immutable key rejects with E0502.
No address/name/hash mismatch substitutes for these operation/type checks.

Immutable key validity, initialized readable elements, extent/stride/provenance
and absence of conflicting mutable/shared element access belong to interpreting
the existing Rust reference types and their checked callers. They are **not new
application requires clauses**. Disjoint key/pad allocations are not required;
disjoint byte footprints within one allocation are permitted. Empty slices need
no readable bytes. The new source evidence checks types/loan facts and concrete
valid/invalid callers. It does not prove the Rust-reference-validity theorem,
shared read-permission interpretation, arbitrary incoming caller validity, or
native raw-pointer alias semantics. Those remain explicit semantic premises of
any future source-to-heap lowering.

Twelve certificate controls reject forged borrow witnesses, wrong borrowed
origins, missing issue facts, wrong layouts, unchecked bodies, forged publication,
and failed/incomplete/timeout/resource-limited proof statuses. Even a forged
`proved, complete=true` result hits the unconditional publication barrier. These
are malformed-certificate controls, not semantic counterexamples or completed
proof checks. A copied spec heap with matching bytes never bypasses the barrier.

## Replay and audit

All accepted ordinary application/new-law trials use `--no-cheating`, full body,
mode and lifetime checking. The final ordinary replay covers 17 expected cases
(16 in `final-ordinary`, the additional unchecked-constructor control separately).
`accepted.json` fixes exact verified/error counts, body inventories, source hashes
and diagnostic markers. Both controllers use `infer_experiment.py` and
`infer_eval_support.py`, immutable source/tool guards and durable failure receipts
under Python `-B -O`. A wrong-solver control rejects before launching any command
and persists its failure receipt.

Use the immutable private copied canonical 112a33 compiler/macros/verified-vstd
bundle and Z3 4.16.0, with hashes/separate inodes in `bundle-identity.json`. It
predates shared observations and is not claimed to match later compiler source.
No engine/library change in this work needs a new vstd build. Never pass
`--no-cheating` to a vstd rebuild; new proofs/applications retain it.

```sh
python3 -B -O source/tools/research/native_borrow_entry/replay.py \
 --verus /tmp/native-borrow-entry-61f21823/bundle/verus \
 --solver /tmp/native-borrow-entry-61f21823/bundle/z3 \
 --output /tmp/fresh-entry-ordinary \
 --case tracked-slice-v2 --case exec-carrier-v3 \
 --case simultaneous-owner-v2 --case native-drop-v1 \
 --case heap-foundation-v5 --case false-duplicate-slot-v1 \
 --case false-lost-slot-v1 --case false-wrong-allocation-v1 \
 --case false-wrong-array-v1 --case false-unauthorized-tail-write-v1 \
 --case false-later-loan-write-v1 --case false-wrong-initial-values-v1 \
 --case false-wrong-future-array-v1 --case choose-permission-v1 \
 --case assume-new-permission-v1

python3 -B -O source/tools/research/native_borrow_entry/extract.py \
 --freeze source/tools/research/native_borrow_entry/extraction-v7-freeze.json \
 --output /tmp/fresh-entry-source
```

Cargo runs from `source/`, nested `[workspace]`, locked/offline, two jobs, private
build/target directories. `RUSTC_BOOTSTRAP=1` enables the pinned compiler's MIR
inspection flags; it disables no checks. Raw MIR, binaries and logs stay under
`/tmp/native-borrow-entry-61f21823/`. Compact receipts/hashes and excluded failures
are in `milestone-results.json` and `final-results.json`. All authored source
candidates were frozen before first trials; unsuccessful versions remain intact.
Excluded setup/API/panic/annotation/proof-development failures are not accepted
semantic evidence. No whole-HMAC or security theorem is claimed.
