# Existing caller-context MODEL writer

`caller.caller_xor_verified` proves the concrete existing shared XOR loop under
an existing `typeG`, ordinary `E/L/I`, a typed mutable owner, a semantic shared
key view, and an arbitrary separating ancestor frame. Its exact contract is
`interface.caller_xor_contract`. No input points-to, exclusive key vector,
range/output-summary premise, second lifetime context, fresh heap or fresh
`typeG` is required.

The proof invokes `shorter_run_resources`. Its generic callback first receives
the already-reborrowed child, then prepares `shared_key_reader_session` on that
child's ordinary lifetime context. Preparation returns `R`, `G`, a reader,
the shared capability, and `G &&> Lchild`. The new
`guarded.byte_child_transform_guarded` uses timeless `G` and that guard to
obtain the child's aliveness guard from the actual `lctx_lft_alive_get_guards`
rule, then opens the actual `uniq_body_acc`. The loop retains `R` for its reads.
The accessor closes the same child body and returns `G`; only then does the
callback join `R/G` back to the single ordinary `Lchild`. The generic protocol
subsequently ends only the shorter child and restores the original parent.

The returned parent is typed at the original lifetime with observed current
`xor_bytes n x (byte_list k key)` and exactly its incoming observed future.
The current's byte list equals the independent `xor_result`: its first
`min(n,k)` bytes are the XOR prefix, and the remaining pad suffix is unchanged.
All bytes remain initialized and the pad length is exactly `n`. The theorem
quantifies over every finite `n,k`, including zero, 128, and values above 128.
It returns the same `E/L/I`, reusable shared view, and arbitrary frame. Existing
shared aliases can remain in that frame. It neither assumes future=current nor
ends the caller's outer lifetime.

The preparation, unique-payload opening, and loop credit indices are explicit
internal MODEL conditions. They do not establish a bounded source-level budget
or a Rust entry proof. The result vector is a pure finite computation; it does
not introduce a trusted transformation specification. The optional root vectors
source and all unfinished root writer proofs were unnecessary and were not read.

Validation and artifact hashes are in `STATUS.json`, `TRUST_INVENTORY.json`, and
the frozen replay. Raw compiler/kernel logs and objects stay in the private
trial directories. Development failures are proof-script failures, not semantic
negative controls.

This is a borrowed, MODEL-typed procedure law only. Incoming source-context
interpretation, native provenance/retag and control-flow correspondence remain
unproved: native 0; 44+3/101 open; HMAC 1/1 open; publication false; overall goal
incomplete.
