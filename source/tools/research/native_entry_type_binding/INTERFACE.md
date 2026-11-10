# Compiler-bound byte-reference entry conditions

This lane checks a concrete open condition in the source-entry audit: the
compiler's `Unpin`, `UnsafeUnpin` and `Freeze` answers and layouts for the actual
byte-array/slice parameters. It does not interpret incoming physical permissions.

The driver queries the pinned rustc 1.99.1 / `58cf8f9c35b355199727627d07ae34a4e2846d67`
after ordinary analysis. It reads the unchanged frozen native HMAC fixture and
the accepted canonical candidate through normal Verus macro erasure. An owned
probe separately requires all three traits for **every** `[u8; N]`, without
adding trait bounds to that function, and for `[u8]`. Negative trait requirements
must fail ordinary checking. `RUSTC_BOOTSTRAP=1` exposes the unstable compiler and
marker-trait interfaces; it does not disable type, lifetime or trait checking.

Inputs/tools are frozen before the first trial. Outputs, compiler diagnostics
and hashes stay private. This is compiler-query/typing evidence, not a Coq
theorem, Miri execution, source-safety proof, or native certificate. In particular,
traits/layout do not imply `valid_block`, live allocation, initialized runtime
bytes, provenance/ancestor/protector rights, a successful argument copy, or C/F
correspondence. Unsized slice layout is a minimum layout, not a zero-length key.

The full original goals remain required. Canonical 6/0 and full HMAC 1/1 are
inherited, not rerun by this lane. Native=0, all44+3/101 obligations open;
publication=false; goal_complete=false. Production is unchanged.

The additive carrier-layout epoch distinguishes the parameter's own backend
representation from its pointee layout: Scalar versus ScalarPair, with actual
size/alignment. It selects no memory/provenance interpretation and proves no
argument-copy path. The earlier type-only receipt remains historical evidence.
