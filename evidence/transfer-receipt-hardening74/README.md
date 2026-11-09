# Relational transfer receipt hardening

## Scope

This follow-up addresses the three findings in the independent six-scenario
harness review at `/private/tmp/verus-six-review72.sgkkne/REPORT.md`.
The genuine application proofs passed before the correction. The defects
permitted inconsistent or tampered relational evidence to be accepted; they
were not newly discovered false Verus theorems or application vulnerabilities.

The engine/runtime remains cache74 / policy68 / schema38. No Rust, vstd,
application body, inferred formula or authored theorem changes are required.
The matching sealed runtime is
`/private/tmp/verus-infer-final74._6rqc4bo/tools74`. The previous immutable
54-file handoff is preserved; this correction receives a new handoff.

## Corrections

- Pin each original helper's inferred requires/ensures and the imported relay's
  contract. These are complete printed-contract SHA-256 identities, not a new
  domain restriction or a trusted application specification.
- Require exact verified/error counts and the complete native-body inventory
  for all 23 non-frontend proof controls. Check collection, selection,
  certification, automatic-derivation flags and checked-body counts, including
  failed runs. An unsuccessful full run does not certify its selected bodies.
- Guard the provider's `libprovider.rlib` and `provider_cold.vir` together around
  every imported positive/negative proof and recheck the pair during independent
  acceptance. Bind clients' dependency identities to the guarded certificate;
  require both helpers' soundness, maximality and exactness evidence.
- Require all five application pin/cleanliness/source-enumeration prerequisite
  commands, each with a real integer zero exit. Expected proof failures remain
  distinct semantic, safety, auxiliary or frontend/import controls.

Synthetic regressions exercise all four reported report mutations, actual
provider artifact tampering and the failed prerequisite, plus every proof
profile, both provider guard inventories, certificate corruption and both
imported clients. They run normally and under optimized Python. Synthetic
fixtures are acceptance tests, not application proofs.

## Evidence and limitations

Fresh validation and reconstruction artifacts are recorded under
`/private/tmp/verus-infer-receipt-hardening74.oQUMpJ`. Source/tool/documentation
changes are finalized before fresh transfers so the complete-source ledger
continues to fail closed. Prior receipts and runtimes are not edited or reused
as if they had been generated from the corrected sources.

The first corrected common run rejected a genuine private `pub(crate)` helper
because the new acceptance check mistakenly expected it to be publicly
exported. Its exact proofs and same-crate availability had succeeded. The
check and synthetic fixtures now distinguish public provider/client exports
from private same-crate helpers. That failed development run remains in
`six-scenarios/transfer-suite-lk_kbdsz`; it is not accepted final evidence.

The earlier compiler build, 2,221 vstd checks, 140 VIR tests and 592 integration
tests belong to the frozen final74 validation; this Python-only correction does
not claim those suites were rerun. Fresh Python discovery passes 219 tests
normally and 219 under `-O`. All six common scenarios pass, and the dedicated
access replay passes all 39 controls. Exact logs, hashes and review status are
recorded in the new handoff's validation manifest. No new hosted Linux execution,
online dependency fetch, commit, PR, merge or ref update is implied.

Fresh common receipt:
`/private/tmp/verus-infer-receipt-hardening74.oQUMpJ/six-scenarios-release/transfer-suite-7ohxey7a/receipt.json`.
Fresh dedicated access receipt:
`/private/tmp/verus-infer-receipt-hardening74.oQUMpJ/access-prefix-release/receipt.json`.

Receipt validation establishes declared internal consistency against preserved
inputs and artifacts. It is not cryptographic attestation of an arbitrarily
forged evidence tree. Trust in the verifier/backend/solvers and existing audited
native specifications/frontend correspondences is unchanged. No new axiom,
unchecked summary, allocator/storage/async-state/crypto or funds-safety theorem
is introduced. Funding remains blocked; application scope limits are unchanged.
