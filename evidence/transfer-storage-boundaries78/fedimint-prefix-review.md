# Independent read-only review: Fedimint prefix bridge

Date: November 2, 2026, America/Chicago.

## Verdict and findings

**BOUNDED NO-FINDINGS VERDICT. ZERO INFERENCE PROGRESS.** No must-fix correctness, source-correspondence, evidence-classification, or fail-closed acceptance defect was found in the specified 15-file prefix-boundary delta. This is not a whole-Fedimint, transaction, backend, consensus, funds-safety, or inference-success endorsement. There are no severity-ranked findings to reproduce; the evidence and trust boundaries below define the verdict precisely. No repairs or subdelegation were performed. The redb companion is excluded from semantic review and fresh testing.

## Exact reconstruction and isolation

- Tool-created review checkout: `/Users/tommyvolk/.t3/worktrees/verus/t3-9a0c9289`, initially clean at `3b05c1912b9d7314ad00c832c4065f4362562e02`, with 1,909 tracked/filesystem files.
- The full dirty-baseline package was actually applied exactly once using `apply_patch`: `/private/tmp/verus-cycle77-storage-integration.PYURYB/final-package-v2/combined.apply_patch`, SHA256 `e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`. All 210 changed-path records, their original HEAD hashes, and the entire 2,092-file reconstruction match.
- The add-only delta was actually applied exactly once using `apply_patch`: `/private/tmp/verus-prefix-boundary78.95TFck/final-package-v4/delta.apply_patch`, SHA256 `4bbd2c97e91aae2c643b21a5491749c672a1aaf3537ed4d04991e8e8252ba3d9`. All 15 paths were absent from the baseline, and all 2,107 candidate files match. No baseline file changes in the delta; 2,092 + 15 = 2,107 before and after review.
- The first attempt to launch the full patch as one argv was rejected by the OS before `apply_patch` spawned. The unchanged clean checkout was rechecked, then the two real applications used stdin. Thus neither package was actually applied twice. Original failed script and successful patch transcripts remain preserved; see `attempts.md`.
- All Git subprocesses used sanitized environments and `GIT_OPTIONAL_LOCKS=0`. Final verifier/test/probe subprocesses used private homes/caches/tmp/Cargo/rustup roots. No network, engine/Cargo/vstd/native-crate build, target/dependency/runtime copy, shared Cargo/rustup write, checkout creation, staging, ref-write command, commit, push, PR, or delegation occurred. Only small evidence/probe libraries were compiled with the existing sealed verifier.
- Scoped native AGENTS and the available codebase/development/coding-style skills were read. The locally referenced linked-specs and multipart-review skills were absent; no attempt was made to install/fetch them.

Evidence: `reconstruct-start.json`, `reconstruction-stage-0.json`, `reconstruction-stage-1.json`, `reconstruction.json`, `apply-0.stdout`, `apply-1.stdout`, `provenance-before.json`, `provenance-after.json`, `final-state.json`, `immutable-audit.json`.

## Source-connected result and limits

Native source remained clean at `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, before and after each replay; all 1,199 native source-ledger entries still match.

- `fedimint-core/src/db/mod.rs:712`, `PrefixDatabase::get_full_key`: original `Vec<u8>`, borrowed slice, clone/extend/final-expression body retained byte-for-byte.
- `fedimint-core/src/db/mod.rs:767`, `PrefixDatabaseTransaction::get_full_key`: the same original key body, also byte-identical.
- `fedimint-core/src/db/mod.rs:773`, `get_full_range`: original `Range` construction and both original endpoint calls, byte-identical.
- Both key-body hashes are `83ae7b53a60df5ab339cc91cd12539c8bceb91877ba0bd6613e05ac0077bb704`; range-body hash is `da45d80a549f9c3928b836681545efdd6f7668a297291168c4a33e2038a65364`.
- Original record field sets, order, and types are retained, including transaction `inner/token/prefix` order, database `prefix/token/inner` order, the database's `Inner: Debug` bound, and `GlobalDBTxAccessToken(u32)`. Moving the method implementation's Debug bound inline is equivalent to the original where-bound. Visibility, named results, and authored contracts change; unused derives/other impls are omitted and disclosed. These are scaffold-local nominal types, not an ABI/layout proof of the native module.
- Exact concatenation, prefix membership, same-prefix cancellation/injectivity, conditional prefix-incomparable namespace separation, a comparable-prefix overlap witness, and exact range endpoints are checked. All semantics/contracts/lemmas/callers are authored, not inferred. There is no new application `assume`, `admit`, `external_body`, or desired-conclusion behavioral summary.

Reviewed source: `evidence/transfer-fedimint-prefix-boundary78/{source-ledger.json,native-items.json,contracts.json,prefix.spec.rs,goals.verus.rs,consumer.verus.rs,expected-scope.json}`; extraction/rendering/correspondence in `source/tools/check_fedimint_prefix_boundary.py:75` and `source/tools/check_fedimint_prefix_boundary.py:125`.

Inspected, but not verified, native obligations agree with the refinement ledger:

- Raw insert/get/remove/prefix/range wrappers at native `db/mod.rs:823`, `:837`, `:860`, `:872` pass prefixed keys/endpoints to async inner dispatch. Backend honoring of keys/ranges, stream-prefix stripping, ordering, effects, and confinement are not derived.
- Module prefixes at native `db/mod.rs:687` start with `MODULE_GLOBAL_PREFIX = 0xff` and append the module-ID encoding. Native `encoding/mod.rs:521` and `:543` encode u16 through `BigSize(u64::from(*self))`, not fixed two-byte big endian; `encoding/btc.rs:324` delegates BigSize encoding to `write`. Neither its correctness nor actual encoded-prefix incomparability is proved here.
- Native `db/mod.rs:1714` borrows the underlying transaction and returns a global-access token. `db/mod.rs:802` intentionally exposes the unprefixed inner transaction on token match. The u32 token/from-prefix implementation at `:1813` is not a capability-security or global-confinement theorem.
- Native base commit at `db/mod.rs:1438` consumes `raw.take()`, awaits backend commit, then submits notifications on success. `commit_tx_result` at `:1846` sets the tracker, awaits the backend, and drains hooks only on success. Consumption/drop, cancellation, atomicity, durability, hooks, notifications, and recovery remain unproved.
- The inspected consensus attempt's marker/processing/audit/commit flow is not a refinement proof. Prior conditional replay/cleanup models and all their assumptions remain separate; session cleanup/index reuse, module effects, and external exactly-once effects do not follow from key concatenation.

Trusted, not freshly established: sealed runtime identity 77 / policy 71 / schema 38; Rust/Verus translation; Z3; reused vstd including standard Vec clone/extend and u8 clone specifications; process/filesystem/hash mechanisms. Baseline vstd `assume_specification` contracts are library trust, not newly verified standard-library implementations or inferred application summaries. Resource exhaustion, allocation failures, panics, and total correctness are excluded as documented.

## Fresh validation, not historical credit

The fresh commands and full raw stdout/stderr are retained in this report directory. Every replay used `--sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin` to avoid discovery through private empty rustup homes.

1. **41 focused tests pass in normal Python; 41 pass under `-O`**: 82 original focused test executions total. See `tests-normal.stderr`, `tests-optimized.stderr`, and `fresh-runs.json`.
2. **13 replay outcomes per mode, 26 total**, independently re-audited from raw JSON/source/command/artifact files:
   - Provider positives in debug assertions on/off: 10 verified / 0 errors, exactly 5 exec bodies (3 native + 2 authored), 5 proof bodies, 3 spec definitions, and 13 selected identities.
   - Matching consumer positives: 3 verified / 0 errors, 3 authored exec identities. The actual `--import` VIR and `--extern` rlib paths match the successful provider from the same assertion mode.
   - Six semantic controls per mode: omit database prefix, reverse concatenation, erase suffix, omit transaction prefix, replace range end with start, and remove namespace incomparability. Each is 9 verified / 1 error, non-VIR postcondition failure, with the exact intended failed-function identity and diagnostic source location. No incidental frontend/resource/panic failure receives semantic credit.
   - One arithmetic safety control per mode: 10 verified / 1 error, exact unchecked-increment failed function and arithmetic underflow/overflow diagnostic.
   - Two original Infer probes per mode: 3 requested, 0 generated, frontend/VIR unsupported at `extend_from_slice`; final seeded Infer failure removes the earlier ordinary export. This does not establish clone support or whole-range inference support.
3. **15 independent check-removal probes per mode, 30 final private mutant suites**, plus a direct malformed-input witness for each guard. With healthy code, each witness raises the specific intended guard's message. Removing only that guard makes the same witness return normally. The focused suite then fails by exception-not-raised assertions, with **zero incidental test errors**. No later unrelated ValueError is accepted as evidence that the removed guard was covered. See `private-adversarial-guards-normal.json`, `private-adversarial-guards-optimized.json`, and `outputs/guard-probes-private-*`. These intentionally failing mutant-suite runs are not added to the 82 original focused tests or counted as new proof successes.
4. **Two additional independent verifier cases**, outside the original 26-case replay count:
   - `independent-verifier/nonvacuity_v2.rs`: 2 verified / 0 errors, one authored proof plus one concrete authored executable caller. It constructs empty inputs and singleton-prefix records, checks concrete key concatenation and reversed range endpoints, establishes a satisfiable incomparable-prefix case, and exhibits distinct comparable prefixes with overlapping keys. It imports only the fresh successful provider pair.
   - `independent-verifier/false_distinct_prefixes.rs`: 0 verified / 1 error, exact postcondition failure in `false_distinct_prefix_separation`. Merely unequal prefixes do not justify unconditional namespace separation.
   - The earlier scratch-only probe failure is preserved and excluded; see `attempts.md`. No reviewed source was changed to make these probes pass.

The separate `independent-receipt-audit.json` checks all 26 replay cases without using the runner's acceptance functions: typed result/work flags/counts, fixed expected identity sets and mode categories, source declaration spans, generics, source/command/output hashes, default-erasure/no-cheating command flags, fresh artifact timestamps, matching positive VIR/rlib import provenance, and failure categories. The guard-mutation probes separately exercise the actual acceptance functions using fresh raw receipts.

Fresh replay commands, from the review checkout:

```sh
python3 -B -m unittest discover -s source/tools -p test_fedimint_prefix_boundary.py -v
python3 -B -O -m unittest discover -s source/tools -p test_fedimint_prefix_boundary.py -v
python3 -B source/tools/check_fedimint_prefix_boundary.py \
  --application /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --solver /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/z3 \
  --sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin \
  --output /private/tmp/CHOOSE-A-NONEXISTENT-NORMAL-OUTPUT
```

Repeat the runner with `python3 -B -O` and a different nonexistent output. Exact executed commands are in `fresh-runs.json` and every case's `command.json`. The runner does not need an old baseline directory or another worktree; old scratch links in the artifact index are provenance, not replay inputs.

## Ordinary exports are not certificates

All seven ordinary proof-failure controls per mode leave `.vir` exports despite `interface_written=false`; the seeded omit-prefix ordinary failure replaces the earlier export. This behavior is observed and accurately disclosed, not hidden or treated as certification. No failed ordinary export is imported. Positive pairs are fresh, their hashes are bound in the receipts, and imported artifacts are guarded across consumer invocations. All imports here are ordinary Verus imports: `functions`, `trusted_models`, and `imported_inference_dependencies` are empty, with zero requested/generated inference functions. No inferred-interface certification, failed-export-cleanup guarantee for ordinary failures, or warm-cache equivalence is claimed. The Infer-aware seeded-cleanup check is separate.

The finalized package adds `artifacts.json` after its original published replays. Those historical receipts guarded 28 inputs; **each fresh independent replay guards 29**, the extra input being `artifacts.json`. Both fresh before/after maps match exactly. This is accounted for as an inventory difference, not a new proof result or an undisclosed validation failure.

## Audited provenance and final immutability

Audited only, not rerun as proof evidence: all 502 original raw artifacts (11,216,644 bytes), all 26 linked artifact records, package inventories/manifests/hash files, canonical reconstruction map hashes, and the original final-audit ledgers. The full before/after provenance checks agree exactly: 1,570 baseline source entries, 1,572 candidate source entries, 1,085 Rust build-input entries, 61 toolchain entries, 3,511 historical guarded-input hashes, 9 sealed-runtime entries, and 1,199 native Fedimint entries. Historical guarded-input hashing includes companion/baseline artifacts only as immutable provenance; no companion semantic validation or old engine suite was run.

Own HEAD, owned branch/ref, and index bytes/hash remain unchanged. Index SHA256: `da7ff71f67950e0fc78f48c3e5be641815370b843bc5f072a8dbab0f3d349036`. After the authorized reconstruction, candidate file bytes and Git status remain unchanged. Native source/pin/clean checks and runtime/package hashes match before/after. No changes to engine or standard specifications occur in the 15-file delta.

One shared repository ref addition was observed separately from this checkout's unchanged owned state:

`refs/t3/orchestration-v2/checkpoints/MTc2ODBlNGY3MGMyYzM5MzBjYjdmYTRiOWI4OThjZDI/ordinal/65`: absent initially; `26e4038bd73b1af72d61296afd3fb53ad7c7af8a` at final snapshot.

No actor is attributed without evidence and no ref was undone. The exact observation is in `shared-ref-observations.json`. This is a snapshot-time integrity audit; future T3-managed shared-ref additions do not alter its statement about the review's unchanged HEAD/index/owned branch or reviewed bytes.

`immutable-audit.json`, `independent-receipt-audit.json`, `independent-verifier-v2.json`, `provenance-before.json`, `provenance-after.json`, all fresh logs, and all preserved attempts accompany this report. `audit-file-hashes.json` seals every pre-existing report/output file; `SEAL.json` identifies that seal and this report. The sealed files are made read-only after writing. No fixes are proposed or applied.
