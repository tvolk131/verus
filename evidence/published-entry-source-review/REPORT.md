# Published entry candidate: source-only review, UNVALIDATED

The twelve declarations are coherent **conditional model algebra by source
inspection**. No source-level contradiction or definite type error was found.
They cannot yet be accepted as compiled proofs, source certificates, or native
correspondences. The decisive missing operation is **callee argument `_1`
initialization by `pass_argument` under FnEntry**, before root bb0[0], rather
than the root reborrow alone. It must bind the actual caller operand to the
callee and justify each write-grant/protector check on its referent. A later
prefix relation must preserve this normalized head up to root bb0[3].

Early scope commit: `798df7cb`. Base: `b008f1874b59dafb1e0a25ebaa938a0f0169a37a`.
Only this NEW directory is written. No build, Coq/Miri/Rust execution,
subdelegation, push/PR, toolchain/global/credential change, worker inspection,
readiness polling, filesystem substitution/race experiment, or source edit
was performed. Git initially lacked author identity; the scope commit used
command-local `-c user.name=Codex -c user.email=codex@openai.com`, matching the
existing commit identity, without configuring the environment. Source reads
used this checkout or the explicitly authorized primary files. Paths inside
receipts were treated as labels and never followed.

## Authenticated evidence and primary citations

`authenticated-inputs.json` records exact SHA256/size for 41 inputs, including
the candidate/freeze, nine frozen SB files plus Git-authenticated lang_base.v, ten pinned Miri files, selected
compiler files, graph and metadata. `authenticate.py` compares committed pins
with base Git objects and named snapshot bytes. SB bytes also match their Git
objects at `d1471b10fe4cb4f506371b2e3acdd927d0a0c076`. Candidate SHA256 is
`557b30289bf080208eb58f64a88e7958d145112926567f98787edf1920b072bf`;
it equals delivered `136d8f82`. The historical `2ccf2962`/`d6b9bd6e` checkpoint
stats were read; current candidate is the source-corrected candidate003.
Status and metadata are authenticated at b008f187, not assumed from branch names.

The committed Miri inspector was run with Python `-B` into
`policy-inspection/`: success means exactly ten primary identities match, not
semantic import. Its original completed receipt was separately authenticated
at SHA256 `3f63a7d764f69031709136bf7415a568fc0fe3388d2c5b2efe6314973bda6d19`.
New official exact-commit `call.rs` and `marker.rs` bytes are retained here;
`downloaded-primary.json` records URLs, SHA256, sizes and Git blob SHA1. Both
also match the official GitHub contents API at the exact commit, byte for byte.
No source was fetched from a moving separate Miri repository.

Citation keys below refer to these exact primary sources; line anchors always
refer to these versions:

- **P** [published steps_progress.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/steps_progress.v):271–294, 752–822.
- **B** [published bor_semantics.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/bor_semantics.v):30–37 stack orientation; 272–366 grant/reborrow/retag; 410–416 RetagIS.
- **R** [published steps_retag.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/steps_retag.v):673–707 top-head; 805–817 nonempty result; 1061–1113 frame; 1673–1736 active-item preservation; 1769–1794 FnEntry membership.
- **I** [published steps_inversion.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/steps_inversion.v):870–892; **W** [steps_wf.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/steps_wf.v):952–969; **D** [defs.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/defs.v):10–49; **T** [type.v](https://gitlab.mpi-sws.org/FP/stacked-borrows/-/blob/d1471b10fe4cb4f506371b2e3acdd927d0a0c076/theories/lang/type.v):17–41,69–79.
- **C** [rustc call.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/call.rs):29–50,340–400,405–666.
- **S** [rustc step.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/step.rs):40–61,94–108,168,214–239,405–433.
- **L** [rustc place.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/place.rs):595–623,687–706,911–942; **K** [stack.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/stack.rs):495–585.
- **V** [rustc validity.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/validity.rs):561–725,1680–1727; **M0** [compiler machine.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/compiler/rustc_const_eval/src/interpret/machine.rs):44–57,198–204,546–587.
- **M** [Miri machine.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/machine.rs):1184–1193,1817–1857,1952–1960; **G** [borrow_tracker/mod.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/mod.rs):30–54,134–174,180–189,275–317,364–400.
- **SB** [current SB mod.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs):60–129,227–326,383–447,593–767,824–896; **ST** [current SB stack.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/borrow_tracker/stacked_borrows/stack.rs):190–260,366–378,427–436.
- **E** [Miri eval.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/tools/miri/src/eval.rs):179–181; **A** [core marker.rs](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/library/core/src/marker.rs):920–943,1014.

## Candidate shape review (not typing/proof acceptance)

| Proposed declaration(s) | Source-level assessment and limit |
|---|---|
| `mutable_top_block` | Per-position head is Unique or SRW, SAME concrete `Tagged p`; protectors/tails vary. It does not encode a parent graph, reachability, initialized bytes, live allocation, or Rust validity. Arbitrary tails in this predicate are further constrained by `Wf` when that is assumed. |
| `top_write_pre`, `top_block_valid` | P:271 permits witness index 0; conditions on strictly earlier indices are vacuous. Unique/SRW both grant write. FixedSize is definitionally freeze and size n (T). Domain membership follows from Some lookup. `cids` can be arbitrary here. |
| `bounded_reference_head_step` | P:805 argument order matches fns/state/location/tag/kind/type/retag-kind/BAR/BLOCK/WF. Nonempty call stack supplies BAR; list injection aligns the actual top call. W:952 takes step then Wf. This constructs an actual MODEL head step and keeps its scalar heap/call stack; no Rust local assignment. |
| `nonempty_reference_result`, `empty_reference_result` | R:805 requires positive model size and matches mutable reference. B:320 returns old tag/unchanged state/counter for size zero for all retag kinds. Correct intended historical behavior, mismatched to modern zero-size instrumentation. |
| outside-stack / active-protector frames | R:1061 and R:1673 take call-id before call-stack in their lemma binder order, despite `retag` taking stack before call-id. Candidate supplies the right order. Active preservation is item membership, not ancestry/whole-program permission ownership. |
| `all_reference_steps_keep_heap` / initialized vector | I:870 has seven existential witnesses and five conjuncts, including final-state equality. Candidate destructuring matches. A MODEL Retag on an already-result operand preserves heap, even for other pk cases allowed by its generic statement. No progress follows from this conditional step fact alone; no source outcomes are quantified. Heap equality implies its given scalar-vector frame. |
| `two_phase_128_progress`, `two_phase_10_progress` | Explicit `(fns : fn_env)` resolves the previously unused table binder. These project only retag progress under the same model premises. Neither proves the canonical source, the full pad-XOR writer, or any byte-layout bridge. |

Imports expose `head_step_wf` via exported steps_wf; the inversion lemma's
section parameter fns explains its explicit first argument. The concrete
lookup, induction-free index-0 argument, tuple/conjunction and call-stack
shapes fit the published statements on inspection. Potential elaboration,
notation/tactic availability (`have`, `done`, `lia`, ssreflect rewrites),
implicit-argument and conversion issues still require the matching checked
baseline and actual Coq run. This review does not diagnose a definite type
failure, add assumptions, or treat textual Qed as verification.

## Source-to-model premise matrix

| Source phase / concrete operands | Independently needed fact and model target | What the pins actually justify / pending condition |
|---|---|---|
| Original entry to extracted root `native_writer::xor_key`; argument `_1 : &mut [u8;128]` | Original arbitrary initialized exclusive 128-byte subobject, including interior subobjects, with same physical allocation/offset and original C/F observations. | graph.json operations 0–3 and loan record bind `_1` deref -> `_4`; loan0/region3 is static reservation at bb0[3], activation bb0[4], not `_1`'s tag. No input semantics or dynamic history follows. |
| Callee MIR selection/phase | Same actual function instance and entry protocol, ordinary argument 1, no shim/foreign/entry bypass; statement-phase mapping to extracted root bb0[3]. | M0:198 uses instance_mir; C has special closure/spread/varargs paths. Extracted borrowck/drop MIR identity is not a proof of matching Miri-executed or compiled native phase. Bind before claiming FnEntry happened. |
| Argument transport `caller_arg` -> `caller_arg_copy` -> callee `_1` | ABI/layout agreement, readable initialized reference carrier, callee storage live, preserved referent address/bytes, actual incoming provenance. | C:340–398 checks ABI then copies with possible transmute. `FnArg::Copy` is by-value; even `InPlace` first degenerates to a copy. S:405–433 selects Copy for Copy/constant and Copy or InPlace for Move based on memory/disjointness. Source by-value reference parameter does not mean a copied referent array. |
| FnEntry reference validation of copied `_1` | Actual hook invocation under FnEntry, not disabled validation or a no-op machine; type `&mut [u8;128]`, ordinary safe-pointer path with reset provenance true and may_dangle false. | C:531 encloses pass_argument. L:918–940 validates copy destination; V:604–649 invokes retag only on the appropriate branch and V:680 writes adjusted pointer back. E/M give default SB + shallow validation. M0's default retag hook returns None. Config/source implementation is not a normative Rust rule. |
| Type-directed FnEntry permissions | Pointee `is_unpin` AND `is_unsafe_unpin` true in the actual typing environment, positive extent. | SB:63–89 gives Unique/write/strong only on that conjunction; otherwise SRW/noaccess/no protector even for FnEntry. Byte arrays are structurally expected to satisfy both auto traits (A); an instantiated typing/layout binding still needs proof. Do not infer Unique for all mutable types. |
| **Earliest missing referent operation:** FnEntry `sb_reborrow(place, size, Unique/write/strong, new_tag, FnEntry)` from copied `_1` | For every touched byte, write authority of the incoming provenance with all displaced incompatible items' protectors inactive; matching live bounds and mutability. In a faithful historical representation derive `valid_block alpha cids l old (RefPtr Mutable) (FixedSize 128)` before normalization. | SB:600 checks bounds, SB:732–752 calls per-range grant, SB:400–406 performs access; SB:227–261 rejects active protectors when invalidated. P:271 spells the represented grant condition without requiring caller tag on top. Original initialization/type metadata alone does not establish it. This fact must be proved from independent original entry semantics, never added as a client requires. |
| Fresh FnEntry result -> actual `_1` | Fresh result tag represents same referent and becomes `_1`'s value; frame protector matches model top call, Wf and bytes preserved. | SB:845 allocates tag; SB:400–412 pushes Unique at source-vector end (top); V:680 stores adjusted pointer. Published list head is top (B:30–37), so relation reverses source stack order and maps tags/calls. R:696 can normalize a justified model FnEntry; see MISSING-LEMMA.md. This is a possible genuine protocol, not established coverage. |
| In-place argument / return carrier protection | Track separate caller storage for the reference VALUE (or return slot), distinguish it from the 128-byte referent, prove relevant storage separation or handle overlap. | After copying, C:396–397 protects original InPlace mplace; C:640–641 also protects in-memory return destination. M:1838–1855 retags/protects THAT place and writes uninit. It does not, by this hook alone, protect the referenced array; FnEntry validation does that separately. Carrier uninit is a real memory effect. |
| Remaining entry hooks and root StorageLive(2/3/4) -> bb0[3] | Preserve `_1`'s tag/head over referent, initialized bytes, and active frame after all arguments, carrier protection and frame hooks. | C:666 completes frame setup; S:103–104 and K:495–585 make locals live with local-write hooks. No source simulation is delivered. A past FnEntry in another frame or an intervening reborrow does not suffice; protection preserves an item, not necessarily its head position. |
| bb0[3] `_4 = &mut (*_1)` | Read actual `_1`; same pointee and extent; writable carrier `_4`; TwoPhase dispatch and all-result relation through validation/write/hooks and PC advance. | S:214–239 evaluates `* _1` with simple-deref validity skipped, forces allocation, makes reference, selects TwoPhase. L:609–618 reads its carrier; it does not get an extra full parent check there. L:692 writes before validation; S:55–61 advances only after success. Candidate Retag alone covers neither carrier operation nor errors. |
| Current TwoPhase with concrete known parent | New SRW/noaccess/unprotected child, correct insertion and unchanged payload under related state. | SB:65–74,413–445; published UniqueRef true also inserts SRW (B:310–314). At a represented top Unique/SRW parent, index-0 grant gives a top child. Away from that class, actual insertion may be below younger items. Top is sufficient, not necessary. No phase-activation theorem is supplied. |
| Wildcard / unknown-bottom provenance | Explicit exposed-tag/unknown-range relation or justified elimination through this exact entry path; preserve original domain. | ST:190–245 admits wildcard exposed matches and unknown bottom (including concrete tags below its limit). SB:423–435 forgets the stack for noaccess SRW grants from wildcard/unknown matches. Published tags are only Tagged/Untagged; Untagged is not a wildcard abstraction theorem. A successful positive Unique FnEntry CAN push a concrete known top even from such parents, but proving success and representing the retained unknown context remain open. |
| Ancestors / protector contexts | Preserve arbitrary represented tails, call-stack membership and protector identity; account for active incompatible younger grants, strong/weak distinctions and call cleanup. | Top avoids displacing anything above parent, so protected tails cause no check. Wf (D) supplies domain agreement, tag/call bounds and uniqueness, not source lineage or permission ownership. Current G separates weak/strong/global maps; published protector is optional call id. Retag active membership frame cannot replace call/return/unwind semantics. |
| Bounds/layout/initialized bytes | Actual live allocation, correct byte extent/offset/alignment/non-nullness and subobject relation, mutability, no machine size overflow; original arbitrary initialized vector independent of borrowing. | V:582–725 and SB:600 check layout/bounds/null/alignment; B:298 explicitly leaves check_bounds TODO. FixedSize has model scalar locations, not a Rust byte layout. Shallow validation checks a pointer but does not recursively prove its referent's arbitrary initialized byte vector. Wf cannot fill this. |
| Freshness, zero and raw cases | Relate unbounded model naturals to finite global nonzero-u64 instrumentation without imposing tag budget on clients; retain all outcomes. Separate raw and zero-size semantics. | G:32–49,171–174 uses checked successor then unwrap; SB:845 allocates even before zero-size handling at SB:668–697, which can adjust provenance without updating stacks. B:323–337 zero returns old tag/no counter increment, raw uses Untagged; current SB raw gets fresh tags, TB raw no new tag (policy pin). No whole-MIR transfer from candidate's empty or generic heap statements. |

## What may be concluded, and what must be proved next

Top-head is currently only a sufficient represented class. It is not necessary
for `valid_block`: for one model location, a stack
`Unique q None :: Unique p (Some c) :: tail` with distinct bounded tags allows
a write by p when the younger q is unprotected, yet p is not head. With suitable
heap/domain/call counters this is wellformed model data. If q instead has an
active protector, the write normalization can fail. These are **model states;
Rust reachability is not established**, so neither is a Rust counterexample.
They show why initialization plus Wf is insufficient and why the predecessor
permission/protector proof matters. The published `valid_block` is stronger
than necessary for some SRW noaccess cases, so failure to establish that
particular representation must not become a restriction of lawful Rust inputs.

A genuine protocol can justify top after **regular positive-size Unique FnEntry**
if its incoming per-byte access/protector checks are independently derived,
the actual result is installed in this `_1`, and no intervening operation
changes that head before bb0[3]. The model normalization sequent and the
concrete source event/operands are in [MISSING-LEMMA.md](MISSING-LEMMA.md).
Do not require top at caller entry. Do not infer coverage from FnEntry comments,
hashes, an unrelated prior FnEntry, successful root reborrow, or a future defined
execution. For wildcard/unknown inputs either prove that exact normalizer and
abstraction path or extend the relation; forbidding them without an independent
entry argument would shrink the domain. The conservative134 model's
registered/fresh_node condition is a separate model registry premise, not this
source-coverage proof.

Even after an entry theorem, bb0[3] requires an all-outcomes source-to-model
operation relation. Published Retag lacks carrier writes, allocation/validation
hooks, data-race instrumentation, stop/panic/UB/errors and unwinding. Several
checks use `?` after changes (copy/write precedes validation; tag allocation and
protector registration precede final grants), so failures are not proved atomic
stutters. No such outcome may be erased by assuming no_unwind, source safety,
or defined execution. Finite tag exhaustion is an instrumentation limitation (the checked-successor unwrap is a host panic,
not a modeled Rust unwind) requiring a justified observation abstraction/semantics treatment; it cannot
become an application tag-budget requires. Source unwind routing and protector
cleanup must be treated separately, including C's ABI-specific unwind behavior.

Next source tasks, in order: bind exact entry MIR/protocol and original input
interpretation; establish pre-FnEntry referent grants/bounds/protectors and
carrier access; prove FnEntry pointer installation and remaining-entry/prefix
frame; relate every root reborrow outcome; extend through unsize/raw/getter/
writer/call/normal and cleanup exit phases and original C/F observations.
This is three distinct layers: (1) unvalidated conditional historical MODEL
algebra, (2) source-policy branch matches under stated conditions, and (3)
unproved all-lawful Rust entry/operation/C-F correspondence. None collapses
into the next.

All canonical every-u64/arbitrary initialized exclusive[10] requirements and
independently frozen original HMAC arbitrary initialized[128], all valid key
lengths including 0 and >128, prefix AND unchanged suffix, exact bodies,
standard contracts and common checked automation remain mandatory. Generic10
is not a canonical-source certificate; a retag vector frame is not a full
writer suffix proof. Normal checking, fidelity, trust, inference, publication
gates and fresh combined independent acceptance are unchanged. Inherited
canonical6/0, full HMAC1/1 unproved, native0, all44+3/101 open, publication=false
and goal incomplete remain unchanged.

## Checks and reproducibility

`commands.json` lists replayable source/static commands. Authentication and
Miri identity inspection succeeded. The candidate declaration inventory was
compared to candidate003; source proof/type/argument-order review is manual.
No candidate compilation, assumption printout, kernel check, all-lawful
coverage or semantic equivalence check ran. Every such result is `not_run` or
false in `results.json`. Final `SHA256SUMS` binds this directory's artifacts
(excluding itself). A compatible independently checked baseline remains a
prerequisite for any future proof trial; neither independent build was inspected.
