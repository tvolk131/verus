# Cycle78 implementation report: bounded native-source prefix bridge

Date: November 2, 2026. **CHECKED SOURCE-CONNECTED PURE PREFIX BOUNDARY;
ZERO INFERENCE PROGRESS. Independent review pending.**

## Delivered result

Three original method bodies are checked byte-for-byte, with original Vec/slice
and Range syntax: both `get_full_key` implementations and transaction
`get_full_range`. Their original records retain all fields/order/types and the
native token's u32 tuple representation. Only visibility, named results and
explicit AUTHOR-OWNED contracts are adapted; derive macros/other impls are omitted.
No pure executable body is replaced by a model or augmented with ghost code.

Checked authored lemmas/callers establish exact concatenation, prefix membership,
same-prefix suffix injectivity/cancellation and conditional separation of
prefix-incomparable namespaces. A checked overlap witness and rejected removal
of the incomparability precondition expose that boundary. Range endpoints are
checked exactly; backend ordering/scans/deletions are not proved.

Native key body SHA256 (same for both originals and both checked bodies):
`83ae7b53a60df5ab339cc91cd12539c8bceb91877ba0bd6613e05ac0077bb704`.
Native range body SHA256:
`da45d80a549f9c3928b836681545efdd6f7668a297291168c4a33e2038a65364`.
Exact declaration/body identities and compiler source spans appear in the raw
positive receipts linked by `artifacts.json`; none is inferred.

## Fresh validation

- **41 focused tests passed in normal Python; 41 passed with `-O`.**
- **13 expected replay outcomes in EACH mode**, all using the sealed runtime.
- Provider positive compiles: debug assertions on/off, default erasure,
  `--no-cheating`, **10 verified / 0 errors**; 5 checked executable bodies
  (3 native + 2 authored), 5 checked proofs, 3 specification definitions,
  exactly 13 selected identities.
- Matching imported-consumer positives in both assertion modes:
  **3 verified / 0 errors**, 3 authored executable identities. Successful
  guarded `.vir`/`.rlib` pairs are the only imported artifacts.
- Six semantic controls each report **9 verified / 1 error**, non-VIR
  postcondition failure, and EXACTLY the intended `failed_functions` identity:
  omit database prefix, reverse concatenation, erase suffix, omit transaction
  prefix, replace range end with start, and remove namespace incomparability.
- Arithmetic safety control: **10 verified / 1 error** at the added unchecked
  increment, with the exact arithmetic underflow/overflow diagnostic.
- Before/after Infer probes each request 3, generate 0, and reject the original
  `extend_from_slice` call as frontend/VIR unsupported. The final probe deletes
  a seeded earlier ordinary export. These are blocker/cleanup observations,
  NOT semantic negative controls or inferred native-body successes.
- Fifteen distinct in-memory acceptance-guard removals are all detected by the
  focused suite in EACH Python mode (30 intentionally failing mutant suites).
  No runner source is edited for these checks; transcripts are retained.
- All 28 current `ArtifactGuard` entries match before/after both fresh replays.
  Native clean pin/source checks match. Compiler JSON uses exact typed success,
  counts, complete selected scope, identity/source spans and failure categories.

Observed ordinary legacy behavior: all seven checked failed-proof controls leave
ordinary `.vir` exports despite `interface_written=false`. The seeded ordinary
omit-prefix failure replaces the earlier export. Neither export existence nor
ordinary failure cleanup is treated as certification; failed exports are NEVER
imported. Infer-aware seeded cleanup is separately checked only on the actual
Infer-requested failure. No warm-cache equivalence claim is made.

## Baseline and immutable integrity: audited, not fresh test credit

Started at clean checkpoint `3b05c1912b9d7314ad00c832c4065f4362562e02`.
Restored the FULL reviewed dirty77 baseline EXACTLY ONCE against clean HEAD with
the supplied `apply_patch` file, SHA256
`e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`.
All 210 inventory paths and all 2,092 restored files match their ledgers.
Package ledgers were checked before/after restoration and fresh validation;
the final delivery audit repeats the checks. This includes 1,570 source files,
1,085 Rust build inputs, 3,511 guarded inputs, 61 toolchain entries, 9 sealed
runtime files, 1,199 native Fedimint and 132 native redb source entries. Both
native pins remain clean; local HEAD/index and this checkout's branch ref remain
unchanged. During final sealing, concurrent shared-repository activity added T3
checkpoint refs and the unrelated `t3/review-redb-full-error` branch. Two strict
global-ref sealing attempts stopped before producing outputs; exact observations
are preserved. No agent Git ref-write command was issued and no ref was undone.
Final auditing enforces this checkout's exact HEAD/index/branch and native pins,
and DISCLOSES every shared-ref difference instead of incorrectly certifying
global immutability while other worktrees are active. Baseline file/package/
runtime/source hashes are still required to match exactly, without exceptions.

Fresh work uses only sealed tools77 (identity77/policy71/schema38), reusing vstd.
No compiler/Cargo/native-crate build, dependency/runtime/target copy, network,
Cargo/rustup shared write, agent commit/ref-write, push, PR or subdelegation occurred.
Only the small scaffold libraries are compiled by the already sealed verifier.
All homes/caches/output directories are private/fresh; inherited `GIT_*` values
are stripped and `GIT_OPTIONAL_LOCKS=0`. Old baseline suites, full vstd,
native transactions/RocksDB/consensus and hosted CI are NOT freshly tested.
Historical raw logs, including whitespace exceptions, remain unchanged.

## Preserved development attempts (not passing evidence)

Persistent scratch: `/private/tmp/verus-prefix-boundary78.95TFck`.
Every generated source, command, raw JSON/stderr and produced artifact is retained
and hashed by its inventory. Initial probes are exploratory, not final replay
credit; the first uses an explicitly authored opaque token scaffold. Final
replays retain the native token representation instead.

1. `development-replay1`: the authored cancellation helper needed explicit
   concatenation-length instantiations. 9 verified / 1 failed assertion; fixed
   only the authored proof, not any native body.
2. `development-replay2`: all proofs passed, but the reader's named-return
   signature span was off by one. It rejected rather than granting credit.
3. `development-replay3`: successful positives; negative setup rejected a
   duplicate method marker. Scoped extraction was tightened.
4. `development-replay4`: the reader incorrectly expected an empty ordinary
   failed-function list. Replaced with an EXACT intended-function check.
5. `development-replay5`: exploratory reversed concatenation used unsupported
   `slice::to_vec`. Preserved as frontend failure, NOT semantic credit. Final
   reverse mutation uses supported Vec new/extend operations and fails its
   intended native-body postcondition.
6. `development-replay6`: safety failure was correct; reader expected the wrong
   diagnostic spelling. Final check matches the exact overflow diagnostic.
7. `development-tests1`: 37-test draft exposed acceptance of a missing nullable
   `generation_failure` field. Explicit key presence was strengthened.

`development-replay7` and `development-tests2` passed, but final fresh normal/
optimized receipts are the published validation. No failed attempt is overwritten
or reclassified as passing. The original first Infer and ordinary three-body
probes are preserved separately.

The first packaging attempt reconstructed all additions correctly, then rejected
Git's ordinary no-index difference exit code 1 despite empty whitespace/error
output. The entire `final-package` attempt and original raw inventory remain
unchanged. Final packaging accepts only exit 0/1 WITH empty stdout/stderr for
this specific whitespace check. Second and third attempts stopped before package
creation on the concurrent shared-ref additions described above. The successful
final package is `final-package-v4`; prior attempts and all observations remain.

## Remaining boundary and handoff

The actual Infer blocker remains missing supported/certified handling of the
native Vec append call; clone/range inference support is NOT inferred from the
first diagnostic. `INFERENCE_BLOCKER.md` records the exact reusable failure and
a bounded future proposal without changing the engine.

`SOURCE_REFINEMENT.md` separates the three newly checked pure bodies from merely
inspected wrappers, BigSize module-ID encoding, token escape, consumption,
commit/notifications/hooks and consensus control flow. Real encoded-prefix
correctness/incomparability, async dispatch, backend honoring keys/ranges,
global-write confinement, serialization, atomic commit/drop and all previous
model-to-native contracts remain unproved. Session cleanup/index reuse and
external hooks remain excluded; fsync durability alone is not atomicity.
Allocation failures, standard-library panics, total-correctness/resource behavior
and actual backend effects are not established by these ordinary library specs.
No ownership-expression slice or whole transaction/backend/funds claim is added.

Only this evidence directory and the two new tools are added relative to dirty77.
`final-package-v4/delta.apply_patch` in the persistent scratch is ADD-ONLY against
that restored baseline, NOT the full dirty77 restoration patch. Its manifest,
SHA256, exact inventory, full file/source hashes, final audit and additions-only
reconstruction hashes accompany it. Reconstruction overlays only the new small
files on the virtually hashed baseline, without copying dependencies/targets or
creating a checkout. Independent review should start from that package.
