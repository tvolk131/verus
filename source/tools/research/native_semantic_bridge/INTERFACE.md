# Borrow-scoped semantic bridge: interface milestone

Publication is false. This new lane owns only this directory. No native fact or
production hook is installed. The original native-drop positive is still 0/1;
the full frozen HMAC writer is still 1/1; all 101 native effects remain open.

The dependency is unmodified VerusBelt at
115a618dd35ff8337112cf2081a7f5391d3973d5, supplied read-only by the parent.
The parent owns its toolchain and baseline build. New proofs compile separately.

## Exact first semantic interface

`candidates/loan-v12/borrow_handoff.v` starts with **the defined**
`ty_own (&uniq{κ} ty) m d g tid [FVal #p]`, not an uninterpreted
RefInterpretation, SourceExec, or owned-heap predicate. This is the resource
interpretation furnished by a semantic input typing derivation. It contains the
original reference's physical value and unique prophetic borrow. The theorem
opens that borrow to its initialized typed contents at exactly `uniq_bor_loc m`,
and returns a linear restoration wand. Opening consumes the borrow resource;
restoring needs the very heap payload, time ticket and original lifetime guard.
No allocation/freeable resource, additional PointsTo argument, extra reference,
or duplicable ghost authority is acquired. The location is arbitrary: no
allocation-base/offset-zero restriction is imposed.

The public `source_context_access` derives its lifetime guard from ordinary
`elctx_interp E` / `llctx_interp L` and `lctx_lft_alive E L κ`, rather than
asking the client for an extra guard resource. These are model typing-context
judgments, not application preconditions. `byte_array128_context_handoff`
specializes the payload to exactly 128 initialized byte cells, exposing the
actual defined heap points-to predicate and all-byte range facts. The Rust input
has NOT yet been given a checked semantic typing derivation in this model.

The initial no-write accessor deliberately restores the exact original `m`,
including its prophecy variable and index. `uniq_resolve` then relates its future
to its current under an alive lifetime. The dependency's read/write rules support
access through that same pointer; future updates must retain the same location
and prophecy identity. These claims are checked about lambda-Verus; they are not claims about Rust's native raw-pointer casts or IterMut.

## Byte-array and adequacy work

Instantiate with 128 initialized bytes (all 0..255, arbitrary values), not
uninitialized memory or a supplied owned allocation. VerusBelt physical cells
hold mathematical values, rather than Rust byte-addressed objects. A cell model
instance does not prove the Rust byte-layout/provenance/retag simulation.

`empty_heap_offset_adequate` instantiates the actual dependency's operational
adequacy theorem for an explicit model-limit control: Offset is safe even on an
empty heap. This is deliberately NOT adequacy for the Rust handoff. A handoff
soundness harness/source typing derivation must still instantiate the actual dependency theorem;
a generic conditional appeal to soundness is insufficient. Print Assumptions and
coqchk must succeed before any model proof is marked kernel checked. The parent
is building the unmodified dependency; current executable model proof status is
`normal_compilation_and_recursive_kernel_check_passed` (loan-v12). The actual
Rust-native handoff/source connection remains unproved. No failed script is a semantic negative.

## Required source connection (still open)

Fresh normal rustc 1.99.1 extraction must bind the six-instance/44-operation
constructor/drop graph and original borrowck operand chain. For each operation,
identify an actual model step and a simulation proof, or reject it explicitly.
In particular byte offset/provenance, valid-range NonNull, retags, fat metadata,
monomorphization versus pre-borrowck FakeRead/cleanup/unwind, and the observation
of the SAME original array's future must be checked. Hash/size/name equality is
only identity evidence. The inherited generated 115/0 trace is not this proof.

No foreign-kernel exit code, status label, or theorem hash may authorize Verus
publication. Future integration requires a checked C/F correspondence and an
independently reviewed inactive module/hook. The full frozen HMAC proof,
untruncated slot inventory and arbitrary key domain remain parent acceptance
work. Disjoint byte subregions in one allocation remain permitted.
