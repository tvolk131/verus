# Supported native constructor operations reduce in the existing heap language

This lane implements Coq source-fragment semantics, a translation into the
unchanged VerusBelt lrust language, and checked operational/resource simulation.
It adds no native Rust theorem or production fact. Only this directory is edited.
The pretrial signatures were committed as c6116d49; implementation checkpoint
54480a6e includes the first normally compiled operational/resource proof layers.
Final validation identities and failed development trials are indexed separately.

`fragment.v` defines typed descriptors (slice, wide raw/NonNull, thin raw/NonNull,
usize and bool), supported primitive operations and an explicit source-step
relation. This relation has no unsupported-step constructors. `translate` rejects
unsupported opcodes and wrong word counts. It is a register-word translation,
not an allocation of a fat pointer object or a new physical unique reference.
Metadata/projections use actual lrust lambda applications; byte Offset uses the
existing OffsetOp; the u8 CTFE false branch uses the existing Case expression.

`supported_operation_head_simulation` proves, for ANY source step, that every
translated output word takes one actual lrust head_step to the exact represented
source output, with exactly the same heap/lock state, no events and no spawned
threads. Its reverse condition proves that every head reduction of that
expression produces that same output and unchanged state. It is not merely an
AST inventory. The target uses the existing language constructors and semantics;
there is no independently authored target heap machine.

`supported_operation_physical_frame` additionally proves WP preservation of an
arbitrary separating iProp R. In particular R may include the single original
model_reference and an exact outside heap frame. The composition lemma
`offset_then_original_read_resolve` computes an end pointer, reads the original
first cell, and uses the existing ModelInputContext open/close/release theorem
for that SAME reference. It retains the frame, local context, ordinary future
resolution observation and lifetime-end restoration continuation. No new loan
or physical owner is constructed from pointer metadata. The protocol needs £2,
unchanged from the imported read/resolution lemma. Offset and its sequencing
binder are two additional real model steps, followed by the original read.
The offset result is not dereferenced. The pure numeric representation checks
and the existing unbounded model OffsetOp remain distinct.

A window carries an arbitrary model base location, a native numeric base address
and allocation byte extent. These bases are related by an explicit convention:
source cursor k maps to exactly model base+_l k and numeric address base+k.
Numeric validity requires nonnull positive base, extent bounded by isize::MAX,
and base+extent bounded by usize::MAX for the frozen 64-bit layout. A valid cursor
may equal allocation extent. Slice and Offset guards constrain it to that same
containing allocation; offset preserves the block and obeys the numerical bounds.
`interior128_offset` permits ANY positive start with start+128 strictly before
the allocation end. `one_past_not_array_dereferenceable` excludes the array's end
from its own dereference domain even when the containing allocation has a suffix.
No allocation-base-only/zero-byte restriction or array-subobject freeability is
introduced. Theorems neither read nor restrict the containing heap's other cells.
These are conditions on a MODEL representation, not new application requires.

The structured selector `reify.py` authenticates source-bindings.json's receipt
and both native/borrowck JSON objects before inspecting actual MIR enums. It
checks exact destination/source locals and types, projections, enum operations,
64-bit layouts, NonNull's transparent one-field nonnull representation, resolved
constructor identity and CTFE false branch. There are eight supported selected
operations: metadata, wide wrap/unwrap, thinning, thin wrap/unwrap, byte Offset and
CTFE branch. Two bb1 operations are CTFE-unreachable; 34 of the frozen graph's
44 operations remain unsupported. The three empty drop-glue instances are query
observations, never extra executed destructors. Whole-graph translation remains
rejected/incomplete; selected fragment edges do not silently bypass a source gap.

The producer and its hash/shape checks are untrusted. Coq verifies the meaning
and execution of the supported fragment operations, but the assignment of that
fragment meaning to a Rust MIR operation is not a metatheorem about Rust. In
particular the raw-reference-creation operation before wide NonNull is an
explicit boundary, as are the root reborrow/unsize, storage lifetime, ownership
aggregate, calls, source phases and incoming/outgoing reference interpretation.
Retag/reservation/activation/FakeRead/unwind/drop are unsupported, never identity
rules. A pure descriptor can be copied, but grants no physical ownership; the
resource countertheorem rejects duplicating the nonempty byte payload.

The original Rust incoming loan, liveness/provenance, suspended ancestors, valid
representation derivation from every original input, original executable entry,
and final same-reference Verus C/F observations remain unproved. Reusing an
already constructed MODEL reference does not close any of those native gaps.
Native correspondences=0; 44 minimal + 3 phase simulations /101 effects remain
open. Full frozen HMAC=1/1 and canonical=6/0 are inherited, not rerun. Their domains,
full predicates, checks and bodies are unchanged. publication=false.

The replay uses only the ready pinned Coq8.20.0/VerusBelt115a618d runner and GMP
manifest. It authenticates the ready and baseline receipts, all 523 sources/284
objects, tool/loadpath identities, exact shared support files and the three
completed read-only prerequisite libraries. The model-input receipt is pinned to
4c6783ad and every prerequisite object/source is checked against its authenticated
receipt. The retained GMP manifest is compared to the committed baseline pin.
All new proof sources are copied into fresh private trial directories and guarded
through compilation and recursive coqchk; final guards precede the success flag.
No installs, shared compiled-input writes, race/substitution experiments, push,
PR or subdelegation occurred. New parent integration and independent review are
still required.

The final concrete instance is `reified.selected_edges_head_simulation`.
It checks the eight selected primitive edges at length 128, from any numeric
window with a strict interior start. These are deliberately independent typed
edges: the slice descriptor used by Metadata and the raw descriptor used by
wide NonNull are NOT joined by a fabricated raw-reference-creation step. Proving
that the actual source locals hold those descriptors at those MIR program
points is part of the unproved source execution/context interpretation.

`physical.interior128_offset_read_adequacy` removes the resource premise from
the MODEL demonstration. From arbitrary raw h with exact array cell lookups and
arbitrary ranged zs of length 128, it uses the accepted initializer to construct
one reference internally. It proves adequacy of the explicit model expression
`Skip;; let end = base+128 in Read ScOrd base`, yielding the original head byte.
This executes five model steps (two setup beta steps, Offset, binder, read) and
uses the accepted £3 initialization convention: £1 for lifetime creation and £2
for read/release. Prefix/suffix and unrelated heap cells are unrestricted. The
resource composition preserves the exact outside frame; ordinary adequacy
concludes only the actual read result. The numeric window is pure metadata and
does not itself prove allocation liveness or create any cell in h. This model
expression is displayed explicitly and is not presented as the Rust source body.

Final trial018 passed normal compilation of four modules and the assumptions
module, then recursive coqchk of all five objects and their dependencies. The
121-entry Print Assumptions inventory covers every new definition, inductive,
constructor, theorem and all 24 generated elimination principles. There are zero
new Program obligations (no Program definitions). 119 declarations are closed;
only the read/release composition and model adequacy inherit existing dependent
functional extensionality. The recursive dependency summary permits only the
existing FE and Eq_rect_eq axioms, with no unsafe recursion/positivity,
type-in-type or rewrite rules. Exact per-declaration results, object/log/receipt
hashes and timing are in results.json. All final guards passed before success.

Seven structured payload controls reject retag, wrong original operand, wrong
arithmetic, cleanup, CTFE=true, wrong layout and altered NonNull valid range.
Coq additionally proves the unsupported-step/translation, wrong-kind,
out-of-window, one-past-dereference and physical-duplication countertheorems.
None of the 16 failed development trials is counted as a semantic negative.
They retain exact pretrial freezes, candidate source copies and failure receipts
under the paths in development.json. Trial014 is the earlier checked milestone;
trial018 is the final checked result. The final freeze is byte-identical to
trial018's pretrial freeze, and replay checks the generated Coq source against
the authenticated structured producer before compilation.

Reproduce from the repository root, using a fresh output directory:

```sh
python3 -B -O source/tools/research/native_mir_lrust_refinement/check.py \
  --freeze source/tools/research/native_mir_lrust_refinement/final-freeze.json \
  --output /tmp/fresh-native-mir-lrust-replay
```

Integration should cherry-pick the lane commits in order: c6116d49, 54480a6e,
0ca503ea, then the final evidence commit. New source files, generated compiler
auxiliaries and all proof products are confined to this lane or its private
trial outputs; the completed prerequisite inputs are read-only. This is a new
supported-operation theorem, not completion of the full native correspondence
or either original writer goal.
