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

Successful ordinary exports retain the legacy format. Explicit ordinary
`--no-verify` remains available and is not a proof certificate. Existing rlibs
are not removed or promoted into evidence of a successful current verification.
The inference policy stays v63, its metadata schema stays 38, and this patch
adds no primitive specifications, application axioms, resource semantics, or
inference eligibility extensions.

The production changes are limited to `driver.rs`, `import_export.rs`, and
`verifier.rs` under `source/rust_verify/src`. The other changes are nine
publication regressions, the assurance paragraph, and this report.

## Reproduce

Use the pinned Rust toolchain and Z3 setup from `BUILD.md`. From `source`:

```sh
cargo build --locked --release \
  -p rust_verify -p verus -p vstd_build \
  -p verus_builtin -p verus_builtin_macros -p verus_state_machines_macros
cargo test --locked --release -p rust_verify_test \
  --test infer_publication -- --test-threads=1
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

This is not a claim that every workspace suite or doctest was run.
The initial archive-only compiler build failed because the version build script
requires Git metadata. Its failure is retained, not counted as a successful
build. Subsequent builds bind the source snapshot to the parent Git identity
explicitly and keep generated artifacts outside the working tree.

## Limits

These checks do not establish whole-redb correctness, native storage or crash
durability, Cargo lifecycle completeness, filesystem atomicity, or independent
proof checking of imported libraries. The fresh regressions cover direct
export paths; Cargo-derived resolution also receives static review, but no new
end-to-end Cargo replay is claimed here. Machine-local campaign evidence and
the unfinished ownership/resource engine are not part of this PR.
