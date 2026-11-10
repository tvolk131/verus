# Indexed shared byte reads (MODEL)

Implemented the committed INTERFACE, confined to this directory. The early typed
handoff is commit `383c94df`; core implementation `31333e95`. The supplemental
session keeps a read token available while mutable-pad access uses the other
context half. No application, baseline, compiler, vstd or root callback changes.

## Interfaces

`shared_read.shared_key_view n κ q x d g tid` is exactly
`ty_own (&shr{κ} (bytes_ty n)) ((q,repeat [] n),x) d g tid [FVal #q]`.
It is persistent; d and g are arbitrary, d=0 is contradictory, and d=1,g=0 is
supported. No time receipt or uniqueness context is needed. The byte type and
flattening are the pinned NativeSemanticBridge definitions; the arbitrary-list
specialization uses pinned ModelInputContext.vector.bytes_of_list.

With `lctx_lft_alive E L κ`, genuine `llft_ctx`, persistent `elctx_interp E`,
ordinary `llctx_interp L`, this same view, arbitrary separating Frame and
`£(2*(g+1))`, `shared_view_read` executes precisely
`Read ScOrd #(q +ₗ i)` for every i<n. It returns the exact same view, L and Frame,
and a byte z with `byte_list n x !! i = Some z`, return value #z and 0<=z<256.
No exclusive PointsTo, supplied guard, disjointness assumption, fixed size or
unconditional first-cell access. Empty keys have no valid index.

Preparation `shared_key_prepare` costs g+1 credits and zero program steps. It
returns L and persistent `shared_key_cap`, the same shared view plus the byte
range derived from its genuine ghost guard. Reusable `shared_key_read` costs
g+1 credits and exactly one SC step, returning L, capability and any Frame.
At g=0 the sufficient explicit budgets are 1 for preparation, 1 per read, 2 for
a direct view read. These are accounted budgets, not claimed lower bounds.
The read law permits additional copies of its capability in Frame; this is also
checked as `shared_read_same_alias`. `shared_read129` checks n=129, index=128;
the generic proofs have no bound at 128. `list_shared_read` handles every finite
input list and any successful indexed lookup.

Baseline `read_shr` was investigated: it is a whole-object Copy accessor with
synchronized d=g and d+1 credits, whose usual dereference consumer is non-atomic.
The indexed SC proof instead specializes the existing physical shared guard
with `bytes_index_guard`, composes lifetime aliveness from ordinary E/L, weakens
NllftG to naN, and uses `wp_read_sc_guarded_cells`. Preparation opens/recloses the
existing ghost guard and uses `bytes_initialized`. No ownership extraction.

## Mutable callback handoff

`session.shared_key_reader_session` starts from the same ordinary E/L and shared
view, using only the g+1 preparation credits. It internally splits L and returns

```
∃ R G : iProp Σ,
  ⌜Timeless R ∧ Timeless G⌝ ∗ R ∗ G ∗ shared_key_cap ... ∗
  (G &&{↑NllftG}&&> llctx_interp L) ∗ key_reader ... R ∗
  (R -∗ G ={⊤}=∗ llctx_interp L).
```

`key_reader ... R` is persistent. `key_reader_read` requires only this law, the
internally generated linear R, arbitrary Frame and g+1 credits. It performs the
same indexed SC read and returns R, the unchanged shared capability and Frame.
It requires no L, E, llft_ctx or caller-supplied guard at read time. Thus G can
support existing mutable-pad access while R stays in the callback's arbitrary
Rin/Rout resources. Pad closure restores G; joining R/G restores ordinary L.
There is no root Rin->Rout signature change. The two tokens prevent using the
persistent reader after closing the session. Session setup has zero program
steps and no credits beyond preparation; each read has one SC step.

This handoff is a checked MODEL interface. Its particular integration with the
parent's mutable loop is not proved here. No extra concrete XOR loop was needed
for the reusable interface, and no native application precondition was added.

## Same heap observation

`observation.shared_key_heap_observe` uses genuine `heap_ctx σ`, E/L, capability,
Frame and g+1 credits. It returns the SAME σ and all resources plus
`∃ z r, byte_list n x !! i = Some z /\
σ.1 !! (q +ₗ i) = Some (RSt r,#z) /\ 0<=z<256`.
The proof calls baseline `atomic_read` on the same authoritative non-atomic map.
The read-state count r is existential; it is not assumed zero. This zero-step
observation does not yield an exclusive vector or change the heap.

## Validation and limits

`check.py` authenticates the mandatory ready gate AND successful baseline
receipt, all 523 foundation sources and 284 objects, tool identities, the GMP
manifest (332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37),
mandatory shared controller/parser support, and exact library source/object/
receipt identities in `dependency-pins.json`. The heap library uses the committed
GMP-positive pin. Only explicitly authorized /tmp library roots are read;
receipt-embedded historical checkout paths are treated as labels.

Every trial freezes before execution, copies sources into a fresh private /tmp
proof directory, compiles normally, inventories every declaration (including
instances) and all obligations (none), prints assumptions for each, and invokes
full recursive `coqchk -silent -o` without exclusions. Source snapshots, inputs,
objects and retained generated assumption sources stay guarded through the
final check; the proof-success flag is set only after all guards pass.
`Typeclasses Opaque` directives control proofmode unfolding only, not kernel
checking or logical assumptions. Raw logs, objects and failed trial snapshots
remain untracked under /tmp; compact identities are in `development.json`.

The core replay passed for 15 constants at
`/tmp/shared-byte-read-final-018/receipt.json`, SHA256
`6b0651a8088649918ac82b7c446c52e81f5a27d3f0cd92330ac71fc9ec80d17c`.
`results.json` records final combined validation, exact receipt/hash, object and
kernel identities, and the complete declaration/assumption inventory.

No new axioms, admissions, unsafe settings or primitives. New semantic proofs
inherit functional extensionality from the pinned representation. The complete
recursive dependency check also reports existing Eq_rect_eq; both are inventoried.

Actual Rust/native correspondences remain ZERO. All 44 minimal + 3 phase
simulations and 101 writer effects remain open. Canonical 6/0 and the frozen full
HMAC prefix+suffix 1/1 are inherited facts, not rerun or strengthened here; shared
observation is causally redundant in that canonical result. Publication and
full-goal completion remain false. New independent review and parent integration
remain outstanding. No push or PR.

Final combined replay passed: 19 declarations, zero obligations, all four modules
plus the generated assumptions inventory, full recursive kernel check without
exclusions. Exact receipt SHA256: `d434bc73bacd2335313b73001436696cf8a305930fa9fa30eb1c79b014311b69`.
Final pretrial freeze SHA256: `a3d3dec0b1c1f64cb1d6459163823b4917b2334d91952be12f453b214e7d144a`.
Implementation checkpoint `4e91856c`; mathematical sources retained exactly.

Replay from the repository root into an unused private output directory:

```sh
python3 -B source/tools/research/model_shared_byte_read/check.py --freeze source/tools/research/model_shared_byte_read/final-freeze.json --output /tmp/shared-byte-read-replay-review
```

Twenty failed development trials are retained: eighteen compile-development
failures, one superseded run rejected by its final source guard after its
recursive checker returned zero, and one explicitly stopped recursive run.
No failed receipt is accepted. Every frozen repository input for these trials
has an exact content-addressed private snapshot, pinned by the compact manifest.
The separate core replay and final combined replay both passed all final guards.
