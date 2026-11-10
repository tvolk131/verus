# Constructor erasure: typed boundary, no source certificate yet

Owned scope: this new directory only. Base: 40a8e281. Early interface is
committed before proof expansion. The frozen constructor is unchanged:
`_1: &mut [u8;128]`, `_4 = &mut (*_1)`, activation at bb0[4],
`_3 = move _4 as &mut [u8]`, `iter_mut(move _3)` into `_2`.
Static loan0 belongs to `_4`; it does not identify the incoming dynamic tag.

`candidates/Interface.v` defines an explicit **uninstantiated** source-semantics
signature, independent entry predicate, physical map and outcomes including
faults. A source instance must independently interpret real Rust entry validity,
operand evaluation, operational transitions and phase events. Its `lawful_entry`
field is a required semantic definition, never a certificate, future-run filter,
Iris ownership, byte equality or successful-proof bit. No source instance exists
in the supplied pinned fragment library. Merely supplying arbitrary functions
of these types would not establish their Rust meaning.

Planned checked composition takes a proof-valued invariant/certificate,
including progress and ALL enabled outcomes, to fault exclusion and preservation
of the original observable physical byte map on normal/unwind exits. Phase and
cleanup transitions cannot disappear through a normal-path run filter. The
invariant may existentially hide history. No history is required empty, no
reference is required at allocation offset0, and no byte is required zero.
A certificate is a theorem argument, not an installed source assumption; until
an actual-source instance and its certificate are proved, source/native status
remains zero. This lane will attempt the first missing root law against the
completed descriptor semantics and retain a checked exact blocking obligation
if the foundation cannot express it.

## Needs routed to integration

1. Independently justified `source_semantics` instance for this pinned MIR/phase
   graph; in particular legal entry validity for arbitrary incoming ancestor/
   protector histories, two-phase root reborrow, activation, unsize, raw-reference
   formation and storage/retag/phase transitions. This is this lane's attempted
   source boundary, not a request for a new application premise.
2. Original-reference C/F exit lane: a proof mapping a `Normal s'` source outcome,
   the actual incoming `_1` reference and final `physical_map` observation to the
   original Verus F view. Do not identify caller-parent future with current.
   The physical byte map is `address -> option Z`; `address` retains allocation
   identity and signed byte offset. `reference` also retains the actual tag.
3. Pointer-distance lane is independent of this constructor milestone; no
   pointer-distance law is needed or assumed here.

Only pinned completed artifacts and current committed documents are read.
Normal coqc, complete declaration/constructor/projection/eliminator assumptions
and recursive coqchk with retained input/product guards are required for all
claimed new Coq results. Native0; 44+3/101 open; publication=false; goal incomplete.
No production hook, C/F law, application requires, trusted native summary,
unsafe-check bypass, rewrite, subdelegate, push or PR is authorized here.
