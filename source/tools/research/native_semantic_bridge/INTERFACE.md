# Borrow-scoped semantic bridge: interface milestone

Publication is false. This new lane owns only this directory. No native fact or
production hook is installed. The original native-drop positive is still 0/1;
the full frozen HMAC writer is still 1/1; all 101 native effects remain open.

The dependency is unmodified VerusBelt at
115a618dd35ff8337112cf2081a7f5391d3973d5, supplied read-only by the parent.
The parent owns its toolchain and baseline build. New proofs compile separately.

## Exact first semantic interface

`candidates/loan-v1/borrow_handoff.v` starts with **the defined**
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

The initial no-write accessor deliberately restores the exact original `m`,
including its prophecy variable and index. `uniq_resolve` then relates its future
to its current under an alive lifetime. The dependency's read/write rules support
access through that same pointer; future updates must retain the same location
and prophecy identity. These claims are about lambda-Verus, pending compilation,
not claims about Rust's native raw-pointer casts or IterMut.

## Byte-array and adequacy work

Instantiate with 128 initialized bytes (all 0..255, arbitrary values), not
uninitialized memory or a supplied owned allocation. VerusBelt physical cells
hold mathematical values, rather than Rust byte-addressed objects. A cell model
instance does not prove the Rust byte-layout/provenance/retag simulation.

A closed type-soundness harness must instantiate the actual dependency theorem;
a generic conditional appeal to soundness is insufficient. Print Assumptions and
coqchk must succeed before any model proof is marked kernel checked. The parent
is building the unmodified dependency; current executable proof status is
`not_run_dependency_build_pending`. No failed script is a semantic negative.

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
