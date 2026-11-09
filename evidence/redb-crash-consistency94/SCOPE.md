# Campaign94: both commit modes, all six properties

Status: ACTIVE. No completion row is closed by this scope document.
The user authorized this goal on November 5, 2026. Milestones are checkpoints,
not substitute completion conditions or automatic implementation-cycle stops.

## Required final theorem

For one exact final native redb candidate, prove crash atomicity, acknowledged
durability, preservation of recoverable state, recovery correctness including
interrupted recovery, error correctness, and logical key-value refinement for
BOTH one-phase/checksum and two-phase durable commit. Mixed histories switching
between these modes are mandatory. All six must hold compositionally, not as
unrelated helper results. Model-only lemmas and finite crash tests are supporting
evidence, not native application acceptance.

Initial native pin: c82ab865bb362bc4af067d332adc44978ff64813 at
`/private/tmp/verus-transfer-redb-miuzm2w3/redb`.
Initial Verus source: the preserved 2304-file dirty campaign93 result.
Exact initial bytes, native/immutable guards, owned Git identity, runtime hashes
and the dirty-baseline archive are recorded in
`/private/tmp/verus-crash-consistency94-20261105/preservation-initial.json`.
Original sources/checkouts, historical seals, toolchains and runtime bundles
must not be mutated. Deliberate candidate source repairs are separately tracked.
Inherited campaign93 Git object-store nonmutation remains NOT_CERTIFIED; that
exception is not erased by a fresh baseline. No Git writes are authorized here.

## Application configuration and quantification

- Single process, exclusive writer, sequential application transactions; no
  simultaneous reader/writer execution is claimed in this first campaign.
- Current-format, initially valid database; ordinary byte-key/byte-value table.
  The native open/create-table, lookup, insertion/replacement, deletion, commit,
  abort, close/reopen and recovery paths are the required operation set.
- Durable commits use `Durability::Immediate`; the transaction's one/two-phase
  choice may change across arbitrary finite histories. Durability::None,
  multimaps, savepoints, compaction, format migration, custom user encoders and
  multiprocess operation are not claimed by this theorem.
- Keys, values, supported native sizes and histories are universally quantified
  within actual representation/resource limits, not restricted to finite tests
  or artificially tiny page/tree limits.
- Naturally reachable native growth, shrink, allocation/reclamation, decoding,
  buffering and repair dependencies must be proved, or explicitly escalated if
  they require a consequential scope decision. They are not silently stubbed,
  assumed correct, or removed from a selected public operation's implementation.
- Initial validity is an explicit input predicate; reopen/recovery validity and
  per-operation invariant preservation are conclusions, not fresh assumptions
  reintroduced at each helper boundary.

## Environmental model and trust boundary

Storage distinguishes issued/visible writes from persistent bytes. Pending
writes may persist partially and out of order; crashes can occur during writes,
sync and recovery. A successful sync persists the required preceding writes;
an interrupted/failed sync is not a successful durability acknowledgment.
Power-loss restart drops volatile bytes, while a process-only restart preserves
backend-visible/kernel-buffered state and its pending persistence choices;
these are separate transitions, not an assumption that process exit flushes or
discards all kernel I/O. Native application buffers must be modeled separately.
One-byte selector updates are atomic as whole byte values, including both mode
and primary bits. Writes do not alter bytes outside their actual target ranges.
File-length/extension semantics and error effects must be explicitly modeled
from the actual backend contract before using them in a native proof.

I/O failures may have partial effects. An unacknowledged or errored commit may
be fully applied or absent, never partially visible as a valid transaction.
Recovery availability is conditional on crashes eventually stopping and needed
I/O succeeding; it cannot be satisfied merely by reporting corruption forever.

Checksum assumptions are recorded per theorem. No finite checksum is declared
universally injective. One-phase integrity arguments may condition on no
undetected relevant corruption/collision in the execution. The native validation
coverage, traversal, slot choice and error propagation remain proof obligations;
an assumed always-correct recovery/validation oracle is forbidden. Any hash-
primitive binding, std/library contract, opaque type or trusted I/O boundary must
be individually inventoried and independently reviewed before final acceptance.
Two-phase publication-order proofs must not borrow one-phase checksum detection
assumptions unnecessarily. Mixed-mode proofs identify which mode/validation
premises they use, including transitions from either prior mode.

## Mandatory acceptance rows

1. Crash atomicity: recovered logical state is a complete legal transaction
   history prefix, not a mixture of partial updates.
2. Acknowledged durability: every successfully acknowledged durable commit is
   included in that recovered history, allowing legitimate later overwrites.
3. Preservation: updates and reclamation preserve pages/state still needed by
   a recoverable committed root; allocator correctness is linked to that state.
4. Recovery: selected roots, decoded structures and reconstructed allocation
   state are valid, operational and safe after interrupted recovery.
5. Errors: no false success; uncertain completion, poisoning/refusal and abort
   behavior follow the actual public contract without assuming Err=rollback.
6. Logical refinement: native byte-table operations implement mathematical map
   updates/lookups and preserve unrelated entries, including across restart.

Each row is required for one-phase, two-phase AND mixed-mode histories.
Required final support: exact native correspondence and checked dependency
closure; assumption/proof-burden inventory; exact runtime/source binding; fresh
imports where applicable; intended semantic/crash/error negative controls;
ordinary regressions; independent final review and reproducible delivery.
Query totals are not counts of certified native bodies. No row closes merely
because a related finite test/model/helper succeeded.

## Repairs, inference and autonomous continuation

A confirmed localized native/verification bug may be repaired and work continue:
retain the original witness, patch minimally, test old/new discrimination,
rebind the candidate revision and reprove affected dependencies. Do not weaken
the theorem to hide a bug or silently alter public/on-disk semantics. Consequent
format/API/architectural tradeoffs require escalation. Known-unsound proof
mechanisms or evidence gates are quarantined until repaired/requalified.

Infer changes require a working authored reference, demonstrably mechanical
annotation removal, a structurally different counterpart and discriminating
negatives. Semantic invariants and contracts remain authored. There is no
feature quota and no new acceptance-framework mandate.

Ordinary proof failures, missing frontend support and local attempt allowances
trigger diagnosis/reallocation, not global-goal termination. Record every launch
and rebuild, test-only verifier descendants separately, and wall-time stages.
Escalate only consequential semantic choices, unisolatable trust failures, major
unrelated infrastructure, sustained independently assessed lack of progress, or
genuine external permission/resource boundaries. Global resource limits, if
provided later, are not silently exceeded. All six/both modes remain the goal.
