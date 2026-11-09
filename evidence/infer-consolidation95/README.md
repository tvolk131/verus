# Consolidation and boundary hardening95

## Authorization and preservation

On November 7, 2026 the user signed off on consolidation, bounded implementation,
build/test work, parallel redb/Fedimint/hex application work, and delegated
trust-sensitive review. Explicit conditional models and default-OFF scoped-proof
proposals may be developed. New primitive axioms, activated certification policy
and changes to final proof scope still require separate approval. Existing
refusal/no-retry restrictions and exhausted old-cycle limits remain unchanged.

Fresh scratch: `/private/tmp/verus-consolidation-hardening-20261107-3uq41d0f`.
BASELINE.json records HEAD, dirty status and 2,369 source/evidence hashes before
the first edit. Entry state: 47 tracked dirty entries and 123 untracked entries.
No staging, branch creation, commit or whole-tree acceptance is authorized.
All18 redb final cells remain OPEN; no trust or admission widening is adopted.

## Patch scope and claim boundaries

- Normalize resource import errors to the span-free diagnostics required by the
  actual import callback, instead of passing serialized foreign spans into its
  assertion. This changes diagnostic handling, not resource admission policy.
- Test real serialization/import/export, private/bodyless markers, explicit and
  Cargo API paths, supplied proof metadata and stale-output invalidation. Cargo
  API tests are not full Cargo-driven application tests.
- Test the actual compiler executable against a marked private/bodyless import
  under eight option configurations and an ordinary-import positive control.
  Require the intended error, exit one, no panic and no stale direct output.
  These controls do not exercise a marked-local-source producer.
- Add a locked/offline two-profile replay, named-test acceptance, raw logs and
  selected source/configuration hashes. Its results are engineering evidence,
  not native ownership or database proofs.
- Publish a concise integration map, preserving historical acceptance records.
- Make the Fedimint insert-link unit tests independent of a vanished historical
  temporary checkout. The checked-in extraction fixture contains five original
  item snippets whose hashes match the existing item ledger, inside a synthetic,
  uncompiled wrapper. Whole-file identities are synthetic unit-test data; the
  production replay still requires the original native pin and file hashes.

## Execution ledger

All preliminary failures and complete command outputs remain in scratch:

- `build-default-off.log`: lock-file refusal after a failed patch used a
  duplicated source/ prefix. No file changed in that failed patch.
- `build-default-off-2.log`: successful compiler build after the focused lock
  update for the already available workspace tempfile test dependency.
- `tests-default-off-1.log`: test-only import/name-resolution errors; corrected.
- `tests-default-off-2.log`: four artifact tests passed; driver positive control
  failed because the fixture omitted the builtin library. No complete matrix.
- `replay1/`: a successful cold build and ordinary-import driver control; intended
  marked-import refusals for ordinary/no-verify/no-lifetime/root options. The
  function-filter fixture then failed CLI validation because it lacked a module
  selector. That test fixture was corrected; the incomplete matrix is not a pass.
- `tests-default-off-3.log`: a preliminary command was mistakenly launched at
  the checkout root with only --manifest-path; it does not establish the
  source/.cargo configuration used by the supported replay.
- `tests-default-off-4.log`: five focused default-OFF tests passed, including
  the actual compiler-driver test after the fixture corrections.
- `replay2/receipt.json` and `replay3/receipt.json`: each successful, independently
  built in a fresh private target. Each records all five named tests passing
  with no ignored tests in both default-OFF and feature-ON profiles, matching
  selected before/after input hashes and raw build/test logs. These are repeated
  engineering controls, not additional resource or application proof credit.
- `python-tooling.log`: the initial broad suite ran 612 tests with one skip and
  one class-setup error caused by the vanished Fedimint source checkout.
- `fedimint-hermetic-unit.log`: a test-only class-setup assertion API error;
  corrected. `fedimint-hermetic-unit-2.log`: ten focused tests passed.
- `python-tooling-2.log`: 622 tests run: 621 passed and one skipped after the hermetic
  unit-fixture change. The reported 49,806 rejected-control checks are synthetic
  validator checks, not proofs or native application execution.

The initial commands set a private final target but inherited Cargo's configured
intermediate directory. The supported replay explicitly sets both directories.
The failed documentation/enhancement patches were atomic and changed no files;
no additional sibling-import, raw seeded-export or JSON assertions from the
failed enhancement patch are claimed. No sealed historical runtime/evidence
package is edited. The narrow candidate snapshot and baseline-preservation
manifest were authenticated by a fresh independent implementation review. Its
result is separate from the earlier source-only boundary audit.

## Independent review and retained evidence

The unchanged [implementation review](INDEPENDENT-REVIEW.md) gives qualified
support with no blocking implementation defect. It authenticated the expected
candidate manifest and all 288 copied-file hashes at entry and exit; both
successful replays' 245 selected input hashes match that frozen snapshot. It
performed no build, test, verifier, solver or native execution. This is not a
full-tree acceptance, closure of F1 or a native proof certificate.

`REVIEWED-CANDIDATE.tar.gz` retains the exact reviewed source/configuration
snapshot, manifest, original hash/status baseline, successful raw logs and
preliminary failures, without toolchain binaries or build caches. Its
`MANIFEST.json` has SHA256
`6ad85bcda481a15479d073c6f1f93b210a6c2b0dfbe809e9c4adab197df4d001`.
The archive has SHA256
`4524798261c3a18b2e80da0b41b4f05254b29e4e259ca5bb52f875052901268c`.
The review has SHA256
`09495f11b1e04784eda432d5101556e9448bd08072876e9e64aba064b2dc1da5`.
`RESULTS.json` records final preservation and post-review documentation changes.
The archive's review-pending text records the historical freeze, not current
pending work. Production and test sources are unchanged after that review;
only later status/documentation reconciliation and evidence retention follow
within that milestone. The user's subsequent instruction to continue through
ordinary blockers authorizes a [separate test-only follow-up96](../infer-boundary-followup96/README.md),
not covered by the original review; the unchanged-source statements here and in
RESULTS.json describe hardening95 closeout, not that later live-tree delta.
An initial native-tar packaging attempt added macOS AppleDouble metadata members
and failed the exact-membership check. That attempt remains in scratch; the
retained file-only archive passes the exact reviewed-file/hash comparison.

## Remaining hardening limits

The audit independently found the foreign-span/driver-assertion mismatch fixed
here. Its report is
`/private/tmp/verus-native-resource-f1-review-20261107-YLG16Z74/REPORT.md`.
It inspected an earlier moving candidate, not the frozen final patch.

No marked-local-source injection or actual lowerer-entrypoint matrix is added.
The actual driver positive covers no-verify, not every matched filter variant;
Cargo coverage is at the import/export API level, not a full Cargo driver run.
The appended suffix in the marked-import test is not an inference-aware malformed
trailer; source placement, not that suffix alone, proves metadata-processing
precedence. Stronger metadata/sibling controls and broader ICE/JSON assertions
are nonblocking follow-ups, not coverage claimed by this patch.
No universal internal-VIR firewall, native ownership correspondence, full E1,
new primitive law, scoped certificate or final redb acceptance follows. F1 as
a whole remains qualified/open, and historical raw-SMT/F2 limitations remain.

## Parallel lanes

The Fedimint assessment proposes source-connected LNURL invoice accounting:
checked fee debit, expiration roundtrip and event reconstruction, with independent
receiver-input overflow boundaries and a fee-versus-net semantic negative.
Original temporary source roots are missing; recovered bytes are not a clean
checkout. Its report and proposed fixture remain UNEXECUTED:
`/private/tmp/fedimint-proof-design-20261107.GtuCkC/REPORT.md` and
`UNEXECUTED-receipt-projection.verus.rs` in that directory.

The hex assessment proposes original Case::table plus Table::byte_to_hex_chars
for all bytes and both cases, unrestricted arbitrary-table helper semantics and
a wrong-case negative. Its report and fixture remain UNEXECUTED:
`/private/tmp/verus-hex-encode-assessment-20261107-TfFS9w/REPORT.md` and
`encoding_slice.proposed.rs` in that directory. No new assumption is adopted.
After the freeze, a separately bounded canonical-byte experiment was delegated
against the authenticated operational91 runtime, not a mixed new-driver/old-vstd
bundle and not the historical hex-v3 engine. It ended **NOT ACCEPTED** after one
frontend-visibility failure, before any body was checked. The constant
constructors and helper postcondition exposed opaque Table internals beyond the
native module. This did not qualify for the bounded lemma-correction slot; no
later invocation or semantic negative was run. The [separate lane ledger](../hex-canonical-encoding95/README.md)
retains its unchanged report, raw handoff and accounting. No retry is authorized
by that stopped handoff, and no current-parent or historical acceptance is inferred.

The new redb padding-boundary delegate returned a provider-safeguard failure and
no usable proof result. It is stopped, receives no proof credit, and will not be
retried, rephrased, performed by the parent or rerouted. Its failure does not
authorize restarting growth049, zero-growth050, mutation030 or refused
drop-pipeline work. No claim of zero execution is inferred from the failure.

Shared source remains parent-owned; one compiler builder avoids write races.
Delegates cannot install trust, export experimental certificates, close final
cells or restart refused tasks. New evidence is distinct from historical credit.
