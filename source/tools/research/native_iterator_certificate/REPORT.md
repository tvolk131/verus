# Executable typed MIR checker; native_bridge_proved=false

This standalone rustc_private prototype queries the **actual monomorphized MIR**
of the frozen local three-line writer and recursively resolves concrete callees.
It does not install facts, change production code, or prove the frozen full
HMAC writer theorem. The original executable loop remains the input; neither
an indexed replacement nor a hand-written native trace is used as evidence.

The first guarded replay (`extraction-receipt.json`, commit `f76fe2a6`) builds and runs with Rust
1.99.1, normal compiler analysis and overflow checks. It extracts 25 concrete
instances and 401 statements/terminators, including the specialized Zip new/next,
both indexed getters, constructors, size calculations, and drop glue actually
encountered. Every MIR local has its instantiated type, target layout and
`needs_drop` query; every block has source locations, operations, successors and
resolved call edges. `mir_sha256` binds the queried body; identity hashes do not
certify its semantics. Foreign generic MIR is available through `instance_mir`.

Extraction is complete as a call inventory, with three explicit semantic leaves:
`ptr_offset_from_unsigned` is an intrinsic without a modeled semantics, and
`panic`/`panic_nounwind_fmt` lack available bodies. These are blockers, never
assumed harmless. Finite instance bounds stop expansion conservatively.

Raw typed MIR and logs remain private under
`/tmp/native-iterator-certificate-20261109-41eec989/dev04/`.
`milestone-extraction.json` commits only the compact inventory. The shared
`infer_experiment.py`/`infer_eval_support.py` guards and receipts run under
Python `-O`. `extraction-freeze.json` predates the accepted build/extraction.
Earlier development compiler/API errors are excluded from accepted evidence.

Reproduce from the repository root:

```sh
python3 -B -O source/tools/research/native_iterator_certificate/replay.py \
  --output /tmp/a-new-private-native-certificate-run
```

## Second executable milestone

The second guarded replay adds typed enum lowering in `src/ir.rs`, a bounded
normal-return value interpreter and conservative inspection in `check.py`,
conditional scalar VC generation in `vc.py`, and explicit controls in
`controls.py`. It extracts **32 concrete instances and 408 MIR operations**.
The additional seven instances are scope-exit drop-glue queries, not invented
runtime calls. Eleven queried/encountered drop-glue bodies are empty-return
bodies for this specialization. This physical no-op observation does not prove
the logical release of retained loans. The remaining three extraction leaves
are still present; no panic branch is excluded by an assumed frame law or
merely because foreign MIR is unavailable.

Structured operations come from matching rustc MIR enums, not parsing the
human `debug` fields. Operands include copy/move, evaluated constants and
runtime-check flags; places carry typed dereference/field/downcast/index
projections. Calls carry instantiated destinations and arguments, resolved
instances and structured unwind actions. Layouts include sizes, alignments,
field offsets, native field names/types and representation flags. Unknown
Rvalues, terminators, intrinsics, projections, types, casts, or assertion forms
are represented as unsupported and rejected. Unknown calls/callbacks, missing
callee/drop bodies, and unproved unwind effects block certificate publication.
The compiler completes normal analysis before extraction; no normal body,
lifetime or trait checks are disabled.

The value interpreter directly consumes these structured operations. It follows
normal call edges recursively with finite operation/call-depth bounds, checks
uninitialized reads and storage/move effects, and rejects loops requiring
induction. On the real next wrapper and specialized implementation it checks
both normal paths at an arbitrary **hypothetical** Zip state:

- Success returns references at the initial `ipad` and `key` pointer origins
  plus the old Zip index, increments only Zip's own index by one, and preserves
  the two physical inner records and length.
- None returns no references and preserves the entire hypothetical Zip record.
- Three constructor/wrapper paths compute the expected pointer origin and end
  extent from the supplied slice value, including the full 128-byte pad range.
  The u8 non-ZST branch is selected by actual CTFE constant evaluation.

These are **conditional computational value diagnostics**, not an independently
proved semantics for rustc MIR, a native ownership theorem, or a proof that the
hypothetical input state corresponds to the client's original array borrow.
The interpreter uses element offsets for the u8 specialization. It records,
but does not discharge, pointer bounds/provenance, validity, retags, transparent
pointer representation and nonunwinding obligations. It does not turn copyable
observations, PhantomData, or a serialized slot list into permissions. Static
writer checks follow the actual local definitions of the two returned pair
references; these are eligibility/effect diagnostics, not a loop proof.

The generated backend queries prove only two **conditional integer implications**:
the actual extracted index update stays within `length`/usize range, and the
indexed yields are within the pad/key lengths, **given**
`0 <= index < length <= 128` and `length <= key_length`. The key length is not
restricted to 128. Both queries require literal `unsat`, exit zero and empty
stderr. Neither query establishes that native construction supplies the
invariant or that arithmetic bounds imply a valid pointer/borrow operation.
The two discharged scalar VCs do not discharge the 101 generated native-effect
obligations or the separate ownership/refinement obligations.

## False-input controls and publication barrier

The guarded controller freezes all code/fixtures before building, hashes the
actual compiler/core/driver metadata inputs, guards the built extractor and
checks repeat extraction for byte-for-byte deterministic JSON. Human installed
Rust source correspondence is additionally bound for every extracted source
span; the compiled core library identity is the actual MIR dependency binding.
It uses an immutable private byte-for-byte solver copy with distinct inode.
Common guards/receipts remain active under Python `-O`.

Eight Rust source variants compile independently with **honestly updated source
and evidence hashes**: changed XOR to OR, omitted write, added tail write,
wrong pair RHS, reversed key origin/index, helper side effect, destructor side
effect, and an indirect callback. Each must hit its designated operation/effect
diagnostic. Identity mismatch alone cannot pass these controls. These are
extractor/checker controls, not SMT postcondition failures or native tests.

The final controls also cover wrong native getter origin/index, wrong Zip field
mutation, wrong constructor extent, missing helper/drop examination, unknown
Rvalue/terminator, type/layout mismatch, forged source binding, lost/duplicated
permission, incorrect logical Zip projection, failed/incomplete/timeout/resource
proof statuses and forged all-proofs-complete status. The mutated native-IR
controls leave debug strings unchanged and update the entire IR binding. They
test interpretation rather than native identity matching; they are not changes
to the installed standard library. Instance/operation-budget exhaustions carry
explicit rejection markers. Actual scalar solver controls reject `sat` and
resource-limited `unknown`; resource failures are not semantic counterexamples.

`validate_certificate` **always rejects** after preliminary consistency gates:
the native refinement proof checker is deliberately not implemented. A forged
JSON `proved` label cannot publish anything, even if all recorded blockers are
deleted. There is no production fact export or production integration patch.
`success` in an experiment receipt means that its expected diagnostic outcomes
were reproduced, never that the target theorem was proved.

## Exact remaining proof boundary

No new actual native theorem is proved. No ordinary Verus proof was added; the
explicit new ordinary-body/helper inventory is empty. The full frozen
`pad_xor` theorem, for every initialized `[u8;128]` and every valid immutable key
slice including empty and longer than 128, remains required and unproved here.
The frozen property and original executable input remain unchanged.

The earliest missing ownership obligation is a **checked relation** from the
original executable exclusive array borrow to 128 distinct initialized linear
slot permissions, with matching initial values and future reconstruction of
that very array. The extracted constructor's physical pointer range supplies
neither the tokens nor their connection to `final(ipad)`.

In this checkout `vstd/raw_ptr.rs:636`'s `ptr_mut_ref` requires an existing
matching initialized tracked `PointsTo<T>`, and connects its current/future
contents to the borrowed reference. The actual MIR getter has no such argument.
`vstd/slice.rs:244`'s `tracked_borrow_mut` requires an already tracked slice and
specifies its update; it does not convert the client's ordinary executable
borrow into a pointer permission inventory. Raw permission splitting APIs
likewise require an input permission. None of these APIs derives the missing
carrier from `IterMut::remaining()`, a raw pointer or PhantomData.

Subsequent obligations are successful-yield loan transfer exactly once;
all-unissued-slot conservation over every source transition; full-domain panic,
unchecked-pointer, unwind and drop accounting; borrow resolution restoring
the original exclusive array; and agreement with inherited observation equations.
In particular a logical Zip projection must use the Zip index even though the
physical inner pointers stay fixed. The full source-to-value/ownership lowering
needs an explicit soundness proof and independent review. The finite value
interpreter and conditional scalar solver queries cannot replace these checks.

The minimal missing implementation is a verified/native-aware borrow lowering
or proof-carrying ownership relation accepted by the ordinary verifier, with
permission creation justified by its input borrow and permission return linked
to its output. Adding an unchecked `PointsTo` constructor or a final/current
frame axiom would change trust and is not implemented. This identifies the
current primitive/model linkage blocker, not an impossibility result for every
future verified compiler extension.

`checker-results.json`, `checker-freeze.json` and `checker-receipt.json` record
the final executable batch. Raw evidence stays in the private path named by
the receipt. Earlier development failures are excluded. Only this research
directory changed; inherited compiler/vstd/policy/cache/schema trust is unchanged.
