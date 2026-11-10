# Executable storage prefix: checked MODEL, Rust interpretation still open

Implemented the actual three root StorageLive operations of the unchanged
six-instance/44-operation constructor graph. This advances execution past
bb0[0..2] to the reborrow at bb0[3] **in an authored partial operational MODEL**.
It supplies executable/formal code and prefix proofs, rather than another
uninstantiated semantics signature. It does not establish an actual Rust source
instance or native certificate. Native0, 44+3 minimal/phase and101 writer effects
remain open; publication=false; goal incomplete. Canonical6/0 and full HMAC1/1
are inherited and were not rerun. No source/body/domain/property was changed.

Only this NEW directory is owned. Prepared checkout base `3f865efc`; early scope
and interface commit `e77da206`. The exact completed constructor artifacts are
pinned at `3d668d29`; historical source-span checkout labels were never opened.
No live sibling/parent checkout or unfinished output, subdelegate, push/PR,
compiler/vstd/controller/global edit, shared build or installation was used.

## Source-derived choices and limits

`pins.json` identifies private raw downloads from official rust-lang/rust,
Rust1.99.1 commit58cf8f9c35b355199727627d07ae34a4e2846d67, cross-checked against
the completed native_pointer_distance compiler pin. These implementation files
are primary **semantic guidance**, not imported formal theorems or proofs:

- [MIR syntax](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_middle/src/mir/syntax.rs#L364): StorageLive/Dead control allocation of a local. Repeated live frees old storage and obtains fresh uninitialized storage; repeated dead is a no-op. This does not drop the referenced object. Locals without storage annotations are always live.
- [Interpreter dispatch](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/step.rs#L37): successful statements advance the statement counter; storage statements dispatch to storage_live/dead.
- [Stack representation and implementation](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/stack.rs#L495): all selected locals are sized. The sized path creates Immediate::Uninit, calls after_local_write(storage_live=true), replaces the local, and deallocates the OLD value only if indirect. Repeated dead works without an access to a dead local. StorageDead of the return place is an interpreter assertion. A backing allocation also exists for an indirect zero-sized local.
- [Physical deallocation](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/memory.rs#L380): indirect storage release removes the allocation, consults a machine deallocation hook and records dead-allocation information. It is not a global physical identity.
- [Immediate-local hooks](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/machine.rs#L635) and [materialization](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/place.rs#L1052): taking a local's address can allocate backing memory; uninitialized storage is not zero-filled. Machine hooks can have effects or fail. Their no-op default is not an established native-machine frame theorem.

The downloaded eval_context/mod files served only to locate the implementation;
they add no formal imports. All bytes and URLs are pinned. Coq's installed primary
standard library (8.20.0) is inherited checking infrastructure; its sources,
objects and tactic plugins are content-pinned as a 1121-file map. The only local
formal dependency is the completed NativeConstructorErasure Interface/Graph;
Composition is also copied unchanged and compiled for interface compatibility.
No additional Rust semantics library or assumed theorem is imported.

## Concrete state and operation implementation

`Storage.v` has separate control, MIR local map, original operand, caller-cell
map, private stack map, fresh stack identifier, arbitrary history type H and a
storage-effect trace. Values distinguish references, raw pointers, slices,
scalars and pairs. Local states distinguish Dead, live immediate Uninit, live
initialized values and live indirect storage. Memory cells distinguish
uninitialized bytes, initialized bytes and pointer/provenance fragments.

Caller addresses reuse `(allocation,native signed byte offset)` and references
retain the existing provenance tag. Private stack addresses use a separate
namespace. `physical_cells` observes both; `caller_map : physical_map` exposes
initialized caller bytes to the old interface. Thus private allocation/freeing
can change physical cells without changing caller observations. Arbitrary
caller cells, interior offsets, reference tags, preexisting private allocations,
prior trace and arbitrary H are accepted by the prefix construction. There is
no zero/fresh caller array, empty-history/protector, client ownership, future-run
or future=current premise.

For sized StorageLive, `live_sized` emits LocalWriteHook, resets the local to
Immediate Uninit, and `retire` deallocates OLD indirect backing if present.
StorageDead changes only that local and releases indirect backing; a dead local
needs no backing check or hook. Deallocation preserves byte contents in a dead
allocation tombstone, makes its cells inaccessible, and emits StackDeallocated.
`materialize_uninit` separately allocates an explicit private buffer of
UninitByte cells. This helper is intentionally limited to previously immediate
uninitialized locals with an unused private identifier; it is not installed as
a MIR address-taking step or as rustc's full initialized spill implementation.

The model records hook requests and storage effects in the trace, retaining the
incoming H unchanged. The **total model history is (H,trace), which changes**.
It does not implement or prove native machine-hook success, race detection,
protector effects or provenance-history updates. The frame theorems below are
about the authored model's caller component. They are not imported source facts.

`Sites.v` is regenerated from the pinned graph: all20 StorageLive/Dead sites,
including the three root live and three root dead sites, have checked identity
and graph-membership facts. `Prefix.execute` checks exact control, executes a
supported storage site and advances it, or returns explicit Unsupported,
StorageFault or ControlFault. Every unimplemented reborrow/raw formation/retag/
unsize/aggregate/call/return/phase operation is Unsupported. No such step can
stutter or claim normal return. Invalid backing states are model representation
failures; repeated live/dead are not invented error cases.

## Checked results

The 142-entry inventory covers definitions, lemmas/theorems, constructors,
record projections and generated eliminators, compared against Coq Print Module.
There are zero Program obligations. Every Print Assumptions result is closed;
recursive coqchk of all new modules and the declaration-free inspection module
reports zero axioms and no unsafe features. This does not discharge universally
supplied inputs or any missing source interpretation.

- `first_three_steps`, `prefix_execution`, `prefix_all_enabled_outcomes`: the
  actual bb0[0..2] operations execute in order without model faults; every enabled
  outcome has the indicated continuing state. These facts quantify over H,
  caller cells, private stack, fresh identifier, prior trace and incoming r.
- `prefix_exact_control/locals`: control reaches bb0[3]; `_0` stays uninitialized,
  `_1` still evaluates to the SAME original reference, and `_2,_3,_4` are live
  uninitialized. Original root layout sizes are16,16,8 bytes respectively in the
  pinned extraction; immediate storage does not allocate those physical buffers.
- `prefix_all_caller_cells`, `prefix_original_bytes`, `prefix_anchors` and
  `prefix_history_effects`: every caller cell, original reference and incoming
  history parameter is retained. Arbitrary initial byte lookups at the same
  original offset are preserved, with no zero/value/length restriction added.
  The prefix records the three local-write hooks and leaves the private map as
  it was. This is a fact about this immediate prefix, not all storage operations.
- `live/dead_anchors`, `live/dead_other`, `live/dead_local`, `execute_caller_frame`:
  supported storage operations retain caller observations and non-target locals.
- `repeated_live`, `repeated_dead_observations`, `dead_twice_locals`: reset and
  repeated-dead behavior, including local-map pointwise identity, control,
  private map, trace and history anchors for an already dead local.
- `materialize_progress`, `backing_release`, `live_releases_backing`: explicit
  uninitialized private allocation and actual private deallocation/tombstones.
  These establish that a universal claim of unchanged private physical memory
  would be wrong. They cover initialized/uninitialized contents without a
  referent Drop, destructor execution or loan-resolution operation.
- `root_storage_dead_sites_progress`: pointwise progress for root bb0[5] `_4`,
  bb1[0] `_3`, bb1[2] `_2`, provided that current local is immediate (including
  moved/uninitialized or initialized reference/pair values). Generic backing
  release additionally handles indirect storage. These are operation laws,
  **not a proof that execution reaches those sites after reborrow/unsize/call**.
- `reborrow_is_explicitly_unsupported`, `all_unimplemented_explicit`: the checked
  prefix ends honestly at the first unimplemented operation.

## Interface for C/F and source-composition work

| Existing component | Concrete storage-layer interface |
| --- | --- |
| configuration / history_type / history_of | `state H` / `H * list storage_effect` / `model_history` |
| source_heap / original incoming operand | `caller_map` / `original_operand`; `Storage.operand_reference s 1` anchors the local |
| physical caller observation | `caller_cells`, `physical_cells s (Caller a)`, `caller_map s a` |
| at_event / step carrier | `current_event s = e` / `operational_step s e o` |
| explicit coverage diagnosis | `execute : state H -> event -> result H` |

The old outcome carrier has no Unsupported constructor. `operational_step`
encodes coverage/control failure as Fault InvalidControl and backing failure as
Fault InvalidStorage, retaining richer diagnoses in execute. This encoding is
**not a judgment of Rust UB**. No Normal/Unwind transition or source_semantics
record with invented lawful-entry/reference-validity predicates is supplied.
These are executable configuration/projection/step components for later reuse.
No Composition.source_entry_certificate or C/F law has been proved.

The precise remaining interpretation relation must map an independently lawful
real entry and interpreter state to these local states and exact control, the
same evaluated `_1`, caller allocation identities/signed offsets and observed
cells, immediate/indirect private storage with disjoint injective allocation
mapping, and the actual incoming histories/protectors. It must interpret the
trace's local-write/allocation/deallocation requests, including success/failure
and their machine effects, and prove simulation plus caller frame for ALL
outcomes of each covered real step. Private separation and H preservation in
the model do not prove that real embedding or its frame. The relation must cover
every lawful original input, not define validity by this model's successful run.
Further reborrow, activation, unsize, raw/reference validity, retag, control,
cleanup and original-reference C/F observation remain unproved. No future-defined
run filter, postcondition-defined validity, Iris embedding or native summary is
introduced to bridge these gaps.

## Validation and reproduction

Successful frozen trial007:
`/tmp/native-source-storage-e77da206-trial007/receipt.json`.
`results.json` records its hash, full inventory, proof/log hashes and zero-axiom
result. `final-freeze.json` is byte-identical to its pretrial `trial-007.json`.
Development trials001..006 retain their freezes and private receipts/logs;
`development.json` classifies their loadpath/type/proof-development failures.
None is a semantic negative or a Rust safety result.

The checker authenticates baseline-ready AND successful named durable-build-2,
immutable VerusBelt115a618d's523 sources/284 objects, runner/coqc/coqchk/opam,
loadpaths, exact GMPmanifest332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37
and both GMP archives, the1121 pinned Coq dependency files, the completed
constructor receipt/products, committed graph/formal sources and completed
extraction receipt/raw objects. Input, private-copy and produced-object/
inventory guards all pass before success. No unsafe flags or kernel exclusions
are used. Only private copies are compiled; baseline objects are read-only.

```sh
python3 -B source/tools/research/native_source_storage/check.py \
  --freeze source/tools/research/native_source_storage/final-freeze.json \
  --output /tmp/fresh-native-source-storage-replay
```

The output must not already exist. The exact pinned private toolchain, completed
artifacts and raw Rust files must remain available; missing Rust files can be
restored only from pins.json's exact official URLs with matching SHA256. No
installation, source extraction or application verification is part of replay.
Root retains integration, independent acceptance, native status and publication
decisions.
