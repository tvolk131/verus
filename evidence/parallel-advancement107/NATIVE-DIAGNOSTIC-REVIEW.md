# native106 independent review107: authorization worksheet

**Decision now: NO-GO for execution.** This source-only review recommends exactly
one conditional diagnostic scope below for a separate explicit authorization.
Parallelism107 does not authorize this build, new trust, or native adoption.
All 18 redb cells remain OPEN; proof credit, new trust, adoption and acceptance: 0.

## Findings and objections

1. **Provenance objection sustained, narrowly.** Independently read
   `build-001/invocation.json` has `compiler_builds=0` and invokes existing
   `runtime/rust_verify` on `vstd/vstd.rs` with `--is-vstd --compile`: a vstd
   receipt, not an engine build. Correspondence remains unestablished, not
   impossible; no executed Arc rejection occurred. See SOURCE-REPORT.md:15,17,27
   and original proposal REPORT.md:31-39.
2. **Three distinct results:** new build custody; exact call binding/rejection;
   checked semantic producer. Neither of the first two supplies native buffer
   identity, ownership/aliveness, fetch correspondence or stable metadata. Seq
   equality is not authority. Native PageNumber/projected-codec nominal identity
   and a checked conversion remain separately missing.
3. **Controls are not substitutes.** Native base.rs:180,212 has `mem: Arc<[u8]>`,
   debug-only `open_pages`, and `PageImpl::memory` returning `self.mem.as_ref()`.
   Pinned alloc/sync.rs:4254 has `T:?Sized` AsRef returning `&**self`. Sized Arc
   and slice successes do not establish unsized Arc support. Isolated fixture
   evidence is not original crate DefId/cfg or frozen-engine behavior.
4. **Feasible in principle, offline buildability untested.** Live Rust 1.99.1
   matches cited std's version, not proven sysroot/source correspondence. Locked
   Git/registry dependencies, rustc-dev, linker/SDK, macros and compatible vstd
   must be available. No availability/build test occurred; no old target reuse.
5. **Concrete build trap:** rust_verify/build.rs:6 requests Git version data
   before VARGO overrides; versions.rs:7,76 reads Git metadata. Bare source copy
   plus VARGO_BUILD_SHA is insufficient. Vargo sets additional cfg/environment
   (tools/vargo/src/commands/mod.rs:42,105); direct Cargo must reproduce that
   recorded wiring. Endpoint hashes alone are not immutable custody.
6. **Write scope must narrow.** resolve_traits.rs:27 and fn_call_to_vir.rs:211
   expose resolution; config.rs:482,484,339 exposes nonverifying SST/log capture.
   Do not edit AST/schema or admission. Preserve native_resource.rs:10,22 and
   import_export.rs:116,277, their submodules and all admission paths, not just
   hashes of two top-level files.

## ONE proposed authorization: diagnostic107-D1

**Purpose:** one new offline diagnostic compiler and original-call matrix, never
the old frozen engine. Matching filenames/version/output hashes do not reconstruct
historical custody. Frozen-provenance recovery, producer/codec implementation,
proof, admission and adoption are out of scope.

### A. Fixed envelope, if explicitly authorized

- Fresh `/private/tmp` only; parent/evidence/runtime/toolchain/dependency/Git
  inputs read-only. No network/downloads, Git writes, historical deletion or
  toolchain/runtime/cache copies. Source-only snapshot permitted, no artifacts.
- 45 minutes total including capture/finalization; no new substantive launches
  after minute 35; retain the final 10 minutes for results. One builder, jobs<=2,
  no concurrent native diagnostic builder. One engine build attempt/profile,
  including only needed builtin/proc-macro support targets; no incremental or
  old target reuse. At most one companion vstd frontend build, with verification
  disabled. At most eight probe frontend invocations, detailed below.
- Scratch <=8 GiB inclusive; source <=128 MiB, logs <=64 MiB. No solver, proof/
  test suite, benchmark or compiled target run. Build-script/compiler processes
  belong to the newly requested build permission, not native application runs.
- Disk check each phase: free >16 GiB plus remaining scratch allowance. Monitor
  allocation; terminate process group before a limit/floor, retain partial bytes
  as incomplete. Never silently truncate a successful receipt.
- No retries/rebuilds, feature experiments, dependency repair or automatic next
  research lane. Controls and the two reserved reproducibility probes are not
  permission to repair a rejected Arc call. Budget failure ends this candidate.

### B. Mandatory preflight gates (otherwise stop before compiler launch)

[ ] Seal source/baseline identity, native extraction, manifests/locks, complete
    path/Git/registry dependency bytes, build scripts, cfg/features/target, macros,
    fixtures and observer diff. Record symlinks/targets and generated inputs.
    Freeze after observation edits; rehash at exit. No unsealed live-parent basis.
[ ] Pin executable, rustc-driver/sysroot/support closure, installed metadata,
    relevant std source, linker/SDK/build tools. Hash the complete declared read
    closure and reconcile consumed dependencies. Unaccounted input/drift or
    inability to capture within the envelope => provenance INCOMPLETE, stop.
[ ] Offline dependencies must work without retained Cargo/Git/cache writes;
    HOME/temp/target/generated output scratch-local. Explicitly bind necessary
    read-only Git metadata; revision is a stamp, not source identity. No fake Git,
    version-helper patch, cache copies or replacement Git repository.
[ ] Before launch seal executable paths, cwd, argv, environment allowlist, locked/
    offline resolution, jobs<=2, cfg/profile/target and clean target. Pin/disable
    Cargo configs, wrappers and inherited flags; exclude secrets. Capture all
    transitive rustc/link/build-script commands, outputs, status and timings.
    No post-hoc command reconstruction or general vargo invocation adding work.
[ ] Pin compatible no-new-law companion vstd/support baseline. Frozen VIR/macros
    are not presumed compatible; historical midpoint proposal is not adopted.
    One nonverifying companion build only; failure => stop, no ABI/trust patch.

Preflight is finite setup, not a history search. Availability is unestablished.
Exact build argv/manifests are required before launch, not post-hoc placeholders.

### C. Minimal instrumentation and exact matrix

Prefer existing logs. Permit only observer logging <=150 added lines across
resolve_traits.rs, fn_call_to_vir.rs, rust_to_vir_func.rs. No change to resolution,
lowering, acceptance, inference, trust or AST/schema/SST semantics. Outside-list
need => STOP. Retain diff for independent noninterference review, not a claimed
theorem. Capture instance/generics/adjustments/spec/body disposition.

Create three separate fixture cases so unsized failure cannot hide controls:

| Case | Required source/instance | Debug | Release |
|---|---|---|---|
| U | Original PageImpl / Page::memory, Arc<[u8]>, `self.mem.as_ref()` | 1 | 1 |
| S | Separate explicitly labeled Sized Arc<[u8; N]> AsRef control | 1 | 1 |
| B | Separate explicitly labeled borrowed &[u8] access control | 1 | 1 |

U preserves native trait, receiver fields/generics/debug declaration and memory
body, with byte/span mapping to base.rs. Referenced definitions must be pinned
native source, not lookalike PageNumber/map or trusted stubs. List placement/
verification wrappers separately; target body cannot be skipped/externalized.
Semantic scaffolding needed => extraction BLOCKED. No constructor/fetch/Drop
changes or native runs. Report new fixture nominal identities/extraction limits.

One compiler profile suffices: these are two **probe configurations**, not two
engine builds. Debug uses `-C opt-level=0 -C debug-assertions=yes`; release uses
`-C opt-level=3 -C debug-assertions=no`; record effective cfg and native features.
Each frontend invocation uses `--no-verify --build-sst`, `--log vir`,
`--log vir-sst` and a separate scratch log directory, with explicit pinned sysroot
and support args. Keep lifetime/erasure/admission checks enabled and
experimental-native-resource disabled. No probe export/import beyond declared
diagnostic support libraries. The seventh/eighth invocations repeat U once per
configuration with identical inputs, comparing normalized binding/body records.
Never interpret these nonverifying invocations as checked proofs.

### D. Minimum validation and stop classifications

Per cell retain source/span, cfg, compiler/support hashes, argv, status, stdout/
stderr and raw logs. Trace `T=[u8]`, default allocator, selected trait/impl/item,
return type, autoderef/borrow/unsizing, external spec/autospec/model route (or
NONE), and actual method-body disposition. Correlate source, VIR target and SST.
Referenced std implementation is not evidence its `&**self` body was lowered.

- **OBSERVED_SUPPORTED:** U reaches the relevant binding/body/lowering stages in
  both cfgs with complete records, controls independently characterized, repeated
  U records agree after documented nonsemantic normalization, and all provenance
  and unchanged-boundary checks pass. This is isolated capability evidence only.
- **OBSERVED_UNSUPPORTED:** a reproducible call-specific diagnostic reaches U's
  actual Arc call, attributed to this new compiler. Preserve stage/span and still
  finish budgeted controls. No inference about the old frozen frontend.
- **INCOMPLETE/BLOCKED:** build/support failure, skipped body, earlier unrelated
  rejection, unresolved trace/spec target, input drift, missing receipt, resource
  cap or timeout. Do not call this an Arc rejection or successful lowering.

Whole-tree baseline diff must contain only allowed observer changes. Hash
native_resource.rs/import_export.rs, submodules and all admission/certification
paths unchanged; do not disable guards by configuration. This validates unchanged
source, not executed import/export regression behavior (separate permission).

No new Arc law, assumption, admit, axiom, external_body, assume_specification,
trusted adapter, producer, codec conversion, admission relaxation, adoption or
resource authority. No growth049/zero-growth050/mutation030/refused-padding95/
drop-pipeline resumption or rerouting. S0 stays fail-closed.

## Decision and handoff

**Authorize diagnostic107-D1 exactly as bounded, or decline.** Not authorized now.
Producer/codec gaps remain OPEN; no automatic follow-on or integration.

Return sealed inputs/diff, command/environment and process ledger, six cells/two
repeats with raw diagnostics, actual loaded output/support hashes, unchanged-guard
diff, resource accounting, separate provenance/binding/semantic results and final
manifest. Retain partial/failure artifacts with honest classifications.

## This review's identities and limits

Required input SHA256 values:

- parallel-advancement107/PLAN.md:
  `16267509886f01118492007eeadb5487294fae0313a8e357378cb78359e2c42f`
- native106 SOURCE-REPORT.md and original scratch REPORT.md (byte-identical):
  `35de42ef4523a80a35b92b6844afbd5792157c62be75bd9fb88e688fc795be99`
- native106 RESULTS.json:
  `ceaef8574583e83c9b36421b136ecd79ca749f8255d7911bd4616bf042bc6130`
- native106 SOURCE-ACCOUNTING.json:
  `07782bca557d18572e8a1d24bc65baf9a9ea89a01f498e8154c02c48d38aafa1`
- native106 INPUT-BINDINGS.json:
  `9427a5fd8e4e88d4c59b648b403a860f2924bc05f1fd969ab72486fd18ed1349`

INPUTS.review.sha256 records inspected-file identities, including native/std
source and invocation/receipt matching supplied native106 descriptors. This is
not reauthentication of all 235 historical inputs. Hashes were consolidated
during/finalizing review, not all before first read; no atomic snapshot,
historical custody or authenticated utility environment is claimed.
ACCOUNTING.json records endpoint recheck, elapsed time, disk and scratch size.
Start: November 7, 2026, 20:42:02 America/Chicago (November 8, 02:42:02 UTC).
Optional work cutoff 20:48:02; absolute deadline 20:50:02. Parent and all retained
inputs were read-only; fresh scratch only. A read-only git status was used for
context; no Git writes, builds, tests, compiler/verifier/solver/native executions,
network, subdelegation or parent edits occurred. Report <=12 KiB, scratch <=8 MiB.
