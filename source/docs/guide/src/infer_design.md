# Infer implementation and testing

## Pipeline and boundaries

The Rust frontend records inferred definitions and their original bodies.
`vir::infer` generates bounded candidates in dependency order; required soundness
and precision checks run through ordinary verification before callers consume a
summary. Proof assistance inserts ordinary annotations and requests the needed
whole-function well-formedness/mode rechecks. Generation/report records never
serve as proof receipts.

Keep these boundaries independent:

1. Eligibility recognizes supported source, types, calls and models.
2. Generation proposes a contract or pure annotation.
3. Structural validation preserves original executable/contract information and
   checks insertion placement and fresh identities.
4. Ordinary verification discharges the required proof obligations.
5. Publication/import validates complete success and compatible dependency,
   candidate and trusted-model identities.

An ID, hash, source match or successful generation step does not discharge a
solver query. Candidate caches contain reusable generation data, not proof results.
Changes to serialized formats, trusted models or safety policy need explicit
compatibility review; their identifiers have distinct roles.

Core generation modules live under `source/vir/src/infer/`; checking/import logic
is in `source/vir/src/infer_checks/` and the frontend API/report boundary.
`assistance.rs` contains shared annotation plumbing. Preserve separate eligibility
and independent projection/proof checks when consolidating builders or traversal.

## Test layers

- Rust unit tests cover generation, IR preservation, identity allocation and
  compatibility boundaries.
- Rust integration tests run the real verifier, selected imports/exports and
  semantic/safety/unsupported cases. Canonical inputs live under
  `source/rust_verify_test/fixtures/infer/`.
- Python tooling tests validate execution, timeout, report, preparation and
  transfer oracles under both normal and optimized Python.
- Application/benchmark entry points run selected pinned external slices and
  preserve failure categories, source fidelity and declared trust.

Normal tests keep solver/resource failures distinct from intended negatives.
Test coverage should name actual checked bodies and helper identities, including
an empty helper set. Positive precision must not be inferred from a total count.

## Running tests

Build a matching Verus/compiler/vstd bundle first, following `BUILD.md`. The
consolidated Infer runner is documented in `source/tools/README.md`. Existing
Cargo/vargo test commands remain supported. `VERUS_TEST_ROOT` optionally selects
an explicit bundle for integration tests; otherwise their normal build-layout
resolution remains. `VERUS_Z3_PATH` selects the solver.

Use normal Rust/Verus checking. Model-only other-pointer-width tests already have
explicit exceptions; those do not establish native erasure for another target.
Use separate output/cache directories when comparing builds or candidates.
Keep raw reports out of Git and upload CI artifacts on failure.

## Regression inputs and source provenance

The fixture manifest records archived origins and exact contents. The Rust
harness supplies modes/options and expected results. The allocator case preserves
its original six-file assembly and cfg-selected controls; the quorum case keeps
its semantic assumptions. These tests do not upgrade the recorded scope into
whole-REDB/Fedimint correctness.

CI validates fixture hashes and executes maintained tooling/compiler regressions.
It no longer requires old review directories or retired experiment controllers.
Historical source guards and their inputs remain available at the archive point.

## Research custody

Complete research and review history is preserved at commit
`7dcabedaa93299a3607746b6ec25f5e4778288a4`, tag
`archive/infer-research-2026-11-12`. The branch
`experiments/infer-native-cell` also preserves unfinished native-cell work.
These are local refs until explicitly published. No history has been rewritten.
Private runtime binaries remain separate artifacts; source history and JSON
receipts alone are not a hermetic reproduction environment.

The live tree keeps implementation, maintained tests, examples and current
limitations. Historical raw JSON/stdout/stderr, copied snapshots, failed-trial
journals and intermediate models belong to the archive. See the
[capability table](infer_capabilities.md) before attributing application guarantees.
