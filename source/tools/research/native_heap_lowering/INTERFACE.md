# Frozen early single-borrow heap-lowering interface

Scope: only new research files. No production rules/facts; publication is false.
Candidate drop-v1 is an exact copy of native_borrow_entry's no-write diagnostic.
fixtures/drop.rs changes the root name to xor_key for both existing extractors,
removes verifier syntax, and preserves the executable constructor/scope exit.
The full frozen HMAC writer remains required and unproved.

Untrusted input: inherited schema-2 typed MIR (actual enum encoding, CTFE,
layouts, instance-resolved call/drop graph), plus local borrowck evidence from
normal analysis before region erasure. Debug strings never select semantics.
Each operation/exit/helper must be interpreted or rejected; reachable panic,
unknown intrinsic, cleanup and raw-reference validity cannot be erased.

Checked semantic target: an explicit bounded byte-allocation semantics with
references carrying allocation, base, extent and loan identity. Native locals
and pointer records carry these identities; they are not additional Rust
references or tracked permission constructors. All 128 arbitrary bytes enter
as one mathematical allocation observation. Reborrow/unsize/constructor operations
retain the same origin and suspend/transfer the single active source loan.
A generic checked interpreter/frame theorem must establish effects in this
semantics; the untrusted extractor/lowerer is not itself a theorem.

Separate obligations (none implied by a JSON flag):
1. Total embedding of EVERY valid Rust input reference into this semantics,
   preserving provenance, existing reference validity and current bytes.
2. Instruction/call/control-flow simulation, including pointer arithmetic,
   retag, layout/transmute, cleanup/panic and drop effects.
3. Exit loan release and observation adequacy to final(original array).
4. Independent review and full frozen writer correspondence before publication.

The first milestone will report exactly which of these are checked versus
conditional. No supplied full-owned heap, raw PointsTo inventory, spec bytes,
PhantomData or source hash is accepted as original physical authority. The
model observation is explicitly non-authoritative. No extra requires is added
to drop-v1 or the original writer. A no-store scan alone proves no native frame.
