# Selected cases and initial baseline checkpoint

Preregistered in [SELECTION.json](SELECTION.json) before new proof trials or engine changes. Sources are the clean pinned REDB `c82ab865` and Fedimint `b8bd3d8e` checkouts. Accepted runtime: `a8194fc0` / engine `f4415999`, schema39/policy75/cache78. Neither provisional Rev mode nor the old native/HMAC dependency chain is used.

| Case | Caller-visible property and connection | Initial baseline | Mechanical burden to measure |
|---|---|---|---|
| **R1: REDB `U64GroupedBitmap::resize`** | Preserve retained pages, set every new logical bit to `full`, preserve outside bits and storage coverage across arbitrary multiword grow/shrink. Used by `BtreeBitmap::resize` and the buddy allocator. | Whole property unproved, fresh solver baseline pending. The earlier 1–64-page `set` proof is a different operation and counts only as scaffolding. | Index/cast/quotient bounds; clamped word intervals; mask membership; loop and Vec-update frames. |
| **F1: Fedimint `PaymentFee` policy/arithmetic closure** | Correct full-domain saturating fee and send/receive amounts, checked component overflow rejection and componentwise fee-limit dominance. Used by send/receive and gateway configuration checks. | Whole component properties unproved, fresh solver baseline pending. Existing request-helper proofs do not cover this. Some direct field/checked arithmetic may already be automatic; do not count them as improvement. | Saturation/min/max case splits, division bounds, observation propagation and monotonicity arithmetic. |

The numeric baseline is deliberately not invented: the first isolated experiments run ordinary and `--infer-goals` modes on frozen semantics, inventory what already verifies, then construct an explicit ordinary manual proof where feasible. Only then can before/after mechanical burden and causal reduction be measured. No engineering result is claimed at this checkpoint.

**F0 inspected but unsupported:** the larger `FundingVerifier::verify_funding` accounting theorem remains unproved. Its actual `Amounts` dependency uses map entry/or-default and owned-map iteration absent from the accepted library models. This stays in the coverage report; it does not become a foreign-MIR or representation project. Native crate/extraction feasibility is not a successful functional proof.

## Ranked practical backlog

1. Baseline R1 and F1 with original bodies/full domains and explicit semantic contracts; classify frontend, safety, semantic and mechanical failures separately.
2. Value checkpoint: identify the SAME missing checked saturation/range/observation law at both actual components. If there is no concrete common need, report it and reassess before writing engine code.
3. Integrate the smallest generic assistance using existing checked laws; compare identical frozen applications on/off and remove nontrivial local proof work. No project-name recognizer.
4. Validate relevant ordinary callers/imports, cold/warm behavior, semantic/safety/stale-export/precision controls and independent review. Run a matching broad suite only at meaningful integration.

Semantic inputs stay authored. Index/mask/frame/loop lemmas remain mechanical even if convenient to name as invariants. Blocker checkpoints occur after at most45 active minutes (excluding build waits); unsupported cases/replacements remain visible. Original HMAC/native objective: **deferred and unachieved**.
