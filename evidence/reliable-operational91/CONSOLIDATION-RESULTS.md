# Campaign91 consolidation handoff

Implementation pass complete; campaign acceptance remains **PARTIAL**. Final
review, final source-built runtime qualification and the operational native
reference belong to the parent. No subdelegation or goal API calls were used.

## Exact changes against the parent's frozen baseline

Nine preexisting paths changed, all within the preimplementation freeze:

1. `source/tools/stateful_bitmap_workflow90/run.py`
2. `source/tools/check_fedimint_prefix_infer79.py`
3. `source/tools/test_stateful_workflow90.py`
4. `source/tools/test_fedimint_prefix_infer79.py`
5. `source/tools/STATEFUL_BITMAP.md`
6. `source/vir/src/ast.rs`
7. `source/vir/src/ast_to_sst.rs`
8. `source/vir/src/modes.rs`
9. `source/vir/src/resolution_inference.rs`

New paths are `source/tools/workflow_runtime.py`,
`source/tools/test_workflow_runtime.py`, this results document and
`evidence/reliable-operational91/CONSOLIDATION.md`. The authorization scope is
parent-owned. No other frozen path changed. Rust changes are **doc-comment only**
against the already-dirty baseline, not against HEAD; executable portions compare
equal. Preexisting dirty edits were not reset, reformatted or replaced.

The one migrated transfer is native Fedimint prefix-infer79. Fifteen specialized
functions remain AST-identical to the frozen baseline: render, correspondence,
command, exact, scope, certificate, check_certificate, imported, failed,
failure_location, native_mutants, mutant_goal, frontend_controls,
identity_sources and completion. Exact theorem/body/source inventories, trusted
models, certificate/import dependencies, negative diagnostics and seeded-export
removal remain local. Shared positive-outcome checks retain exact boolean/integer
types and complete-crate flags. Stateful qualification, fixture hashes, original
model oracle, independent executable hashes, two runtime destinations and all
specialized ON/OFF checks remain intact.

The shared module is a collection of functions, not a new receipt architecture:
streaming digests, regular-file artifacts, strict typed equality/JSON, environment
overlays, runtime staging, exact command/stream/input binding, subprocess logging
and common outcomes. Stateful orchestration retains reference88's builder and
application logic and its launch budgets, substituting the shared executor.
Fedimint binds the command's actual input path and bytes before/after execution;
an independently supplied source/report pathname is insufficient. Logs establish
consistency, not cryptographic execution attestation. Each reused provider input
is validated before the next case overwrites it; label snapshots retain case text.

## Fresh validation and commands

All private logs are under
`/private/tmp/verus-operational91.x12fMK/consolidation`. Commands below ran in the
current checkout, with no network. The existing development build target was
reused; proof outputs were new directories, not cached proof results.

### Full VIR, unit plus documentation

Fresh final command, invoked through the existing absolute Rust toolchain:

```sh
/Users/tommyvolk/.cargo/bin/rustup run 1.99.1-aarch64-apple-darwin \
  /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/cargo \
  test --offline --locked --release -p vir -- --nocapture
```

Result: **178 unit tests and 13 executable doctests pass, zero ignored**.
The other five formerly failing blocks are explicitly labelled VIR/prophecy/
two-phase pseudocode (`text`). Real borrowing, partial moves, array patterns,
closures, returned references and two-phase method calls execute assertions;
there is no doctest disable, blanket ignore, no-run or executable Rust change.

Final wall time was **107.073 s**; unit execution 2.70 s and executable doctests
0.01 s (rustdoc total 0.44 s, merged compilation 0.26 s). The exact environment,
argv and complete logs are `vir-full-invocation.json`, `vir-full-result.json`,
`vir-full.stdout` and `vir-full.stderr`.

Cargo home is exactly
`/private/tmp/verus-workflow90-repair2/build/cargo-home`. HOME, TMPDIR,
XDG cache and Rustup home also point into that private build directory. The test
target is `/private/tmp/verus-workflow90-cycle2.DJ31Jt/build/target`. Shared
toolchain paths are read-only inputs. This is a fresh regression with reused
development build artifacts, **not** a fresh verifier/vstd build or cold benchmark.

### Python readers and adversarial controls

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=source/tools python3 -B -m unittest \
  test_workflow_runtime test_stateful_workflow90 test_fedimint_prefix_infer79
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=source/tools python3 -O -B -m unittest \
  test_workflow_runtime test_stateful_workflow90 test_fedimint_prefix_infer79
```

Both pass **53 tests**: normal **10.856 s** wall / 10.638 s unittest;
optimized **9.274 s** wall / 8.965 s unittest. Full command/environment/results
are in `checks/python-{normal,optimized}/{invocation.json,result.json,stdout,stderr}`.

New controls include coherently rehashed malformed/duplicate-key reports,
scope-changing commands, source/environment splits, same-bytes/different-path
artifacts, input/output drift, symlinks, boolean counts/status, timeout/spawn
failures and shared-executor budget exhaustion. Ten isolated in-memory guard
removals are killed under both modes: shared command equality, output binding,
environment binding, input binding, artifact symlink exclusion, inference
outcome; specialized Fedimint scope and seeded-export removal; specialized
stateful compiler-manifest and trusted-model/import-oracle acceptance.
No production guards are disabled; unit fixtures are explicitly synthetic.

### Migrated actual Fedimint proof replay

```sh
python3 -B source/tools/check_fedimint_prefix_infer79.py \
  --verus /private/tmp/verus-workflow90-cycle2.DJ31Jt/runtime/verus \
  --solver /private/tmp/verus-workflow90-cycle2.DJ31Jt/runtime/z3 \
  --application /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin \
  --output /private/tmp/verus-operational91.x12fMK/consolidation/fedimint-replay
```

A second fresh replay uses `python3 -O -B` and output
`/private/tmp/verus-operational91.x12fMK/consolidation/fedimint-replay-optimized`.
Both return **PASS, 29 cases**: all provider/consumer and mutant-exact positives,
four native semantic mutants, false-goal and safety negatives, six frontend
controls, malformed-goal and five metadata/import negatives. Optional historical
`--baseline-verus` was not requested or run. There are **58 actual verifier
launches total**, with no setup failures within these two replays.

Normal wall time **33.380 s**, summed invocation time **14.243 s**, summed
validation time **0.381 s**. Optimized wall time **36.115 s**, invocation
**15.623 s**, validation **0.398 s**. Case receipts are in each replay's
`receipt.json`, command/stream bindings in `runs/<case>/`, and outer commands in
`checks/fedimint-replay{,-optimized}/`. The optimized replay follows the final
test additions; production Python is identical between the two actual replays.

Cold, warm and hostile candidate-cache cases continue to require regeneration
and fresh proof work. Neither a warm preparation nor a candidate-cache file is
proof-result acceptance. These two runs differ in Python mode and were not three
matched repetitions; **no speedup or cold/warm performance claim is made**.

## Timing instrumentation

Invocation `seconds`, runtime staging `staging_seconds` and validation
`validation_seconds` use `time.monotonic()`. Fedimint rows expose
`invocation_seconds`; timed validation includes input/stream binding and the
specialized positive/negative validators. Setup, guard snapshot hashing and
some certificate/completion checks remain outside that validation subtotal;
subtracting it from overall wall time is not a clean isolated cost measurement.
Stateful classifications time valid and invalid evidence. Staging timing covers
copying and runtime artifact hashes. The source-qualified stateful gate stopped
before fresh matrix staging, so there is no accepted fresh end-to-end stateful
staging/verification measurement. Instrumentation is ready for the parent's
later controlled before/after measurements; no optimization cycle was spent.

## Failures, bounds and remaining blocker

All test/setup failures in this pass:

1. First full VIR test-build: 178 units passed, 12/13 doctests passed; one added
   assertion attempted indexing a partially moved non-Copy array (E0382).
   Removed that invalid assertion, retained the real move and its assertion,
   then reran the full command green. Initial wall **164.143 s**; complete logs
   are in `vir-initial/`. The prior 18-failure logs remain untouched.
2. Initial focused Python run: **39 tests, one failure**, because an existing
   rejection-message regex expected `inference status`. The common check
   correctly rejected the forged outcome, but its new message differed. Kept
   the check and made the diagnostic compatible. Logs are
   `python-initial.{stdout,stderr}`; unittest time 6.334 s. Intermediate expanded
   run: **51 tests pass**, 7.843 s, `python-expanded.{stdout,stderr}`.
3. Actual stateful replay attempt: **7.193 s**, expected fail-closed
   `ValueError: compiler source drift`, before any verifier/test/build/vstd
   launch. The attempted command used `run.py --runtime` with the runtime above,
   native `/private/tmp/verus-transfer-redb-miuzm2w3/redb`, the private Cargo
   cache above, and repair2's `test-executables.json`. Full argv/logs are in
   `checks/stateful-replay/`; `stateful-replay/launch-counts.json` contains four
   zeroes. No changed qualification or bypass was used.

Minor discovery/edit diagnostics: one unmatched shell glob and one unrelated
denied `/private/tmp/codex-daemon-501` directory during read-only discovery;
existing pinned checkout paths were used. One documentation patch context
mismatch was reapplied without changing scope. An initial private ref audit
compared two different output formats; the final audit uses the baseline's
`refname objectname` format. None is proof evidence or a verifier launch.

Resource usage: **zero verifier compiler builds, zero matched-vstd runs; two
Cargo VIR test-build invocations**, separately itemized above. No further build
is attempted in this pass, even if those two test builds are counted toward the
compiler-build bound. Focused Python process/timeout controls are tooling tests,
not application verifier launches. No network/shared Cargo/toolchain writes,
commits, pushes, branch/ref mutations, native edits or new application axioms.

The old runtime is byte-identical to its historical qualification, but **is not
built from the newly edited full source**. Its frozen compiler identity is
`d0c7852d41e69749e079c2a94bb22f736f040e2607bc94e6aeb14af01d25bb37`;
current compiler identity is
`b6294b48c66cb799b0793b2bdbddc02ff489f23084eaee4eb7464ffcbdd94337`.
`qualification.json` and `expected.json` are unchanged. The old runtime's
matched vstd is used honestly for the migrated transfer, not relabelled as a
campaign91 source build. A final source-built runtime, exact qualification,
fresh stateful matrix/regressions and parent-owned independent review remain
necessary before full workflow acceptance. This pass does not claim an
operational allocator theorem or close the old blocked goal.

## Preservation evidence

Private `baseline-changes.json`, `doc-comment-only.json`,
`fedimint-specialized-oracles.json` and `preservation.json` record:

- All **1401 immutable files unchanged** and **1331 native files unchanged**;
  Fedimint/redb remain clean at their original pins.
- Owned HEAD `3b05c1912b9d7314ad00c832c4065f4362562e02`, branch and index hash
  `0cbac76f360da036c17112dde3cf940baf5e069bb82de7f5996da12c9f0cabfa` unchanged.
- Two shared orchestration checkpoint-ref additions observed relative to the
  parent's baseline, no removals; names/hashes recorded without attribution.
- Nine frozen-path changes only, four new owned files, and all four Rust
  executable portions unchanged. All fifteen inventoried Fedimint specialized
  oracles compare equal. Current top-level RFC/direction documents had no
  directly relevant literal stale identity claim to repair; no broad cleanup.

Stop here. The implementation is ready for parent review, with the source-bound
runtime/stateful replay gate explicitly still open.
