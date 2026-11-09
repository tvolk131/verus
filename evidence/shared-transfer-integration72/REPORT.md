# Shared transfer integration, cache72 / policy66 / schema38

## Scope and reconstruction

This is a new local harness/setup integration report, not a replacement for the
historical application reports or a claim of a hosted nightly run. Work was
performed directly in the prepared checkout at checkpoint
`3b05c1912b9d7314ad00c832c4065f4362562e02`, without subdelegation, commits, PRs,
manual branch/ref changes, compiler rebuilding, copied targets, or edits to
parent/original candidate directories, frozen tools or pinned application clones.
PR10 was not operated on.

Only the authorized `combined.apply_patch` was applied to reconstruct the
reviewed application composition. Its SHA-256 is
`171397882113a0eb7e20b27a71a2a9c5b73884e498c000524d029193efa17f57`.
All 25 changed-file hashes, the complete 1,542-file source ledger and nine frozen
runtime hashes were independently verified before implementation. The source
ledger's SHA-256 is
`b4de2bc2a8feaad9d0a54d0ecf0bd4a83439d8016c52cac3d9a8cc9e9302deef`.
No old 87-file restore patch was used. The checkpoint's `--is-core` regression
and all production Rust/inference version markers remain unchanged relative to
that reconstruction. The baseline-relative package is against this reconstructed
snapshot, not against the raw checkpoint.

## Implemented acceptance and preparation

The common frontdoor retains the existing four scenarios and registers
`redb_page_relations` and the actual hyphenated `fedimint-full-request` ID.
Independent receipt checks re-read raw reports, exits, original/generated source
bytes, publication state, mandatory exact checks, scope, dependency identities
and guards; controller flags alone do not certify a result. JSON duplicates,
malformed booleans, incomplete controls, source/artifact drift, skipped checks,
partial verification and resource/frontend errors posed as proof fail closed.

The redb slice independently requires eight positives, three authored semantic
negatives, eleven caller-safety negatives, one separately counted polynomial
auxiliary negative and four exact metadata corruptions. The NIA diagnostic is
`Verus Infer: polynomial proposal not satisfied`; its `caller_proof_failed`
envelope is not authored semantic-counterexample evidence. Existing BV
auxiliary classification remains intact. Source reconstruction preserves the
original bodies, domains, debug guard and semantic controls.

The complete request checks all seven fields, complete item/source/lock hashes,
explicit external Hash correspondence, exact unconditional two-helper equations,
all six helper checks, and honest regenerated cold/warm misses. Providers are
11/0; import-only clients are 4/0 with zero local inferred functions and matching
nonempty dependencies bound to the provider certificate plus rlib/vir guards.
Guard deletion and false policy are 10/1 authored deadline postcondition failures.
Overflow and panic are 11/1 arithmetic and requires-false precondition failures,
respectively, not semantic counterexamples. Nine current frontend/profile and
six exact corrupted-import controls require nonzero ordinary exits, the correct
diagnostic, a seed bound to the successful export and absence afterward. The
optional baseline69 comparisons are separate and were not requested here.

Explicit preparation fetches the reviewed real Bitcoin 0.32.8 / bitcoin_hashes
0.14.0 graph using a minimal lock checked against the application's Cargo.lock,
then verifies all 21 registry identities/checksums, archive/source bytes and
dependency edges offline. Its isolated Cargo home is fresh; Git context, Cargo
overrides and compiler wrappers are sanitized. Application roots/pins are
checked before and after, destinations are fresh/nonsymlinked/disjoint, and old
receipts cannot be overwritten. Proof builds remain offline in a fresh small
native dependency target; no compiler target is built or copied. The only
standalone-runner change supplies this explicit prepared Cargo home/environment.

## Fresh validation actually executed

Raw evidence and the exact command/log-hash index are retained at
`/private/tmp/verus-shared-integration-evidence`; the handoff/package directory
contains `validation.json`, `manifest.json`, `changed-file-hashes.json`,
`integration.apply_patch` and its replay verification. Local validation used
the matching immutable `/private/tmp/verus-infer-combined72.bPzT4g/tools72` bundle.

- All Python tooling tests: **206 passed normally and 206 under `-O`**.
- Focused receipt, poison and preparation tests: **58 passed per mode**.
- YAML parsed with Ruby Psych; all seven shell blocks pass `bash -n`.
- All ten CI step deadlines validated: 80 minutes active work, five upload,
  five margin within the 90-minute job. The explicit dependency step has a
  four-minute process bound within five minutes; six 270-second controller
  limits leave three minutes of the 30-minute proof step for shared overhead.
- Real pinned archive/cache preparation succeeded in a fresh isolated Cargo
  home, using read-only cached inputs. Online fetch behavior/failure and
  sanitization were tested with mocked endpoints/real-format synthetic inputs.
- The fresh **common** six-scenario receipt has `success: true` and
  `application_proofs_proved: true`; every scenario is `proved`.
- The relational runner contributes 27 controls (23 proof cases plus four
  metadata controls). The full-request runner contributes 23 current controls,
  including 15 frontend/import rejections; baseline comparisons are empty.
- Funding caller status remains explicitly **blocked**.

The final suite receipt is
`/private/tmp/verus-shared-integration-evidence/final-six-scenarios/transfer-suite-vqb01s66/receipt.json`.
Its six child receipt paths and every raw command/stdout/stderr hash are indexed
in the package. Python commands use `python3 -B [-O] -m unittest discover -s
source/tools -p 'test_*.py' -v`; focused commands use `test_infer_transfer*.py`.
The actual frontdoor command is:

```sh
python3 -B source/tools/check_infer_transfers.py \
  --fedimint /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --redb /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-infer-combined72.bPzT4g/tools72/verus \
  --solver /private/tmp/verus-infer-combined72.bPzT4g/tools72/z3 \
  --dependency-home /private/tmp/verus-shared-integration-evidence/final-dependencies/cargo-home \
  --output /private/tmp/verus-shared-integration-evidence/final-six-scenarios \
  --timeout 180 --scenario-timeout 270
```

Initial unsuccessful controller trials remain under `initial-replays/` and
`six-scenarios/`. They exposed two acceptance bugs during implementation:
ordinary proof errors also set generic `encountered-error`, and caller trace
artifacts are named `.source.rs`, not `.rs`. Both were corrected with regression
coverage. These trials and development-only diagnostic inspections with mocked
source guards are not accepted proof evidence. The final replay uses all real
source/tool/dependency guards without mocking.

## Precise integration inventory

Eight updates relative to the reconstructed snapshot:

- `.github/workflows/infer-transfers.yml`
- `source/docs/guide/src/infer_adoption.md`
- `source/tools/INFER_TRANSFERS.md`
- `source/tools/check_fedimint_full_request.py`
- `source/tools/check_infer_transfers.py`
- `source/tools/prepare_infer_transfers.py`
- `source/tools/test_infer_transfer_preparation.py`
- `source/tools/test_infer_transfers.py`

Eight additions:

- `source/tools/infer_full_request.Cargo.lock`
- `source/tools/infer_transfer_dependencies.py`
- `source/tools/infer_transfer_receipts.py`
- `source/tools/prepare_infer_dependencies.py`
- `source/tools/test_infer_transfer_dependencies.py`
- `source/tools/test_infer_transfer_pages.py`
- `source/tools/test_infer_transfer_receipts.py`
- `evidence/shared-transfer-integration72/REPORT.md`

## Qualifications

No fresh hosted Linux run, online real-registry download or manual workflow
dispatch was executed here. Scheduled workflow definitions activate only after
publication on default `main`; neither `main` nor PR10 was changed. The parent's
139 VIR tests, 581 integration tests and 2,221 vstd checks were already recorded
in its validation handoff, **not rerun by this worker**. No compiler/macros/vstd
rebuild or new inference semantics/version change is claimed.

These are selected helper/caller proofs under Rust typing/erasure, the ordinary
Verus backend, solvers, vstd/native Option/Range correspondences and an explicit
unused opaque Hash identity boundary. Bitcoin default features plus serde are
not full application feature equivalence. No allocator-state, IO, whole-database
or whole-gateway theorem, cryptographic/hash law, amount/origin authenticity,
fee relation or stateful funding-safety theorem follows. The funding source trace
does not cross its first async persistent lookup and subsequent state/external
effects boundary.
