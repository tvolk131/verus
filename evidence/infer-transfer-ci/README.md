# Hardened external-transfer CI checkpoint

This extends the validated Infer checkpoint, cache **69**, policy **63**, schema
**38**. The implementation commit is `87aa581ba4b606de7fa371ddb9531b2ce2fa50be`,
based on `55bd67ad980e90e92c77efe4453a927f097c3386` (`origin/verus-infer`).
All 87 implementation/evidence files match the previously validated snapshot,
apart from one EOF-only normalization in the historical early-return Python
runner. Its parsed AST is identical; the Fedimint harness tests were rerun in
both modes after that cleanup. Every Rust source hash is unchanged.

## Durable proof workflow

`.github/workflows/infer-transfers.yml` builds matching compiler/macros and
verified vstd, prepares fresh exact-pin application sources, runs every common
frontdoor scenario, and uploads raw setup/build/proof/failure evidence. It uses
read-only repository permissions, does not retain checkout credentials, has no
PR trigger, and defaults its manual/nightly source selection to `verus-infer`.
Application pins come from the common scenario registry rather than YAML copies.
Preparation success explicitly does not assert application proof success.

This PR targets `verus-infer`, not the default branch `main`. GitHub schedules
require the workflow to be published on the default branch, so this change by
itself does not activate a nightly schedule. No `main` changes or hosted manual
workflow dispatch were made as part of local validation.

## Independent review findings and local fixes

The independent CI review found three P2 issues in the first CI candidate:

1. **Inherited Git context:** preparation now removes inherited `GIT_*` values,
   disables global/system Git configuration, and checks actual repository roots
   immediately after initialization and after checkout. A real local-Git test
   injects directory/worktree/index/object/config overrides and proves the
   unrelated repository's HEAD, source, index and remotes remain unchanged while
   both fresh application checkouts have the expected roots, commits and status.
2. **Evidence reuse/symlinks:** both destinations must be fresh and disjoint;
   evidence is created privately before the receipt writer is entered. Reuse,
   directory symlinks, dangling symlinks and preexisting pending-receipt links
   reject without overwriting old evidence or victim source. Failures occurring
   after safe fresh allocation retain their command logs and failure receipt.
3. **Setup deadlines:** all nine steps have finite deadlines. Active-work step
   maxima total 80 minutes, upload has five minutes, and the 90-minute job leaves
   another five minutes of margin. Solver/compiler/vstd commands also use GNU
   timeout with kill grace periods on Linux. Pipefail still propagates failures.
   External cancellation or runner loss can nevertheless prevent artifact upload.

Test call recording retains controlled Git settings, not ambient environment
values, avoiding accidental secret exposure in assertion diagnostics. The
independent reviewer did not modify the candidate or rerun these final fixes;
the fix regressions and final acceptance below are parent validation.

## Validation

- **160 Python tests normally and 160 under `-O`** pass: 136 infer, 14 redb and
  10 Fedimint tests per mode. The 28 focused preparation/frontdoor tests and six
  shared execution/receipt tests are included, not extra distinct test totals.
- YAML parses; all six shell blocks pass `bash -n`; budget assertions cover all
  nine steps, 85 minutes total including upload, against the 90-minute job limit.
- Rust formatting and staged whitespace checks pass. The one newly tracked EOF
  warning was removed without changing its Python AST or any Rust source.
- The unchanged engine has the prior **134 VIR unit**, **573 integration** and
  **2,221 vstd** passing results, recorded in the preserved implementation report.
  Those broad Rust/proof suites were not rerun merely for Python/YAML changes.
- A fresh final common frontdoor replay proves all four source-connected slices
  using the immutable matching nine-artifact runtime and the final source ledger.
  Semantic, safety and auxiliary controls remain distinct; mandatory exact checks,
  original-body verification, source/tool guards and stale-export rejection remain.

No hosted Linux build, actual Linux GNU-timeout execution or manual workflow
dispatch is claimed. Existing primitive/library contracts, native frontend
correspondence and Verus backend remain trust boundaries; this CI work adds no
application axioms or trusted executable summaries. Application scope remains
extracted helpers, not whole-database/gateway/cryptographic correctness.

## Evidence

The adjacent `manifest.json` records exact source/runtime hashes, review and
validation receipts, the implementation commit and final proof outcomes. Raw
artifacts live under `/private/tmp/verus-infer-application-next-4dp08w2a`:

- `ci-hardening-validation/receipt.json`: all final Python command/log hashes.
- `checkpoint-replay-final/receipt.json`: fresh four-slice acceptance and raw
  nested proof receipt references.
- `/private/tmp/verus-ci-review-cu__c7jn/report.md` and `integrity.json`: original
  independent review, ten adversarial probes and frozen-input integrity audit.

Use `source/tools/INFER_TRANSFERS.md` for fresh clone, build and replay commands
without relying on another worktree or these historical temporary artifacts.
Earlier evidence reports retain their original dirty-checkpoint HEAD/uncommitted
descriptions; committing those source bytes does not retroactively alter them.
