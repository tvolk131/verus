# Independent review — frozen test-only boundary follow-up96

Date: November 7, 2026. Engineering review only; no native proof or application acceptance.

## Verdict

**Qualified support for this frozen TEST-ONLY follow-up. No blocking defect found.** The sibling, malformed-trailer precedence, and eight-option executable controls strengthen the tests without relaxing conservative reporting. Retained logs support five passing tests in both final profiles.

This is a fresh review, not an amendment to the hardening95 report, approval of all inherited dirty work, or blanket/full-F1 approval. Local marked-source production, an actual lowerer-entrypoint matrix, full Cargo-driver controls, and complete F1/native correspondence remain **OPEN**. All eighteen final redb property/history cells remain **OPEN**. S0, full-scope/ordinary Infer/import/export barriers and trust adoption receive no new credit or authorization.

## Review protocol and authentication

Frozen root: `/private/tmp/verus-boundary-followup96-20261107/candidate1`. All source/evidence references below are relative to this root, not the moving checkout.

Actions: read-only inspection, JSON/text analysis, SHA256 hashing, `diff`, disk checks, and this one report. No builds, tests, compiler/verifier/solver/native execution, supplied-tool execution/import, network, Git, parent/snapshot edits, trust installation, proof work, or subdelegation occurred. No refused task was investigated/retried; hex repair was not reviewed.

- Manifest SHA256 matched the supplied value at entry and again after inspection: `0fd5038a8c0c6d52acddb89d2794c036db093b75db3a73bc0a1a99b79b583bce`.
- **All 273 copied SHA256 entries matched**, with zero missing files/mismatches: 252 tree files, 19 evidence files, and the top-level patch/baseline test, totaling **6,762,650 bytes**. No symlink or unlisted snapshot file was found; the manifest is the sole file outside its copied-file map.
- `evidence/BASELINE.json:1` contains 2,382 baseline entries at HEAD `b32846b61dcf724618f7345e219e5007a0c79643`. All five changed-file baseline/candidate hash pairs agree with that record and frozen candidate bytes: the Rust test, `VERUS_INFER_STATUS.md`, the hardening95 README, the redb STATUS JSON, and `source/vir/NATIVE_RESOURCE_IR.md`. The follow-up README is a new, non-baseline document.
- Independently rehashed **246 copied unchanged baseline files**, including the selected production compiler/resource/reporting sources and runner. All match their entry baseline. The manifest/sealer's **2,377 unchanged unrelated baseline files** comprise these 246 plus **2,131 uncopied baseline files**. The latter are sealed accounting, not bytes independently available to rehash in this snapshot. I did not substitute the moving parent or inspect unrelated work to inflate that assurance.
- `baseline-test.rs` SHA256 is `970dfdaff86799fcb7a328529182dc1f388fd70564d2aeefe8a55983de2133de`, matching the entry baseline. Candidate test SHA256 is `6346a63ce616fd710f28c7b01405dde52bd1926b8283ee0aed8727da2c85f3a0`. A read-only unified diff is **byte-for-byte equal to `TEST-ONLY.patch`**. Exact historical document deltas cannot be reconstructed without their original bytes.
- Read `tree/evidence/infer-consolidation95/INDEPENDENT-REVIEW.md:1` in full (SHA256 `09495f11b1e04784eda432d5101556e9448bd08072876e9e64aba064b2dc1da5`). Its authority is not extended. Baseline binding uses the entry baseline/sealer, not independent reconstruction of the old archive.

Post-inspection authentication at **2026-11-07T23:39:13.034640+00:00** and exit authentication at **2026-11-07T23:42:50.551572+00:00** both matched the manifest and all 273 hashes. Free space: entry approximately 55.216 GiB; post-inspection 55.232 GiB; exit **51.899 GiB**. All exceed 16 GiB. The report passes the 16-KiB size limit.

## Source review

### 1. Leading unmarked member is finite, distinct, and relevant

`tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:21` builds synthetic VIR, not native Rust resource input. Ordinary/resource names are distinct (`:48`). Only the marked branch calls `fixture(false, false, false)` once (`:88`); that call cannot recurse. Its public/bodyful unmarked function precedes the resource member, whose requested visibility/body presence remains intact. Crate/module paths and empty unit bodies are consistent; no duplicate name or lost marker was found.

All marked artifact paths use this helper, including import, export, stale-output, and executable-negative fixtures. Consequently a first-member-only resource check would miss the marker. The actual production scan remains `functions.iter().find(...)` at `tree/source/vir/src/native_resource.rs:10`, with unconditional prefix matching at `:6`; the feature does not disable artifact refusal.

The ordinary roundtrip at `tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:156` confirms real export/decoding, body stripping and no inferred dependencies. Export negatives (`:224`) retain visibility/body/API/proof-tuple combinations and require the resource diagnostic plus an empty output directory. The default proof tuple is not native proof evidence. Production rejection at `tree/source/rust_verify/src/import_export.rs:277` precedes pruning, body stripping and certificate construction; unrelated malformed-fixture errors cannot satisfy the intended diagnostic.

### 2. Metadata precedence is now a real competing-parser control

At `tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:184`, visibility/body variants use both import APIs and all eight modes. The header sets `inference_aware=true` (`:193`); invalid JSON ends with actual `INFER_MAGIC` (`:201`). These are 64 marked refusals and 64 unmarked metadata controls, not native proof obligations.

The unmarked variants must return the inference-metadata diagnostic and no primary spans. Marked variants must return the intended native-resource import diagnostic and no primary spans through the shared helper at `:147`. Bincode decoding failures or unrelated errors cannot satisfy the resource-specific assertion. The production seam at `tree/source/rust_verify/src/import_export.rs:111` decodes first, refuses marked members at `:116`, and only then reads/processes trailers at `:132`. With these exact bytes the unmarked route reaches JSON decoding at `:153`. Span-free normalization at `:117` remains unchanged and matches the import driver's assertions at `tree/source/rust_verify/src/verifier.rs:3426`.

API flags are not complete CLI/filter execution. This establishes refusal precedence over this malformed trailer, not a native producer, valid-certificate theorem, or full Cargo-driver path.

### 3. Executable controls and reporting preserve the real boundary

The normally ignored executable test (`tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:277`) requires matching driver/builtin paths. Its root and child bodies (`:289`) avoid the absent-root fixture problem. Both marked states run eight configurations (`:306`), with valid module/function selection (`:329`). Build-SST specifically means `--no-verify --build-sst`.

At `:348`, both streams are checked for three panic/ICE spellings and stdout must parse as JSON. This guards against generic-failure false positives despite caught unwinds (`tree/source/rust_verify/src/driver.rs:29`).

At `:353`, root/function/module/only-module runs must OMIT `verification-results.success`. Other modes require success equal to the unmarked state. All modes assert error/crate bookkeeping. Marked cases additionally require exit **exactly 1**, the intended import refusal, `interface_written=false`, and absence of the direct artifact seeded before that invocation (`:307`, `:364`). Ordinary positives require successful exit, an output artifact, and positive verifier counts whenever verification was requested. Filtered positives and no-verify/build-SST explicitly require zero full-scope checked-body counts and `complete_selected_scope_verified=false` (`:374`). No-verify/build-SST therefore receive no body-proof credit.

These expectations agree with unchanged production: `tree/source/rust_verify/src/main.rs:543` emits whole-crate success only without filters, and `tree/source/rust_verify/src/infer_report.rs:84` withholds full-scope certification under filters/no-verify. An ordinary `.vir` output may exist while `interface_written=false`; it is not inferred-interface publication. The ordinary controls are ordinary Verus plumbing/body-verification evidence, not native ownership/resource evidence.

Stale semantics remain bounded: `tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:259` explicitly calls `invalidate_export` before raw API export. The executable matrix instead exercises actual callback cleanup of a seeded direct output. Neither proves that `export_crate` alone deletes stale files, nor general concurrent-publication behavior or Cargo-driver cleanup.

The selected production local guard still precedes ordinary user filtering (`tree/source/rust_verify/src/verifier.rs:2914`, `:3050`), and full-crate certified-dependency/inferred-function restrictions remain at `:2925` and `:2959`. These are unchanged-source observations, not newly executed local-source/lowerer coverage.

## Retained evidence: inspected, not rerun

All receipt-referenced log hashes match. Each receipt has 245 selected inputs; failed replay/replay2 differ from final only in the test. Focused3/replay3 agree, and all 245 final hashes match frozen bytes; only the test differs from entry baseline. This supports no selected-production accommodation, not complete runtime provenance.

- **`evidence/replay/receipt.json:1`: failed.** Build exit 0, test exit 101, four passed/one failed. `evidence/replay/default-off-tests.log:398` records the driver's existing `user_filter.rs:137` assertion, then the test panic-oracle failure (`:403`), consistent with the missing selected root body. That production path remains unrepaired; this is not passing rejection evidence.
- **`evidence/replay2/receipt.json:1`: failed.** Build exit 0, test exit 101, four passed/one failed. `evidence/replay2/default-off-tests.log:457` shows the erroneous absent-success versus `Some(true)` test comparison. Production reporting was not relaxed.
- **`evidence/focused3/receipt.json:1`: bounded passing reuse.** Five passed, zero failed/ignored, no selected drift. Its `target_reuse` explicitly names replay2's target. This is not cold-build provenance and does not independently rebuild the production driver.
- **`evidence/replay3/receipt.json:1`: final successful two-profile replay.** Exactly four ordered commands: default-OFF build/test, then feature-ON build/test. All use offline/locked; both builds select driver and builtin, and tests include the ignored executable test. All four exit 0, no timeout, and `input_drift=[]`. Both Cargo target/build directories name replay3's new target, with two jobs. Feature-ON rebuilds into that same replay target after default-OFF; it is not a second independently cold target.
- Both final logs report **5 passed; 0 failed; 0 ignored; 2 filtered out** (`evidence/replay3/default-off-tests.log:1573`, `evidence/replay3/feature-on-tests.log:1573`). Independently verified the five completed named outcomes, including the driver's nocapture-split trailing `ok`, rather than relying only on name substrings.
- Independently parsed all **16 driver JSON/status records in each of the three successful logs**, including focused3. All three have the same semantic matrix: ordinary/no-lifetime unmarked cases verify two bodies and report two full-scope checked exec bodies; each filtered positive verifies one but omits success and reports zero full-scope checked bodies; no-verify/build-SST positives report zero verified/checked bodies. All marked cases exit 1 with the resource diagnostic, error and VIR-error true, crate success/interface-written false, and zero full-scope checked bodies. None contains the three panic/ICE markers. File absence is enforced by the passing test, not independently retained artifact bytes.
- `evidence/python-unit.log:1` records **three runner unit tests passing**, not a broad suite. Its source covers command planning, basic acceptance-log checks and identity inclusion (`tree/source/tools/test_native_resource_boundaries.py:6`). No 622-suite rerun is claimed here; the 622-run/one-skip statement in top-level status is explicitly historical hardening95 evidence. The Python log has no complete command/environment receipt of its own.

## Bounded nonblocking findings

1. **“Matched” means consumer/options, not an artifact differing only by marker.** The executable positive clears imported functions/modules at `tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:300`; the negative contains a public/bodyful leading sibling plus a private/bodyless marked member. Likewise metadata positives have one ordinary member, while marked fixtures have two and a different resource name. These are useful positive/negative option controls, but do not demonstrate whole-pipeline acceptance of the identical imported structure with only its marker removed. The exact intended refusal and metadata controls prevent this from becoming a generic-error false positive, so it is not blocking for the stated boundary slice. A future marker-only paired dependency would close this narrower precision gap. Read README “matched inputs” (`tree/evidence/infer-boundary-followup96/README.md:17`) with this qualification, not as structural identity.

2. **A few stronger observations remain log-only rather than regression assertions.** The unmarked metadata substring at `tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:214` also matches “missing or invalid inference proof metadata”; the present source/bytes establish the JSON stage, but the assertion alone would not distinguish future loss of trailer/header reachability. An exact error distinction would be stronger. Driver assertions do not require exact selected function identities/counts, `encountered-vir-error`, marked scope counters, or zero `verified` for no-verify/build-SST; the retained logs show the expected values. Positive `verified>0` prevents vacuous verification but is not full filter-conformance testing. These omissions do not invalidate the refusal/reporting properties actually asserted.

3. **Runner/provenance limits remain inherited and bounded.** The unchanged runner's `checked_test_log` checks name substrings plus aggregate counts (`tree/source/tools/check_native_resource_boundaries.py:49`), not each named completed outcome; this review independently checked the actual outcomes. The three Python tests do not establish timeout cleanup, failure short-circuiting, disk refusal or drift rejection end-to-end. Selected identity omits runtime/toolchain/dependency binaries and support inputs; build logs visibly include support crates outside the selected map. Offline/locked and before/after hashes are neither hermetic provenance nor continuous immutability. The 16-GiB runner check (`:110`) is a pre-command refusal gate, not an in-build reserve guarantee. None is upgraded into stronger assurance here.

## Documentation and handoff

The follow-up README preserves failures, target reuse and the unrepaired absent-root assertion (`tree/evidence/infer-boundary-followup96/README.md:40`). Its filtered/full-scope wording matches source/logs. Top-level status separates historical review from this pending-review slice (`tree/VERUS_INFER_STATUS.md:58`, `:67`); the old ledger confines unchanged-source claims to its closeout (`tree/evidence/infer-consolidation95/README.md:102`). Resource docs claim no new production semantics or full Cargo/native closure (`tree/source/vir/NATIVE_RESOURCE_IR.md:85`). Subject to the matching qualification, no blocking overclaim was found.

Frozen STATUS retains all eighteen OPEN. This report grants no native theorem, primitive-law adoption, application acceptance, S0/full-scope change, trust installation, or permission to resume refused tasks. Stop at handoff.
