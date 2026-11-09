# Start here: native redb research handoff

This is the current entry point for a new AI thread or another machine. The
code/evidence checkpoint is `e64321b13a1d2258f46ebdac99fbd3cb729bd25a` on
`t3/infer-reviewed-transfer-followups`; the portability additions postdate it.
Check out that branch including these additions, not just the default branch.

## Goal and actual status

The goal is six original redb properties across one-phase, two-phase and mixed
histories: [all eighteen cells](evidence/redb-crash-consistency94/GOAL.md).
**All eighteen remain OPEN.** The first target is full original two-phase
acknowledged durability, as specified by
[FIRST-CELL139](evidence/redb-native-mir-campaign115/FIRST-CELL139.md).
Do not substitute a local allocation/store lemma, a model-only transaction,
a fixed file size or a diagnostic inventory for that cell.

- The latest kernel170 is a **partial, unreviewed, unsealed candidate**, not the
  active compiler in the checkout's root `source/`. Its final build succeeded
  and its final retained report has 45 passing structural controls. Those are
  not native proofs. Its worker did not finish final authentication/sealing;
  the later parent archive preserves endpoints, not a retroactive worker seal.
- Layout171 is **source-only, uncompiled, unverified and unexecuted**. Its exact
  staged consumer and scalar lemmas are committed under `candidates/layout171`.
- Conditional mathematical allocator schemas: one. Operative new laws,
  actual native issuance and accepted original allocations/stores/bodies: zero.
  Native export remains **REFUSED**; G1/G2 and original layout applicability
  remain OPEN. Conditional generated proofs do not establish original authority.
- Prior worker failures, command exhaustion, disk refusals and interrupted116
  remain historical facts. No worker budget, goal-service state or review is
  reset by this handoff. Old documents saying a worker is running are historical,
  not evidence of a live worker or a slot reservation.

## Trust authorization and next implementation edge

[AUTHORIZATION169](evidence/redb-native-mir-campaign115/AUTHORIZATION169.md)
approves developing a narrow native MIR/heap/full-pointer interpreter and
integrated affine-flow checker as an explicit compiler TCB component. It does
not claim machine-checked simulation of native Rust execution. A **new independent
exact-code review and pinned disposition are required before activation**.
No extra allocator/frame axiom, trusted caller summary, assumed caller validity
or relaxed admission is authorized.

Read these next, rather than relying on the old chat:

1. [Kernel170 contract](evidence/redb-native-mir-campaign115/IMPLEMENTATION-CONTRACT170.md)
   and [boundary168](evidence/redb-native-mir-campaign115/NATIVE-SEMANTICS-BOUNDARY168.md).
2. [Preserved candidate/status](evidence/redb-native-mir-campaign115/GIT-BACKUP.md),
   then the full prepared kernel source and its retained final receipt/report.
3. [Layout171 disposition](evidence/redb-native-mir-campaign115/LAYOUT171-PARENT-DISPOSITION.md).
4. [Allocation156 boundary](evidence/redb-native-mir-campaign115/ALLOCATION156-DISPOSITION157.md)
   for the remaining same-original-event/resource-issuance gap.

The next proof-directed work is to review the exact kernel (including any new
path adapter), then finish original statement/caller-CFG and activation-return
threading beyond the primitive cut. Establish actual layout, nonnull and live
resource-frontier applicability, and compose actual native store/deallocation.
The original checkout/reservation, returned-loan/copy, normal/cleanup writeback
and cache-return chain must then discharge the full durability/recovery inputs.
Do not start another inventory-only chain or count conditional kernel controls
as closing a database cell. Layout171 is a dependency, not a new allocation.

## Prepare using only the cloned repository

Python 3.9 or newer with its standard `gzip`, `tarfile` and LZMA support suffices
for preparation. No Rust toolchain, Cargo cache, Z3, network or previous `/tmp`
directory is needed for these two commands:

```sh
python3 -B source/tools/prepare_native_research_handoff.py verify
python3 -B source/tools/prepare_native_research_handoff.py prepare \
  --output "$HOME/verus-research-work"
```

The destination must not already exist. Inputs are size/hash checked; links,
traversal paths, duplicate archive paths and excessive output are refused.
Preparation never executes archived code, compilers, solvers or applications.
On failure, any partial output is not a successful handoff; no `PREPARED.json`
completion record is written. Choose a fresh destination for another attempt.

The prepared directory contains:

| Path | Contents |
| --- | --- |
| `kernel170/candidate` | Complete latest 1,122-file source snapshot, not just the eleven-file delta |
| `kernel170/receipts` and `kernel170/evidence` | Retained commands, source/tool bindings, failures and structural results |
| `kernel170/runtime/canonical-vstd170.json` | Exact preserved JSON projection of the genuine historical library AST |
| `inputs/vstd156.vir` | Exact historical serialized library artifact, not newly issued proof authority |
| `redb115/redb` | Original selected application sources; the adjacent dependency envelope has relative paths |
| `rust-source/rust-src-1.99.1/rust-src/lib/rustlib/src/rust/library` | Exact selected Rust library sources |
| `layout171` | Exact staged scalar module, consumer, proposed cases and historical authentication |
| `provenance` | Historical source binding, inventory and acquisition helper; not a current runner |
| `PREPARED.json` | Input-manifest digest, prepared paths and path-adapter before/after hashes |

The archived kernel contains one diagnostic hardcoded runtime-library path.
Preparation replaces **only that expression** with an explicit
`VERUS_NATIVE_RUNTIME_LIBDIR` lookup. This is a new **source-only, uncompiled,
unreviewed** path adapter. The original archive stays unchanged; the adapted
source has a different hash and does not inherit the historical build/control
results as validation. Missing configuration refuses instead of falling back to
the previous machine. The adapter does not establish runtime/source equivalence.

## Ordinary setup and build recipe

Preparation is portable; historical compiler execution was
`aarch64-apple-darwin` with Rust 1.99.1. Other targets require fresh validation,
not reuse of the old platform's acceptance or executable hashes.

Install the matching ordinary Rust/compiler-development components and build
dependencies using [BUILD.md](BUILD.md) and `rust-toolchain.toml`. Fetch Cargo
dependencies normally if necessary; the registry/git caches are not bundled.
Z3 is needed for later verifier work, not for preparation or a compiler-only
candidate build. Do not run a broad test suite that silently attempts a solver.

This is a **documented but UNEXECUTED-on-a-new-machine build recipe**, not a
claim of reproducible binaries. After the recorded resource constraints are met:

```sh
export WORK="$HOME/verus-research-work"
export SYSROOT="$(rustc +1.99.1 --print sysroot)"
export TARGET="aarch64-apple-darwin"
export VERUS_NATIVE_RUNTIME_LIBDIR="$SYSROOT/lib/rustlib/$TARGET/lib"
export RUSTC="$SYSROOT/bin/rustc"
export RUSTDOC="$SYSROOT/bin/rustdoc"
export RUSTC_BOOTSTRAP=1
export VSTD_KIND=Imported
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_DEV_DEBUG=0
export RUST_MIN_STACK=20971520
export CARGO_TARGET_DIR="$WORK/build/kernel170"
cd "$WORK/kernel170/candidate/source"
cargo +1.99.1 fetch --locked
cargo +1.99.1 build --offline --locked -j1 -p rust_verify \
  --bin continuation160 --features native-semantics170-inspect
```

Use the recorded historical target only with its matching components and host
setup; do not blindly carry that `TARGET` value to another architecture/OS.
Rebind tools and source hashes in a **fresh** receipt directory. Do not run the
old `run170.py` or mutate `STATE170.json` to reuse its remaining allowance.

For a later compiler-only original-route diagnostic, start from the preserved
`kernel170/receipts/055-original-route-final-structural-controls.json` argv:
substitute the fresh observer/output, prepared JSON plus its manifest-bound
digest, actual sysroot/target, freshly built matching libc rlib and prepared
`redb115/redb/src/lib.rs`. The archived relative dependency envelope pins
`libc 0.2.191` in its lockfile. Select the actual fresh Cargo artifact explicitly;
do not silently pick one of several `liblibc-*` files. Do not use the opaque old
`.vir` as a JSON file: final170 inspected the separately preserved canonical JSON.
The intended diagnostic refuses native export; compiler return is not proof.

## What is preserved versus intentionally rebuildable

The [capsule manifest](evidence/redb-native-mir-campaign115/portable-inputs/MANIFEST.json)
classifies all selected inputs. The additional stored payload is about 8.1 MiB:
the exact library-source archive, compressed historical `.vir`, and acquisition
provenance. Those no longer depend on local scratch/cache survival.

Ordinary toolchains, dependency caches, Z3, machine-specific executables/rlibs
and target build directories are intentionally not copied. Kernel170's converter
source and JSON output are preserved; allocation156's full source/proof/receipt
archive is also committed if a matching historical engine must be rebuilt.
Fresh builds need fresh bindings and are not promised to reproduce old hashes.
No unique implementation source for the current milestone is knowingly left
only in the old machine's scratch directories.

The [relocation validation](evidence/redb-native-mir-campaign115/portable-inputs/validation/RESULT.json)
prepared all inputs from a disposable Git-index export, with an empty HOME and
unrelated working directory. All 15 input identities and 11 unit tests passed;
all 1,122 source files were accounted for, with only the recorded path adapter
changed. Tampered input and an existing output directory were refused; restoring
the input passed again. The raw `.vir`, canonical JSON and four selected Rust
library members matched their pins. This was a same-machine relocation test,
not a second-machine compiler build, and ran no compiler, solver or application.

This is not closure of every historical pilot's replay dependencies, an offline
compiler installation, upstream source authentication, continuous custody,
bootstrap correspondence, native proof, or admission approval. Keep all existing
review/activation barriers. Heavy compiler/solver runs retain the strictly
greater-than-16-GiB free-space floor and their separately assigned bounds; this
packaging pass did not authorize broad cleanup or new proof execution.
