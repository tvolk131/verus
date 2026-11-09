# Reviewed storage-boundary cycle77

November 2, 2026. Engine identity: cache77 / interface-policy71 / schema38.
Both independent Sol reviews approve their explicitly bounded scopes. This is
NOT approval of native crash durability, whole-database recovery, consensus,
availability, external exactly-once effects or funds safety.

## What changed and what is proved

### Infer compatibility and evidence repairs

The preceding cycle76 review requested three corrections. The independently
reviewed repairs now admit the already checked nested-record Euclidean quotient
during certified import, bind bitmap success to exact native/proof-body scope
and strictly typed completion/count fields, and bind the private fee positive
to its actual `PaymentFee::absolute_fee` identity. Nonzero-divisor requirements,
mandatory inference proofs and publication/import barriers remain enforced.
No new arithmetic specification or application axiom is added by these repairs.
The existing unsigned-standard-library division correspondence remains trusted.

### redb: slot-selection branch and state-frame proof

The original extracted `select_primary_slot`, `primary_slot`, `secondary_slot`
and `swap_primary_slot` executable bodies check against explicit authored
contracts in a **reduced-error scaffold**. It retains all five record field
sets, full-u64 transaction IDs, corruption observations and selection branches.
Its only representation precondition is `primary_slot < 2`.

The proof establishes the two-phase rule, both-corrupt failure, primary-corrupt
fallback, strictly-newer/noncorrupt secondary selection, equal/stale/corrupt
secondary preservation, exact kept-original-primary return flag, and logical
field framing: only the physical primary index can toggle. Corruption flags
are entry observations, not checksum-to-tree-validity or durability theorems.

The error sum is reduced to `Corrupted(String)`. The original full enum still
fails the frozen unsupported-type probe. Reduced-to-full error refinement and
native caller establishment of the representation precondition are UNPROVED.
Each provider has 23 verified obligations, 15 checked executable bodies and one
checked proof body; the complete 40-row scope is bound by the replay. Selection
contracts, seven clients and the bit-vector toggle lemma are authored and
checked. Only newtype comparison plumbing is inferred. The full-enum finite
native tests do not discharge the formal declaration/refinement gap.

See `../transfer-redb-slot-selection/REPORT.md` and the preserved independent
`redb-review.md` for source fidelity, exact outcomes and limitations.

### Fedimint: conditional transaction and replay model

This result is **CHECKED CONDITIONAL MODEL-ONLY**. A finite-trace induction
proves marker/effect pairing and at most one committed effect bundle within
the current marker lifetime. Matching replay changes no durable state;
mismatching item or peer rejects. A committed-but-unobserved result can be
recovered and replayed without a second bundle. Checked non-atomic-write and
missing-dedup witnesses demonstrate why those hypotheses are necessary.

Atomic all-or-none commit and durable recovery are separate explicit authored
transition semantics. Serialization, module-write confinement, exact abstract
equality, reliable marker reads and marker lifetime are author-owned boundaries.
Native `AcceptedItemKey` is an index; session completion deletes its markers.
This is not a forever-exactly-once theorem or an external-hook guarantee.

The scope is 18 verified obligations, one authored executable body, sixteen
proof bodies and fifteen spec definitions, with exactly 32 named identities.
Native source-to-model correspondences, unchanged native functions proved and
new inferred functions are all ZERO. The detailed native source/DB/Rust/C++
inspection ledger identifies remaining obligations; inspection is not proof.

See `../transfer-fedimint-atomic-replay/SOURCE_REFINEMENT.md` and the preserved
independent `fedimint-review.md`.

## Fresh validation and review

- Parent: all six existing common-frontdoor scenarios pass at77; 368 Python
  tests pass normally and another 368 under `-O`.
- Parent: 29 new redb outcomes and 11 Fedimint outcomes pass in each mode.
  All 1,976 baseline files and 107 integrated additions match their ledgers.
- Independent redb/repair review: 113 focused tests and 29 replay outcomes per
  mode, 42 independent compiler probes, four focused provider runs and 43
  receipt checks. No new actionable findings; F1-F3 repairs approved.
- Independent Fedimint review: 27 tests and 11 outcomes per mode, constructive
  non-vacuity and eight weakened-model mutations, 362 rejection probes and
  thirteen guard-removal mutants per mode. Bounded no-findings verdict.

The parent freshly built/rechecked the repaired compiler earlier at77, with
144 VIR unit and seven division integration tests. Reviewers audited those
large historical results, not reran them. Vstd is reused unchanged from the
sealed cycle76 artifact: there is NO fresh full-vstd claim here. Hosted CI was
not dispatched. No commits, pushes, merges, user-ref changes or PR actions.

The first combined suite attempt correctly rejected an incomplete engine-source
ledger after the two Fedimint scripts were added mid-replay. It is preserved as
a failed attempt. The stable-tree replay then passed without weakening a gate.
The archived parent audit predates review, so its `independent_acceptance`
field remains honestly `pending`; the two reports and this record provide the
subsequent bounded decisions, not a rewrite of historical evidence.

## Reproduction and next boundaries

Both new storage pilots intentionally audit the exact frozen dirty77 snapshot
and depend on its sealed runtime, native clones and provenance ledgers. They
are NOT yet portable clean-machine or hosted-CI scenarios. The existing six
common scenarios remain separate. Do not disable snapshot gates to replay a
different engine or silently apply the complete baseline patch twice.

The immediate redb follow-up is retaining the complete native error declaration
using explicitly authored opaque standard-type declarations, then binding its
full scope and rerunning positive/import/semantic/safety controls. A parent
scratch probe with `io::Error` and `panic::Location` declarations, all enum
variants and original enum attributes already passes 23/0 and an imported
caller passes 5/0. This probe is NOT independently reviewed or included in the
accepted milestone; it may avoid another inference-engine extension.

For higher-level redb durability, an author must next state storage/crash
semantics and the commit/publication/recovery invariant, then prove native
refinement, allocation/reuse and torn-write handling. For Fedimint, the next
substantive step is proving a small native transaction/marker correspondence,
including ownership/discard, conflict/error behavior, write confinement and
marker cleanup. An fsync durability premise alone establishes none of those
atomicity or protocol obligations. Infer should automate the mechanical
plumbing inside those authored boundaries, not invent their guarantees.
