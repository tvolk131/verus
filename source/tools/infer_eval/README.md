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

The manifest contains 40 curated cases: nineteen positive examples and twenty-one
expected failures. They exercise unsigned and signed arithmetic, casts, arrays,
checked arithmetic, `Option`/`Result`, early returns, `let-else`, guards/or-patterns,
widened `?` errors, fixed-array outputs, shared subslices, exact and sound
composition, counted-loop invariants/termination, constant/parameter steps,
subtraction/countdown counters, symbolic `usize` bounds, bitwise/shifts, wrapping/saturation, mixed-sign checks and negation,
`isize` arithmetic and checked methods, architecture-specific interval
summaries, 32/64-bit casts, caller
properties, call-graph depth, unsupported loops/mutable slices,
analysis budgets, required solver limits, and frontend errors. Depth cases
generate deterministic chains of four and sixteen functions. The eleven
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
Broader real-world suites, statistical timing repetitions, peak-memory profiling,
and Cargo incremental-build timing remain future evaluation work. Do not compare
debug/release timings or different machines as though they were controlled runs.
Benchmark sources intentionally include invalid code and proof failures; run
them through the manifest runner instead of compiling the directory as a crate.
