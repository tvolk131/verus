# Independent LOCAL checked foreign-bool review

Review source: **35bc7b12fff34551da06a9b81e65f1f20d530992**. Scope milestone:
`d62a2896`. Production unchanged; only this evidence directory is owned. No
subdelegation, push, PR, parent live outputs, sibling checkout/build/process
inspection, shared cache/config/toolchain writes, or security/race experiments.
No applicable AGENTS.md was found in this checkout or its exact ancestors.

## Scope and source audit

The reviewed boundary is a trusted frontend extension for same-crate direct
semantic core `From<bool> for u64`, enabled by `--infer-goals`. It is not native
or provenance correspondence. Recognition resolves the actual method in the
caller TypingEnv, checks the core lang-item/From diagnostic relationship, Item
instance, empty final substitutions, bool/u64 signature and safe Rust ABI.
Every instantiated optimized typed MIR local/block/operation/terminator and
possible drop is inspected. The admitted executable body is exactly
`_0 = Copy(_1) as u64 (IntToInt); Return`. Metadata does not admit extra execution.

`corresponds_vir` separately checks the original MIR against the generated full
ordinary Function: parameter/return types and modes, distinct binders, original
local mapping, Copy read, If branches 1/0, and excluded effects. The ordinary
no-requires functional postcondition remains a proposal. The distinct
BodyBackedForeign function/stub is registered before Query's stub maps. Query
finishes the body and applies ordinary trait demotion, WF, mode, resolution,
read and solver processing. Shape matching and lookup are not body proofs.

The direct-call adapter records the original erasure/runtime call and performs
inherited resolution, argument evaluation, generic and impl-path processing
before selecting the local verification target. It changes no FnDef type
translation or type equation. Private/unreferenced uses are detected before
publication pruning. Complete checks are mandatory; selected exports refuse
before Query and selected stale files are invalidated even on early failures.

Packing proposals contain no application variables or hypotheses. Each fact is
`forall low,high:uW. ranges ==> conclusion`, for unsigned W in 8/16/32/64 and
0<shift<W. The conclusions relate OR/shift packing to mathematical addition and
multiplication, and high-field positivity to the masked packed result. BigInt
constants avoid host overflow. The existing BV backend widens unclipped
arithmetic losslessly; ordinary quantification includes unsigned type ranges.
Constant proposals contain only closed finite unsigned literal/bitwise syntax;
shape discovery only chooses generic theorems, even when its hint is imprecise.
Budgets/width exclusions conservatively omit automation.

The same expression Arc is placed in AssertBitVector with **no requires** and
then Assume. `bv_to_queries` creates isolated local queries with only their
closed expressions; caller conditions and function axioms do not discharge them.
Every generated query enters ordinary proof-error accounting. Main-body success
cannot hide a failed auxiliary query. Proposals are confined to ordinary normal
queries, excluding inferred functions and precision queries; no application
assertion, contract, safety condition, body, manual packing help, model or vstd
assumption is weakened. This automation still trusts the existing arithmetic/BV
translations and solver.

Candidate v4 canonicalizes only proposed expression locations for hashing,
recursively through nested expressions. Installed formula nodes and diagnostic
spans remain intact. Original Function/body, typed formula operators/binders,
configuration and transitive-dependency provenance remain bound. Cache73 and
policy70 are active; schema38 is unchanged. The cache stores candidates, never
proof outcomes. This review's actual identical-input experiment, rather than the
worker's failed pre-root experiment, determines reconciliation acceptance.

## Fresh evidence

All applications retain `--no-cheating`, complete scope and normal checks. The
private compiler/macros and standard vstd are built from these source bytes,
offline/locked, jobs=2, with private CARGO_HOME, target-dir **and build-dir**.
Standard vstd uses its normal driver without `--no-cheating` on inherited
admissions. Setup uses only the named completed worker's checksum-validated crate
archives/sparse index and getopts Git objects, plus the authenticated unchanged
solver. No compiled worker engine is used for fresh proof results.

| Fresh trial | Verified/errors |
|---|---:|
| Full symbolic bool, identical-input cold/warm | 2/0 each |
| False caller, healthy helper; only caller fails | 1/1 |
| Byte-identical bool source, goals off | 0/1 |
| Exact original full-help Item.new cold/warm | 52/0 each |
| Original independent support | 36/0 |
| Standard matching vstd | 2250/0 |
| Full carriers runtime build | 52/0; execution exit 0 |

The full-help source matches both pure source.py/fidelity and the completed
worker bytes. Genuine private NonZero/BorTag/Item carriers, all original runtime
and debug assertions, `u64::from(protected)` and all authored packing help remain.
Emitted native MIR retains the core From call and contains no synthetic helper.
This concrete smoke execution supplies no native theorem.

Full native-owner inventory is 16 exec +1 proof; complete ordinary inventory
contains 39 bodies, including both constructors, checked foreign helper and all
spec/derive/constant helpers. Selected/excluded/declaration-only native owner
paths/spans and every ordinary body hash are retained in RESULTS.json.

Cold/warm argv, environment, source and all eight tools are byte-identical.
Completed outputs were archived before working path reuse. Original typed
bodies and generated helper body/binders/read identities match, as does its
actual ordinary `(check-valid ...)` query. Eq candidate ID is
`ea35cc1148ad4a1557d605eb0e9f34637f359ffb770aa553780076ae0f3f68c5`
on both miss and hit. Requires remains true; soundness, full-domain maximality
and exactness freshly prove on both runs. Each Eq AIR contains three actual
check-valid queries and no packing facts. Worker cold/warm identity is expressly
not counted as a pass.

Focused local5, Infer-goals12, NonZero/Eq3 and constructor13 all pass; all149 VIR
unit tests pass, including both Contract tests. A separate private exact VIR
copy with one appended test also passes nested-expression span invariance and
type/formula/source/dependency distinctions, preserving installed Arc identities.
The test checks identity binding, not well-formedness or a native correspondence.

The 24 explicit/actual --VIA-CARGO wrapper-entry controls and eight incomplete
local refusals are independently replayed. Genuine ordinary legacy VIR seeds
are removed; cap-free ordinary and infer-aware exports each succeed 1/0 through
both explicit export and actual cargo wrapper entries.
Nine additional real wrapper controls cover extra-filename, explicit metadata
filename and single `-o`, each with complete/parse/type outcomes. This exercises
the actual wrapper entry and its selected output calculation; it is not a
cargo-spawn integration claim.

Bounded independent controls additionally cover qualified/trait/alias forms,
caller TypingEnv, evaluated-once side effects and their false assertion,
branch/local scope, private-unused uses, false body/caller precondition,
nearby input/output/generic/custom-trait/FnDef/coercion behavior, finite-width
packing with/without its low-field premise, shift boundaries, signed and nested
constant expressions. Receipts distinguish frontend/type/lifetime errors from
solver failures and successful proofs.

## Findings, exclusions and limits

**No blocking correctness finding. Accept this bounded LOCAL milestone.**
The final receipt audit and exact identities are in RESULTS.json.

Informational finding R1: zero-shift arithmetic automation is incomplete. Minimal
repro: `fn call(b:bool,x:u64)->(r:u64) ensures r==x {let z=u64::from(b);x<<0u64}`.
It fails 1/1 with the healthy helper and 0/1 goals-off on the same source; adding
an ordinary BV assertion proves it at 3/0. This is the deliberate k=0 omission,
not a false success or a defect in the bool body translation. Receipts are
`semantic-shift-zero`, `semantic-shift-zero-off` and
`semantic-shift-zero-ordinary-bv`. No scope requirement was relaxed. No acceptance is inferred from an incomplete run or structural
check. Six intentional source controls are built only in a private Git-byte
fixture; shipping source and tool bundle are untouched. Their exact source
manifests, patches, actual tool hashes, commands and proof receipts are retained.
The false helper proposal fails its ordinary helper postcondition at 1/1; the
false constant/BV proposal fails full verification at 45/7. These distinguish
actual body/proposal checking from correspondence alone.

RESULTS.json lists all setup exclusions separately: sandbox startup failure,
optional whole-workspace archive inventory, reduced private test-workspace lock
mismatch, private build-dir harness ENOENT, unsuffixed spec-literal ambiguity and
an initially incomplete private tools-source fixture. The false-BV diagnostic
classifier initially expected a different error string; it was corrected to the
actual isolated bitvector error on the unchanged archived 45/7 failure. Corrected trials use fresh
receipt paths; failed completed receipts are preserved. No oracle or production
requirement was relaxed. The completed worker's 52 selected proof/control/vstd receipts, source
manifests, mutation bytes and tool bundles authenticate against the explicitly
allowed Git objects/artifacts; historical checkout names are only labels. The
additional final-build receipt and final product identities also authenticate.
Its prebuild snapshot refers to a superseded compiler
`80aa23c06acab915474781da22d1f3c8e2d8d59718f0c6ef9d0c44bea6e9b5b3`
whose bytes are not retained in the allowed bundles. This unused prebuild
snapshot is not a compiler-build input or accepted proof tool. It is recorded
as unavailable, not equated to the authenticated final compiler. The historical
build shell script is hash-only here; the independent build uses the explicit
recorded Cargo invocation instead.

Private root is recorded in PRIVATE_ROOT. RESULTS.json binds exact source/Git,
compiler/macros/vstd/solver and rustc/core/target identities; the executed runner
is archived before setup-only recipe corrections; complete frozen
commands/env/source manifests, raw AIR/MIR, caches and logs remain private.
Replays are in review.py, proofs.py, controls.py, cargo_paths.py, cargo_capfree.py,
extra_semantic.py, shift_controls.py, tests.py, fault_controls.py and audit.py. Only the explicitly
permitted pure source.py/inventory.py readers were imported from earlier work;
no earlier writing controller was executed.

Recommendation is scoped to LOCAL capability only. Trusted compiler/frontend,
whole-MIR translation/correspondence code, ordinary arithmetic/BV encoding,
solver, existing standard models and inherited vstd admissions remain trusted.
No FromSpecImpl<bool>, obeys-from-spec bridge, external body, new std/vstd
assumption, native theorem or exported/imported foreign law is introduced.
The inherited normal-return tag bound and authored views/contracts remain; no
out-of-range/error/unwind or physical permission/provenance theorem follows.
Certified publication/import, current compiler/core/target/model certificates
and import-time revalidation remain open. Full HMAC1/1 remains unproved and
unrerun; canonical parent6/0 is inherited, not rerun; all44+3/101 obligations stay
open. **native=0, publication=false, goal_complete=false.**
