# Array-to-slice carrier: partial MODEL interface

This lane models the data operation inside the frozen root `bb0[4]` unsizing
cast. It does not interpret the complete Rust statement. The pinned interpreter
constructs a pointer/length pair and performs machine-dependent reference
validation, including retagging. The latter is an open source obligation.

`carrier_store s src dst n` reads an initialized `RefValue` from an immediate
source local, requires a distinct uninitialized immediate destination and a
representable 64-bit length, then stores `SliceValue r n`. It preserves the
source local, physical cells, private storage, incoming history, storage trace,
original reference and control location. Control does **not** advance: the
validation/retag/write-hook phase has not executed.

This deliberately follows the carrier portion of the pinned interpreter, whose
operand evaluator currently reads `Move` and `Copy` identically. It does not
assert Rust move invalidation or permission duplication. A slice carrier is
data, not a second right to use the reference.

The generic operation covers representable lengths, with 10/128 corollaries.
Only the selected 128-byte root statement is linked to the frozen source graph.
Generic length 10 is not a canonical native certificate. No permissions,
initialization, provenance validity, or C/F equations follow from carrier data.

The complete source transition still needs actual operand evaluation, reference
validation and retag effects, local write hooks, phase/control advancement and
source-to-model interpretation over lawful incoming histories. All native
statuses remain zero/open, publication=false, and the full goal is incomplete.
