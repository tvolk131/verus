# Independent source replay and original-reference exit review

Base bc2a0b37. Decisions: **ACCEPT F1 replay fix**, **ACCEPT F2 complete
pointer inventory fix**, **ACCEPT the exact exit conditional/algebra/MODEL
scope**. Each current finding has severity none; F1/F2 historically were P2.
**No native theorem is accepted or publishable.** This review changes only the
new `evidence/source-replay-exit-review/` directory.

## Authentication and execution

The early typed scope is 5e965b6e; the initial harness is 02512d57, with historical
path remapping corrected in afb6c2fc. Historical authentication/finalization was
frozen in 9e1dfd79; final record changes retaining the parser failure are
8d44f1b3/75fe2573. Product sealing was frozen in d9bb3ccb. The independent exit output
parser correction is 80d5e037. Exact final delivery is supplied in the handoff.

The current README/integration freeze is the replay entrypoint. Historical
REPORT/final-freeze files were preserved. `review.py prepare` authenticated each
integration-results.json delivery, integration and manager receipt chain,
success flags, recorded products, and identical mathematical candidates/objects.
`finalize.py history` additionally authenticated every historical frozen source
and controller through immutable Git objects at constructor3d668d29,
pointer2c0b8f23 and exit173d169e, and checked them against original receipt input
pins (27/15/50 files respectively). The predecessor review's REPORT/results/
controller match delivered review170e24b6. Historical paths were remapped to this
checkout or read as Git objects, subject to the process deviation below.

Before proof trials, both baseline-ready.json and its named successful
rocq-toolchain/durable-build-2/receipt.json were authenticated against committed
foundation pins. The fixed VerusBelt115a618dd35ff8337112cf2081a7f5391d3973d5,
523 sources/284 objects, private Coq8.20/OCaml4.14 tools, runner/loadpaths,
completed proof/source dependencies and exact GMP manifest
332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37
are retained in the 2,808-file pretrial map. Environment/builds stayed read-only.

All three current README commands ran with Python -B -O and fresh private
outputs. Normal coqc and full recursive coqchk passed without exclusions or
unsafe options. An additional independent declaration-free Coq inspection
prints every module, checks every discovered statement and individual
assumption list, and recursively checks all original and review modules again.
Its compiler-discovered constants agree exactly with recursive kernel visitation;
constructors/inductive types are inventoried separately. Complete individual
inventories, including eliminators, projections and generated helpers, are in
`assumptions.json`. No runtime Python assert gates acceptance. The independent exit parser needed
short-name normalization for inherited extensionality; its first successful
compilation outputs and parsing failure were retained before the corrected
independent normal compilation/kernel replay (80d5e037).

All mathematical .v/.vo files match delivery byte for byte. Pointer's generated
Assumptions/Inspection products legitimately differ from the old incomplete
inspection products. Exit printed proof-term hashes also match delivery.
`hashes.json` distinguishes those products; `results.json` retains exact command
vectors, receipt hashes, inventories and explicit limits. Raw logs are not in Git.

## F1 — ACCEPT stable requirements and complete current snapshot retention

`goal_scope.py` requires exactly one START/END heading, in order, and SHA256
11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0
for the exact domain/completion block. The matching frozen
`goal_requirements_sha256` is mandatory: missing, empty and wrong pins reject.
It separately hashes the complete current TARGET snapshot and compares it to
the retained initial input. This is neither a stale TARGET substitute nor a
requirement pin computed from whatever weakened domain happens to be present.

Both constructor and exit read the full current TARGET, include it in
ArtifactGuard, authenticate the snapshot, pin the helper itself in their
integration freeze, use the guard around every command via LoggedCommands, and
check it again before success. Constructor/exit controller and support identities
are frozen too. Progress prose may differ between runs; the required block
cannot differ, and no part of the current charter may change within a run.
The storage lane is outside this review.

All seven provided tests passed normally and under -O (14 total). The pure
minimal rejection checks are `test_missing_or_wrong_pin_rejected` and
`test_retained_snapshot_identity_required`; boundary uniqueness/order and domain/
completion mutations are covered by the other tests. No source mutation or
filesystem race/substitution/security experiment was used. Both fresh direct
README replays succeed without reconstructing a historical TARGET. Historical
F1 remains an accurate account of the old controller, not an unresolved defect
in the current replay entrypoint.

## F2 — ACCEPT inventory completeness separately from mathematical scope

Pointer's mathematical candidate remains exactly 77 source declarations. The
current controller emits 154 named `inventory_begin/end` helper proofs,
individually prints all their assumptions, and rejects any unexpected tail
output. A separate compiled `Inspection` imports Assumptions and discovers the
complete distance/branches/Assumptions inventories with `Print Module`.
Its discovered source list must equal the frozen source inventory; its helper
list must equal the complete generated list. Expected-name construction and
compiler discovery therefore are not just the same source regex counted twice.

Kernel output independently must cover every discovered actual constant exactly
once, with all helpers in order. Inductive types and constructors are not
misreported as `checking cst` entries: 231 declarations total comprise 203
constants and 28 inductive/constructor declarations. The 203 constants comprise
49 source constants and 154 helpers. Full recursive kernel replay checks the
inductive bodies as well. The independently compiled review discovers the same
77+154 declarations and checks each individual statement/assumptions list;
Inspection is declaration-free. All 231 individual lists and recursive kernel
axioms are empty. No silent omitted helper remains.

Acceptance of the 77 mathematical declarations is conditional scalar/span and
control-fragment acceptance. Completeness acceptance of all 231 declarations is
a distinct evidence result. Neither provides source permission, source-to-span
validity, native intrinsic preservation or source-machine control correspondence.
The historical report's anonymous-helper omission remains true of the original
worker receipt; no historical artifact was rewritten to disguise it.

## Exit — ACCEPT exact 17+68 scope; native obligations remain explicit

The 11 frontend declarations model current/future pairs and the existing
reborrow/update equations. Direct source inspection confirms phase1 current
equality, phase2 future substitution and update-current preserving the original
future. This is an algebraic interpretation; no source/reference representation
is derived from those equations. `conditional_original_exit` separately requires
original merge resolution, pending same-future, actual final current binding and
pending release. It derives the whole original future. Drop substitutes no-write
preservation; full-vector substitution can retain every unissued suffix byte.
The two countermodels are closed logical witnesses, not Rust executions.

The three resources lemmas have distinct cuts:

* `pending_reference_normal_release` consumes the particular typed pending unique
  reference using the foundation's type_resolve_instr/source_loan_release and
  ordinary lifetime aliveness. Resolving opaque IterMut does not release arbitrary
  pending observations or resolve a live caller parent.
* `model_drop_continues` executes the existing shorter no-write MODEL protocol,
  then invokes the continuation with the same E/L/I, parent ownership, original
  parent future and arbitrary ancestor Frame. Only the shorter child ends; the
  parent remains live. Its model program is not the MIR constructor/drop body.
* `live_original_current_observation` is a fancy update, not a runtime WP wrapper.
  It opens/recloses the same typed unique body via ordinary lifetime context,
  observes its whole current byte vector against heap_ctx of the same state at
  physical p, and returns that heap authority, context and same reference m.
  It changes no future and imposes no allocation-base constraint. The one later
  credit is a MODEL resource. Heap authority is not carried externally across a
  runtime WP allocation. There is no claim that these separate cuts have already
  been bound to an actual source execution.

The three source declarations import exactly the delivered constructor Interface
and Composition. `initialized_bytes_unique` uses equal-length initialized lookups
at the same tagged reference's allocation/offset addresses. It introduces no
abstract F=heap predicate. `conditional_source_final_vector` retains explicit
same-heap CURRENT/physical agreement plus merge, same-future and release premises.
`conditional_source_drop_exit` additionally requires the universally lawful-entry
source certificate, V_entry's actual operand1 and normal execution. It retains
fault exclusion from the full certificate, which covers every enabled outcome
and has progress; no defined-execution or no_unwind premise is added. Its normal
result does not prove termination or guarantee normal return.

The signature M remains universally quantified and uninstantiated. There is no
actual source certificate, source safety interpretation, source-to-current/future
bridge, or syntax-indexed compiler transport. OriginalOperandCF supplies original
entry/current and merge interpretation; PendingExitCF supplies actual final
pending-current/physical binding, unchanged pending future and eligible typed
release/no escape. These are independent open source obligations, not consequences
of constructor byte stuttering or of importing checked MODEL resource lemmas.

B0 (incoming `_1`) is distinct from internal A (`_4`) and caller parent P.
The desired native equation concerns F(B0). Releasing B0 recovers C(P) while
preserving P's independent original F(P); it never establishes F(P)=C(P).
The continuation theorem and the closed released-child countermodel preserve
that distinction. No fresh heap/world, zero initial bytes, root-only caller,
empty history, borrowed permission assumption, runtime credit requirement,
no-unwind assumption or manufactured C/F binding is installed for clients.

Independent discovery and individual checking cover exactly 17 source constants
and 68 generated helper theorems, all 85 recursively checked. All helpers,
frontend declarations, pending release and three source adapters are closed.
Only model_drop_continues and live_original_current_observation use inherited
dependent functional extensionality. Recursive dependencies additionally use
inherited Eq_rect_eq. No new axioms, obligations or constructors occur.

## Raw receipts, failures and process limits

All new raw evidence is under `/tmp/source-replay-exit-review-bc2a0b37-002/`:
`pretrial.json`, `history-authentication.json`, `constructor/receipt.json`,
`pointer/receipt.json`, `exit/receipt.json`, `constructor-independent.json`,
`pointer-independent.json`, `exit-independent.json`, all `*-command.json`,
`preparation-failure.json`, `exit-inventory-parser-failure.json`,
`seal-receipt.json`, and `final-receipt.json`. Exact digests are in the
committed compact files/manifests. `commands.md` gives replay instructions.

One review preflight failed before any proof command: the initial harness only
recognized historical `.t3/worktrees` labels and opened the historical
`/home/dev/projects/verus/.../TARGET.md` path while hashing an input. Eight
historical repository files had been opened for hashing before this rejection
(the exact source/evidence path list is retained in preparation-failure.json and
results.json). That violates
the requested checkout isolation boundary. It yielded an authentication mismatch;
no proof/semantic conclusion uses those bytes. The mapping was corrected to handle
all historical repository source/evidence labels, committed, and preparation
restarted in a fresh private directory before every proof trial. An initial
ancestor `find .. -name AGENTS.md` traversal was also too broad; it was interrupted
before output, with no file-content reads by that command. These procedural
mistakes cannot be described as perfect checkout isolation. All substantive
source review, authentication and successful proof trials use this checkout,
immutable Git objects or named completed /tmp dependencies. No further historical
checkout paths were opened after the correction. The raw preflight failure receipt retains the details.

All ten explicitly indexed earlier exit-development receipts were authenticated.
Trials1–6/10 are development/controller failures, not semantic negatives;
trials7/8 are superseded kernel-valid combined-WP prototypes, not execution
bridges; trial9 is the checked split predecessor. No earlier prototype is accepted
as a native observation proof. Two new failed attempts were review-tool failures: preparation and independent
assumptions-output parsing, neither mathematical compilation nor kernel rejection.
The latter accepted only fully qualified axiom names and rejected the inherited
short printed name; exact short-name normalization and the exact two inherited
recursive kernel axioms are required in the corrected independently frozen run.
That rerun reused the two top-level exit-discover/exit-individual command JSON
names: their original hashes remain, but the original metadata bytes/timings
were replaced. All first-attempt Coq sources/objects, individual outputs and raw
stdout/stderr remain identical to their frozen originals; accepted corrected-run
command vectors/logs/timings are complete. The failure receipt records this
retention limitation rather than claiming those two old metadata files survive.

No subdelegation, push/PR, Rust rebuild/full suite, dependency install, executable
fixture, engine/vstd/global policy or other lane edit occurred. Canonical6/0 and
full exact HMAC1/1 unproved are inherited, not rerun. The full goal still demands
every-u64 canonical/arbitrary initialized[10], and independently frozen exact
HMAC pad XOR/arbitrary initialized[128]/all valid key lengths including0/>128,
prefix AND unchanged suffix, same actual checked automation and combined fresh
independent acceptance. Native correspondences remain0; all44 minimal+3 phase
and101 writer effects remain open. Publication=false; whole goal incomplete.
