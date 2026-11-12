# Root coverage and metadata fixes: parent validation

Parent corrections ef662ee3 and c4df45e8 pass fresh validation. Source/runner
freeze is d18c0f62. NEW independent recheck remains pending; this is not full
F1, native authority or owned-cell stage acceptance.

- Full VIR:172/172, including nine metadata placements with both rejected
  C<u32> and accepted C<u64> at the guard and actual Ctx entry. These fixtures
  are synthetic internal boundary checks, not full source/WF or native proofs.
- Loaded-native control unit:1/1; existing58 native and45 emitted-body controls
  remain. All three full native Functions still fail normal opaque-field WF.
  No authority, native body query, state proof or helper fact is enabled.
- Focused suites:12/12 tests,226 invocations. All198 prior cases remain plus28
  root/item/module/default-option and retained-root-exclusion controls.
- Direct ordinary/Infer and feature-on/off root coverage:20 expected outcomes.
  Unmarked root opt-ins no longer count as body checking; actual item/module
  contexts and the real default option retain ordinary coverage. Experimental
  requests still refuse with zero helper/body/export credit.

All source/tool guards passed. Exact hashes/argv/receipts/counts/costs are in
ROOT_GATE_FIXES_RESULTS.json. The fresh no-debug/non-incremental compiler build
took77.230s; VIR47.089s, loaded-native6.035s and focused61.056s (Cargo wall times
include compilation). The build directory is approximately1.6GiB. No build-cache
copy or new vstd verification; the seven supporting artifacts retain the
previous NORMAL2257/0 identity. No verifier check or runtime source was disabled.

Immutable runtime and four ORIGINAL test executables:
/tmp/owned-cell-root-gate-fixes-d18c0f62-parent-snapshot.
Receipt SHA256:
fb56f75c515ab64a6c87b8ac465a1bb282b819f0d5aa10a9069eb102cf8c8f81.
The snapshot contains the exact Cargo-executed VIR/native/refusal/admission test
binaries under checks/. Each binary's source build, original path and hash is
bound by the receipt. Relocation of the original executables was tested on the
root coverage group, metadata group and native controls, with zero rebuilds or
source/harness transformations. Reviewer replay can avoid another large build.
The admission/refusal executable's parent-parent directory is the snapshot,
so its existing InferCrates profile lookup resolves the frozen eight tools.
Set RUSTUP_HOME=/home/dev/.rustup, RUSTUP_TOOLCHAIN=1.99.1, VERUS_IN_VARGO=1,
VERUS_ROOT to the snapshot, VERUS_Z3_PATH to its z3, and LD_LIBRARY_PATH to the
toolchain lib plus lib/rustlib/x86_64-unknown-linux-gnu/lib. Use a fresh private
TMPDIR/cache. Filter VIR for internal_representation when only these gates are
needed; the native test filter is recorded in the command receipts.

The old parent build failed ENOSPC (exit101) before focused testing. Its source
guard and command logs are retained; the wrapper receipt's success does not
mean compilation passed. Only its disposable caches were removed, alongside
stale completed-experiment incremental caches during the requested disk audit.
The initial snapshot preparation refused an ambiguous main/test executable
choice; selection was then bound to Cargo's actual execution paths. Neither
event is semantic negative evidence or a suppressed verifier failure.

Actual admitted-owner versus translated/checked-Function reconciliation remains
a distinct activation gate. Full native contract/current/future correspondence,
compiler-private field authority, F2 same-client final-input state and F3 original
native erasure remain open. D2's separate branch is not accepted here. Complete
REDB Mutex/cache/publication/recovery coupling is unproved. CP1/CP2=false;
HMAC/native remains DEFERRED AND UNACHIEVED.
