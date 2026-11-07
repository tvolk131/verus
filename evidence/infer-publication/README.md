# Export publication after complete requested checks

This follow-up is based on merged PR #10, commit
`f808257383b353ff9c11c43e846bfd3ade1ef8c8`. It isolates publication hardening
from the larger, still-uncommitted inference and redb experiments.

## Scope

All requested VIR exports wait until verification and any requested erased-code
checks/native compilation have succeeded. This includes ordinary exports when
Infer is disabled, not just interfaces containing inferred contracts.

Known direct export paths are invalidated before parsing. Cargo-derived paths
are invalidated when the compiler resolves them. Invalidation does not depend
on successfully decoding an old interface or finding an inference marker.
An export that fails late is reported through the selected Rust diagnostic
format, and the invocation and publication report remain unsuccessful.

Cargo-verified invocations disable rustc's early artifact notifications in the
parsed compiler configuration for both compiler passes. Cargo consequently
waits for successful process completion, after interface publication, before
starting dependent crates. Unverified direct-rustc wrappers retain their usual
behavior. This does not rewrite JSON options: response-file options are covered,
malformed options still receive rustc's own errors, and other JSON diagnostics
and ordinary direct-compilation artifact notifications remain available.

Successful ordinary exports retain the legacy format. Explicit ordinary
`--no-verify` remains available and is not a proof certificate. Existing rlibs
are not removed or promoted into evidence of a successful current verification.
The inference policy stays v63, its metadata schema stays 38, and this patch
adds no primitive specifications, application axioms, resource semantics, or
inference eligibility extensions.

The production changes are limited to `driver.rs`, `import_export.rs`, and
`verifier.rs` under `source/rust_verify/src`. The other changes are publication
and Cargo regressions, the assurance paragraph, and this report.

## Reproduce

Use the pinned Rust toolchain and Z3 setup from `BUILD.md`. From `source`:

```sh
cargo build --locked --release \
  -p rust_verify -p verus -p vstd_build \
  -p verus_builtin -p verus_builtin_macros -p verus_state_machines_macros
cargo test --locked --release -p rust_verify_test \
  --test infer_publication -- --test-threads=1
cargo test --locked --release -p rust_verify_test --test cargo -- --test-threads=2
```

The publication fixtures do not require a vstd build. They use freshly built
compiler/macros, temporary provider/consumer files, and the configured Z3.
No private temporary source tree or historical runtime is needed to reproduce
them from this commit.

## Review and validation

Independent static review found no publication-barrier or soundness blocker
in the isolated candidate. It identified one P2 diagnostic-format regression:
the first late-error path printed plain text even for JSON diagnostics. The
parent repairs that path using `EarlyDiagCtxt` with the compiler's selected
format. A new filesystem-failure regression requires every stderr line to be
JSON and the failed interface to remain unpublished. Another new regression
exercises erased-code checking without requesting native compilation.

Original independent report SHA256:
`c2fe5b454eb8150a7b970e0962b5faeea5c1228c304f58875d32d7d2beca9ba6`.
That review is static inspection of the original candidate, not a claim of
independent execution of the parent's repair or new tests.

Fresh validation of the isolated source snapshot passes:

- Release compiler and matching macro builds.
- Matching vstd: 2,221 verified, zero errors.
- VIR library: 134 tests passed.
- All 28 Infer integration binaries: 582 tests passed, zero failed, ignored,
  or filtered out. The nine publication tests are included in that total,
  not nine additional distinct tests. Their separate focused run also passed.
- Infer tooling: the same 136 tests pass normally and with Python `-O`.
- Formatting of the changed Rust files and patch whitespace checks.

These totals describe the initial PR snapshot, before the Cargo-readiness
correction. Its first full-test CI run exposed a missing dependency interface:
the three-crate Cargo test failed in parallel and passed with one build job.
The first correction filtered literal JSON options; a static review identified
response-file and malformed-option gaps, plus a unit test outside the selected
CI packages. The final correction instead changes parsed configuration and
places its regressions in the CI-selected integration binaries. The Cargo
regression requests four build jobs and response-file artifact notifications,
with separate cold/warm build and verify targets. Additional regressions check
malformed joined/separate JSON options and preserved direct artifact reporting.
The earlier failed run and superseded filtering experiment are not relabeled
as successful validation of the final correction.

Fresh validation of the final Cargo-readiness correction passes:

- Matching release compiler rebuild.
- All 28 Infer integration binaries: 583 tests passed, zero failed, ignored,
  or filtered out. The ten publication tests are included, not additional
  distinct tests; their separate focused run also passes.
- All 21 Cargo integration tests, including forced-parallel cold/warm build
  and verify with response-file notifications, malformed JSON options, and
  temporary-project creation.
- Formatting of all four changed Rust files and patch whitespace checks.

The first broad local Cargo run passed 19 tests and failed two temporary-project
creation tests because inherited snapshot-build `GIT_DIR`/`GIT_WORK_TREE`
variables redirected `git init` to the protected parent repository. The same
compiled test binary passes all 21 with those variables removed and the pinned
compiler library path configured. An initial direct launch lacked that library
path and executed no tests. These setup failures remain failures, not successful
validation; no product-code workaround was added for them.

This is not a claim that every workspace suite or doctest was run.
The initial archive-only compiler build failed because the version build script
requires Git metadata. Its failure is retained, not counted as a successful
build. Subsequent builds bind the source snapshot to the parent Git identity
explicitly and keep generated artifacts outside the working tree.

## Limits

These checks do not establish whole-redb correctness, native storage or crash
durability, Cargo lifecycle completeness, filesystem atomicity, or independent
proof checking of imported libraries. The Cargo cases are regression scenarios,
not an exhaustive concurrency or scheduling proof. Machine-local campaign evidence and
the unfinished ownership/resource engine are not part of this PR.
