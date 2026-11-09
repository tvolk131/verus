# Campaign91 consolidation freeze

Frozen before implementation. This is the single authorized consolidation pass,
not acceptance of workflow90 or the operational native theorem. The parent owns
final review. No subdelegation, native/source-body edits, commits or ref writes.

## Chosen transfer and owned files

The ONE migrated Fedimint checker is
`source/tools/check_fedimint_prefix_infer79.py` (native prefix composition).
Its byte-correspondence, exact contract/scope, certificate/import, cache-miss,
publication, semantic/safety/frontend and metadata-negative oracles remain local
and unchanged in meaning. Other transfer checkers are not migrated.

Implementation ownership is frozen to:

- New `source/tools/workflow_runtime.py` and
  `source/tools/test_workflow_runtime.py`.
- `source/tools/stateful_bitmap_workflow90/run.py` and
  `source/tools/test_stateful_workflow90.py`.
- `source/tools/check_fedimint_prefix_infer79.py` and
  `source/tools/test_fedimint_prefix_infer79.py`.
- Doc-comment-only changes in `source/vir/src/{ast,ast_to_sst,modes,resolution_inference}.rs`.
- `source/tools/STATEFUL_BITMAP.md`, directly relevant current-identity claims
  in `VERUS_INFER.md` / `VERUS_INFER_DIRECTION.md` if stale, this document and
  `evidence/reliable-operational91/CONSOLIDATION-RESULTS.md`.

Existing dirty edits are preserved against the parent's 2266-path
`/private/tmp/verus-operational91.x12fMK/baseline-source.json` and tar archive.
The 1401 immutable / 1331 native source guards under baseline82 stay read-only.

## Shared design

Extract small functions, not a new receipt framework: streaming digests and
regular-file artifacts, strict typed equality/command binding, environment
sanitization/overlays, exact runtime file checking/staging, logged subprocess
execution with timeout/spawn results, and common whole-crate outcome/schema
validation. Require source bytes, actual argv/cwd/environment and output bytes
to agree; a caller-supplied source/evidence pathname is never sufficient.
Preserve existing receipt filenames/shapes where practical. Specialized theorem
inventories and negative diagnostics stay in their existing readers.

The stateful runner continues to reuse the reference88 builder/application
orchestration, but supplies the shared executor with the same category budgets.
Its two executable-derived runtime destinations, independent binary hashes,
loader/solver pins, exact qualification, fixture hashes and ON/OFF matrix remain.
Do not update the historical qualification to pretend the old runtime was built
from newly edited documentation. A fresh source-bound qualification requires a
real build/review and may remain a disclosed blocker.

Use `time.monotonic()` for invocation, staging and validation durations. Timings
are observations, not proof evidence or speed claims. Acceptance output is new;
development build/candidate caches never replace fresh proof launches.

## Rustdoc repair policy

Repair the 18 historical failures without changing executable Rust or disabling
doctests. Supply real types/setup/assertions for meaningful Rust borrow, array,
closure and move examples. Explicitly label VIR tree/prophecy/two-phase notation
as `text` pseudocode; do not use blanket `ignore` / `no_run` / `doctest=false`.

## Validation and resource bounds

Run focused Python suites under normal and `-O`, integrated malformed
report/command/artifact and isolated guard-removal controls, and a fresh full
`cargo test --offline --locked --release -p vir -- --nocapture` including docs.
Use only private Cargo home
`/private/tmp/verus-workflow90-repair2/build/cargo-home` and existing matched
runtime `/private/tmp/verus-workflow90-cycle2.DJ31Jt/runtime`; no network/shared
Cargo/toolchain writes. At most TWO compiler builds and ONE matched-vstd run;
enumerate test builds separately. Attempt migrated actual proof replay where
the frozen runtime/oracles permit it; record any precise binding/setup blocker
instead of relaxing expectations. Record all commands, timings, freshness,
setup/test failures, changed paths and guard/ref preservation in results.
