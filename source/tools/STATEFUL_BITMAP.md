# Source-built stateful bitmap replay

The default track consolidates the **accepted transition85** reference: one
native growing-range word transition, including the inferred `bits_in_range`
helper. It does **not** establish complete `U64GroupedBitmap::resize`
correctness. The permanent transition reference is
`stateful_bitmap/transition.rs`; its checked bit-vector
mask lemma, ghost snapshot, and quantified proof skeleton are authored, not
automatically invented or inferred. A separate explicit-contract fixture proves
seven unchanged native access/helper bodies and actual get/set/clear composition,
with a handwritten checked BV lemma; this is not Infer automation credit.
Generic no-instantiation clients are also
checked with goal assistance enabled and rejected with it disabled.

The explicit `--resize-provider reference88` track additionally checks the
complete authored reference in `stateful_bitmap_reference88/reference.rs`:
the pinned native struct and all nine executable bodies, four actual
composition clients, and an ordinary imported-contract consumer. Contracts,
loop invariants, snapshots, and checked semantic lemmas remain authored. This
track adds no Infer feature or inferred-interface certification, and requires
a new independent review before acceptance of its complete semantic scope.

## Offline command

Run from the repository root, with Python 3.9 or newer, on the canonical
`1.99.1-aarch64-apple-darwin` Rust toolchain:

```sh
python3 -B source/tools/check_redb_stateful.py \
  --native-checkout /path/to/clean/redb \
  --output /path/outside/the/repository/new-replay
```

To check the complete authored reference instead of the limited default track,
add `--resize-provider reference88`. Use a different new output directory for
each replay.

The native checkout must be clean at
`c82ab865bb362bc4af067d332adc44978ff64813`. Its
`src/tree_store/page_store/bitmap.rs` must have SHA-256
`d57724dcbc9701e8fb7a2e268febba1fbe2ff2a00e14e000c27a703afe3af9ff`.
The checker compares executable tokens against the actual helper and native
loop iteration, allowing only the disclosed `data`/`self.data` receiver mapping
and removal of authored proof scaffolding. It does not substitute a handpicked
summary of native behavior.

The output directory must not exist. The default Cargo cache seed is the actual
`~/.cargo`; `--cargo-cache /path/to/existing/cargo-home` selects another existing
seed. Only its `registry` and `git` trees are **copied**, never linked, into a
private Cargo home (macOS copy-on-write cloning). Missing offline dependencies
fail the build; there is no network fallback, dependency installation, lockfile
repair, historical runtime lookup, target-cache reuse, or package archaeology.
`--rustup`, `--sysroot`, and `--solver` allow explicit existing input locations;
the default solver is the repository's `source/z3`.

The runner creates private `HOME`, `CARGO_HOME`, target, `RUSTUP_HOME`, cache, and
`TMPDIR`. Its only toolchain link points from private `RUSTUP_HOME/toolchains`
to the existing canonical sysroot, which is used read-only. It invokes an
absolute rustup executable, supplies the absolute sysroot explicitly, and uses
the canonical **name**, not an absolute path, for `VARGO_TOOLCHAIN`. No inherited
Cargo, Rust, solver, or bootstrap environment is used. Cache sources, native
checkout, compiler source, toolchain, and existing runtimes are not modified.

## What runs and what is bound

One offline, locked, release Cargo invocation builds the current local compiler,
launcher, vstd builder, builtin library, and macro libraries into a new target.
The producer is derived from a sorted SHA-256 manifest of current workspace
member/local-path source trees, vstd, workspace manifests/lockfile, Cargo config,
and root toolchain specification. It is not a manually selected producer pin.
The manifest includes Rust compiler fork and local dependency trees. Current
literal Cargo member/path declarations are scanned; compiler inputs must remain
local regular files. New files inside these roots change the manifest.
In addition, a full nonignored repository manifest (tracked and untracked files,
including the frozen campaign SCOPE and evidence) is captured before the build
and checked for drift around execution. This broader freeze is independent of
the compiler-relevant producer hash. Coordinate changes to **any** nonignored
repository file before starting the command, not while it is running.

The newly built `rust_verify` verifies and compiles matched local vstd once, with
`VSTD_KIND=IsVstd` and the explicit std/alloc/nonzero bootstrap arguments. This
bootstrap intentionally does not use application `--no-cheating`: existing vstd
axioms/external bodies remain audited trust, **not newly proved application
facts**. Its exact output, argv, and return status are retained; its verified
count is recorded, not a manufactured fixed historical total.

Every application uses `--no-cheating --compile --output-json --num-threads 1`,
fresh inference cache/output paths, and no inherited `VSTD_KIND`. Checks include:

- Exact accepted positive transition, with both executable bodies, authored
  proof body, specification definition, inferred contract/proofs, and query total.
- Generic no-explicit-instantiation positive and assistance-disabled rejection.
- Wrong branch semantics, false frame, out-of-bounds indexing, and missing
  quantified guard negatives. These must be real proof failures of the expected
  diagnostic class, not panics, frontend errors, timeouts, or successful runs.
- Real compiled/exported provider and imported consumer composition: the
  consumer calls the transition contract and the inferred mask contract. Both
  `--extern` library and `--import` interface kinds and paths are checked, with
  the actual provider certificate fingerprint/crate identity bound to the report.
  The trusted-model function inventory is fixed independently. Model fingerprints
  bind the freshly built vstd artifact and therefore vary across rebuilds; they
  are read from the source-producer-bound exported certificate, compared to the
  report, and authenticated by the real consumer import, not pinned to an old
  runtime or learned from the report being checked.
- Failure publication: no negative may leave a library or exported interface.
- `access_composition.rs` checks seven native bodies byte-for-byte against pinned
  source, the native struct, eight total executable bodies and one authored proof
  body. Two semantic controls reject a false post-clear read and a zero-mask
  contract. Tuple indexing needs no Infer or trust-boundary change.
  Its compiled `.vir` is an ordinary export: `interface_written=false` in the
  inference report is expected, not silently upgraded to certified inference.
- Exact native `required_words_native.rs` now verifies under the explicitly
  authorized `u32::div_ceil` correspondence. A separate bridge checks the full
  nonzero-divisor domain and extreme numerators. No other integer width or
  method acquires this specification.
- The explicit complete track checks the native struct and all nine bodies,
  the full authored stored-bit/length/frame property, four actual composition
  clients, and a fresh ordinary imported-contract consumer. It reuses the
  strong artifact, command, source, and full-body-scope gates above without
  claiming certified inference for these ordinary contracts.

`source-manifest.json`, `native.json`, `native-projection.json`,
`build-receipt.json`, per-command invocation/result/expected/accepted records,
and `receipt.json` retain exact commands and hashes of source, runtime,
toolchain binaries/libraries, native bytes, application inputs, output artifacts,
and full body-scope expectations/reports. Inputs are checked for drift before
and after execution. These are execution records plus consistency gates, not
cryptographic third-party attestations; a self-authored JSON receipt alone is
not proof evidence. Setup failures are retained and consume their launch slot.
The per-command limits are one compiler build, one matched-vstd run, and 24
application launches. The default component track uses thirteen applications;
the explicit complete-reference track uses fourteen, including its actual
ordinary imported-contract consumer.

## Exit status and complete reference

On successful default replay, status is
`transition_access_established_resize_incomplete`, `transition_established` and
`access_established` are true, `full_resize_complete` is false, and the process exits **2**. Use
`--transition-only` to request exit 0 specifically for that limited track.
Actual failures exit 1; an incomplete full goal is never silently called passed.

The explicit `--resize-provider reference88` track returns exit 0 only after
the whole native projection, exact complete verification scope, and fresh
ordinary imported composition pass. Its status is
`complete_resize_established` with `full_resize_complete=true`. This describes
the authored source-bound proof, not automatic proof discovery, allocator
correctness, or whole-database correctness. The complete runner does not close
the earlier automation goal by merely checking this reference.

Parent integration can register another `CompleteResizeProvider` in
`COMPLETE_RESIZE_PROVIDERS`, selected by `--resize-provider NAME`, or pass one
directly to `replay(runner, complete_provider=...)`. The registered `reference88`
provider is real checked source, not placeholder acceptance. A provider supplies:

- Its permanent source path and independent `expected_report` in the same
  schema as `stateful_bitmap/expected.json`, using crate name `stateful_resize`.
  Include **all** executable/proof/specification bodies and exact query total,
  helper contracts/proof requirements, dependencies, and trust inventory.
- Its fully qualified `resize_function`, required to be an ordinary-contract
  `checked_exec_body` in that exact complete scope.
- An audited `project_native_resize(source_text)` callable erasing only proof
  instrumentation and returning the **whole** native resize executable body.
  The runner compares that projection to the pinned native function, not just
  its inner transition. Register the callable in guarded checker source.

The registered reference also audits the exact struct and all nine native
bodies, not only `resize`. It declares ordinary contracts explicitly and does
not attach an inference certificate to ordinary VIR imports.

Register the completed source/expectations permanently in this fixture
directory. The complete provider is checked by the same current-source runtime
and acceptance gates after the transition campaign. Only a successful whole-body
projection plus successful exact-scope verification permits
`complete_resize_established` and `full_resize_complete=true`. The parent still
owns independent final review and semantic adequacy of the complete property;
this runner does not infer missing resize invariants or contracts.
The current runtime includes the explicitly authorized correspondence for only
`u32::div_ceil`, over all numerators and nonzero divisors. Its unbounded
mathematical ceiling formula and zero-divisor precondition are new library
trust, not a proof of Rust core or a new application axiom. The replay checks a
three-query bridge fixture; the default track now verifies the exact
`required_words` helper positively. The old `EXPECTED_FRONTEND_BLOCKER` reader
and frozen expectations remain solely for historical receipts and tests: that
unsupported-method expectation is stale on this runtime and is not executed.

## Gate regression tests

```sh
python3 -B -m unittest discover -s source/tools -p test_redb_stateful.py
python3 -B -O -m unittest discover -s source/tools -p test_redb_stateful.py
```

Tests use explicitly synthetic files/reports and mocked Git identity; they do
not count as solver evidence. They call the integrated acceptance entry points
to reject malformed totals, missing/substituted exec/proof scopes, helper and
failure-report corruption, missing/drifted artifacts, wrong/dirty native pins,
import-kind substitution, and compiler source drift. Production gates do not
use Python `assert`, so optimized Python preserves them.

All result classes use the same canonical application-command check. Runtime,
rustup, sysroot, input/output paths, crate name, goal-assistance mode, mandatory
flags, and imports must match the independently constructed execution context;
selected executables and libraries must belong to its guarded artifact set.
The input must match the registered permanent fixture, and stdout/stderr bytes
are consumed only through their authenticated, canonical artifact paths.
Absent failed outputs exclude dangling symlinks as well as regular files.

`stateful_bitmap/failure_expected.json` freezes the eight intended failures:
exact verification totals, complete inference/body/helper/generation schemas,
failed-function sets, dependency/model inventories, and primary diagnostics with
source locations. These expectations were authored from the independently
reproduced original controls, not learned from the report being accepted.
Only measured timings, well-formed candidate identifiers, the canonical input
path, and model fingerprints already authenticated by the successful provider
certificate are normalized. All other fields must match exactly. Negative
reports never acquire theorem or publication credit through this validation.
Mixed/unrelated errors, an inserted false assertion, wrong locations, malformed
schemas, empty commands, alternate runtimes, and evidence-path splits reject.

## Bounded stateful workflow90 candidate

### Reviewed-gate repair and exact-runtime replay

The two implementation cycles are closed. Campaign91 separately authorizes the
source-bound rebuild after its documentation-only compiler consolidation; no new
Infer design is included. The exact runtime-specific binding is
`stateful_bitmap_workflow90/qualification.json`; the original model oracle remains
unchanged in `expected.json`. Fresh artifact comparison establishes 33,347,277
byte-identical serialized vstd logical bytes and 382 identical approved model
functions; only source-file metadata ordering changes. Model fingerprints are
derived from those functions and the exact new artifact, not learned from the
candidate report. This qualification is not a new independent review. No general
normalization or stale-model/import-policy exception is allowed.

Replay the qualified bundle without another compiler/vstd build:

```sh
python3 -B source/tools/stateful_bitmap_workflow90/run.py \
  --output /private/tmp/campaign91-qualified-replay \
  --native /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --cargo-cache /private/tmp/verus-operational91-runtime.q91/build/cargo-home \
  --runtime /private/tmp/verus-operational91-runtime.q91/runtime \
  --test-manifest /private/tmp/verus-operational91-runtime.q91/test-executables.json
```

The output must be new. Test binaries are freshly built against a clone of the
private development cache, then copied into this new scratch tree before runtime
staging; no historical test/runtime directory is modified. This is fresh
qualification work with warm development artifacts, not a cold build. Their full
paths and bytes are bound and checked before execution. Both harness runtime
layouts and pinned loader libraries are checked. Every matrix case binds the exact fixture, source, argv,
cwd, environment, compiler/runtime/solver, complete selected-body inventory,
verification flags, output artifacts and intended primary failure diagnostic.
These receipts establish consistency, not cryptographic execution attestation.

The ordinary imported provider legitimately carries an empty-proof metadata
envelope. Its consumer must bind exactly that provider ID and certificate digest;
this is not an inferred-constructor or inferred-contract certification. The
earlier eighteen rustdoc compilation failures are historical, not a current green
baseline. Campaign91 repairs them with thirteen executable examples and five
explicitly labelled pseudocode blocks; its fresh full VIR unit-and-doc run passes.
Neither that tooling result nor the earlier failed command closes the pending
independent workflow/native review.

### Campaign91 shared mechanics and freshness

`workflow_runtime.py` now supplies small runtime/environment, strict command and
artifact binding, subprocess and whole-crate outcome helpers to this runner and
the ONE migrated Fedimint checker, `check_fedimint_prefix_infer79.py`. Specialized
fixture, source, native body, report, import, certificate and negative-diagnostic
oracles remain local. Duplicate JSON keys, coherently rehashed malformed reports,
scope-changing commands and same-bytes/different-path artifacts reject. The
Fedimint checker still runs every semantic, safety, frontend, import and poisoned
cache control; warm candidate preparation is not cached proof acceptance.

Run both focused Python readers, including isolated guard-removal controls:

```sh
PYTHONPATH=source/tools python3 -B -m unittest \
  test_workflow_runtime test_stateful_workflow90 test_fedimint_prefix_infer79
PYTHONPATH=source/tools python3 -O -B -m unittest \
  test_workflow_runtime test_stateful_workflow90 test_fedimint_prefix_infer79
```

Invocation `seconds`, staging `staging_seconds` and validation
`validation_seconds` use monotonic clocks. Fedimint case rows additionally record
`invocation_seconds`; validation includes input/output binding and specialized
positive/negative checks, not all setup/guard hashing. These are raw observations,
not measured speedups. Outputs must be new, while development compiler/candidate
caches may be reused only with fresh verification.

The original workflow90 qualification and expectations are preserved as
`/private/tmp/verus-operational91-runtime.q91/original-{qualification,expected}.json`.
Editing VIR doc comments correctly made the old runtime fail `compiler source
drift`. Campaign91 now rebuilds once from frozen source, runs matched vstd once
(2,221 verified), explicitly compares logical payloads/model functions, and pins
the exact fresh runtime/test/model identities. Three fresh-output replays pass
all eight native cases, eleven loop methods, five entry methods and both runtime
smokes. Related Python tests pass 110 methods in each normal/optimized mode.
No source/negative/trust oracle is weakened, and the seeded stale-rlib distinction
remains checked. Exact differences, private replay paths, costs and the one
scratch setup failure are in
`git show archive/verus-infer-pre-integration-2026-11-13:evidence/reliable-operational91/RUNTIME-AND-COSTS.md`. The migrated Fedimint
measurements remain honestly labelled as using the historical matched runtime.
The operational allocator theorem and new independent workflow/native reviews
remain pending; neither these local passes nor requalification closes the old
goal or accepts the full campaign.

### Original cycle-one handoff (historical)

This was one implementation cycle, not acceptance of the original bitmap goal.
The two hash-frozen manual/reduced fixtures are in
`stateful_bitmap_workflow90/fixtures`. A retains all nine native bitmap bodies,
resize and four clients. B retains the all-u32 constructor, required_words and
new_full without preconditions. The only reductions are the frozen three-line
and two-line point assertions. All semantic invariants, contracts, snapshots,
checked lemmas, safety conditions and executable tokens remain authored.

One offline reproduction command on the campaign's pinned macOS toolchain is:

```sh
python3 -B source/tools/stateful_bitmap_workflow90/run.py \
  --output /private/tmp/workflow90-reproduction \
  --native /private/tmp/verus-transfer-redb-miuzm2w3/redb/src/tree_store/page_store/bitmap.rs \
  --cargo-cache /private/tmp/verus-native-reference88.w3Ws2g/parent-final-replay/build/cargo-home
```

Use a fresh output directory. The command reuses reference88's offline builder,
matched-vstd verifier and invocation ledger. It tests typed helpers before the
compiler build, obtains actual test executable paths, stages the same complete
runtime into both current-executable-derived directories, and checks the solver
that InferCrates actually overrides. It does not modify the test harness, native
checkout, shared Cargo home or toolchain. It runs the exact native ON/OFF matrix
and both harness smokes; any failed native gate returns PARTIAL, not acceptance.
Individual logs and preparation declines remain in the private output directory.

Preparation observations are not theorem evidence or cache hits. A successful
ordinary constructor exports only its checked authored contract, not an inferred
constructor summary. Repeated ordinary proofs are not Infer cache reuse. Unknown
value representations, nonempty condition setup, unsupported scopes and bounded
resource exhaustion conservatively decline. Full campaign acceptance still needs
the remaining behavioral/publication/cache/regression gates and the parent's new
independent review; do not infer it from this command's compilation success.
