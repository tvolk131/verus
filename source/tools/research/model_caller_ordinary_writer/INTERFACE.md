# Ordinary-memory writer in an existing model caller context

Adapt the checked caller-preserving MODEL procedure `2c4795b5` to the checked
ordinary reader/loop `212044b1`. Reuse its generic guarded child accessor,
computed typed XOR vector and single-context lifetime protocol; do not duplicate
their proofs or substitute a new algorithm. Only the concrete loop's memory
orders and the corresponding checked reader and internal credit index change.

The pretrial `caller_xor_na_contract` retains existing typeG, E/L/I, mutable-pad
type ownership, shared-key view and arbitrary ancestor Frame. It returns the
original parent at the same lifetime with exact min-XOR current bytes, unchanged
suffix and the original future unchanged. Do not end the caller's lifetime,
initialize a fresh world or assume parent future=current. No physical PointsTo,
exclusive key vector, separate copy of L, range/output summary, or native-entry
premise may be introduced. All finite n/k and contents remain quantified.

Prepare the Na reader session after child reborrow; keep R for actual reads
while G guards unique child access; close that same child payload and join R/G
before ending the child. The explicit MODEL credit index is
`(g+1) + (1 + min(n,k)*3*(g+1))`. This is a sufficient internal symbolic typing
index, not a supplied credit requirement or runtime bound for the original
Rust caller. Type/lifetime aliveness premises retain their original model scope.

Freeze the typed expression/contract before trial, compile normally, inventory
every declaration/helper/assumption and recursively kernel check without
exclusions. Pin completed libraries, baseline/tools/GMP and preserve final
artifact guards. Failed proof-development trials are not semantic evidence.

MODEL only: no actual Rust input interpretation, native pointer/retag/cleanup
simulation or C/F import. Native0, 44+3/101 open, HMAC1/1 unproved,
publication=false and goal incomplete. No production/compiler/vstd changes,
new axioms or trusted summaries, source rewrites, installations or push/PR.
