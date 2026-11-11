# Active priority: practical Infer for REDB and Fedimint

The user replaced the active goal on 2026-11-11. [Goal and execution constraints](GOAL.md) govern this work. The previous HMAC/native objective is [deferred and unachieved](DEFERRED_HMAC_NATIVE.md), not a prerequisite or a completed result.

Active branch starts from accepted `a8194fc0` / engine `f4415999`, schema39/policy75/cache78. Original provisional work is preserved on local `verus-infer` at `b76abf27`; no files/evidence were deleted. Runtime tests must use the matching accepted bundle, not the provisional binaries left in target directories.

## Useful accepted building blocks

- Bounded scalar and scalar-pair inference, range/bounds and checked unsigned arithmetic/bitvector assistance.
- Checked word-view laws, source-connected packed-field observations and typed packing deduplication; finite collection/sequence and bounded native-loop assistance within documented eligibility.
- Tuple constructor function items, genuine NonZero equality, and checked bool conversion evidence with ordinary import/publication checks. Local NonZero ordering has no ordering export authority.
- Checked canonical-varint assistance and readonly mutable-buffer observations are bounded capabilities, not general loop/ownership inference.

Historical single-word REDB bitmap and Fedimint request-helper transfers are reusable scaffolding, not new progress for this goal. Whole database/protocol correctness is not claimed.

## Provisional/research-only, off the critical path

Rev Stage B at `98931edc` with independent review `5f9533d6` is an opt-in NEW compiler-semantics prototype; parent checks completed but it is not promoted as this goal's accepted baseline. All Miri, native borrowing, Coq/Iris foundations, foreign-MIR expansion and HMAC-suffix work are deferred. Do not follow old Miri next-dependency notes.

## Current experiment state

R1 resize and F1 fee-policy baselines are **unsupported/unproved**, not successes: the accepted runtime rejects their original `div_ceil`/`saturating_div` calls before checking bodies. Their source/property freezes and results remain unchanged and in [coverage](COVERAGE.md), together with the F0 funding-map boundary.

[Replacement registration](SELECTION_R2_F2.json) selects R2 bitmap-tree set/clear ancestor-summary propagation and F2 the complete guardian/quorum admission policy, before their trials. Both selected properties now have source-connected ordinary manual proofs; the parent has authenticated both deliveries and replayed F2 with ordinary/imported clients. R2 parent replay is in progress. Independent acceptance remains pending. Neither lane shows an Infer reduction. No new engine layer is approved before a concrete common mechanism is justified. See [value checkpoint1](VALUE_CHECKPOINT.md) for why unsupported native APIs were deferred rather than becoming prerequisites.

## Current application checkpoint

- **R2:** full registered multiheight leaf membership/count, ancestor-summary and storage/padding/frame mutation property proved manually, ordinary/infer. Cost: 8 mechanical helper bodies, 15 static lemma call sites, 9 exec-local proof blocks, 9 snapshots, 6 loop-entry invariants, plus the detailed assertions/exit facts in [R2 report](redb_tree/REPORT.md). Infer removes zero annotations. Default insert/remove length algebra is already automatic after allocated-set equality.
- **F2:** complete selected NumPeers policy and quorum intersection/honest-margin consequences proved manually, ordinary/infer; ordinary imported client passes. Four existing set-lemma calls and two conditional blocks remain. Policy arithmetic already verifies automatically. [F2 report](fedimint_quorum/REPORT.md) discloses the checked FromSpec adaptation, explicit fault/set/count assumptions and omitted consensus implementation.
- R2's finite-set constructor correction and F2's missing conversion-spec adapter are specification corrections, not automation gains. Original executable bodies and registered domains/properties remain preserved. Neither proves complete original allocator/consensus callers.

## Next bounded work

1. Complete parent R2 replay and fresh independent application-proof review of both results, their source/domain fidelity, negative/import/publication gates and manual inventories.
2. Compare actual demands: R2 needs word/nested-sequence frames, ancestor-loop transport and a finite-range predicate-update equality; F2 needs subset cardinality and inclusion-exclusion. Already automatic arithmetic/count facts cannot be charged as removable work.
3. Authorize a common checked engineering layer only if its concrete mechanism removes nontrivial local work in both unchanged selected properties. A label covering two unrelated fixes or synthetic examples is insufficient. If no direct common mechanism is justified, report that bounded-experiment shortfall; do not widen into another prerequisite program.
4. Retain the45-active-minute blocker limit, focused existing-utility checks and independent review. No foundations, new trusted contracts, disabled checks, narrowed inputs or unregistered replacement cases.

No push/PR without authorization. All ordinary source/body/domain/fidelity/lifetime/trait/WF/mode/precision/erasure/publication checks remain required. No new trusted axioms, stronger hidden contracts, disabled checks or narrowed application inputs.
