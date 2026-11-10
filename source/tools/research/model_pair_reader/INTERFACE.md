# Checked pair-setup to reusable shared-reader handoff

Compose actual same-Outer/TypeG pad/key initialization with the checked shared
reader session, without a caller-supplied owner, PointsTo, guard or read token.
Pair setup returns a shared tctx with existential synchronized depth d/g; the
reader consumes g+1 preparation credits and g+1 per read. Isolate and prove the
budget interface rather than assuming that existential depth has a public bound.

Prefer a proved fixed-depth shared view (the read library supports d=1,g=0), or
a proved uniform bound computable from the actual setup and input lengths.
Preserve the genuine checked Share operation and exact key view. If strengthening
the handoff requires a new derivation of its actual typed rule, keep it checked
in this directory; do not modify frozen sibling libraries or add a trusted rule.
Do not demand credits indexed by an opaque fresh existential as a new public
application requirement. Public model adequacy supplies credits internally.

Handoff must retain pad model_reference/Lpad, exact outside frame, same Outer,
and a persistent key_reader with its linear R, restoration half G and closure to
ordinary Lkey. Support all key lengths, empty key q=p, same-block disjoint
interiors and shared aliases. Root will thread R and per-read/store credits through
the changing pad callback, retain G outside it and join R/G afterwards.

If possible, also prove an actual same-heap whole-key observation adapter from
the returned genuine shared capability, ordinary Lkey, heap_ctx sigma and an
explicit sufficient reserved budget. Return the same sigma/context/capability,
exact initialized key values, arbitrary Frame; do not extract an exclusive key
vector or assume final read-state counters zero. Empty keys need no cell access.

Only this new directory is owned. Freeze before trials, normal compile, full
assumptions/obligations inventory and recursive kernel checking with exact pins.
No baseline/vstd/compiler/shared edits, new axioms/primitives/unsafe shortcuts,
parent/sibling checkout reads, application preconditions or source changes.
Fresh model initialization is not actual Rust entry/provenance/ancestor history.
Native correspondences remain zero; publication and full-goal completion false.
