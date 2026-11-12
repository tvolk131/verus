# Persistence-model interface freeze M1

Fixed parent: c008326bfed88eab958335412cdc5ec0e9c8130e. Frozen before proof trials.
Source: REDB c82ab865bb362bc4af067d332adc44978ff64813 and the ten CASES.json file hashes.
Application source coupling proved: **false**. HMAC/native: **DEFERRED AND UNACHIEVED**;
old native0/44+3/101 untouched. No source-lane, parent, native-test or shared-cache writes.

## Typed operational boundary

`Bytes = Seq<u8>`, arbitrary finite length. `Write { offset: int, data: Bytes }`.
`Storage { volatile: Bytes, persistent: Bytes, base: Bytes, pending: Seq<Write> }`.
`base` is the last successful barrier image, initially the durable existing file.
Application write buffers have a separate sequence `buffered: Seq<Write>`: buffering
alone cannot satisfy backend coverage. A submitted write updates volatile bytes and
appends pending work. Write errors may submit only a prefix/subrange; this is an
explicit source obligation, not an implicit rollback.

`Event = Buffer(Write) | Submit(Write) | Persist(Bytes) | SyncSuccess |
SyncError(Bytes) | Crash(Bytes) | Ack | ReturnError`.
`Phase = Preparing | FirstBarrier | Publishing | SecondBarrier | Acknowledged | Failed | Crashed`.
A protocol state carries storage, buffered writes, phase, previous two-phase flag,
and last acknowledgement. `Ack` is allowed only after SecondBarrier. It means the
modeled public Immediate Ok, not a lower-level return or a poisoned-transaction error.

Allowed persistent images choose **independently per byte** from the base byte or
any submitted pending write covering that address. This permits arbitrary tearing,
reordering, full or zero progress and stale overlapping writes. No other byte changes.
Single-byte values are atomic; the publication primary/two-phase/recovery/unflushed
bits must occupy the SAME byte and persist as one submitted byte value. This is an
explicit environment/encoding premise. No whole-header atomicity. There is no
spontaneous media corruption in this model; independent detected-corrupt input slots
are also represented at the recovery boundary. Collisions/undetected corruption are
not excluded by claiming XXH3 perfection.

Successful sync copies volatile to persistent and base, clears pending, and requires
that the relevant application buffer has been drained. Failed sync permits **any**
allowed persistent image, including the entire volatile image, retains pending work,
and does not advance the protocol to a successful barrier. Crash similarly chooses an
allowed image, discards application buffers/pending, and restarts volatile/base from
that image. Subsequent writes are restricted by explicit frame/retention conditions.
These are MODEL device premises, not verified StorageBackend summaries.

## Candidate, header, and served-state separation

`Candidate { transaction: nat, required: Map<int,u8> }` describes the **complete**
user/system tree footprint and other semantically required pages. `matches(bytes,c)`
means agreement at every required address; it says nothing about header checksum.
A separate candidate-consistency premise says those intended bytes encode a valid
snapshot, including roots, graph closure and allocation/metadata consistency.
Coverage says volatile contains every required byte and the application buffer is
drained before the successful barrier; retention says no intervening submitted write
changes any required old-root byte. Neither says a future barrier succeeds.

`Slot { transaction: nat, checksum_valid: bool, tree_complete: bool,
raw: Bytes, detected_corrupt: bool }`; `Control { primary: bool, two_phase: bool,
recovery_required: bool, primary_unflushed: bool }`; `Header { control, slots }`.
`HeaderSelection = Reject | Slot0 | Slot1` models header-only select_primary_slot.
With two_phase=true it uses the recorded primary or rejects a corrupt primary;
with false it chooses a checksum-valid newer secondary when applicable.
`ServedRecovery = Reject | Serve(slot)` is a DIFFERENT layer: any served candidate
must pass whole-candidate validation; a valid admissible fallback must be served
when present. A prospective conditional recovery law may use an explicitly defined
validate-then-fallback operation; this is a target specification, not an assertion
that actual REDB recovery implements it. Header-only selection is not coherence.

Detected-corrupt serialization returns the ORIGINAL raw bytes. A deterministic
checksum observation on the identical bytes stays invalid. No property claims that
all torn bytes are detected or that equal hashes imply equal bytes. Deliberately
replacing a slot with a new commit resets corrupt_bytes and is outside preservation.

## Entry invariants and exact obligations

Commit entry: existing valid durable published root, fixed layout, one writer, no
pending None commits; candidate has greater transaction ID. Previous two_phase is
**either false or true**. Recovery marker is established by writer opening (actual
establishment must be proved); primary_unflushed=false in this single-process
configuration. Requested new mode is two-phase Immediate. Old secondary may be
stale or detected-corrupt; it is not assumed coherent. No resize or shrink;
ShrinkPolicy::Never source caller eligibility remains a source-lane obligation.

Recovery entry: arbitrary reachable persistent crash image, both prior modes,
possibly new control and torn slots, markers decoded from that image. Do NOT
reimpose commit-entry consistency on secondary or infer tree completeness from
header checksum validity. Before barrier1 with previous two_phase=false, a newer
checksum-valid secondary can name incomplete pages. This must remain a countermodel.
After barrier1 complete candidate coverage can support publication; after barrier2
publication is durable. Ordinary Err can coexist with full durability.

Individually inventoried premises/debt:

1. Device byte persistence/frame and successful-sync contract as above (MODEL).
2. Atomic control-byte encoding and fixed-layout interpretation (MODEL/source debt).
3. Intended candidate consistency: complete tree and metadata (semantic/source debt).
4. Buffer/submitted-write coverage at barrier1 (source debt).
5. Retained-old-root byte footprint throughout prepublication and recovery (source debt).
6. Serialization/decode/checksum observations and correspondence of slot ID to footprint
   (source debt; no trusted hash summary).
7. Public Ack follows both successful flushes on the selected mode, including all
   intervening/postcommit fallible calls (source debt).
8. Recovery validates selected complete candidate, falls back when admissible, preserves
   detected-corrupt raw bytes, and only then serves it (source debt).
9. Simulation of each original native body to the operational steps, including error,
   open/repair marker writes and crashes during recovery (source debt).

The next minimum source sequent is an ordered effect trace for the selected commit:
new-secondary header submission; complete drain plus successful barrier1;
atomic control publication submission; drain plus successful barrier2; only then Ack.
Each error stops success advancement but can persist writes. Need typed footprints
for old/new required pages and the buffer-to-backend submission relation. Header
selection alone cannot discharge item8. Route these needs through the parent.

## Prospective validation and limits

Ordinary Verus definitions/proofs, accepted78 private complete bundle with verus-root,
Rust1.99.1, strict --no-cheating, fresh outputs/caches, both ordinary/Infer modes.
All selected exec/proof bodies and Infer helpers will have exact named inventories
(explicit zero Infer helpers if none); guards bind original source/tools/helpers.
Per-candidate freeze precedes compilation, including negative controls. Preserve
failed trials. Relevant controls: positive fallback, false no-tearing, early Ack,
publication before barrier1, prior-false header-only coherence, Err-implies-rollback,
and reserialization laundering detected corruption. Arbitrary finite bytes/traces
in a checked quantified/inductive law are distinguished from finite witnesses.
No new dependency, compiler rebuild, native Cargo graph, or Infer engineering.

M1 baseline: no model proof or native result yet. Expected value: falsifiable storage,
ordering and recovery contracts, useful conditional laws, retained countermodels;
model-only work does not achieve the application goal.
