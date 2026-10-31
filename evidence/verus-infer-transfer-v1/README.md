# Frozen transfer v1: unsigned-varint 0.8.0

**Result: frontend-blocked; no application proof, inference certificate, or
application defect established.** Frozen on October 31, 2026. One crate, one
encoder, one unchanged required domain. No engine, macro-library, vstd source,
trusted-model, parent checkout, or parent release-artifact modifications.

## Selection and pre-proof freeze [E1]

Selected `unsigned-varint` **0.8.0**, Parity Technologies, MIT, upstream repository
`https://github.com/paritytech/unsigned-varint`. Packaged VCS revision:
`10d55e5dbdd7f61b6495bd12a49d06df82e08612`. The cached published archive SHA-256 is
`eb066959b24b5196ae73cb057f45598450d2c5f71460e98c49b738086eff9c06`.
The archive agrees with the cached registry index; selected snapshot files agree
byte-for-byte with archive members. This is local package provenance, not an
independently fetched Git revision or authenticated supply-chain attestation.

The selected original items are `src/encode.rs`'s `encode!` macro, `pub fn u64`,
and `const U64_LEN: usize = 10;`. Both original executable bodies remain
byte-for-byte present in every trial fixture. `original-selected.rs` SHA-256:
`faef7ad4eec15afcffa66e445b8a63b12f5f6cb0139336f53c87e224fd6ae7c0`.
The full original encode/decode/lib files, package metadata, license and package
archive are retained under `upstream/`; other upstream algorithms are not proof
targets. Original copyright/license notices are preserved there.

**Static suitability:** this is a small no-allocation/no-I/O encoder with a
10-byte output buffer. Its scalar shifts are familiar to the prototype, but a
mutable iterator, early `break`, macro expansion, inclusive subslice and returned
borrow make complete frontend/inference support uncertain. Only this crate's
bodies were inspected; a cache filename inventory also showed other crates, but
none was tried. The selection was not contingent on a passing proof.

**Required domain:** every `u64` (including zero and `u64::MAX`) and every
initialized, exclusively borrowed `&mut [u8; 10]`, regardless of previous byte
contents; aarch64-apple-darwin, 64-bit `usize`. There is **no `requires` clause**,
small-number bound, zero-buffer premise, or post-failure domain reduction.

**Required property:** the returned slice is the canonical unsigned base-128
varint of the input. Its length is 1..=10; byte `j` has payload
`floor(number / 128^j) mod 128` and continuation bit exactly when another byte
follows. `number < 128^length`, and a multi-byte encoding has
`number >= 128^(length - 1)`. Safety/debug assertions must hold throughout the
domain. Untouched buffer suffix preservation is additionally tested natively,
but is not claimed as part of the frozen proof goal.

`selection.json` was written at **19:06:45.788222 UTC**, SHA-256
`099873723b641260e5bf712b66a32e2a73bd7d0be283f75cfac534fc61a5c96b`.
It predates vstd verification (19:09:47 UTC), artifact freezing (19:11:27 UTC)
and the first application trial (19:12:16 UTC). Initial fixture hashes are in
`pre-proof-fixtures.json`. Those records and original fixtures remain unchanged.

## Engine provenance [E2]

The provided checkout was initially clean. HEAD was exactly
`45f93e51f4d0c0f1b71ef645932d54a7862ec983`; both HEAD and requested baseline had
tree `10a1d83c1e462f9a8980973bfe9435b44aaedb5d`. No applicable `AGENTS.md` was
found in the checkout or checked ancestors. The direction/reference/adoption,
TCB/unverified-code guidance, relevant assurance sections, contribution policy,
build instructions and existing experiment conventions informed this trial.

`source/target-verus` did not exist initially. Following the current guide's
manual Cargo recipe, a new independent engine was built **offline** here:

```sh
cd source
cargo build --release --locked --offline -p rust_verify -p verus -p cargo-verus
cargo generate-lockfile --offline --manifest-path vstd/Cargo.toml
PATH="$PWD/target-verus/release:$PATH" VERUS_Z3_PATH="$PWD/z3" \
  target-verus/release/cargo-verus verus build --release --locked --offline \
  --manifest-path vstd/Cargo.toml
```

Engine build: exit 0, 50.013 s. Matching vstd build: exit 0, 20.771 s,
**2219 verified, 0 errors**. The generated ignored vstd lockfile is retained as
`vstd-build.Cargo.lock`; no vstd source was edited. An initial locked vstd build
failed because that lockfile did not yet exist; its diagnostic is retained.
No unrelated cargo-make installation, IDE tool or repository-wide test build
was needed.

Identities: Verus `0.2026.10.31.45f93e5`, release/macos_aarch64; Rust **1.99.1**,
commit `58cf8f9c35b355199727627d07ae34a4e2846d67`; Z3 **4.16.0**. Full SHA-256,
size, real path and inode receipts for verus, rust_verify, both macro libraries,
builtin, vstd interface/rlib, actual rustc/rustc_driver, Cargo and Z3 are in
`artifacts-before-proofs.json`; the audit rechecks their hashes after all trials.

The sole copied parent binary was **standalone Z3**, independently copied to
this checkout's ignored `source/z3`, with a different inode and matching hash.
Its installation origin is not independently authenticated. Parent release
verifier/macros/vstd were never used: their uncommitted-work provenance is not
relabelled as this committed baseline. Shared installed Rust and Cargo registry
dependencies are trust/reproducibility inputs, not independently rebuilt tools.

## Adapters, authored intent and proof work [E3]

| Item | Role and audit |
| --- | --- |
| Original `encode!`, `u64`, `U64_LEN` | Exact original text retained; no algorithm replacement, mutable-iterator rewrite or input restriction. |
| Prelude, proc-macro hygiene feature, Verus macro enclosures | Frontend plumbing only; no dependency contract or axiom added. |
| `global size_of usize == 8` | Preselected target model, not a discovered numerical precondition. |
| `canonical` and `radix_power` | Authored intended encoding relation and mathematical radix definition; not inferred behavior. Neither received application-run proof evidence. |
| `decreases exponent` on the spec definition | Authored spec-recursion termination decoration, not an encoder invariant or implementation proof. |
| `#[trigger] encoded[index]` | One authored specification-instantiation hint; not behavioral intent, an assumption or generated assistance. |
| Ordinary `#[verus_spec]` postcondition | Required property on the original `u64`; signature and executable body unchanged. |
| `#[verus_infer]` plus `encode_policy` wrapper | Separate exact-inference probe on the same original body; ordinary one-call wrapper requests the same full-domain policy. No authored contract is combined with inference on one function. |
| Constant-only enclosure correction | After initial failures, wrap unchanged `U64_LEN` in `verus!`; no original body/property/domain edits. Before/after hashes and rationale retained in `constant-enclosure-adapter.json`. |

No views, encoder loop invariants, encoder decreases measures, handwritten
implementation proof blocks, external bodies/specifications, assumes, admits,
termination assumptions, verification filters, erasure-check bypasses or trusted
models were added. The upstream `debug_assert_eq!(n, 0)` is an original executable
check, not an injected proof assumption. Ordinary Verus/rustc, Z3 and matching
vstd models remain the toolchain TCB. Empty reported model dependencies at an
early frontend failure do not mean that the toolchain has no trust assumptions.

## Actual trials [E4, E5]

The first 8 trials reported **two frontend errors**: macro-generated `for`
unsupported and `U64_LEN` accidentally outside verification scope. The latter
was an adapter omission, not an upstream defect or automation limitation. Only
that unchanged constant was enclosed, and 8 further trials isolated the former.
All initial source, JSON, diagnostics and timings are preserved separately.

The isolated diagnostic is:

```text
The verifier does not yet support the following Rust feature:
`for` loops produced by a macro expansion
(wrap the loop in the macro body with `verus_exec_expr! { ... }`)
```

The original macro loop has not been rewritten. Current implementation source
at `source/rust_verify/src/rust_to_vir_expr.rs:3588` explains that loops not
visited by Verus's syntax macro reach this unsupported native-for-loop case.
This is **not** evidence of failure of a mutable-iterator model, invariant
discovery, the authored postcondition, or an application bug; those stages were
not reached. A documented proof-only workaround exists but was not implemented
in this frozen unchanged-macro trial.

Corrected-constant whole-process observations (seconds, one ordered repetition):

| Annotation style | Assistance | Cold | Warm | Both exit statuses |
| --- | --- | ---: | ---: | --- |
| Original body + authored policy | Off | 0.831915 | 0.295998 | 1, 1 |
| Original body + authored policy | `--infer-goals` | 0.288328 | 0.293167 | 1, 1 |
| Inferred original + policy wrapper | Off | 0.287849 | 0.295733 | 1, 1 |
| Inferred original + policy wrapper | `--infer-goals` | 0.291084 | 0.294901 | 1, 1 |

Every application invocation used `--no-cheating --output-json --time`, a
requested export path and a separate process. Cold starts with a nonexistent
candidate cache; warm keeps the same source, flags, export path and candidate
cache. Neither is an OS-cache cold measurement. All application caches remain
empty and no requested interface exists after any run.

All 16 JSON reports have `success=false`, `encountered-vir-error=true`,
`status=verification_failed`, `crate_success=false`, zero checked executable and
proof bodies, zero generated candidates, zero body-proof/solver time and no
interface written. Even the inference-annotated cases report zero discovered
requests: translation failed before discovery. With assistance requested,
`enabled=true` but preparation never starts, decisions are empty and no proposal
is applied. These are **frontend timings, not proof times**, cache speedups or
assistance-quality measurements. No fresh application proofs existed to compare.

## Native tests and engine sanity [E6, E7]

The unmodified upstream library was compiled directly with rustc, avoiding
unrelated optional/dev dependency builds. `native_tests.rs` checks the actual
upstream `encode::u64` **and** the exact extracted selection against an independent
u128 division/remainder encoder, canonical termination, reconstructed value and
unchanged suffix. Each profile covers **173,503 input/buffer pairs**: exhaustive
low 16-bit values, all base-128 length boundaries with every repeated byte buffer
pattern, and 100,000 deterministic full-width samples with varied buffers.

Debug: **3/3 pass**, exit 0, process 0.231278 s. Optimized with debug assertions
disabled: **3/3 pass**, exit 0, process 0.455465 s. These are authored finite slice
tests, not a proof for all u64/buffers or the crate's complete upstream Cargo test
suite. An initial native test link failed solely because the rlib filename lacked
the required `lib` prefix; the corrected commands and both diagnostics remain.

As a tooling check, the baseline's existing `verus_infer_quickstart.rs` runs in
fresh cold/warm processes: both exit 0, **9 verified, 0 errors**, all 3 functions'
mandatory soundness/maximality/exactness checks proved; cold candidates miss,
warm candidates hit, with fresh solver evidence in both reports. Process times
are 1.000490/0.402170 s. This is not another transfer candidate or application
result. vstd/tooling proofs do not rescue the blocked encoder proof.

**Semantic mutation control: not run**, because no meaningful positive
application proof succeeded. A frontend rejection of a mutation would not test
the required property, so no such rejection is counted as a semantic control.

## Reproduction and limits [E8]

All meaningful build/native/verification command receipts contain literal argv,
cwd, relevant environment, exit status, UTC start, monotonic wall time and raw
stdout/stderr hashes in `raw/`. Compiler JSON is saved verbatim as `.stdout`.
The two trial summaries preserve cache inventories and export absence. Final
input/artifact/body/receipt checks are in `audit.json`; the bounded SHA-256
inventory is `evidence-index.json` (not a proof certificate).

With the independently built tools still present, replay to a **new** directory:

```sh
python3 evidence/verus-infer-transfer-v1/replay.py \
  --output /private/tmp/verus-infer-transfer-v1-new-replay
```

This replays native debug/release, the isolated 8-trial matrix and fresh cold/warm
engine sanity, validates the specific frontend diagnostic and required sanity
proofs, and retains new raw logs. The successful local portability replay and
its complete receipts are in `replay-corrected/`. Its 16 commands validated.
A first portability replay had an incorrect path substitution; it is isolated
under `replay-failed/`, is not trial/proof evidence, and was corrected before the
successful replay. Setup observations not automatically logged are identified
as transcript-derived in `setup-events.json`, with no invented timings.

Compiled artifacts, target directories, native test binaries and candidate
caches are **not committed**. Rebuilding requires the pinned Rust toolchain,
baseline source/dependency cache, solver and generated vstd lockfile. The build
recipe is above; source checks and artifacts were independently frozen, but no
reproducible-build or full dependency supply-chain claim is made. A separately
built unchanged engine may use `--engine-root` and explicit
`--allow-rebuilt-artifacts`; the replay then records hash differences as distinct
evidence, not byte-identical original provenance. OS state, host load, absolute
paths and process startup affect these single-repetition timings. Harness
launch-error handling and replay validation were improved after capture; literal
raw commands, frozen fixture hashes and retained original logs are authoritative.

## One next extension, not implemented

Extend **macro-loop intake for this same frozen slice** with a proof-only
`verus_exec_expr!` adapter around the original macro loop, the existing frontend's
suggested route. Check erasure/body equivalence and rerun this exact property and
full domain before considering engine changes. Add an independent macro-loop
regression when implementing that extension. This is one bounded adapter/front-
end integration step, **not** a claim that arbitrary mutable iterator inference,
early-break invariants or this encoder's postcondition will then verify. There
is no evidence yet justifying a new trusted model or a broader inference feature.

## Evidence index

- **E1:** `selection.json`, `pre-proof-fixtures.json`, `original-selected.rs`,
  `upstream/` (including license, metadata and immutable crate archive).
- **E2:** `artifacts-before-proofs.json`, `vstd-build.Cargo.lock`, build raw logs,
  `audit.json` (final source and artifact preservation).
- **E3:** `fixtures/`, `constant-enclosure-adapter.json`, fixture generators;
  adapter/specification audit in this report.
- **E4:** `raw/*.json`, `raw/*.stdout`, `raw/*.stderr`: exact primary commands,
  statuses, timing, complete compiler JSON and diagnostics.
- **E5:** `trials.json`, `constant-enclosed-trials.json`: all 16 original attempts.
- **E6:** `native_tests.rs` and native build/test raw logs.
- **E7:** engine-sanity cold/warm raw logs, including all fresh proof outcomes.
- **E8:** `replay.py`, `replay-corrected/`, `replay-failed/`, `setup-events.json`,
  `audit.py`, `evidence-index.json`. Ignored `work/` is not retained evidence.

AI-assisted research/code/report: **OpenAI Codex:gpt-6.1-sol**, the authorized
replacement after Claude Opus 5.5 was rate-limited before work. No other agents,
push, PR, merge, watch or schedule. The experiment commit changes only this
directory; its local Git commit is the publication boundary, not an engine pin.
