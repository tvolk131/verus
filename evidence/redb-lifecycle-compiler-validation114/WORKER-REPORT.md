# Compiler-validation114 isolated execution handoff

The authorized implementation/validation completed: the pinned, offline, locked
full `rust_verify` package check passed, and the separately built filtered unit
test harness passed exactly the five requested capture-status tests. Both actual
Cargo child exit codes were 0, with owned process groups absent after completion.
There were exactly two compiler-producing commands, one Cargo job each, no retry,
no timeout/cancellation, and no in-scope source correction was needed.

The private retained root is
`/private/tmp/redb-compiler-validation114-worker-20261108T151020Z`.
No parent, frozen, native, runtime, old sealed evidence, or Git source was edited.
All authored files and the metadata adaptation were written through apply_patch
inside this new private root. Necessary dependency source staging was limited to
the frozen workspace's existing dependencies; installed registry/Git sources and
toolchains were read in place. No existing target or toolchain was copied, no
network/download/fetch or cache cleaning occurred, and no subagent was launched.

## Authentication and the actual source

- Revision2 MANIFEST.json matched the authorized SHA256
  `a586fe4665ee832499ea233969eb2914b941093186596f849d8cfeea9321c342`.
  All 21 listed payload hashes and byte lengths matched before overlay and again
  at final reconciliation. The authenticated payloads and manifest are retained
  in `revision2-authenticated-copy/`.
- BASELINE.json's three historical handoff paths matched their expected bytes
  and hashes. The corresponding three files in the frozen whole source matched
  independently before overlay. See the two `*-three-authentication.json` files.
- The complete frozen staged workspace inventory has 1,095 files. Its fingerprint
  is `cd8219020bced835a9c1853d80ecef22d569e2209f4530a5f74275818f34b020`.
  All selected overlapping source comparisons against the old runtime-binding
  matched. This is input authentication only; runtime adoption is not granted.
- Only the three authorized revision2 candidate files were overlaid. The frozen
  workspace's `source/`, `dependencies/{syn,prettyplease}`, `tools/`, README and
  rust-toolchain.toml were copied into `snapshot/`. The changed live parent source
  was not used for compilation.
- All 1,478 entry parent records matched before and after. Final audits also
  matched the frozen tree, original historical paths, revision2 payloads, 364
  inventoried external resolved package source trees, all 3,340 inventoried
  toolchain files, and selected platform-support identities.

`command-{1,2}.source-before.json` and `.source-after.json` bind both commands to
the same exact snapshot fingerprint:
`c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23`.
The source was unchanged during each command. Inventories use SHA256 of the
compact sorted-key JSON record array, sorted by relative path, as implemented in
worker.py. Individual file byte counts and SHA256 values are retained as well.

The preserved historical Cargo.lock SHA256 is
`30cd48d355d863cc714c0768bb1b69adb618d77fcd395c916acedb8e63456bc4`.
It was unchanged; this is the historical lock, not the live parent's different
lock. Metadata resolution used `--offline --locked` and succeeded without a
missing dependency or a sandbox/cache-lock escalation.

## Exact metadata and source deltas

`baseline-metadata-build.rs` preserves the historical build script.
`metadata-adaptation.patch` contains the isolated metadata-only delta. It adds an
explicit `VERUS_SOURCE_SNAPSHOT_SHA256` branch, validates the digest format,
avoids eager Git version/HEAD calls only for that explicit snapshot mode, and
uses unversioned identities rather than fabricated Git metadata. Normal Git
behavior remains in the other branch. Snapshot mode does not accept
VARGO_BUILD_VERSION/VARGO_BUILD_SHA overrides. The validation environment sets
VARGO_TOOLCHAIN explicitly so no rustup shim is used by the build script.

Both build scripts actually emitted:

```
VARGO_BUILD_PROFILE=debug
VARGO_BUILD_VERSION=unversioned-source-snapshot
VARGO_BUILD_SHA=unversioned:snapshot-sha256:c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23
VARGO_TOOLCHAIN=1.99.1-aarch64-apple-darwin
```

`actual-command-scopes.json` retains each actual raw build-script output and its
hash. The digest names observed source bytes, never a Git commit.

`complete-source-delta.patch` and `final-source-delta.json` show exactly four
changed source paths: the three exact revision2 candidate files and build.rs.
`source-correction.patch` is empty because there was no correction to revision2.
No semantic/authentication gate, assumption, admission, primitive, proof, native
producer, or trusted-law change was introduced.

## Command ledger and actual results

Both slots were reserved in RESERVATIONS.json before staging/setup. Full exact
argv, cwd, allowlisted environment, identities, initially empty target state,
resource samples, and expected scope were recorded before each launch in
`command-N.scope.json` and `.launch.json`. COMMAND-LEDGER.json combines scopes
and the final receipts. The direct Cargo executable for both was:
`/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/cargo`.
The cwd for both was `snapshot/source` under this retained root.

1. `cargo check --offline --locked --jobs 1 --package rust_verify --verbose`
   ran 2026-11-08T15:13:08.687506Z through 15:14:01.891616Z (53.203 seconds),
   exit 0. It used fresh `target-check/`. Raw Cargo verbose output confirms
   rustc_mir_build and rust_verify compilation/typechecking, including the
   rust_verify library and binary targets. The check emitted compiler metadata
   for the checked targets; it did not build a working rust_verify engine.
2. `cargo test --offline --locked --jobs 1 --package rust_verify --lib --verbose checked_normal_drop::origin_capture_status_tests:: -- --nocapture --test-threads=1`
   ran 2026-11-08T15:14:31.474817Z through 15:15:41.009236Z (69.533 seconds),
   exit 0. It used independent fresh `target-tests/`, without reusing command 1
   artifacts. Cargo compiled the dependency libraries and rust_verify library
   test harness and executed only the filtered five tests. The result was
   **5 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out**.

The exact selected names, all `ok`, are:

- `checked_normal_drop::origin_capture_status_tests::missing_capture_is_explicit`
- `checked_normal_drop::origin_capture_status_tests::unfinished_capture_is_not_completed_capture`
- `checked_normal_drop::origin_capture_status_tests::completed_empty_capture_is_not_missing`
- `checked_normal_drop::origin_capture_status_tests::completed_capture_without_matching_local_is_explicit`
- `checked_normal_drop::origin_capture_status_tests::matching_observations_do_not_establish_a_unique_occurrence`

`selected-test-results.json` checks the exact expected set and five passing
results. Both builds retained two dead-code warnings for the unestablished drop
site metadata fields; no warnings or diagnostics were suppressed by this task.
The test binary is retained at
`target-tests/debug/deps/rust_verify-051e18c17d4cd6f0`, with its individual hash in
`command-2.artifacts.json`. Both complete target inventories and targets remain
retained. No general test suite, engine release build, vstd build, native redb
run, standalone Arc probe, solver/proof replay, or diagnostic107-D1 was run.

## Tools, receipts, resource limits, and capture limits

Direct cargo/rustc/rustdoc paths point into the installed pinned 1.99.1 sysroot.
Raw version output reports cargo 1.99.1 (5f94df478 2026-08-27), rustc 1.99.1
(58cf8f9c3 2026-10-13), host aarch64-apple-darwin, LLVM 23.1.1. Sysroot,
target-libdir, CommandLineTools/SDK paths, clang/ld/ar paths, selected tool/SDK
hashes, and rustc linked-library declarations are retained in the preflight
logs and inventories. The configured four rustflags and Cargo [env] settings
were inherited from the preserved historical source/.cargo/config.toml. The
explicit environment disables incremental/debug-info growth and fixes jobs=1;
it adds no proof or acceptance gate relaxation.

Every managed version/support/Cargo-metadata/check/test launch has full raw
`.stdout` and `.stderr`, an exact launch record, a final receipt with true child
exit code, and resource samples. Check/test raw output is not truncated on disk.
EXPLORATORY-PREFLIGHT-NOTES.md separately discloses read-only shell inspection
failures and tool-output truncation before/outside the managed harness. Those
exploratory tool outputs remain in the conversation, not a claimed full raw
machine capture in this bundle. No compiler/setup failure was hidden or retried.

The wrapper owned separate process groups, sampled resources every roughly ten
seconds, imposed <=1,200-second command limits and the earlier task cutoff, and
verified groups absent after completion. There was no timeout, cancellation, or
incomplete process reported as success. The first clock was 15:10:20Z; the own
hard deadline was 15:50:20Z and optional/compile cutoff 15:38:20Z, earlier than
the global deadlines. Execution and retention finish well within that allowance.
Resource usage remains below 1 GiB in this fresh root, versus the 8 GiB cap;
observed free space remained above 16 GiB, including concurrent global disk
changes. final-resource.json records the final retained allocation and minimum
sampled free space rather than assuming free space is constant.

Cargo verbose records direct rustc/build-script/test invocations, not a complete
OS process trace. Inventoried installed compiler/shared-library/SDK inputs are
observations, not proof of Git continuity, bootstrap correspondence, OS dyld
shared-cache closure, or compiler correctness. Dependency inventories are a
conservative superset of resolved workspace packages; actual invocations and a
resolved rust_verify package graph are separately retained. No source-to-binary
or source-to-core attestation is claimed.

## Unchanged acceptance boundaries

The five tests exercise only the pure origin_status classification helper.
They do not execute actual original Builder temporary capture, elaborated MIR,
drop glue, destructor/resource behavior, cache correspondence, or database
obligations. Typechecking those modules does not establish those semantics.
No output is adopted or integrated. All 18 native redb final cells remain OPEN;
source path remains FAILED; Git continuity remains UNAVAILABLE;
compiler/source remains UNESTABLISHED. Conditional/unadopted TCB restrictions,
the native Mutex/BuildHasherDefault admission and original lifecycle proof,
S0/Infer/full-scope/import/export fail-closed gates, and earlier CHANGES REQUIRED
review status remain in force. Goal service is last observed BLOCKED and was
not altered or agent-resumed.

The parent must independently audit this handoff and decide any next work.
No additional review, task, compile, test, adoption, or integration is scheduled.
SEALED-MANIFEST.json inventories the retained payloads, including full source,
raw logs, exact deltas, receipts, targets and inventories. Its checksum is in
SEALED-MANIFEST.sha256 and the handoff response.
