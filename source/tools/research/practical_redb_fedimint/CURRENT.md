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

## Immediate execution order

1. Inspect real pinned REDB allocator/page-state and Fedimint amount/accounting/validation code. Select and preregister one substantive component each BEFORE proof trials/engine edits.
2. Present property, actual domain, accepted-runtime baseline and remaining authored semantic/mechanical burden; freeze sources/definitions and rank common practical fixes.
3. Run independent project lanes in isolated worktrees. Integrator owns common fixes. Every brief names actual call sites, obligation, expected shared benefit and stopping condition.
4. Before each new engineering layer, record which selected property gets closer and what local proof work disappears. Prefer common mechanisms over project-specific recognizers.
5. At most45 active-investigation minutes per blocker, excluding build waits, before a recorded checkpoint. Unsupported cases stay in the denominator. Foundations/ownership/private representation prerequisites mean defer the case, not launch another prerequisite.
6. Reuse existing proof/replay utilities, focused checks first, broader suite at integration and independent review. No new custom evidence subsystem. Proof-glue reduction and causal reuse in BOTH projects are the primary outcomes.

No push/PR without authorization. All ordinary source/body/domain/fidelity/lifetime/trait/WF/mode/precision/erasure/publication checks remain required. No new trusted axioms, stronger hidden contracts, disabled checks or narrowed application inputs.
