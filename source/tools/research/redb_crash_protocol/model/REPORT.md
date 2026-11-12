# Persistence-model lane result

**Conditional MODEL laws checked; application goal unachieved.**
`source_coupling_proved=false`. No original REDB executable body is certified by
this lane. Parent owns source integration and native tests. HMAC/native remains
**DEFERRED AND UNACHIEVED; native0/44+3/101 untouched**.

## Ordered milestones and input fidelity

1. `4f376735`: early typed interface/premise freeze before proof trials.
2. `a5a3dca7`: operational definitions, checked proofs, prospective controls and replay.
3. Subsequent evidence commit: full successful/failed receipts, cost inventory and this report.

Fixed parent c008326bfed88eab958335412cdc5ec0e9c8130e; no ancestor AGENTS.md
files were present, including tracked ancestor instructions. The five named charter
files were read at that parent. Each trial authenticated all ten pinned REDB source
files, both replay helpers, the receipt and all eight accepted78 tools, then copied
the **complete** bundle (including verus-root) into a fresh private /tmp directory.
All copies were rehashed. Rust reports `rustc 1.99.1 (58cf8f9c3 2026-10-13)`;
the selected compiler binary is guarded in every trial. No rebuild, dependency,
network request, Cargo graph, shared cache, sibling/parent lane access or native
mutation was needed. The pinned REDB checkout was only read.

The initial shell attempts failed before execution with bwrap loopback
`Failed RTM_NEWADDR: Operation not permitted`; authorized safe require_escalated
local commands succeeded. Git initially lacked identity; per-command identity was
copied from the fixed parent's recorded Codex identity, with no config change.

The initial application domain remains existing valid database, one writer,
fixed layout, no pending None commits, requested two-phase Immediate. Previous
commit mode is an unconstrained bool. These application facts are projection
obligations; the operational layer deliberately generalizes over byte sequences
and footprints rather than pretending to construct a complete REDB database.
The nonempty positive construction retains a distinct old footprint and checks
both prior flags through a universally quantified `prior: bool` parameter.

## Operational definitions and exact assumptions

`protocol.rs` distinguishes backend volatile bytes, persistent bytes, successful
barrier baseline, pending submitted writes, and separate application buffered
writes. Bytes outside all submitted writes remain unchanged. Pending writes can
persist in any order, one byte at a time, partially or completely. Overlapping
pending writes may persist stale values. This is a conservative byte-level model,
not a claim that all devices exhibit every such behavior. There is no whole-header
atomicity and no spontaneous change outside submitted writes.

The god/control byte is atomic only as one byte value: its primary/two-phase bits
cannot be mixed from different values of that byte. Mapping the actual encoded
bits to `Control` is NOT proved. Other header bytes may tear independently.

A successful sync makes the backend volatile image durable, establishes a new
baseline and clears pending writes. `protocol_step` additionally requires the
application buffer be empty. It does **not** assume candidate coverage merely
from buffer emptiness. Failed sync permits every allowed partial image including
all writes, retains pending writes, and enters Failed rather than a successful
barrier phase. A crash discards volatile-only and pending/buffered state. No theorem
uses Err as a rollback fact. A failed write must be interpreted as actual submitted
subranges/progress plus ReturnError; that source interpretation is unproved.

Candidate footprints are `Map<int,u8>`. Agreement with a footprint does not itself
prove tree graph closure, checksums, allocation consistency or logical snapshot
validity. Those are separate candidate-consistency source obligations. Likewise,
volatile agreement/flush submission coverage and old-root write framing are
separate premises. None promises that a future barrier will succeed.

Commit-entry has an old valid durable published root, a newer candidate, prior
mode either false or true, the writer-established recovery marker, and no unflushed
primary in this single-process configuration. Recovery-entry is a crash image:
markers and slots are decoded from that image, with no reimposition of secondary
coherence or candidate completeness. These are distinct interpretation invariants,
not a single validity predicate. Marker establishment and the exact eligibility
of ShrinkPolicy::Never remain source obligations.

## Checked scope and useful laws

The final positive receipt is
`evidence/redb-model-accepted78-tug4tj3e/receipt.json`. Both ordinary and Infer
report **26 verified obligations, zero errors**, with entire selected crate checked.
The exact inventory is **21 named proof bodies, zero exec bodies, zero Infer
helpers**. Full `model::...` proof identities, the 21 handwritten spec definitions,
and every native scope entry (including generated enum accessors) are retained.
The final proof-body names and all contract premises are in its prospective
`freeze.json` and `candidate.rs`; RESULTS.json binds the final protocol hash.

| Laws | Exact MODEL premise and conclusion |
| --- | --- |
| `outside_writes_unchanged`, `control_byte_from_one_write` | Allowed persistence preserves uncovered bytes; a control byte equals one complete base/submitted byte value. |
| `retained_image` | Base matches the footprint, every pending overlapping write frames it, image is allowed: image matches that footprint. |
| `submit_frame`, `step_retention` | Matching volatile/base plus pending retention, well-formed storage, one allowed step, and framing of a submitted write preserve the required bytes. |
| `arbitrary_trace_retains` | Induction for arbitrary finite Seq traces, under the preceding entry/step/frame conditions and WF at each source state; retains footprint in the last image. WF preservation for all reachable states is not separately proved. |
| `successful_barrier_covers`, `covered_barrier_then_crash` | Required bytes already agree in backend volatile state, sync succeeds, then allowed crash: required bytes survive. |
| `publication_requires_first`, `ack_is_after_second` | The defined publication/Ack transitions enforce their named phase guards. These are interface ordering checks, not a proof of native call order. |
| `required_two_barriers_ack_and_crash` | Candidate-byte coverage before barrier1, explicit successful two-barrier/publication/Ack steps, and header-write framing of pages imply required pages persist after crash. Prior-mode history is preserved. |
| `write_establishes`, `acknowledged_header_and_pages` | Separately inventory the header write's complete publication footprint. Its submission establishes those bytes; barrier2 plus crash preserves BOTH page and publication footprints. No bytes-to-native-header decoding theorem is assumed proved. |
| `coherent_nonvacuous_target` | The explicitly defined validate-then-fallback TARGET serves only good slots and serves one if either is good. It is not actual REDB recovery. |
| `header_selection_with_complete_candidates` | Header selection is coherent IF every checksum-valid candidate is complete and undetected-corrupt=false. This extra premise is intentionally unavailable in the pre-barrier prior-false case, is not an entry invariant, and is not used for the publication proof. |
| `detected_corrupt_preserved` | Detected-corrupt serialization returns identical raw bytes; a deterministic detection predicate true of those bytes remains true. No XXH3 implementation, arbitrary-corruption detection, or collision freedom is assumed proved. |
| `both_prior_modes_remain` | Ack preserves either previous-mode history value. This does not establish native mode transitions. |

Four checked constructions witness torn header bytes, both full and zero
persistence on failed sync, prior-false selection of a newer incomplete secondary
with valid fallback, and a nonempty two-barrier success retaining old bytes and
new publication/pages for either prior flag. They are witnesses, not a finite
exhaustive enumeration. Quantified/inductive laws cover arbitrary finite byte
lengths, footprint maps and finite trace lengths under their explicit premises.
They do not cover infinite liveness, future successful commits that retire pages,
or physical device behavior.

The retained prior-false witness is particularly relevant: before barrier1,
`header_select` chooses the newer checksum-valid incomplete secondary while the
old slot is complete. Changing the previous flag to true keeps the old primary.
Neither outcome licenses treating header selection as final served recovery.
Actual REDB can also reject a corrupt two-phase primary before the target fallback
layer; the abstract target does not erase that source eligibility question.

## Controls and preserved failures

All eight controls fail only the intended postcondition in both ordinary and
Infer modes. Their exact candidates are frozen in CONTROLS.json and each trial's
prospective freeze, with qualified body inventories and explicit zero helpers.

- early Ack guard weakened to Preparing;
- publication guard weakened to Preparing;
- detected-corrupt raw bytes replaced by canonical reserialization;
- header-only selection claimed coherent without complete-candidate premises;
- failed sync claimed to roll back;
- whole-write atomicity claimed despite byte tearing;
- successful sync claimed to cover a candidate without write coverage;
- retention claimed despite an unframed old-root overwrite.

The first two mutants omit the positive success construction, which would fail
an assertion under intentionally broken guards. Their precise smaller scopes
are frozen; this keeps each control a clean postcondition test. Every remaining
selected body is still checked. No control uses assume/admit/axioms, poisoned
helpers, hidden external bodies, frontend failure or resource exhaustion.

Five unsuccessful development/harness receipts remain archived:

1. Initial candidate: three actual proof failures (two quantifier instantiation
   gaps, one sequence extensional equality gap); explicit proof steps repaired
   them without changing semantic assumptions.
2. Ordinary export incorrectly tested with inferred-publication status.
3. and 4. Ordinary passed, Infer passed verification but harness incorrectly
   expected an empty accepted78 trusted-model registry.
5. Early-Ack negative correctly failed its proof, but ordinary --export still
   emitted raw VIR; the initial harness incorrectly expected its absence.

Final replay uses ordinary exports only after successful runs, and tests Infer
negative export suppression directly. It never imports any local VIR. Ordinary
raw export presence is explicitly **not** certification. All generated VIR files
were moved after trials to their private runtime's quarantined-exports directory;
ARTIFACT_LOCATIONS.json records location and digest. Full source snapshots,
stdout/stderr, prospective freezes, guards, commands and failure receipts stay
in the committed text evidence. No stale local interface is a proof input.

Infer's 359-entry accepted78 registry is frozen byte-for-byte as existing trust;
ordinary mode reports no Infer registry. No trusted backend/hash/std summary was
added. Imported inference dependencies and generated Infer helpers are both zero.
No new Infer annotation reduction or engineering gain is claimed.

## Cost and reproducibility

RESULTS.json inventories all 17 trial receipts and 31 compiler invocations:
**52.288 seconds** compiler wall time and **15.999 seconds** setup time. The final
acceptance subset is 18 invocations: two positive and sixteen semantic negative.
All runs used fresh private bundle/cache/output paths and strict --no-cheating;
normal body/lifetime/trait/WF/mode checks were not disabled. No builds or native
runs occurred. Manual effort was approximately 15 active minutes for source/
contract reading, specification/proof writing, harness correction and evidence
packaging; this is a coarse human estimate, not a measured annotation-reduction
claim. No single blocker approached the45-active-minute cap. Exact timestamps,
per-command costs and source sizes remain available in the receipts/commits.

From the worktree root, run:

```
python3 source/tools/research/redb_crash_protocol/model/replay.py --label replay
python3 source/tools/research/redb_crash_protocol/model/controls.py
```

Then invoke replay.py for each frozen controls/<name>.rs with
`--expected postcondition --label <name>`. The same safe require_escalated local
path may be necessary if bwrap again fails before execution. No Cargo/native
graph is required. The evidence packaging script validates receipt digests and
quarantines exports; it does not run or certify proofs.

## Source debt and next minimum sequent

Nothing here discharges actual source ordering, backend compliance, buffer
ownership effects, header encoding, tree consistency, or final recovery behavior.
There is no original-source exec proof, application recovery theorem or native
fault baseline in this lane. The ordinary model proof does not activate any fact
inside the native crate. No unsupported source obligation justifies new Infer
engineering yet; such engineering needs a measured source-lane mechanical demand.

The parent/source lane needs to discharge this narrow ordered-effect sequent:

```
commit-entry(old valid, newer candidate, previous two_phase in {false,true})
  -> submit new-secondary header preserving old-root bytes
  -> drain all candidate-required pages from CachedFile buffers to backend writes
  -> successful sync_data #1
  -> submit header with atomically encoded primary + new two_phase bits
  -> drain and successful sync_data #2
  -> all remaining fallible commit epilogue calls succeed -> public Immediate Ok
```

At every error edge, success advancement stops but any allowed pending bytes may
already be durable. Show exact required-page footprints and that submitted writes
retain the old root until final recovery no longer needs it. Prove serialization
and decode connect the durable header footprint to the candidate identity. A
complete-candidate verifier and actual recovery fallback must connect byte
agreement to an admissible served snapshot; the prior-false pre-barrier secondary
must be validated or rolled back rather than assumed coherent. For corrupt
slots, prove from_bytes records the original bytes and every recovery header
rewrite preserves their exact slot footprint; retained_image/arbitrary_trace_retains
can then handle partial persistence and crashes during those rewrites. The final
XXH3 observation remains actual source work, without a perfect detector premise.

Current one-phase requests, None commits, create/resize/shrink, multiple writers/
readers, full allocator/reachability/tree establishment and physical devices
remain uncovered. Root integration and a NEW independent review of its concrete
result are still required to finish the application goal.
