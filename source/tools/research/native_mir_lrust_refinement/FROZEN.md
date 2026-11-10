# Pretrial interface and candidate statements

Prepared baseline cd48c42f. This file freezes the following interfaces before any
Coq trial. No source proof has yet been attempted. Every later candidate is
snapshotted before compilation and failed scripts are retained.

* `window`: containing model block/base location, independent numeric native base
  address and byte extent. Its validity is numeric nonnull, isize/usize bounds.
  `pointer`: window plus cursor; representation is the SAME base+cursor lrust
  location. A slice/NonNull/raw descriptor is PURE metadata, never ty_own.
* Typed descriptor operations: PtrMetadata on slice; wide NonNull wrap/unwrap;
  pointer thinning; thin NonNull wrap/unwrap; byte Offset bounded within the
  containing allocation and isize/usize limits; CTFE false branch on u8.
  Each operation has explicit input/output descriptor constructors. Runtime
  reference creation/retag, unsizing source references, reservation/activation,
  FakeRead, storage lifetime, aggregate ownership, calls, unwind/drop and source
  phases are unsupported (no evaluation constructor and no translation).
* `source_step op inputs heap outputs heap'`: explicitly defined fragment
  semantics. Outputs follow descriptor meaning; bounded Offset requires the
  window and cursor bounds. Heap is unchanged by supported operations.
* `translate op represented_inputs : option (list expr)`: existing lrust
  multiword register representation, NOT allocation of a new heap object. Fat
  descriptors have [location;length], thin descriptors [location], usize [int].
  Projections use real lambda/beta expressions; Offset uses existing OffsetOp;
  CTFE false uses existing Case. Unsupported operations return None.
* `supported_operation_simulation`: any source_step yields translated expressions
  with one actual lrust head_step per output word, exact represented outputs,
  SAME model heap, no events and no fork. Include a converse/determinacy result
  for those target steps so the theorem is operational, not AST reification.
* Window/interior laws: arbitrary containing allocation, arbitrary 128 bytes,
  strict interior start, numerical bounds and same block, one-past relative to
  the array allowed without dereference. No array freeability or second loan.
* `physical_frame`: translated operation WP preserves the original model
  reference and arbitrary separating frame. Use existing model ownership/loan
  interfaces for a composition example; never construct fresh authority from
  a descriptor. Exact postcondition/step costs remain visible in the scripts.
* Structured source reifier authenticates source-bindings receipt and both MIR
  JSON outputs, selects enum payloads and exact layouts/operands, and emits a
  checked Coq supported-operation inventory. It does not certify Rust semantics.
  Source creation of the raw slice descriptor remains a required interpretation
  boundary, never silently erased in a whole-program translation.
* Coq controls refute one-past-as-in-array-dereference, out-of-window Offset,
  wrong descriptor kinds, and physical ownership duplication. Python controls
  exercise unsupported MIR and changed layouts/operands with honest payload
  mutations, not identity/race/substitution experiments.

Toolchain: read-only Coq 8.20.0, VerusBelt 115a618d, existing runner and pinned
GMP. Authenticate all baseline source/object pins and completed library receipts
before use; retain guard snapshots/copies through final recursive coqchk.
Inspect assumptions for every new definition, inductive/constructor, fixpoint,
lemma and any generated Program obligation; no new axioms or unchecked modes.

This is a source-fragment-to-existing-lrust theorem conditional on an explicitly
stated descriptor representation. It does not establish an actual Rust incoming
loan, pointer provenance, ancestor contexts, full native body/phases or C/F
observations. Native correspondence 0; 44 minimal + 3 phase /101 effects remain
open; full HMAC 1/1 inherited; publication=false.
