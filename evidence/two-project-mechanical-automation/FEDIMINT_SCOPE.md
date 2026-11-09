# Frozen Fedimint native fee-validation target

Frozen November 3, 2026, before implementation, against accepted cycle80
cache79 / policy73 / schema38. This is one half of the active two-project goal,
not a success claim. Implementation/review cycles used: 0 of 3.

## Source-faithful executable scope

Native pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.

| Selected original item | Complete item SHA-256 |
| --- | --- |
| `fedimint-core/src/amount.rs:118`, `Amount::checked_add` | `87524f34513de2ac447c5bbd0f1f62cefff41732b3a896aadfb8bf974d8a1477` |
| `modules/fedimint-lnv2-common/src/gateway_api.rs:252`, `PaymentFee::checked_add` | `6ad96dedd30054c5632c26b1fe5a5704caa0034d09946833bb34ab6945081aff` |
| `modules/fedimint-lnv2-common/src/gateway_api.rs:243`, `PaymentFee::is_within` | `075bc37005bd569abcefde7a9493774e8cc70c470970aaa37b8096e3ba8844ed` |

Retain `Amount { msats: u64 }` and
`PaymentFee { base: Amount, parts_per_million: u64 }`, not tuple replacements
or flattened executable records. Retain the actual relevant built-in Amount
comparison derives and check their generated bodies through the existing
checked-derive mechanism. Unrelated serialization/encoding derives and methods
may be omitted from the extraction, explicitly recorded. No replacement
comparison body or assumed application ordering specification is permitted.

All original `u64` component values are included. Both checked-add functions
have a total safe domain, including either/both component overflows: `None` is
a normal result, not a precondition failure. The two `is_within` arguments may
alias, including equality. Do not introduce nonoverflow or limit preconditions
to evade rejected-input behavior.

Existing native `absolute_fee` and `fee` bodies and the verified component
monotonicity theorem are composed, not counted as newly inferred capability.
Keep their original multiplication saturation BEFORE division by 1,000,000;
keep private `absolute_fee` private. Their pinned item hashes are respectively
`b1f6426cc31ebf585f2a898d4593338f47b89c9cf64b5dcfe643975efb430d2d`
and `7ddbad236c417465b7bb056c4d7583b2f7b270f587f08292ca067a9af6a91784`.

## Frozen intended properties

Let M = 2^64 - 1. All additions in statements of intent are mathematical.

1. `Amount::checked_add` returns `None` iff the msat sum exceeds M, otherwise
   `Some` with the exact sum, including sums exactly M.
2. `PaymentFee::checked_add` returns `None` iff either component sum exceeds M;
   otherwise `Some` with both exact sums. Preserve native `?` early exits and
   the differing intermediate `Amount` and final `PaymentFee` payloads.
3. The actual native `is_within` is equivalent to the conjunction of msat and
   ppm bounds. Either excessive component rejects; equality accepts. The
   actual generated Amount comparison implementations must justify the msat
   comparison, rather than having that law assumed.
4. A labeled, source-connected validation composition rejects overflow first,
   then send-limit violations, then receive-limit violations. Over unrestricted
   inputs, acceptance is equivalent to nonoverflow and both native checks.
5. Acceptance implies the exact summed send fee is componentwise bounded by
   the send limit and the transaction fee by the receive limit.
6. For every original `u64` amount, acceptance implies the actual computed
   send fee is no greater than the send limit's fee, and likewise for receive.
   Use the existing checked semantic monotonicity theorem; do not add manual
   overflow/field/Option/algebraic proof plumbing for the new native helpers.

## Real caller boundary — explicitly accepted correspondence seam

Primary source is `handle_set_fees_msg` in
`gateway/fedimint-gateway-server/src/lib.rs`, native lines 2627–2648. Exact
validation slice SHA-256:
`0bbbf702c708e9dc5065f0d5e03723e48afd1f06d900ad192f0f85bdb5e54e71`.
The source performs checked addition, `ok_or_else(...)?`, the send-limit check,
and the receive-limit check, before assignments at 2650–2651 and the awaited
save at 2652. Commit at 2655 is outside the theorem.

Secondary correspondence source is `to_gateway_parameters` in
`gateway/fedimint-gateway-server/src/config.rs`, native lines 187–207, slice
SHA-256 `155f60d6877d4f6764b22686032d7b009638ec825be548fa0085e3f6b6c1aeb2`.
Its validation order is the same. Native send/receive constants and their
`from_sats` construction must be source-bound; do not invent unrelated limits.

The parent explicitly accepts an authored pure composition that abstracts
formatted error payloads while preserving native calls, data flow, and decision
order. Pin and independently audit correspondence to both native regions.
That audit is NOT a verified extraction compiler or a proof of either whole
native caller. Whole async/configuration methods, error formatting/allocation,
configuration provenance, connected-federation lookup, database writes/commit,
backend, startup, all-federation, durability and funds safety remain unproved.
The distinction must remain visible in the final report and acceptance tests.

## Baseline-to-final automation requirement

The sealed feasibility package is
`/private/tmp/verus-fedimint-scope81.vFWtlc34`.

- Native Amount checked-add declines scalar/unit Option payload eligibility.
- Native PaymentFee checked-add declines canonical `?` lowering for its nested
  record payload, even with an ordinary verified Amount summary.
- Named-field checked derives fail the tuple-only macro frontend. A separate
  non-native tuple control reveals the second-borrowed-record profile decline.
- Both unchanged native checked-add bodies verify with two handwritten ordinary
  contracts: 6 verified, 0 errors. Those contracts are the concrete manual work
  to remove. Existing fee arithmetic already verifies and is not new progress.

Reusable work is bounded named-singleton checked comparisons, two-root shared
transparent record queries (aliasing allowed), and transparent scalar-leaf
record Option payloads through canonical typed `?`. Do not merely remove shape
guards. All generated formulas remain proposals subject to soundness,
maximality and exactness against original typed bodies.

## Completion and rejection controls

The feasibility `control-matrix.json` is part of the frozen requested control
set; it is a proposal, not evidence of execution. Require full-domain local
and genuinely certified imported composition, cold/warm provider runs, fresh
mandatory proof work and exact body scope. Bound both VIR and native library
artifacts, imported certificates and actual comparison dependencies.

Include separate controls for base/ppm/both overflow, exact-M sums, either
excessive limit component, equality, receive-only rejection, wrong decision
order, altered denominator/saturation, incorrect comparison bodies, malformed
residuals, hidden effects, unsupported shapes, incomplete proof checking,
resource incompleteness and stale/tampered imports. A safe `None` result is
not an invalid-input failure. Frontend declines cannot replace semantic false-
property failures. Seeded inferred exports must disappear after failed runs
where Infer publication applies. Existing regression suites and new isolated
acceptance-guard regression tests must pass, followed by independent review.

## Trust and sealed feasibility references

No new application axiom, assumed application summary, native executable
replacement, or standard-library axiom family is authorized. Existing Verus,
Rust derive generation, solvers and approved vstd numeric/Option/comparison
correspondence remain trusted. The caller-correspondence seam above remains
an explicit limitation.

- Feasibility report SHA-256:
  `46001956ff35e3161cf385a97fae9c79802cc798880fcd9a8524fdb0334a93e7`.
- Machine proposal SHA-256:
  `66ed5a500e9fe4d3eb9420ef0619d9b118cd08ce45bd2226057f8848ae144744`.
- Control matrix SHA-256:
  `c777179a32b38980dda4b0794c7ec9d22e289e759e6baa6963a3a2845856264c`.
- Full package `SHA256SUMS` SHA-256:
  `855f8026ca6cd6b4cde0c04442724bc56539c3315960e3cb4ffc5a1bb73bd30a`.

The parent read the report, checked the original probe sources/diagnostics and
manual-contract control, and verified the complete package seal before this
freeze. No implementation or independent acceptance is claimed by this file.
