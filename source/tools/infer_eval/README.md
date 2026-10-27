# Verus Infer evaluation corpus

From the repository root, after building Verus:

```sh
python3 source/tools/eval_verus_infer.py \
  --verus source/target-verus/release/verus \
  --output-dir /tmp/infer-evaluation
```

The output directory must be new. With no `--output-dir`, the runner creates a
persistent temporary directory and prints its location. `--case signed` selects
one case; repeat the option to select several. `--timeout` bounds each compiler
process and defaults to 180 seconds. No packages or network access are needed.

The manifest contains 47 curated cases: twenty-two positive examples and twenty-five
expected failures. They exercise unsigned and signed arithmetic, casts, arrays,
checked arithmetic, `Option`/`Result`, early returns, `let-else`, guards/or-patterns,
widened `?` errors, fixed-array outputs, shared subslices, exact and sound
composition, counted-loop invariants/termination, constant/parameter steps,
subtraction/countdown counters, symbolic `usize` bounds, bitwise/shifts, wrapping/saturation, mixed-sign checks and negation,
`isize` arithmetic and checked methods, architecture-specific interval
summaries, 32/64-bit casts, caller
properties, call-graph depth, unsupported loops/mutable slices,
analysis budgets, required solver limits, and frontend errors. Depth cases
generate deterministic chains of four and sixteen functions. The twelve
standalone examples are shared with the documentation.

The explicit 32/64-bit cast and platform-integer probes use `--no-erasure-check` to inspect the
verification model without evaluating the native target's global-size assertions.
Their exact arguments are recorded; they do not test cross-target native codegen.
The portable `usize` example retains normal erasure checks and is separately
compiled and run on the host.

Each case runs with its own empty candidate cache, then again with that cache.
The runner checks the exit status, report schema and result, actual interface
publication, required proof outcomes, and cache observations. Warm candidates
must have the same displayed predicates and freshly established required proofs.
It does not compare candidate IDs across runs: these bind actual typed formulas,
including source spans, and can change when cached formulas are reconstructed.
The incremental case then changes one leaf, measuring transitive invalidation
and reuse of an independent candidate. Solver results are never cached.

Artifacts include the source and its hash, exact command, available compiler
stdout/stderr, full structured reports, timing data, verifier build information,
executable hash, manifest hash, `results.json`, and `summary.md`. The runner exits
nonzero on any unexpected outcome. A process timeout, missing report, malformed
JSON, or zero contracts in a positive case is a harness failure, never a successful
negative test. The Rust integration tests separately exercise corrupt caches,
stale-export removal, imported fingerprints, and multi-crate composition; the
three-crate Cargo example remains the Cargo regression fixture.

Measurements cover:

- Coverage and failure categories within this corpus, with feature labels.
- Requested precision, separately observed soundness/maximality/exactness, and
  success or failure of explicit caller properties.
- Expanded precondition and postcondition AST node counts.
- Candidate-generation time, candidate SMT time, whole invocation time, and
  the existing `--time` compiler/solver breakdown.
- Cold/warm candidate hits and incremental invalidation cost.
- Verification cost for two call-graph depths.

Times are observations without pass/fail thresholds. Generation time includes
eligibility, dependency traversal and cache I/O; per-candidate time includes the
completed candidate's eligibility, decoding or synthesis, and cache write. SMT
time includes ordinary and precision queries and diagnostic reruns. AST counts
include repeated subexpressions, rather than counting shared allocations once.
Failed generation can leave fewer completed candidates than requested functions;
the per-function metrics describe only completed candidates. The overall
generation timer still includes failed work. Total invocation time also includes
imports and compiler startup.

This is a reproducible baseline for selected features and known limits, not a
general Rust coverage percentage, a soundness proof, or a performance guarantee.
Broader external application suites and simultaneous process-tree peak-memory
profiling remain future evaluation work. Do not compare
debug/release timings or different machines as though they were controlled runs.
Benchmark sources intentionally include invalid code and proof failures; run
them through the manifest runner instead of compiling the directory as a crate.


## Repeated measurements and artifact stability

```sh
python3 source/tools/eval_verus_infer.py --case applications \
  --verus source/target-verus/release/verus --repetitions 3 --memory \
  --output-dir /tmp/infer-app-repetitions
python3 source/tools/test_infer_evaluation.py
```

Every repetition has an independent source directory and empty candidate cache.
“Cold” means a cold **candidate cache**, not cold OS page caches or a new solver
installation. Each warm run uses its paired cache and checks the actual predicates
and fresh required proof outcomes. Runs are sequential within the runner. Complete
builds before evaluation, and avoid competing workloads when comparing timings.
A single repetition remains the default; repetitions are explicit observations,
not a claim of statistical significance.

Result schema 2 records the planned/actual run count, repetitions, raw observations,
and per-case/per-phase count, minimum, median, nearest-rank p90 and maximum wall
time. Expected proof failures are valid observations when they match the manifest;
unexpected outcomes and harness failures are excluded from timing statistics and
counted separately. Candidate generation and solver times remain distinct from
whole-invocation wall time. Artifact hashing and optional memory-worker startup
are outside the measured invocation. Candidate IDs can vary across repetitions
because source spans and source paths are part of their identity.

`--memory` uses a fresh Python worker per invocation and reads
`getrusage(RUSAGE_CHILDREN).ru_maxrss` after the compiler exits. Values are converted
from bytes on macOS and KiB on Linux. The metric is named
`waited_child_max_rss_bytes`: it is the OS high-water observation for waited child
work, not a summed or simultaneous process-tree peak. Unsupported systems report
an unavailable reason. It includes compiler startup, imports and checking; it does
not isolate the inference pass or measure retained memory. A host timeout remains
a harness failure and supplies no proof evidence.

The runner fingerprints the launcher, driver, solver, vstd interface/native library,
builtin/macro libraries, manifest and selected source files before starting. It
checks them before and after every invocation and again at completion. A missing,
replaced or changed artifact aborts remaining runs and writes an explicit harness
failure plus observed artifact changes to `results.json` and `summary.md`. This
prevents combining observed different builds; it is not continuous monitoring.
The result schema and corpus-manifest schema are independent (manifest remains 1).

## Cargo incremental edits

```sh
python3 source/tools/eval_verus_infer_cargo.py \
  --verus source/target-verus/release/verus --repetitions 3 --memory \
  --output-dir /tmp/infer-cargo-repetitions
```

This runner materializes a reproducible three-crate fixture in each repetition's
output directory, using the checkout's vstd. Cargo runs offline with an isolated
target directory and candidate cache. The verifier profile is selected by
`--verus`; the generated application fixture uses Cargo's default development
profile. Each repetition measures five sequential scenarios:

1. Cold Cargo compilation and verification, including dependencies.
2. A no-edit warm run; Cargo should reuse outputs without fresh inference reports.
3. A body-only leaf edit with unchanged displayed contracts; all three crates
   verify again, recording candidate-level hits/misses. Source-span or build
   provenance changes may invalidate even an independent candidate; this is an
   observation, not a benchmark failure or a promise of unchanged cache keys.
4. A leaf semantic contract edit; the unchanged application property must fail,
   and its old inferred interface must disappear.
5. An updated application property; verification and interface publication resume.

It records exact source snapshots, commands, outputs, fresh per-crate inference
reports, ancillary no-inference reports, interface hashes/timestamps, elapsed time,
optional memory and per-phase statistics. It checks required proof outcomes and
actual publication, not merely elapsed time. Cargo's normal no-edit cache is
explicitly distinct from Infer's formula-only candidate cache: a skipped Cargo
invocation supplies no new proofs. Compiler/model/source stability is checked
around every invocation. These small controlled edits complement the broader
`examples/verus_infer_workspace` regression; they do not estimate general Cargo
incremental performance.

The iterator checksum is rejected by the existing Verus frontend before Infer
generation. Its structured status is `verification_failed`, with an explicitly
checked “is not supported” diagnostic; it is not mislabeled as an Infer analysis
rejection. Unsupported Infer syntax and incomplete proofs retain their distinct
manifest/report categories.
