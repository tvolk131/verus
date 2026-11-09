# Independent frozen implementation review — consolidation/hardening95

Date: November 7, 2026. Engineering review only; no theorem or native-application acceptance.

## Verdict

**Qualified support for this narrow consolidation/hardening implementation. No blocking implementation defect found in the reviewed change.** The import diagnostic normalization fixes the foreign-span/driver-assertion mismatch without changing the refusal decision. The new tests exercise real bincode import/export APIs and a real matching compiler executable, rather than merely relabeling calls to the shared rejection predicate. The retained successful logs support the stated bounded engineering results.

This verdict does not accept the inherited dirty tree, close F1 as a whole, certify a native application, establish native correspondence/resource soundness, or adopt any primitive law. All eighteen final redb property/history obligations remain **OPEN**. S0, ordinary Infer/full-scope/import/export barriers, trust adoption and certification policy remain unchanged within this reviewed scope. The broader local-admission/lowerer/Cargo-driver gaps remain open.

No builds, tests, verifier/solver/native application executions, network access, Git operations/writes, trust installation, application proof work or subdelegation occurred during this review. Actions were frozen-file reading, standard-library hashing/text/JSON analysis, disk checks and writing this one fresh report. No parent or candidate file was modified. No refused task was investigated, performed, retried or rerouted.

## Authentication and attribution boundary

Frozen root:
`/private/tmp/verus-consolidation-hardening-20261107-3uq41d0f/candidate1`.
All source/evidence anchors below are relative to that root, not the moving checkout.

- Manifest SHA256 independently matched the supplied value at entry and post-review:
  `6ad85bcda481a15479d073c6f1f93b210a6c2b0dfbe809e9c4adab197df4d001`.
- All **288 copied-file SHA256 entries** matched at entry and post-review: **zero missing files or hash mismatches**. They cover 259 tree files and 29 evidence files, totaling 6,800,492 copied bytes. No copied-file symlink or unlisted snapshot file was found; the manifest itself is separate from its copied-file map.
- Scope is the manifest's **16 candidates: 10 changed baseline files and six new files**, plus necessary frozen supporting source/logs. `evidence/BASELINE.json:1` records HEAD `b32846b61dcf724618f7345e219e5007a0c79643`, 2,369 original file hashes and 170 dirty-status lines, including 123 untracked entries. The changed-file baseline/candidate hash pairs are internally consistent with that record and the copied bytes.
- Independently compared all **243 copied supporting tree files** outside the 16 candidates to their baseline hashes: all matched. In particular, `verifier.rs`, the resource predicate/lowerer code and the production Fedimint controller are unchanged relative to this baseline.
- The assertion that **2,359 unrelated baseline files were unchanged** comes from the authenticated sealer/manifest, not a fresh review of the moving parent. The other 2,116 uncopied baseline files cannot be independently rehashed from this snapshot. The baseline preserves original hashes/status, not original bytes for changed files; therefore this review cannot reconstruct an exact baseline patch or attribute every full-file/HEAD difference to hardening95. Inherited inference/serialization policy code and pre-existing documentation claims are not newly accepted here.

`evidence/boundary-source-audit.md:1` is a prior moving-source audit, not the final implementation review. Its source-only diagnosis was useful background, not an executed pre-fix reproduction. The older frozen resource review retains its original, narrower authority; neither substitutes for this review.

## Source-supported implementation observations

### 1. Diagnostic repair preserves rejection

`tree/source/rust_verify/src/import_export.rs:111` decodes the real `CrateWithMetadata` bincode envelope. At `:116`, the same unconditional `vir::native_resource::reject_crate(&krate, "import")` is still called before approved-model handling, trailer inspection or adding the crate to import results. Its error is now mapped through `crate::util::error(error.note.clone())`.

`tree/source/rust_verify/src/util.rs:119` constructs `vir::messages::error_bare`, while `tree/source/vir/src/native_resource.rs:10` originally creates an error with the marked function's span. Deserialization retains a `Span` object even though its raw payload is replaced (`tree/source/vir/src/messages.rs:14`). The actual callback requires a spanless error before reporting the note and setting `encountered_vir_error` (`tree/source/rust_verify/src/verifier.rs:3423`). Rebuilding the diagnostic at this seam satisfies that convention and avoids attempting to render foreign spans before the import span context exists.

The mapping is `Err -> Err`, preserving the rejection note; it does not return success, alter the marker prefix, gate refusal on a feature, ignore a marked member, or weaken certificate checks. Local-source/export errors are not globally stripped of their useful spans. No admission widening was found.

The unchanged local guard is at `tree/source/rust_verify/src/verifier.rs:2914`, before ordinary user filtering at `:3050`; query construction gathers checked functions before that filtering (`tree/source/rust_verify/src/query.rs:497`). Existing full-crate restrictions for certified dependencies and inferred functions remain visible at `tree/source/rust_verify/src/verifier.rs:2925` and `:2959`. These are source observations, not newly executed local-admission controls.

### 2. Pruning/bodyless/proof-supply behavior is meaningfully exercised

The new test module is private and test-only (`tree/source/rust_verify/src/import_export.rs:367`); `tempfile` is a dev dependency (`tree/source/rust_verify/Cargo.toml:32`), consistent with the `rust_verify` lock entry at `tree/source/Cargo.lock:2324`. The final locked/offline build logs support dependency-graph consistency. Without original baseline lock bytes, this review does not claim an independently reconstructed one-line-only lock delta.

The fixture (`tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:21`) changes an internal marker, visibility and body presence. It is hand-built adversarial VIR with synthetic spans, not a marked Rust-front-end producer.

- `ordinary_exports_roundtrip_through_explicit_and_cargo_imports` (`:149`) uses the real exporter/decoder and checks imported body removal and absence of inferred dependencies. Both direct arguments and the Cargo API vector/path are exercised.
- `serialized_markers_are_rejected_before_import_filters_and_metadata` (`:177`) serializes the actual envelope through `atomic_export`, including public/private and bodyful/bodyless variants. It then calls the real importer across both API routes and eight argument configurations, requiring the resource diagnostic and empty primary-span list. Because this is the API, setting those flags is not equivalent to exercising all driver branches; the separate executable test addresses marked imports at the driver seam.
- The appended bytes in that test do **not** form a valid inference trailer and do not end in `INFER_MAGIC`; with `inference_aware=false`, they alone do not force the ordinary trailer parser to fail. Therefore the test supports early resource refusal, not comprehensive inference-metadata precedence/malformed-trailer coverage. The source placement, not that suffix alone, establishes ordering before trailer processing.
- `marked_exports_are_rejected_before_private_pruning_and_verified_metadata` (`:208`) requires the intended export diagnostic and no final/temp output across visibility/body/API variants, with and without a supplied proof tuple. This is useful: `reject_crate` runs at `tree/source/rust_verify/src/import_export.rs:277`, before private-function pruning at `:284` and body stripping at `:289`. The supplied tuple uses empty default proof data; it is not a genuine resource certificate or a newly established inference theorem.

The guard scans all functions in source, but each new adversarial fixture has only one function. There is no unmarked-leading-sibling regression that would catch a future erroneous first-function-only implementation. That is a bounded test-strengthening opportunity, not a current bypass.

### 3. Stale-output semantics are correctly bounded

`tree/source/rust_verify/src/import_export.rs:251` removes the requested artifact through `invalidate_export`. The direct path is invalidated in callback configuration before parsing (`tree/source/rust_verify/src/verifier.rs:3291`), and a resolved Cargo output is invalidated later (`:3385`). Explicit `args.export` takes precedence over the Cargo path (`tree/source/rust_verify/src/import_export.rs:245`).

The new API stale test (`tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:242`) seeds a file, **explicitly calls invalidation**, and then confirms marked export refusal leaves it absent, for each API route. It does not show that calling `export_crate` alone removes a pre-existing file: the latter rejects before publishing but does not itself perform stale invalidation. Export tests using an initially empty directory establish nonpublication, not raw-API stale cleanup. No broader transactional/concurrent-publication or combined-direct-plus-Cargo cleanup claim follows.

The executable matrix (`:261`) separately seeds the actual direct output before every invocation and asserts its absence after each marked-import error. This meaningfully checks the callback cleanup path on these import failures. There is no corresponding full Cargo-driver seeded-output run or general early-failure matrix here.

### 4. Driver oracle catches the diagnosed panic class, with remaining limits

The executable test is normally ignored and requires explicitly selected matching driver/builtin paths (`tree/source/rust_verify/src/import_export/native_resource_boundary_tests.rs:261`). The supported runner deliberately includes ignored tests. It uses a normal temporary Rust consumer plus a serialized private/bodyless marked dependency, direct `--import`/`--export`, `--internal-test-mode`, `--no-vstd` and the matching builtin library (`:269`).

For each of ordinary, no-verify, no-lifetime, root, function, module, only-module and no-verify/build-SST options it requires **exit code exactly 1**, the intended resource import diagnostic, no `panicked at` text in either stream, and no stale direct artifact (`:321`). Merely failing a compiler run would be inadequate because `tree/source/rust_verify/src/driver.rs:29` catches panics. The current oracle is substantially stronger and would detect the previous span assertion's ordinary panic output.

Limits: it does not assert every ICE/panic spelling or JSON bookkeeping; its sole unmarked executable positive is **no-verify** with an empty imported crate. That control is plumbing, not matched verification of all filtered positive variants or any native body. I independently parsed the retained successful logs: marked cases have error/vir-error bookkeeping true, `verus-infer.crate_success=false`, and `interface_written=false`; none reports `panicked at` or `internal compiler error`. Filtered reports omit top-level `verification-results.success`, rather than always emitting false. Those are **log observations**, not properties enforced by the new Rust assertions. A future JSON assertion should respect that existing filtered-report shape.

## Retained execution evidence, not reviewer executions

### Successful replays

For each of `evidence/replay2/receipt.json:1` and `evidence/replay3/receipt.json:1`, independently checked:

- Exactly four ordered commands: default-OFF build, default-OFF tests, feature-ON build, feature-ON tests. All are `--offline --locked`; builds include both `rust_verify` and `verus_builtin`; tests select the boundary module with `--include-ignored --nocapture --test-threads=1`. Feature-ON uses `vir/experimental-native-resource` and default-OFF has no such feature argument.
- All four command results have exit 0 and `timed_out=false`. All eight raw-log hashes across the two receipts match their files and the manifest. The separate launch logs agree with receipt ordering.
- Each of the four test logs contains all five named boundary tests and `5 passed; 0 failed; 0 ignored`, with **two other tests filtered out**. This is not a full Rust test-suite result. Each contains one successful unmarked/no-verify driver invocation and eight intended marked-driver exit-1 refusals, not nine application proofs. The diagnostic is present for every marked case.
- Each receipt records **245 selected input hashes**, `input_drift=[]`, and a distinct private target root for that replay. The two selected maps are identical, and every selected hash matches the frozen tree. The runner computes before/after drift; it retains the before map and drift list, not a separate complete after map. This supports the stated selected-input match, not continuous immutability or a trusted execution attestation.

For example, `evidence/replay3/default-off-tests.log:100` records the ordinary marked refusal and `:178` the intended diagnostic; its summary is at `:765`. The feature-ON summary is at `evidence/replay3/feature-on-tests.log:765`.

### Failures remain failures

The preserved initial lock-file refusal, test import/name-resolution errors, missing-builtin driver positive failure and root-CWD configuration failure are consistent with the frozen execution ledger (`tree/evidence/infer-consolidation95/README.md:40`). They are not passing boundary evidence.

`evidence/replay1/receipt.json:1` correctly says failure: build exit 0 followed by test exit 101, four passed/one failed. Its selected map differs from the final snapshot only in the corrected boundary test file. `evidence/replay1/default-off-tests.log:431` shows `--verify-function` without the required module/root selector; `:436` records CLI validation failure and `:439` the resulting **test assertion panic**. Do not mislabel that as a runtime reproduction of the original foreign-span compiler panic. The complete replay2/3 matrices use the corrected selector. The earlier five-test default-OFF pass is retained separately, not silently substituted for either complete two-profile replay.

### Python and synthetic unit fixture

`evidence/python-tooling-2.log:5` records **622 tests run, OK with one skip** (621 non-skipped successes), after an initial 612-test run with one setup error/one skip (`evidence/python-tooling.log:20`). `evidence/fedimint-hermetic-unit-2.log:1` records ten focused successes; its earlier setup-assertion error is also retained. The 49,806 rejected-control count is synthetic validator-check accounting, not native executions or theorem coverage. The broad log has no complete command/input-hash receipt or included full Python suite, so its binding to every final Python input is weaker than the 245-input Rust replay receipts; this review did not reproduce it.

The new fixture is an **uncompiled synthetic extraction wrapper** (`tree/source/tools/fixtures/fedimint_insert_link80a.rs:1`). Independently extracted and hashed its record, token, key function, insertion function and declaration as text: all five match the frozen existing item ledger (`tree/evidence/transfer-fedimint-insert-link80a/source-ledger.json:1`). Its whole-file hash is not an original native file hash.

`tree/source/tools/test_fedimint_insert_link80a.py:64` constructs temporary synthetic whole files and a synthetic file ledger, patches `NATIVE` only in unit-test scopes, and asserts real item-ledger equality. The original-input guard test still rejects missing/wrong file hashes/items (`:331`). The unchanged production controller still requires the clean original native pin (`tree/source/tools/check_fedimint_insert_link80a.py:74`) and original whole-file/item hashes (`:113`). This repairs a vanished-checkout test dependency without admitting the wrapper as a native replay source. Names such as “native receipt” in these unit tests describe the validator being mocked, not new original-native execution evidence.

## Replay runner adequacy and bounded follow-ups

`tree/source/tools/check_native_resource_boundaries.py:26` builds the matching executable before its tests in each profile. Both final and intermediate build directories point to the fresh output target; jobs are limited to two (`:90`). New-output exclusivity prevents accidental receipt/log reuse. On POSIX, the per-command timeout kills and waits for the process group (`:54`), covering the nested driver processes as well as Cargo. Result acceptance combines exit status, non-timeout, named-test/count checks and empty selected-input drift (`:120`); a failed command stops subsequent commands. These are appropriate engineering controls for the recorded platform.

The following are **nonblocking follow-ups**, not work performed or authorization to resume refused tasks:

1. **Keep the larger F1 gap explicit.** No marked local Rust-body injection; no actual lowerer-entrypoint feature/mode/safe-API/bodyless matrix; no complete Cargo-driver import/export run; no matched executable positive across all filters. Existing `check_lowering` is a body-check hook (`tree/source/vir/src/ast_to_sst_func.rs:741`), not a universal firewall over arbitrary skipped/bodyless internal VIR (`tree/source/vir/src/ast_to_sst_crate.rs:21`). None of these missing controls supplies a present admission bypass, but they prevent declaring F1 universally closed.
2. **Strengthen regression precision.** Add an unmarked-leading sibling to adversarial artifacts; use a deliberately inference-aware/malformed trailer if claiming trailer-precedence coverage; distinguish raw exporter nonpublication from caller-managed invalidation. Consider asserting supported JSON error bookkeeping and a broader ICE oracle. These would protect the actual seams rather than inflate predicate-test counts.
3. **Tighten runner-oracle tests.** `checked_test_log` at `tree/source/tools/check_native_resource_boundaries.py:49` checks name substrings plus an aggregate five-pass summary, not a completed passing result for each named test. With today's exact five-test module and successful harness exit this is sufficient for the inspected logs, but future extra tests/output could weaken the association. Parse completed named outcomes, including the nocapture-split driver result. The three Python tests (`tree/source/tools/test_native_resource_boundaries.py:6`) cover plan/basic log/identity behavior, not timeout cleanup, failed-build short-circuiting, free-space refusal, or input-drift rejection end-to-end; add mocked orchestration controls if hardening the runner further.
4. **Do not oversell provenance or reserve enforcement.** The selected identity omits dependency/toolchain/runtime binaries, several workspace support crates/configuration inputs and the complete inherited environment; offline/locked does not make this hermetic. Receipts record selected environment variables before Cargo applies configuration defaults, not the effective complete subprocess environment. The free-space check (`tree/source/tools/check_native_resource_boundaries.py:110`) only refuses the next command below 16 GiB; it cannot guarantee a running build preserves that reserve. The snapshot/documentation already qualify provenance and phrase the reserve as a pre-command gate. A future stronger reserve guarantee requires separate enforcement, not reinterpretation of these receipts.

## Documentation/status assessment and handoff

The added integration map and hardening ledger correctly separate artifact/API work, driver work, synthetic unit fixtures, historical evidence, and unexecuted application proposals. They explicitly retain local/lowerer/full-Cargo/universal-firewall limits and distinguish prior audit from pending final implementation review (`tree/VERUS_INFER_STATUS.md:35`, `tree/evidence/infer-consolidation95/README.md:73`, `tree/source/vir/NATIVE_RESOURCE_IR.md:73`). The new harness documentation does not add an application scenario or claim hosted-CI acceptance (`tree/source/tools/INFER_TRANSFERS.md:454`).

The later GOAL/TAKEOVER/STATUS checkpoint keeps all18 OPEN and primitive adoption/S0 unchanged; its “review pending” text accurately describes the freeze. This report is a separate result for the parent to reconcile, not permission to rewrite historical acceptance records or reinterpret old allowances. When reconciling counts, prefer the precise Python wording “622 run; one skip” over implying 622 independently executed non-skipped successes. No blocking status/documentation overclaim was found within the new bounded claims.

Post-review authentication at **2026-11-07T22:47:58.619783+00:00** again matched the expected manifest and all 288 copied hashes, with zero mismatches. Free space was **81.357 GiB**, comfortably above 16 GiB (entry approximately 82 GiB). Only this report is written afterward; final handoff is conditioned on one more identical input-hash check and report size <=24 KiB. No native proof credit is granted, no primitive law is adopted, and all18 remain OPEN. Stop at handoff.
