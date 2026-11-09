# Native hex 1.3.0 canonical single-byte encoding: bounded application experiment

Date: 2026-11-07, America/Chicago. Implementation sub-agent handoff.

## Result: NOT ACCEPTED — FRONTEND_VISIBILITY_BLOCKER

Exactly **one** direct verifier invocation was executed. It failed before body
verification with three datatype-visibility/well-formedness errors. This is NOT
the bounded nibble/table proof-failure case permitting invocation 2. Verification
therefore stopped immediately: no lemma correction, second invocation, semantic
mutant, fresh-cache positive, or warm-cache replay was performed.

There is no positive proof, no semantic-negative result, no accepted encoding
capability, no integration, and no trust adoption. This does not establish an
upstream defect or impossibility of every other annotation design. It establishes
the exact frontend blocker of this candidate with the permitted frozen engine.

All outputs are preserved in this fresh, non-Git directory:

`/private/tmp/verus-native-hex130-encoding-20261107-1VZM03`

All paths below are relative to that directory unless explicitly absolute.

## Exact blocker and strict stop decision

The frozen runtime reports that `Table([Char; 16])` is opaque outside its native
`hex_conservative::table` module because its field is private. Selecting the
actual `pub(crate)` associated constants produces crate-visible open spec bodies:

1. `native/src/lib.rs:406:40`, actual `Table::LOWER` initializer:
   `disallowed: constructor for an opaque datatype`.
2. `native/src/lib.rs:414:40`, actual `Table::UPPER` initializer: same error.
3. `native/src/lib.rs:448:27`, unconditional selection equation in the
   `pub(crate)` helper's required postcondition:
   `disallowed: field expression for an opaque datatype` at `self.0`.

In each case the diagnostic explains that the constructor/field must be
well-formed throughout crate `hex_conservative`, which is wider than module
`hex_conservative::table`. The result aborts with **3 errors, 6 warnings**.
Warnings concern inherited Clone autoderivation, not a permission to change
derives. All diagnostics are preserved unabridged in `invocation-1/stderr`.

This is frontend source/visibility incompatibility, not an SMT proof failure of
shift/mask arithmetic or finite-table correspondence. The strict plan explicitly
prohibits spending invocation 2 on frontend/model/build repairs. No visibility
widening, constant substitution, opaque wrapper, model alteration, trusted
summary, algorithm replacement, or alternate engine was attempted after failure.

`invocation-1/stdout` is complete parseable verifier JSON. Its result is:

```
encountered-error: true
encountered-vir-error: true
success: false
native_collection_completed: true
complete_selected_scope_verified: false
checked_exec_bodies: 0
checked_proof_bodies: 0
inference status: verification_failed
crate_success: false
interface_written: false
```

Collection is not verification. No reported source selection or unproved
contract earns proof credit. No `.vir` export or inference interface was written;
the private inference cache remained empty.

## Authenticated application inputs and preserved identities

Parent evidence was read only from:

`/Users/tommyvolk/.t3/worktrees/verus/t3-60978068/evidence/verus-infer-hex-v3`

- Native acceptance commit: `54b6672c00c2cbeac8bcd7c6761cf1b0bec1a6a2`.
- Engine acceptance commit: `6d80948ce8c981bd650ae851cd99c64cf7625f5d`.
- Upstream native version: **1.3.0**, commit
  `129db6ac54d25b08e0b927c225aefe4a8df05a57`.
- `native-source.tar.gz` SHA256:
  `e7f4b63eced039d437b54858fc600071b41614aa66b2a9fdc27caa94c9e0de5b`.
- Authenticated archive member `src/lib.rs` SHA256:
  `c5d584fb99c620c8e086c970afcf14b36fa4bb8838d9ead1a415ef3142c67073`.
- `closure-summary.json` SHA256:
  `762e7ac7cfc54b8a16624e36099086f524c2dd0f35b61be26b04157da98570af`.
- `requirements-preservation.json` SHA256:
  `9dd33548c9bc4ca813aa480affc38cb92464d6facbcc2f3b8ce67ade2cc0ab44`.

Archive/member and both evidence pins matched before extraction and at exit.
Only regular files/directories with safe relative paths and no `.git` member
were extracted as disposable application input. All 53 archive files were
inventoried, and the historical receipt's native file bindings matched the
extraction. No Git checkout/worktree was created.

The assessment and proposed encoding fixture were read first and preserved in
`inputs/assessment-REPORT.md` and `inputs/assessment-encoding_slice.proposed.rs`.
They remain proposals, not proof evidence. Historical receipt and source audit
are data only (`inputs/historical-v3-receipt.json`, `inputs/source-audit.json`):
no old acceptance controller was modified or executed. Historical v3's 37
results, 32 named executable bodies, and one inferred pair helper with three
mandatory certificates were NOT revalidated by this failed experiment.

## Frozen engine, source binding and loader qualification

Used **READ ONLY IN PLACE**, without copying the runtime:

`/private/tmp/verus-operational91-runtime.q91/runtime`

Every artifact in the supplied manifest was SHA256-authenticated at entry,
again before invocation 1, and at exit. The manifest itself is:

`/private/tmp/verus-operational91-runtime.q91/runtime-manifest.json`

Manifest SHA256:
`1fa38f105d5b9fda9bda2ad3b0ee4518f083f917bc77dcbce6585ff152e476a3`.

| Matching artifact | SHA256 |
| --- | --- |
| verus | `819f6665ed710b7c9a7f641ff499c104eba32e4c6da3f7cfe802676f967294e0` |
| rust_verify | `468229c6de195c436973a2a1fdd0b7744eb0a1099f3f659ce282831556536e5f` |
| libverus_builtin.rlib | `941acbc2b162f5d94508af7dc3123ba8b427a22582ce105eafc21fe36ead2935` |
| libverus_builtin_macros.dylib | `a183d6ad52cac685b97f30a7cea8d6ccbe05d31c8d2a0c157441a46107ff9dd5` |
| libverus_state_machines_macros.dylib | `8a9744e44216b0a1a7e99156aa9a1706bdfa0856b5d75c9870b39bf51305c1bf` |
| libvstd.rlib | `48daa990c92f620393b69f1469858f1264d9c0f6365615d0a324bad14ab16e18` |
| vstd.vir | `8017cdac8cbc55979129802085c6b47f7d966d3e63017bed6b480a498699ec13` |
| vstd_build (not executed) | `14a7df403de736cd53e507a380dd1a83d3603d7fab1f0c4b319c4a952bfab306` |
| z3 | `edae32f9e37ea4b5bb35310d72f0e352d0dc07626cac4e9e30bc1ea9a5bc8efb` |

Historical source/build binding authentication checked canonical manifest
identities, successful recorded compiler and matching-vstd build receipts and
their log hashes, qualification-to-runtime equality, every listed qualification
source binding, the recorded Rust compiler hash, and all 30 loader libraries.
No historical build was replayed. Detailed bindings and full hashes are in
`authentication-entry.json`, `authentication-pre-invocation-1.json` and
`authentication-exit.json`.

- Frozen compiler source-manifest canonical identity:
  `e474a327b1a90258200b82eb9b6525e35cc9741fcca99f488cc5b040a61c019b`.
- Frozen compiler identity excluding its recorded test-only source:
  `b6294b48c66cb799b0793b2bdbddc02ff489f23084eaee4eb7464ffcbdd94337`.
- Frozen full source-manifest canonical identity:
  `488e3d4aa250ff236b54af45ab0a7cd6a868445476bcdcb9e8887911ec6d1fca`.
- Recorded runtime version/platform: `stateful-source`, release,
  macOS/aarch64, commit label `source-e474a327b1a90258200b82eb9b6525e35cc9741fcca99f488cc5b040a61c019b`.
- Toolchain: `1.99.1-aarch64-apple-darwin`; exact sysroot:
  `/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`.
- Recorded rustc identity: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`;
  rustc binary SHA256
  `1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1`.
- Exact `DYLD_LIBRARY_PATH` from existing runtime execution metadata:
  `/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib:/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/lib`.

The launcher was given `VERUS_USE_RUSTUP=0`, explicit `--sysroot`, the exact
loader environment and matching solver; it directly executes its sibling
`rust_verify`. No toolchain/version/proof metadata probe launched verification.
Only the one budgeted application invocation did so. The inspected launcher,
configuration and attribute-rewriter sources matched their frozen compiler
manifest entries; this is not a claim that today's entire parent source matches.

**Qualification boundary:** this is a frozen campaign91-engine experiment. It is
NOT current-parent or hardening95-built acceptance, NOT the historical hex-v3
engine, and NOT the separate cached rust-hex 0.4.3 lane. No new hardening driver
was combined with older vstd. Historical model/qualification receipts are
preserved provenance, not a newly verified toolchain or newly adopted trust.

## Candidate source, contracts and preservation audit

Deliverables:

- `native/src/lib.rs`: disposable annotated native source.
- `native/src/encoding_spec.rs`: cfg-gated independent arithmetic policy and goal.
- `source-delta-invocation1.patch`: complete new source/proof delta.
- `selected-source-audit-invocation1.json`: hashes, exact additive spans,
  original/erased function hashes, constant hashes and domains.
- `source-invocation1/lib.rs`, `source-invocation1/encoding_spec.rs`: frozen
  pre-invocation candidate snapshots.
- `native-authenticated-manifest.json`: all authenticated archive file hashes.

Frozen candidate hashes:

- `native/src/lib.rs` SHA256:
  `48c433d5a672db9a00ea45cdbdd5f52d0a0ae0750db0e707f440b2ebe053fc24`.
- `native/src/encoding_spec.rs` SHA256:
  `9f2b2bd5056f1d5955f7787dc54a50d4d145781752aa740b763d76f242fc491c`.

Only `src/lib.rs` differs among the 53 archived files, by additive native
selection/contracts and cfg-gated declaration/imports. The new module is entirely
cfg-gated. Original executable statements, original function signatures, enum
variants/discriminants, types, derives and both literal constant initializers
remain byte-for-byte unchanged. No erased proof block or checked lemma was added
inside the original helper; invocation 2's optional augmentation was not used.

Fresh audit checks insertion-only deltas against the authenticated member,
compares both original methods including signatures/bodies, both selected real
initializers, and confirms all other archive files unchanged. The historical
218-function preservation audit is inherited evidence; the current additive
audit does not claim to rerun its original parser/compiler. Exact direct-body
comparison and unchanged other files guard the new delta.

Intended, **unproved** requirements:

- Every `u8` (all 256 values) and both original `Case` variants, without input
  preconditions. Numeric ASCII policy is independent of tables and decoding:
  `48+n` for `n<10`, otherwise Lower `87+n` or Upper `55+n`.
- Original `Case::table` must return the logically equal original LOWER/UPPER
  table selected by the actual case.
- Original helper must satisfy unconditional table-selection equations for
  **every legal `Table([Char;16])`**, not just canonical tables. Canonical ASCII
  conclusions are conditional on logical value equality to LOWER or UPPER.
  There is no call to the original native `PartialEq::eq` in the specification.
- Actual LOWER and UPPER initializers were explicitly selected; no copied array,
  assumed initializer semantics or canonical-table constructor invariant.

No new trusted spec, primitive axiom, external body, assume/admit, library-model
change, Infer change, domain narrowing, function filtering, no-verify or
no-lifetime setting was added. Authored required contracts are not new trust.

## Retained scope, proof/assistance/publication distinctions

The command retained the historical full application configuration: crate
`hex_conservative`, library mode, `std`, `alloc`, `verus_hex_vec_phase3`,
`--infer-goals`, `--no-cheating`, and no scope filters. It selected the complete
retained v3 scope plus candidate items. Comparing stable `rust_function`
inventories finds all **71** previously selected native function entries still
selected, with **8** additional entries (including derived Case Clone, both
initializer proxies, both original encoding methods, two policy definitions and
the canonical goal). This is a collected inventory, not checked coverage.

`proof-result-checks.json` retains complete collection/status details and the
stable-name multiset comparison. Frontend abort makes all selected items
untranslated/unverified; no ordinary executable/proof bodies were checked.
The retained inferred pair's historical soundness/maximality/exactness
certificates were not freshly checked. New inferred helper count is zero.

Assistance explanations report enabled profiles but preparation did not start
or complete, with no decisions. Assistance arrays and inferred function output
are empty. Reported generation time is zero; total internal metric is 579906
microseconds, not a checked-proof time. No candidate cache entries, interface or
published export exist. No generated telemetry is treated as a proof certificate.

## Execution and resource accounting

| Strict slot | Execution | Outcome |
| --- | --- | --- |
| 1 | Full retained scope + candidate, fresh private cache | exit 1; frontend visibility failure; 0.781733667 s |
| 2 | NOT RUN | not eligible: failure is not bounded nibble/table lemma work |
| 3 | NOT RUN | no passing full-scope positive, so wrong-case negative forbidden |
| 4 | NOT RUN | no accepted positive/control; no fresh-cache replay |
| 5 | NOT RUN | no prerequisite fresh positive; no warm-cache replay |

Total direct verifier invocations: **1**, failures: **1**, total measured direct
verifier wall time: **0.781733667 s**, workers: **1** (`--num-threads 1`). No
timeout, interruption, semantic rejection, successful proof or certificate
result is claimed. Unused invocations were not repurposed.

Before invocation 1: **87,201,316,864 bytes free**, greater than 16 GiB.
Allocated fresh scratch before invocation: **7,106,560 bytes**; after invocation:
**7,258,112 bytes**. Final totals are recorded by finalization and remain below
64 MiB. Runtime files were not copied. Disk guards and a 59.5-second process-group
kill guard were armed; none fired. Individual output files were capped at 8 MiB,
scratch at a 60 MiB early-stop guard, and core dumps disabled.

One preparation-only failure occurred before verification: a conservative
4 MiB historical-receipt read-size guard rejected its 5,278,011-byte size.
The guard was increased to 8 MiB, within the overall 64 MiB allocation bound,
and extraction preparation resumed. All runtime checks had already matched.
This was neither a verifier execution nor a source/frontend repair or proof
retry. `preparation-attempts.json` preserves its argv, failure and observed
0.3817-second command duration. Preparation then succeeded in an observed
0.3549 seconds. `discovery-failures.json` also records one absent optional
metadata filename and a deliberate `head` pipe's tar broken-pipe diagnostic;
neither launched verification or selected another runtime.

## Raw receipts and boundaries

- `invocation-1/invocation.json`: exact raw argv, complete explicitly supplied
  secret-free environment, cwd, source hashes, empty initial cache, runtime
  binding, UTC start, resource settings and disk observation.
- `invocation-1/result.json`: exact exit/timing, logs' SHA256, cache state,
  allocation/free-space observation, export state.
- `invocation-1/stdout`, `invocation-1/stderr`: complete raw verifier streams.
- stdout SHA256:
  `19ec7179a2a15712ef9ccdef02de94440f0b440792311061de2bcafc7d1afb96`.
- stderr SHA256:
  `e99d0e9713099cff6d771f28a2e0f724206c34c900f66891d707ac42ac7fe1e4`.
- `proof-result-checks.json`: result classification, complete scope/proof checks,
  controls, ordinary/inferred/assistance/publication distinctions and budget.
- `authentication-entry.json`, `authentication-pre-invocation-1.json`,
  `authentication-exit.json`: all runtime/loader/source bindings and rechecks.
- `input-bindings.json`, `output-manifest.json`: preserved input/output hashes.
- `prepare.py`, `audit.py`, `invoke.py`, `finalize.py`: new scratch-only
  instrumentation; not old acceptance controllers or an integration harness.

No parent file, frozen input, runtime artifact, redb cell, S0/Infer/import/export
barrier, engine/model source or historical controller was modified. No build,
Cargo command, native executable test, network operation, Git operation,
subdelegation, alternative-engine search or cleanup occurred. All18 redb cells
and all stated barriers remain outside this experiment and unchanged. The
separate rust-hex 0.4.3 lane was not touched. This is not a retry/reroute of any
refused/exhausted task; no historical without-assistance control was replayed.

No whole encoder, iterator, vector, whole-crate, other-target or trust claim.
The parent alone handles any final integration or independent review.
**Stop at handoff: NOT ACCEPTED.**
