# Infer testing and evaluation

Run from a clean checkout with the repository's Rust toolchain and a normally
built matching compiler/vstd bundle. The runner uses existing Rust and Python
oracles; its receipt is test execution metadata, not an application proof.

```sh
# Fast harness/provenance tests, normal AND optimized Python; no Verus build.
python3 -B source/tools/run_infer_tests.py --suite tooling

# Real compiler/unit/integration tests against an explicit matching bundle.
python3 -B source/tools/run_infer_tests.py --suite compiler --release \
  --verus-root "$PWD/source/target-verus/release"

# Retained REDB allocator and Fedimint quorum proofs and semantic/safety controls.
python3 -B source/tools/run_infer_tests.py --suite applications --release \
  --verus-root "$PWD/source/target-verus/release"

# Focused rerun; compiler unit tests still run.
python3 -B source/tools/run_infer_tests.py --suite compiler --release \
  --target infer_buffer_observation_goals --target infer_word_sequence_goals \
  --verus-root "$PWD/source/target-verus/release"

# Heavy source-mutation and cold/warm/Cargo evaluations, explicitly requested.
python3 -B source/tools/run_infer_tests.py --suite nightly --release \
  --verus-root "$PWD/source/target-verus/release" --timeout 3600
```

`--suite all` runs tooling/compiler/applications, not nightly. `--list` prints the
selected targets without building/running. `--output` must name a new directory;
the default is a persistent temporary directory. Commands/stdout/stderr and one
compact receipt go there, never into the repository. `--offline`, `--cargo`,
`--solver`, `--jobs`, `--batch-size`, `--test-threads` and `--timeout` support explicit local setups.
Existing Cargo environment variables remain available for private caches/builds.
Rust targets are built once and run in bounded batches of exact test names;
selection and executed/ignored counts are checked. Integration tests use two
workers by default; compiler units run serially. `--target` narrows integration
coverage explicitly, and the receipt records the selected targets. A timeout fails the run and
retains output. Existing ignored tests are reported, not counted as executed.

The nightly mutation harness builds private source mutations; allow its build
and solver costs explicitly. Full reports and performance data are CI artifacts,
not golden proof receipts. Pin the toolchain and compare matching inputs before
claiming a speedup.

## Existing components

- `infer_experiment.py`: shared required proof/body/export validation and logged
  commands. Scenario-specific oracles retain source/domain/function expectations.
- `infer_eval_support.py`: subprocess timeouts, artifact stability and measurements.
- `infer_eval/cases.json`: maintained scalar/loop/evaluation case catalog; failures
  and unsupported cases retain separate status categories.
- `check_infer_transfers.py` / `prepare_infer_transfers.py`: pinned external helper
  transfers and fresh source checkout preparation. Pass explicit application,
  verifier, solver and output paths; source fidelity and exclusions remain required.
- `test_infer_assurance_mutations.py`: source mutations must be rejected by the
  actual policy/precision tests; `--policy-only` is the cheap acceptance core.

Canonical proof inputs and compact provenance are under
`source/rust_verify_test/fixtures/infer/`. Historical experiment/review controllers
and reports are at tag `archive/infer-research-2026-11-12`, not live test dependencies.
The [guide](../docs/guide/src/infer.md) and
[capability table](../docs/guide/src/infer_capabilities.md) describe current scope.
