# Exact model boundary

VerusBelt 115a618dd35ff8337112cf2081a7f5391d3973d5 is the only proof
foundation selected here. Its actual defined unique reference type combines:

* physical `[FVal (LitV l.1)]`,
* `uniq_body`, containing a linear prophetic indexed borrow of the typed heap
  payload at `l`, and
* lifetime, persistent guards and current/future prophecy interpretation.

`source_loan_access` opens that actual body, retaining its location, prophecy
identifier and indexed-borrow identity, then closes it with the SAME payload.
The input resource follows from semantic reference typing. It is not the owned
`PPtr` permission type: `points_to_ty` is `tracked_ty (own_ptr ...)`, which
includes a freeable allocation capability. A source borrowed subobject supplies
no such capability. The accessor quantifies over arbitrary `cloc`; it never
identifies the borrowed location with an allocation base.

The new 128-element byte type represents initialized cells with values 0..255.
This is a cell-level instance. A VerusBelt `LitInt` physical cell is not a
Rust byte, and a VerusBelt integer is not a fixed-width machine integer.
`lang.BinOpOffset` accepts any Z offset without a live-allocation, address-space
or isize bound. Our model-limit proof records this directly. There are no MIR
retags, fat-pointer metadata, Rust transparent valid-range types, or native
exception/cleanup operations in this language. The raw-pointer observation
lemma retains the original unique resource exactly once; `ptr_ty` itself is
copyable and has ghost interpretation True. The empty-heap raw-read control
shows why a pointer value alone supplies no readable contents.

A true Rust-native handoff requires a simulation of that Rust primitive's
physical/provenance/retag behavior AND its logical-resource transfer. Neither a
resource-preserving lambda-Verus identity nor the inherited 115/0 ghost trace
establishes that. Our native operation audit therefore rejects every unproved
operation and the pre-borrowck FakeRead/unwind phase differences. No extra C/F
observation or same-size layout argument repairs that missing interpretation.

# Additional foundation inspected, not selected or built

The official [RefinedRust project](https://plv.mpi-sws.org/refinedrust/) links the
[development repository](https://gitlab.mpi-sws.org/lgaeher/refinedrust-dev).
Inspection pin: `e39cd5f1c2d6fde6ab8be70a847e0547a2fa9156`.
Private sparse checkout: `/tmp/native-semantic-bridge-e4a8dcab/refinedrust-inspect-e39cd5f1`.
No RefinedRust library or theorem is imported by the proofs in this lane.

It offers actual candidate primitive foundations missing from VerusBelt:

* `theories/refinedrust/radium/loc.v` defines a location as provenance plus
  numeric address; provenance may identify an allocation or be absent.
* `radium/lang.v`, `eval_bin_op.PtrOffsetOpIP`, requires in-bounds start/result,
  byte offset fitting isize, and representable result. Its own TODO says the
  validity/live-block condition should be strengthened. This is not silently
  treated as a proof of current Rust Offset or retag behavior.
* That language has byte-level values, XorOp, typed validity/casts, call return,
  stack local live/dead, unwind cleanup/terminate, and link-time panic strategy.
* Frontend source fidelity is a separate question: the inspected translator ignores
  MIR FakeRead in `body/function_translation/basicblock.rs`. The source-to-Radium
  correctness of doing so is not supplied by its type-system soundness theorem.
  `rr_frontend/rust-toolchain.toml` pins nightly-2026-11-02, not this task's
  Rust 1.99.1 extractor/toolchain. No current-Rust retag correctness theorem was
  identified in the inspected Radium syntax/heap rules.

Adopting it would introduce a different Rocq/Iris/dependency tree, the Radium
memory-model choices, translation/ABI/exception obligations and a new
cross-logic relation to VerusBelt's prophetic C/F observations. None is assumed
here. It may support a future explicitly pinned primitive simulation lane; it
is not a replacement algorithm, native certificate, or automatic closure of
any of the 101 obligations.

For the full writer, input key reference interpretation must distinguish empty
slices: a valid zero-length key can be nonnull/aligned/dangling, with no readable
bytes and no live containing allocation. Nonempty pad128 retains all initialized
slots. Nonempty key/pad regions may be disjoint within ONE allocation. Neither
allocation inequality nor key-length narrowing is an acceptable premise.
