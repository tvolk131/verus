# Independent review of exact MODEL byte/cell and register integrations

Verdict: **accept the exact conditional MODEL statements; no actionable
implementation findings**. Review base is
`91eee431526e9006e2a58c46c8c73fd73a238f60`, containing register implementation
`703f9544` and byte implementation `240eb3dc`. Typed interface and normal replay
controller were committed before trials in `e8cf60ec`. Only this evidence
directory is changed. [STATEMENTS.md](STATEMENTS.md) records all 30 register
and 26 byte declarations, including all eight generated register eliminators.
Exact receipts, object identities, freezes and replay commands are in
[EVIDENCE.json](EVIDENCE.json).

## Register data dependency and physical frame

Line references here are to
`source/tools/research/native_register_segment/candidates/segment.v`.
`register_arguments` requires successful lookups of the supplied ordered names
(10–11). `RegisterCompute` requires the existing fragment `source_step`, not
merely a descriptor-shaped result, and changes exactly its destination (12–15).
`SegmentCons` threads the actual intermediate register map and state (16–20).
The only two selected instructions are unwrap 3→6 then Offset (6,2)→5 (41–44).
In `end_segment_exec`, the second lookup of 6 is proved from the first insertion,
and the length lookup of 2 survives that insertion (46–60). This closes the
MODEL data dependency; the pointer is not independently resupplied to an
unrelated second edge. Both output lookups are exact (63–66). Registers other
than 5/6 are unchanged (68–74), including input 2 and 3. Generic step/run
inversion proves complete physical-state equality (22–37).

`end_code` is one function, with an actual lexical `r6` binding that feeds the
Offset operation (76–79). Its full application performs four deterministic
pure primitive reductions: outer beta, projection beta, let beta and Offset
(81–98). `PureExec` in the pinned `iris/program_logic/language.v:224–235` includes
reducibility for every state and determinism with equal state, empty observations
and no forks; it is not just an existence claim or AST equality.
`wp_end_code` retains arbitrary `R` once (102–109). The combined theorem uses
the same p,n in the source run and whole target application, relates returned
location to register 5 and retains R (111–124). It introduces no cell ownership.
The source σ is a parameter of the pure fragment fact; this theorem is not a
joint native/target state relation or a new adequacy theorem.

The imported `fragment.v:15–30` explicitly requires a positive numeric base,
extent≤isize::MAX, base+extent≤usize::MAX, an in-window cursor, nonnegative
n≤isize::MAX and cursor+n≤extent. The same `offset_valid p n` is present in
both segment execution and combined theorem (48,112). These finite conditions
remain separate from the target's unbounded `(block,Z)` location arithmetic:
`lang.v:245–246` has an unconditional Offset rule, and the target-only lemmas
rightly have no finite-address guard. An endpoint may be one-past; none of this
segment dereferences it. A descriptor's numeric address is not physical
ownership, provenance, liveness or a Rust pointer interpretation.

The authenticated primitive source pins select constructor bb2:1 and bb2:2:
`_6 = Transmute(copy _3)` and `_5 = Offset(copy _6, copy _2)`. The source-only
review authenticates both source JSON objects and successful extraction receipt,
recomputes the existing reifier's 8/2/34 selected/unreachable/unsupported coverage,
and confirms exact generated `reified.v`. Its seven deepcopy-only controls
reject. These are untrusted identity/shape checks. No earlier raw-reference,
StorageLive/Dead, reborrow, retag, call, aggregate, source-phase or cleanup gap is
crossed by an identity rule. Actual source locals at those program points and
incoming Rust context remain obligations.

## Independent Boolean XOR, actual cell effects and move19 scope

Line references here are to
`source/tools/research/native_byte_xor/candidates/native_xor.v`.
The four definitions (9–25) use Z.odd/div2, Boolean xorb and positional
addition/multiplication; none calls Z.lxor. Length/range/decoding lemmas (27–47)
cover arbitrary widths and all Boolean words, not samples. The low-bit and
shift laws for literal Z.lxor establish full decoded-word agreement (48–56).
`scalar_xor8_lxor` then proves equality for every pair of unsigned bytes
(57–62), using the already checked range lemma from ModelByteXor. Independent
`scalar_xor8_range` holds for all integer inputs (63–68). The domain is explicit:
for nonbyte integers this function is an eight-bit result, not a claim of
unbounded Z.lxor agreement.

`scalar_xor8_pure` and `scalar_xor8_normalizes` concern only `xor_call a b` under
the two byte bounds (70–77). The imported concrete ModelByteXor operation is
the existing beta plus two case selections, exactly three primitive steps.
There is no added XOR language primitive. These lemmas do not assert that
`cell_assignment` or the whole native assignment is pure.

`cell_assignment` actually reads SC at p, supplies the read result as the first
argument of the actual byte_xor function, supplies b as the second, and writes
SC at that same p (81–84). The WP requires the old cell, one explicit £1 and
arbitrary R (116–120). Its read uses the supplied cell and gets it back
(125–127); its scalar call is proved from pure byte facts, with cell, credit and
frame retained in the continuation (128–131); its store receives that same cell
and the one explicit credit (132). The post returns updated ownership, the
unchanged R and byte range (133–134). No copied register/pointer grants a cell,
and no ownership is made persistent or duplicated. The baseline SC read rule
returns its token (`lifting.v:440–451`); the SC store requires the old token
and £1 and returns updated ownership (`lifting.v:549–560`). The operational
rules read the heap at that location and store by updating that location
(`lang.v:261–263,276–281`). This is a conditional MODEL resource proof, not a
proof that native reference accesses use these ordering/provenance rules.

`without19` and `prepare_assignment` are ordinary Coq data (88–104). A present
19 yields the expression with exactly that scalar and a returned map with 19
unavailable; missing 19 rejects; all other indices are unchanged. The packaged
WP retains all those facts AND still requires the original physical resources
and both byte bounds (136–148). The original pure rs remains a reusable Coq
value: no linear/native invalidation, register machine execution, pointer-local17
lookup or global single-use claim is proved. This limited bookkeeping is stated
in the implementation report and is acceptable in the exact interface scope.

The byte controller authenticates SOURCE.json, source-results, both successful
outer/inherited source receipts, the raw JSON hash and distinct canonical hash
before selecting bb7:s6. The exact enum is
`(*_17) = BitXor(copy (*_17), move _19)`, with u8 places and &mut u8 at 17.
Its full-payload match rejects extra fields and nine in-memory controls reject.
Only one payload in the 408-operation graph is identified. The selector and its
Python checks remain untrusted; none supplies a Coq semantic premise.

## Evidence authentication, normal replay and trust

Both committed controllers and the shared ArtifactGuard, LoggedCommands,
receipt helper, assumptions/kernel parsers, environment helper and all pins
were read. The review requires exact worker receipt hashes, retained freezes,
all returned proof source/object hashes, every recorded input identity and
retained JSON hash. Historical checkout paths in receipts are rebased by
repository-relative name into this checkout; none is accessed. Only the final
completed worker outputs and explicitly pinned dependencies were read.
Earlier excluded experiments and the parent's live byte replay were not read.

Readiness AND the successful durable baseline receipt are checked separately,
with successful version/build/kernel stages, immutable foundation commit
115a618d, 523 sources/284 objects, tool binaries, loadpaths and GMP manifest
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
The byte lane also freezes the Coq/OCaml/stub/GMP external inventory and the
completed 26-constant ModelByteXor and 8-constant ModelBufferTransform libraries,
including exact receipt/source/object identities. Register imports bind the
completed primitive proof receipt and all five objects; the independent
source audit additionally authenticates its outer receipt and all four proof
sources against this checkout. Dependency status booleans alone do not suffice.

Fresh normal `coqc -q` and full recursive `coqchk -o` run in separate new private
module directories, without exclusions or admission flags. The register
controller's silent check is supplemented by a printed type/body/module
inventory and another verbose recursive check. The compiled module print inventories all three
new inductives and their four constructors; the complete object is recursively
checked. Coq 8.20 logs only the 23 constants, including eight generated
eliminators, plus 60 markers each in Assumptions and ReviewTerms.
The byte check covers all 26 constants and 52 markers each in Assumptions and
ProofTerms, with no datatype or Program obligations. All five returned worker
objects, all returned proof sources, byte printed-term hashes, and the raw
30/26 closed assumption reports match the frozen worker evidence exactly.

Every candidate declaration is closed under the global context. The recursive
context contains only the two inherited baseline logical axioms, dependent
functional extensionality and Eq_rect_eq; neither occurs in an individual
candidate assumption report. Summaries report predicative Set, no rewrite
rules, no type-in-type, no unsafe (co)fixpoint reliance and no assumed positivity.
No axiom, admission, primitive, trusted contract or unsafe mode was added.

Input/snapshot/product identities remain guarded through the final checks.
All raw logs, copied proof sources, `.vo` files and compiler auxiliaries remain
in private `/tmp` outputs. Review-tooling failures are listed separately in EVIDENCE.json: the first
verbose-log parser expected nonexistent inductive log lines; the next attempt
was stopped after an offline print-format check exposed another parser issue
(opaque Qed constants print as Parameter in a module signature). The corrected
controller uses the compiled module print for datatype inventory and both
Definition/Parameter labels for checked constants. This is not a new source
Parameter or axiom. Fresh final validation reruns all proof commands under a
new freeze; the earlier failures are not semantic rejections. No source
edits, global tool/config changes, shared build, Rust build, filesystem control
experiment, subdelegation, push or PR occurred.

## Adequacy and remaining obligations

Neither reviewed module adds an initialized-heap adequacy theorem for its
whole program. The cell result needs supplied old-cell ownership/credit; it
cannot be instantiated as an empty-heap read merely from a raw pointer. The
source segment has no reads, and its pure execution does not give allocation
or dereference safety in Rust. Existing imported MODEL adequacy theorems (and
the foundation's `lrust_adequacy`, which starts from its own empty model state)
do not establish native execution or final original C/F observations here.
WP is ordinary partial correctness; do not turn the cell WP into a new total
native termination/effect theorem. The scalar and register PureExec lemmas do
supply their stated finite MODEL executions.

The segment descriptor gmap and byte adapter scalar function are distinct
MODEL interfaces. No theorem transports the constructor result into writer
local 17 or composes the constructor and cell assignment as an original Rust
execution. Acceptance of both modules does not supply that missing connection.

Actual native correspondences remain **ZERO**. Incoming Rust descriptors and
physical context, original provenance/ancestor loans, reference accesses,
register-state simulation, source phases, whole constructor/writer execution
and original C/F observations remain unproved. All **44 minimal + 3 phase /
101 writer effects** remain open; full frozen HMAC prefix **AND** suffix 1/1,
canonical 6/0 (shared observer causally redundant there) are inherited markers,
not recertified here. publication=false and goal incomplete. This acceptance
adds independent evidence for exact MODEL prerequisites only.

## Final receipt and replay

Final successful review receipt:
`/tmp/native-byte-segment-review-17a8c04d-trial003/receipt.json`

SHA-256: `814f5459ca72bf9448ae704d744b73cd0984518df1e86b7875903c17cdef206c`.

Source-only receipt SHA-256:
`745e53dab1ced3cd82586eb995e094e0fca0ec635a1b33004eaf76c36904410b`.

```sh
python3 -B evidence/native-byte-segment-review/review.py \
  --freeze evidence/native-byte-segment-review/trial003-freeze.json \
  --output /tmp/native-byte-segment-review-NEW
python3 -B evidence/native-byte-segment-review/source_audit.py \
  --freeze evidence/native-byte-segment-review/source-freeze.json \
  --output /tmp/native-byte-segment-source-review-NEW
```

Use new output directories; the retained trial directories already exist.

All five returned objects match the frozen worker evidence:

| Module / object | SHA-256 |
| --- | --- |
| `native_register_segment/Assumptions.vo` | `b476936a01edad5d11aafb9798d9849834b5a8ca762a9e7992bbed981eae37af` |
| `native_register_segment/segment.vo` | `1d854264d06e2f547d719638169fe70fbd7822ad3d0a363b9b5b9def9890c334` |
| `native_byte_xor/Assumptions.vo` | `799dec12b6d566f0b145911f4ababd39739e37f449d21d7dbf20cdb36cdb7d22` |
| `native_byte_xor/ProofTerms.vo` | `fb2f7ab1bd0013c38cef64b02ffc08030d8122cedb2b02d709464f26518cee68` |
| `native_byte_xor/native_xor.vo` | `de23bd93791532fe6087fb83710397b0cdb9c85bde0f354deaea95ac047a6efb` |
