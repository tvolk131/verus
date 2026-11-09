# Independent bounded cycle79 candidate review

## Decision

**PASS for the stated bounded native-prefix inference candidate; no new blocking soundness or acceptance defect found. Not approval of shared integration or the whole application.**

I independently reconstructed and reviewed the exact candidate in my initially clean checkout. I used no subagent, rebuild, commit, branch/ref write, push, PR action, shared Cargo/home/toolchain write, or native-application edit. Candidate/parent code received no fixes. The only candidate-tree changes were the two requested patches, each applied once. Review tools and generated evidence are in this private review directory.

The two frozen shared identity-reader failures and the nested macOS loader failure remain parent-owned integration work. They are not silently accepted, fixed, or represented as passing shared CI. Earlier baseline reviews are background, not approval of this delta.

## Exact subject and reconstruction

- Checkout: `/Users/tommyvolk/.t3/worktrees/verus/t3-30da317f`.
- Initial and final HEAD: `3b05c1912b9d7314ad00c832c4065f4362562e02`.
- Own branch: `refs/heads/t3/review-cycle79-candidate`; own HEAD, branch and raw index SHA256 are unchanged.
- Accepted baseline: `/private/tmp/verus-storage-cycle78.G9sUIt/final-package/combined.apply_patch`, SHA256 `b9df7a504753f419997ec8ae0fa35e37f28de165e37dfd39d604c79993420f75`.
- Candidate delta: `/private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/baseline-relative.apply_patch`, SHA256 `0b5fa2b368840f0e3254692f1ff903e8127527c5b99cac9049f33b1df52d3636`.
- Read sealed delivery README before applying either patch. Baseline application and candidate application each occurred exactly once through `apply_patch` stdin, on this checkout; no worktree was shell-created.
- Before the candidate delta: all2122 file hashes and the exact tracked/untracked nonignored inventory matched; all1574 source hashes and1085 Rust build-input hashes matched.
- After the delta and again after tests: all2133 file hashes and the exact inventory matched; all1579 source hashes and1088 build-input hashes matched. All2133 recorded source modes matched too. The17-file delta is six modifications plus11 additions.
- Delivery seal: all6351 regular files listed in its `SHA256SUMS` matched. The12 baseline-package ledger/patch hashes matched. Frozen source snapshot, eight runtime regular files,61 toolchain entries, solver, and native ledgers were independently checked.

See `git-initial.json`, `git-final.json`, `before-audit.json`, `after-audit.json`, `delivery-seal-audit.json`, `matched-evidence-audit.json`, and the two apply stdout/stderr pairs. Git audit subprocesses remove inherited `GIT_*` and set `GIT_OPTIONAL_LOCKS=0`; runner subprocesses additionally use private HOME/Cargo/rustup/cache/tmp locations. The initial shell environment exposed only the harmless inherited `GIT_PAGER=cat`; all subsequent Python Git audit environments strip every `GIT_*` before setting optional-locks0.

## Authority and provenance

The only new-engine runtime executed was delivery `runtime78/`, the final compiler10 runtime. Its actual `rust_verify` SHA256 is `2dc428e7717fb2ba77b6640a7aed18dae5c7ecd44265046898ec2637627355e6`; its launcher SHA256 is `93b2d370189f3b529908bbfad0fad80285c9a166546f2a813a853fce80303700`.

Explicit toolchain: `/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`. Solver was passed through `delivery/runtime78/z3`, whose read-only target is `/private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/z3`. The requested baseline77 verifier was used only for the old-engine decline, never as new-engine proof authority. Earlier compiler9 artifacts/replays receive no proof credit here; checking the overall delivery hash seal does not credit them as proofs.

Audited separately, without Cargo/compiler/vstd rebuild:

- Final `attempts/build-final.stdout` has successful Cargo build-finished status and names all six recorded Cargo-produced runtime products. All six runtime product hashes match `build-receipt.json` and `runtime-hashes.json`. Five products are Cargo-fresh reused build products; `rust_verify` is recompiled in that final build. These are not six fresh recompilations.
- The recorded build-source-ledger digest matches the1088-input Rust ledger, which matches this candidate and the frozen source. All11 named validation-log digests match.
- `attempts/vstd-compiler10.stdout` records2221 verified/0 errors, with exit0, matched final runtime artifacts and recorded command. No reused compiler9 vstd proof is credited.
- `attempts/rust-focused-compiler10-2.stdout` contains nine suite counts6+6+13+18+17+3+7+21+2 = **93**, not113. These are audited final-runtime Rust integration tests, not fresh reviewer Rust test executions.
- `attempts/vir-unit-4.stdout` records148 unit tests. Source-identical generated-term/native-u8 append oracle evidence covers14641 pairs. This is audited unit evidence, not freshly rerun compiler10 integration.
- Both supplied final `replay-compiler10` receipts were independently parsed and rechecked against complete contracts/scopes and completion requirements:30 cases each. These audited replays are separate from the two fresh reviewer runs below.

This is a matched provenance/log audit, not a reproducible-byte-build proof or cryptographic authentication of arbitrary maliciously reconstructed VIR files.

## Fresh reviewer execution counts

| Fresh execution | Result | Evidence |
| --- | --- | --- |
| New runner, ordinary Python |30/30 accepted cases, PASS | `replay/receipt.json` |
| New runner, Python `-O` |30/30 accepted cases, PASS | `replay-optimized/receipt.json` |
| New acceptance unittest module |18 tests pass | `acceptance.stderr` |
| Same acceptance module, `-O` |18 tests pass | `acceptance-optimized.stderr` |
| Independent compiler probes |35 completed invocations; classified below | `independent-probes-final.json`, `additional/independent-probes-final.json` |
| Independent receipt corruptions |18/18 rejected per mode | `receipt-probes.json`, `receipt-probes-optimized.json` |
| Guard-removal regression sweeps |six per mode; each detected by the18-test suite | `regression-sensitivity*.json` |

Do not sum audited and fresh results into one fresh test count. Regression sweeps intentionally weaken an in-memory review target and intentionally fail tests; they are sensitivity evidence, not normal passing tests.

Both fresh runner modes verify provider23/0 with five inferred functions and consumer6/0 with one inferred composition. Provider selected-body scope is eight executable bodies, five proof bodies and three specification definitions; consumer is four executable bodies. Full-scope collection/completion, exact typed scope records, verified counts and complete contracts are checked, not merely exit codes or the presence of an inference report. Cold/warm/poisoned phases all report regenerated-state misses and fresh positive solver time for every inferred function.

Each mode's30 cases includes the baseline unsupported/zero-generated decline; cold/warm/import/composition/cache checks; four exact native mutants followed by specifically located semantic failures; false/malformed goals; unsafe indexing; six frontend controls; five import-metadata faults; final restored provider and consumer. Failed negative exports are seeded from real successful exports and must disappear. Categories remain distinct.

## Native fidelity and scope

Fedimint is clean at `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`; its1199 native-ledger entries match. Redb is clean at `c82ab865bb362bc4af067d332adc44978ff64813`; its132 entries match. Both full ledgers and pins were guarded before and after review, without edits.

All three selected native method items and executable bodies are byte-identical to the native source: `PrefixDatabase::get_full_key`, `PrefixDatabaseTransaction::get_full_key`, and `PrefixDatabaseTransaction::get_full_range`. Only Infer attributes are inserted outside method items. The original private records and methods stay private. Native `PrefixDatabase<Inner>` retains its `Inner: Debug` domain; the transaction helpers retain unconstrained `Inner`. Record field order, types and bounds, and `GlobalDBTxAccessToken(u32)` are preserved. The detached impl spells the same Debug bound inline rather than changing its domain.

None of these three native methods has authored pre/postconditions. Frozen inference gives `requires true` and complete byte-view concatenation or both complete Range endpoint equations, not merely lengths or some elements.

Two **auxiliary** public inferred wrappers construct `Inner=u8` transaction records. They are not extracted native methods. Consumer `composed` is also auxiliary; other executable goals and namespace/cancellation lemmas are explicit authored proof goals, not inferred provider contracts. The actual exported provider certificate contains exactly `key_contents` and `range_contents`; private native methods are not exported by making them public. A fresh independent import attempt naming the private native record fails with Rust E0603, without publishing an export. This is a language visibility rejection, not a semantic proof failure.

## Implementation audit

### Identity, types and standard-library authority

`source/vir/src/infer/owned/byte_append.rs:10` and `:44` recognize closed canonical interfaces, not names alone: correct crate identities/paths, canonical proxy, function kind/modes, no executable model body, exact parameter/return shape, generic arity and Sized/Clone/Allocator bounds. Shared model gates reject Infer/unsafe/async/user-mut/proof-existential/handwritten-requirement alternatives. Slice View is a canonical resolved View trait implementation; Seq concatenation is `vstd::seq::Seq::add` with its actual spec signature, not arbitrary overloaded Add.

The append call gate at `source/vir/src/infer/owned/byte_append.rs:116` requires concrete u8 and alloc::alloc::Global, correct two arguments/unit result, a direct mutable local destination, no parameter destination, complete sequence state, and a distinct inert immutable local slice read. `u8` primitive Clone semantics and existing vstd Vec clone/extend specifications justify value-preserving byte copying; no extrapolation to arbitrary `T:Clone` is permitted. The candidate adds no vstd behavioral contract, axiom, assume, trusted application summary, or desired-conclusion helper.

The trusted-model certificate contains380 registered standard models. This is an inventory, not an invented minimal per-function dependency list. Ordinary verifier/solver plus existing vstd and primitive Clone authority remain explicit trust.

### Mutable state, aliases and evaluation

Append composes the prior sequence and the slice view in source order, invalidates the destination's stale concrete-object observation, and updates only that local's complete sequence state. The tail gate excludes effectful calls, projections and expression blocks rather than assuming their effects away. Existing bounded state machinery also rejects unsupported mutable aliases or local reassignments.

Fresh independent positives cover repeated appends, a moved accumulator, immutable aliases to the same slice, and a cloned snapshot followed by later mutation. Complete equations distinguish saved old state from the subsequently modified vector. Reversal, wrong length and wrong snapshot goals first infer exact candidates, then fail the intended `probe::goal` with one postcondition error; seeded exports disappear.

### Range and generic domains

`source/vir/src/infer/records/shape.rs:76` preserves canonical Range<u64> and adds only `Range<&[u8]>` and `Range<Vec<u8,Global>>`. Canonical core identity, vstd ExRange proxy, public transparent two named fields, their types, generic/Sized shape and other record gates remain checked. No global generic-call or mutable-record policy is relaxed. Range properties concern both complete endpoint views; no claim follows about capacity, allocator state or object identity.

A separately authored independent implementation with inert local endpoint slices infers exact Range output, proves both correct endpoint equations, and rejects an incorrect end=start equation after inference. Range<u16>, mutable Range destinations, RangeInclusive and direct projected append tails decline rather than receiving an unsafe generic widening.

### Proofs, import, publication and cache

Required soundness/safety, safe-domain maximality and exactness remain independent obligations. They are freshly required and proved for accepted inferred functions; `requires true` is not obtained by manually constraining input domains. Ordinary callers and authored false goals still verify normally and block publication on failure. The standard-library model domain remains the domain of these claims, not allocation/OOM or durability guarantees.

`source/rust_verify/src/import_export.rs:98` checks inference-aware headers, bounded trailer/schema/policy/producer, unique complete function proof identities, aggregate/per-function guarantees, body pruning, summary fingerprints, dependency fingerprints, trusted-model identity and imported-summary shape. Final publication remains gated; no unchecked interface is retained after the tested semantic/frontend/import failures. Cache78/policy72/schema38 identities are intentional. Owned-value profiles regenerate candidates instead of trusting cache-decoded proof results. Cold/warm misses and poisoned-file noninjection are proved; cache hits or hostile decoder coverage are not claimed.

Eight additional fresh import corruptions use the real successful provider and rlib: duplicate/missing proof rows, renamed proof function, changed producer/schema, false per-function exactness, self/stale dependency and wrong trusted-model fingerprint. All reject at the intended importer consistency gate and remove a seeded inference-aware export.

## Receipt-reader audit and sensitivity

Acceptance is the full pipeline, not a standalone `check_certificate` helper: exact typed contracts/scopes/counts, command validation, runtime importer checks, actual artifact hashes, proof statuses and publication are combined. Compiler JSON is descriptive evidence, not itself a proof certificate.

Fresh independent corruptions reject changes to contracts, scope count/rows/source, bool-for-int verified/requested counts, duplicate inferred function rows, missing maximality/safety requirements, and accidental native publication. Actual recorded consumer vectors reject joined extra import/extern flags, duplicate imports, missing extern kind and scope-changing flags. Exactly one distinct `--import` VIR and `--extern` rlib kind is required with the exact provider name/path; actual bytes are hashed. Dependency cardinality, integer crate identity and provider-certificate fingerprint reject independently. These18 corruptions reject identically under normal Python and `-O`.

Disabling one relevant in-memory require guard at a time makes the shipped acceptance suite fail. Per mode, detected subtest-failure counts are: exact contract1; full scope24; verified count5; import-kind/cardinality1; certificate binding3; exact argv5. There are no candidate file edits in these sweeps. Together with the fresh real compiler controls, this establishes meaningful regression sensitivity rather than relying only on self-consistent passing fixtures.

## Declined cases and exploratory harness limitations

The35 independent completed compiler invocations classify as five successful exact programs, four genuine intended wrong-goal failures,17 frontend unsupported results, eight importer rejections, and one Rust privacy error. Raw initial expectation match is32/35, not35/35: the three mismatches are reviewer hypotheses disproved by conservative rejection, not candidate soundness failures. They remain visible in the raw records.

- Correct programs using a rebound mutable slice local, mutable local alias, by-value mutable parameter, projected field tail, expression-block tail, ordinary inert function-returned tail, direct Range field tail, and a reset/reassigned accumulator are outside this bounded profile and decline. These are completeness/frontier observations, not successful semantic controls.
- In particular `reset-local-good` declines before inference, and `reset-local-wrong` also declines; the latter receives **no** wrong-goal proof credit. Neither reset probe supplies seeded-export-removal evidence. The E0603 privacy check was originally expected to be an inference/frontend decline but is correctly classified as a Rust visibility error.
- Callback/custom-element/generic/effectful-call controls do not prove arbitrary callback semantics. Several decline earlier in the bounded frontend, which is recorded in stderr.
- An initial exploratory seven-invocation run stopped in the review harness on a missing `verified` key in an unsupported report; six completed rows plus the seventh raw stdout/stderr are preserved in `probes/`, `independent-probes.json`, `probes.stdout` and `probes.stderr`. The private harness was made tolerant of absent failure-only counters, and the final26+9 probes used fresh directories. This was not a candidate fix. The seven exploratory invocations are not added to the35 primary focused counts.

## Parent-owned integration and external state observations

The sealed current-source discovery logs show453 run/451 pass/two identity-reader failures in both normal and optimized modes. The unchanged reader expects cache77/policy71 while the candidate reserves78/72. These are audited failures, not freshly rerun all-source discovery. Shared-controller final evidence is separately loader-blocked at `@rpath/librustc_driver-451a19fa7a19fc36.dylib` before empty-JSON parsing; it does not establish a reached identity rejection or a passing new-engine shared replay. See the sealed `attempts/python-all-final.stderr`, `python-all-optimized.stderr`, and `attempts/shared-on78-final/transfer-suite-5fip4tqp/redb_page_relations/redb-page-relations-j4c0ahsi/relations_cold.stderr`. Parent owns reconciliation; I did not patch any of it.

Original guarded live-input ledger comparison is **not globally unchanged**. Seven parent-checkout paths already differed from the historical ledger at review start (six candidate Rust source modifications plus the identity reader). Six further live parent paths differed at final read: `source/docs/guide/src/infer.md`, `source/tools/INFER_TRANSFERS.md`, `source/tools/check_redb_slot_selection.py`, `source/tools/check_redb_slot_selection_opaque.py`, `source/tools/test_redb_slot_selection.py`, and `source/tools/test_redb_slot_selection_opaque.py`. Their before/after expected/actual hashes are preserved in `guards-initial.json` and `guards-final.json`. I made no writes to that parent checkout and make no attribution about these observations. The frozen2133-file candidate snapshot and my candidate checkout continue to match exactly; native/runtime/toolchain ledgers remain matched.

Three shared-ref additions were observed, without attribution: `refs/heads/t3/review-cycle79-compatibility` at3b05c1912b9d7314ad00c832c4065f4362562e02; orchestration checkpoint ordinal73 atdd05e934fa714fcf296fef369eed8e5f052e82c2; and another orchestration checkpoint ordinal0 atb2837798b6cb4ea3513f29dd7217a4f2880a75d6. Full ref names and before/after values are in `git-final.json`. Own HEAD/branch/index did not move. No shared-ref authorship is inferred.

## Claims not established

This is detached source-connected selection verification, not a native Fedimint module/application build theorem. The exact helper contents do not establish namespace incomparability. Conditional namespace goals still require the explicit high-level premise; actual variable-length BigSize encodings have no newly proved fixed-length or incomparability property. There is no transaction-wide confinement, storage/backend correctness, durability/fsync, async/consensus, external exactly-once or funds claim. There is no arbitrary Clone/allocator/effectful callback support, universal alias handling, cache-hit claim, or cryptographic authenticity claim for hostile reconstructed interfaces.

## Reproduction and sealed output

Exact review commands and all per-invocation compiler vectors/hashes are retained in `replay-commands.json`, the two fresh replay receipts, both independent compiler JSON ledgers, and the scripts. Reproduction must use a fresh output directory; existing outputs are intentionally not reused as fresh proof evidence.

From an initially clean checkout at the exact HEAD, the reconstruction sequence used was:

```sh
apply_patch < /private/tmp/verus-storage-cycle78.G9sUIt/final-package/combined.apply_patch
python3 -B /private/tmp/verus-cycle79-independent-review.5GYSxK/audit.py before
apply_patch < /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/baseline-relative.apply_patch
python3 -B /private/tmp/verus-cycle79-independent-review.5GYSxK/audit.py after
```

Representative fresh runner command (repeat with `-O` and a different fresh output):

```sh
python3 -B source/tools/check_fedimint_prefix_infer79.py \
  --verus /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78/verus \
  --solver /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78/z3 \
  --application /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin \
  --baseline-verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --output /private/tmp/verus-cycle79-independent-review.5GYSxK/replay
python3 -B -m unittest discover -s source/tools -p test_fedimint_prefix_infer79.py
```

`replay.py` sanitizes the parent environment and records exact commands. Independent probe scripts are `probes.py`, `additional.py` and `receipt_replay.py`; `evidence_audit.py` performs the separate build/vstd/log audit. Raw sources, stdout/stderr and seeded/export hashes are retained, including exploratory failures. `review-receipt.json` is the structured review result. `SHA256SUMS` binds all regular review artifacts except itself and `seal.json`; `seal.json` binds that manifest, this report and the review receipt. Only this private review output is made read-only; no symlink targets or external evidence are chmodded.
