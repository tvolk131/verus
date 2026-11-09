# Campaign91 source-runtime qualification and measured costs

Local implementation result, November 4, 2026 (America/Chicago): **source-bound
qualification and three measured stateful replays pass; full campaign remains
PARTIAL**. No new Infer design, operational allocator theorem, optimization,
independent acceptance, goal transition or subdelegation is claimed.

## Ownership, freeze and artifacts

Private root: `/private/tmp/verus-operational91-runtime.q91` (abbreviated `Q`
below). Only these repository paths change in this task:

1. `source/tools/stateful_bitmap_workflow90/qualification.json`
2. `source/tools/STATEFUL_BITMAP.md`
3. This document.

Before building, `Q/compiler-source-manifest.json` freezes **1,084 files** and
`Q/full-source-manifest.json` freezes **2,272 nonignored files**. Both are checked
unchanged through compiler, vstd and harness builds. `Q/freeze.json` records:

- Compiler manifest identity, including test source:
  `e474a327b1a90258200b82eb9b6525e35cc9741fcca99f488cc5b040a61c019b`.
- Existing gate's compiler identity, retaining its existing single test-only
  source exclusion:
  `b6294b48c66cb799b0793b2bdbddc02ff489f23084eaee4eb7464ffcbdd94337`.
- Prebuild full manifest identity:
  `488e3d4aa250ff236b54af45ab0a7cd6a868445476bcdcb9e8887911ec6d1fca`.

The only compiler-source differences from the prior qualified source are the
four consolidation doc-comment changes in `ast.rs`, `ast_to_sst.rs`, `modes.rs`
and `resolution_inference.rs`. `Q/compiler-source-comparison.json` binds their
exact before/after hashes and checks executable content against the campaign's
original source archive. This comparison does **not** normalize the compiler
manifest or exempt the real source drift.

The exact qualification update then changes the full manifest identity to
`faf00d4aa3cbcfbfe6f5c42cdcac1cfed1176dbc4616038bbfbd1ab5d8b5e2e1`.
All three replays freeze precisely that same full source, compiler source,
qualification, expected oracle and test/runtime artifact set. Documentation and
this evidence are updated only after those runs; final source deltas are
separately recorded, not attributed to the earlier build freeze.

Qualified runtime: `Q/runtime`. Qualified copied test manifest:
`Q/test-executables.json`; independent executable hashes are in
`Q/test-executable-bindings.json`. Both binaries live under
`Q/harness/target/release/deps`, away from historical build/handoff directories.
The entry harness hash remains
`5f2b542edf124cb508166b7e8ed475b4dc82e4b47fa5f82eccfcbb7d6bfbaf7b`;
the newly built loop harness hash is
`ecfa31d4c3f08a7a79c1a3cb065721922c45c4de38e40770aa9c49065463a523`.
`Q/runtime-manifest.json` inventories all nine exact runtime artifacts.

## One-command replay

Run from the repository root; choose a nonexistent output directory. This command
is supplied for the parent/reviewer and is **not an extra fourth replay** here:

```sh
python3 -B source/tools/stateful_bitmap_workflow90/run.py --output /private/tmp/campaign91-qualified-replay --native /private/tmp/verus-transfer-redb-miuzm2w3/redb --cargo-cache /private/tmp/verus-operational91-runtime.q91/build/cargo-home --runtime /private/tmp/verus-operational91-runtime.q91/runtime --test-manifest /private/tmp/verus-operational91-runtime.q91/test-executables.json
```

The three executed outputs are `Q/replay-1`, `Q/replay-2`, `Q/replay-3`. Measured
invocations use `Q/measured_replay.py`, a scratch-only wrapper around the unchanged
production runner that adds monotonic phase timers without changing arguments,
results, exceptions or gates. `Q/measurement-instrumentation.json` hashes that
wrapper, driver and production runner. Each outer command binds the wrapper,
test manifest, qualification and expected file before/after execution. Inner
commands retain canonical argv/cwd/environment and stdout/stderr/input bindings.
Fresh output-specific paths are the canonical differences between repetitions;
no earlier successful report or proof cache substitutes for execution.

## Explicit logical-payload and model qualification

`Q/original-qualification.json` and `Q/original-expected.json` preserve the exact
pre-task files. `Q/qualification.diff` records the entire qualification change.
The expected file is untouched, SHA256
`02e13ab9a35ac26758311d44a4c885cd7085de5c1b816a5237ab47d5248c0118`.

The original reviewed vstd whole hash is
`5338ee8b4aa0cfb8912c0a53d585e2ccd6e0ed89b2cd5e698d28af4708798ff2`;
the previous cycle2 whole hash is
`c38b7f8563e3d6952668b62c124b8e01d952eed9b1b4fa8f83f14a4a324e5b87`;
the fresh whole hash is
`8017cdac8cbc55979129802085c6b47f7d966d3e63017bed6b480a498699ec13`.
All three files are 33,364,097 bytes. Copied existing artifact-layout/dump support
decodes the original, previous and fresh files in new scratch; tool/source hashes,
commands and streams are retained in `Q/comparison-support.json` and `Q/runs`.

`Q/logical-payload-comparison.json` proves the exact serialized logical Krate
prefix is **33,347,277 byte-identical bytes**, hash
`18c39d82d6bc823552fa397bda0958edcffe34e884c23aea4f796f6688855c3e`.
The 152-entry source-file mapping, offsets/filenames, crate ID, inference-aware
flag, metadata length and trailer agree exactly; only map serialization order
differs. Original/fresh decoded Krates also compare equal. No logical difference
was observed or waived.

All **382 approved model functions** have identical identities, complete
serialized bytes and decoded structure. `Q/model-functions.json` inventories
every function's serialized length/hash. The independently recomputed original
fingerprints bijectively recover the original expected identities/order, then
fresh fingerprints are calculated using the exact artifact hash and existing
`trusted_model_fingerprint` serialization recipe. The candidate positive report
is **not** the source of its own oracle. `Q/model-derivation.json` and
`Q/derived-{original,previous,fresh}-models.json` retain the results:

- Original native oracle, unchanged in expected/qualification:
  `62713b5aaac6741035e5b8097d136853c9f44c759921a9e22d3fa58d89efa326`.
- Previous native oracle:
  `5c26fa1069d7d486e9754730c2c1898abff89e5b77b7ed456daf483443bd672f`.
- Fresh exact native oracle:
  `f2e52fa93f23698d9f937d42b89a4fcd53b3e16be06f217527200a7b6207499b`.
- Tree and assistance-OFF model inventories remain exactly empty.

Only scope, compiler identity, changed runtime/test hashes, exact new vstd/model
identity and existing-style comparison-artifact bindings change in qualification.
Original logical/oracle hashes, pinned rustc/rustup/loader, test-only exclusion
and historical independent-review artifact references remain unchanged. Those
historical references are not represented as a new independent campaign review.
No general hash normalization, stale-model exception or weakened theorem oracle
is introduced.

## Fresh build, vstd and harness costs

The canonical compiler recipe is
`/private/tmp/verus-workflow90-cycle2.DJ31Jt/runs/compiler-build-3/invocation.json`.
It builds the same six packages offline/locked/release with the same pinned
toolchain, RUSTFLAGS and two-job limit; private directories, solver and source
producer identity change as recorded in the new invocation. Private registry/git
caches are copied from `/private/tmp/verus-workflow90-repair2/build/cargo-home`;
its private development target is APFS-cloned into `Q/build/target`. Private
HOME/RUSTUP_HOME/TMPDIR/cache destinations prevent shared-cache writes. The
historical target, runtime, executables and sealed handoff are never staged into
or modified. No network or shared Cargo/toolchain write occurs.

| Fresh operation | Wall seconds | Outcome |
| --- | ---: | --- |
| Initial freeze/private setup, including clone | 6.200 | target clone subprocess 0.626 |
| One verifier compiler build | 266.440 | exit 0 |
| One matched-vstd run | 19.087 | 2,221 verified, 0 errors |
| One focused harness build (`--no-run`) | 111.383 | both executables built |

Compiler/vstd/harness subprocess sum: **396.910 s**. This excludes setup,
qualification, source checks and an orchestration repair interval; it is not an
uninterrupted end-to-end stopwatch. These are **fresh qualification executions
using cloned warm development artifacts, NOT cold-build measurements**. Copying
the target did not prevent substantial recompilation. Cold-cache cost was not
measured and no second build is spent to estimate it.

One scratch setup failure occurred after the successful compiler build, before
any vstd launch: existing reference88 support expected `source-manifest.json`,
while the task had only written `compiler-source-manifest.json`. The identical
frozen manifest was copied to that canonical alias and existing `matched_vstd`
was resumed directly. No compiler rebuild, changed input, weakened check or
extra application/vstd launch was used. `Q/setup-failure.json` records it.
Separate registry/git-copy streams/timings are not exposed by existing
`seed_cache`; exact successful commands are reconstructed from that executed
support in `Q/cache-setup-inventory.json`, with this limitation explicit.

## Three exact comparable stateful repetitions

Each repetition passes all eight cases: native/tree manual ON and OFF, reduced
ON, and the original reduced OFF backend rejection. There are six positive and
two intended negative cases per replay. Native positives retain 22 verified
queries, 13 executable and 3 proof bodies; reduced native OFF retains exactly
21 verified/1 error. Tree positives retain 5 verified/3 executable bodies;
reduced tree OFF retains 4 verified/1 error. Scope/identity/flags/diagnostics,
zero inferred functions, model inventories, and publication checks remain exact.

Exclusive measured phases below partition each runner total; rounded cells may
differ slightly from the sum. Hashing covers compiler/full-source manifests.
Validation/preflight includes runtime/loader/test artifact hashing and exact
outcome checks. Native includes eight subprocesses and their orchestration.
Setup/other includes initialization, preservation snapshot/copy and untimed glue.

| Phase, seconds | Replay 1 (first) | Replay 2 | Replay 3 | Range |
| --- | ---: | ---: | ---: | ---: |
| Setup/other | 5.016 | 4.366 | 5.304 | 4.366–5.304 |
| Runtime/harness staging | 1.544 | 1.528 | 2.040 | 1.528–2.040 |
| Compiler/full-source hashing | 5.987 | 6.295 | 7.094 | 5.987–7.094 |
| Validation/preflight | 14.518 | 14.835 | 16.693 | 14.518–16.693 |
| Native cases/orchestration | 7.945 | 7.933 | 10.002 | 7.933–10.002 |
| Focused regression processes | 5.397 | 5.014 | 4.758 | 4.758–5.397 |
| **Runner total** | **40.407** | **39.970** | **45.892** | **39.970–45.892** |
| Outer process total (not additive) | 40.630 | 40.148 | 46.081 | 40.148–46.081 |

Exact `Q/replay-N/phase-timings.json` retains both inclusive and exclusive timers;
inclusive nested values must not be added twice. `Q/measurement-summary.json`
contains the partition, ranges, invariant identities and raw subprocess subtotals.
Native subprocess-only sums are **7.481 / 7.449 / 9.466 s** (not additional work).
The four regression subprocesses, each with exit 0, are:

| Regression process, seconds | Replay 1 | Replay 2 | Replay 3 |
| --- | ---: | ---: | ---: |
| Exact common-runtime smoke | 0.744 | 0.424 | 0.360 |
| Exact InferCrates smoke | 1.263 | 0.885 | 0.858 |
| Full 11-method loop suite | 2.180 | 2.269 | 2.327 |
| Full 5-method entry suite | 1.198 | 1.431 | 1.208 |

`Q/focused-test-inventory.json` lists all exact method names. Repeated ordinary
providers, a real imported consumer and certificate/provider dependency bindings
run inside the loop suite. The failure/publication control still distinguishes
fresh failure (no library) from seeded failure (known `.vir` withdrawn, prior
`.rlib` bytes retained unchanged and not refreshed); attempted imports fail and
publish no consumer interface. Wrong writes/frame, missing lower/upper/domain
guards, stale snapshot, condition effects, original executable safety, entry
OFF/renaming/replacement/guard controls all remain checked.

These three repetitions share source/runtime/oracle/test bytes, not machine-wide
cache or load guarantees. Run 1 is reported separately. Run 3 is slower, not a
claimed warm-cache speedup; three local observations do not characterize all
hardware or Infer uses.

## Material cost and recommendation, not an optimization

Preflight alone is **11.164 / 11.353 / 12.476 s** across 12 boundaries per replay,
roughly 27–29% of total. Validation/preflight is about 36–37%; source hashing is
15–16%. Staging is only about 4%, so no extra staging microbenchmark or staging
optimization is justified.

Because repeated artifact validation is material, `Q/component-benchmarks.json`
records three isolated samples with unchanged source and the same nine runtime
files, two executables and two staged destinations. All existing preflight checks
still execute; these samples launch no application/verifier:

| Isolated operation, seconds | Sample 1 | Sample 2 | Sample 3 |
| --- | ---: | ---: | ---: |
| One compiler + full-source manifest hash | 0.905 | 0.522 | 0.334 |
| Unique runtime + test artifact hashing | 0.079 | 0.104 | 0.065 |
| One complete two-harness preflight | 1.752 | 1.430 | 1.158 |

The three micro-samples have uncontrolled cache/load variation and are not added
to replay totals. Recommendation **one only**: investigate deduplicating reads
of identical artifact paths **inside a single preflight boundary**, while keeping
both executable oracles, both destinations, exact path/hash/loader checks and
fresh validation before every launch. No cross-launch cached acceptance, skipped
negative, weaker source/model guard or hash normalization is proposed. Measure
three exact before/after replays if implemented later. **No optimization is
implemented now and no speedup credit is claimed.**

## Launch inventory, regressions and historical labels

`Q/invocations-results.json` inventories **53 logged command receipts** with
argv, invocation/result hashes, categories, status and duration: 1 compiler,
1 vstd, 24 direct native applications, 15 test processes and 12 setup/helper/
outer-replay processes. The 15 tests are one harness build, twelve focused
harness processes and two Python modes. These are not an inventory of every
descendant OS process. Eight successful cache-copy commands are separately
disclosed as helper-level setup: two initial copies in
`Q/cache-setup-inventory.json`, and two per replay in its `setup.json`. Existing
support records command/status, not separate streams/timings for these copies;
they are not fabricated full process receipts. The focused loop method also
runs its existing Python envelope reader once per suite (three total), and
verifier/solver and read-only Git support processes remain nested in their
parent measurements. Source/projection/hash support calls and exploratory file
reads are not counted as application launches.

Application budget: **24/32 direct applications; zero diagnostic applications**.
Focused test verifier calls are separately source-enumerated: 19 in the full
loop suite (including repeat/import/fresh/seeded failure paths), 6 in the entry
suite and 2 exact smokes, **27 per replay / 81 total**; these are not concealed
inside the 24 direct-case count. Method executions are 16 + 2 smokes per replay,
54 total. Matched-vstd is separately counted once. No fourth replay runs here.

Related Python commands use `-B -m unittest -v` and `-B -O -m unittest -v`, with
`PYTHONPATH=source/tools` and the private environment, for
`test_workflow_runtime`, `test_stateful_workflow90`,
`test_fedimint_prefix_infer79`, `test_redb_stateful`,
`test_bitmap_stateful_acceptance`: **110 tests pass in each mode**. Process walls
are **11.941 / 12.670 s**; unittest reports **11.485 / 12.267 s**. Malformed input,
command/environment/path splits, negative outcome/scope/model/publication and
isolated guard-removal controls are retained. Exact methods/commands/streams are
in `Q/runs/python-{normal,optimized}`.

Consolidation's preceding two Cargo VIR test-build commands remain separately
itemized in `CONSOLIDATION-RESULTS.md`: final 178 units + 13 docs pass, initial
invalid doctest repaired. They are not relabelled as verifier builds. This task
uses **one of the two allowed verifier builds, one matched-vstd run and one
harness build**; no spare build or optimization cycle is spent.

`Q/historical-costs.json` reads and hashes, without modifying,
`/private/tmp/verus-workflow90-cycle2.DJ31Jt/invocations-results.json`. Historical
observations include compiler 229.757 s, cached rebuilds 0.661/0.624 s, vstd
19.015 s, first harness build 86.357 s, old Infer suite 184.256 s and pinned
transfers 151.801 s. The 123.653 s final-VIR command failed; its status remains
failed. These are **historical observations, not fresh measurements, matched
repetitions or evidence of a speedup**.

## Preservation and remaining gates

`Q/preservation-before-docs.json` and final `Q/preservation-final.json` check all
**1,401 immutable artifacts and 1,331 native files**, pinned clean redb/Fedimint
checkouts, the old nine-artifact runtime, both historical harnesses and all
**2,437 sealed repair2 handoff payloads**: no mismatch. All original 2,266 source
paths remain present; prior dirty edits are preserved, with authorized campaign
changes explicitly inventoried. Own HEAD
`3b05c1912b9d7314ad00c832c4065f4362562e02`, branch
`refs/heads/t3/infer-reviewed-transfer-followups` and index hash
`0cbac76f360da036c17112dde3cf940baf5e069bb82de7f5996da12c9f0cabfa` remain
unchanged. Shared-ref drift is recorded without attribution; no commit, branch,
push, history/ref operation, native edit, network or shared cache write is made
by this task.

There is no unresolved local runtime/matrix/Python blocker. The scratch alias
setup failure is repaired and disclosed. **The actual operational native
allocator theorem, body-correspondence/semantic controls and new independent
reference/final workflow reviews remain parent-owned and pending.** This task
does not accept its own evidence, close the old goal or claim campaign success.
The parent should use the concrete qualified bundle for its final native
acceptance only when those separate gates are met. Work stops at this handoff.
