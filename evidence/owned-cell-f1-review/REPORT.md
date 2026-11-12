# Frozen F1 admission review

Recommendation: **request changes to F1 admission** for F1-REVIEW-1 below. Keep the entire native stage inactive. The original focused matrix reproduces, canonical scalar headers and the main positive/negative grammar work, and no default-off regression was found. Passing that matrix does not cover the source-forged internal skip markers independently identified here.

Scope is prepared root `f95473bb8376e9f5ef05885c2d18d276b2363b1b`, implementation `2cb432c6` integrated as `58e74d9b` / `6373f994` / `c43894e2` / `7217d20c`. Early scope/authentication commit: `26682715`. Only this NEW evidence directory is owned. No production changes, recursive delegates, continuing native-body implementation inspection, sibling/live-parent reads, shared mutable build/cache/output, push or PR occurred. Python imports initially generated three untracked bytecode files in source-tool directories; those were removed, further bytecode writes disabled, and final status confirms no outside changes.

## F1-REVIEW-1 — admission counts frontend-skipped bodies as checked local functions

Priority: **P1 for F1 acceptance**, not a current native proof exploit. The exact later-phase inactive refusal continues to prevent every requested proof. This finding blocks the claimed complete checked-local-unit admission prerequisite.

Location: `source/rust_verify/src/checked_owned_cell_admission.rs:637` (`checked_item`, exclusion predicate at 649–655). It rejects external/trusted markers but omits `VerifierAttrs.internal_const_body` and `internal_reveal_fn`. `source/rust_verify/src/rust_to_vir.rs:67` and `:70` explicitly return without translating either marked item. These are not merely unusual source names: the markers are parsed by `attributes.rs` into the actual skip flags.

Minimal native witness (also saved verbatim in `followup/forged_const_body_cell-ordinary/input.rs`):

```rust
use vstd::prelude::*;
#[verifier::verify]
#[verus::internal(const_body)]
pub fn skipped(v: u64) -> u64 {
    core::cell::UnsafeCell::new(v).into_inner()
}
```

Invoke the frozen authenticated verifier with `--crate-type=lib --edition=2024 --no-cheating --compile --experimental-owned-unsafe-cell --output-json` (full argv in receipts). **Expected:** specific skipped-local admission rejection before the later refusal. **Observed in ordinary and Infer:** `inactive: F1 admission passed; functions=1, bodies=1, headers=0; ...`. Replacing `const_body` with `reveal_fn` has the same result. A marker on an ancestor module also passes F1 in the follow-up probe.

The stronger false-contract witness is saved in `confirmation/const_body_false-False-True/input.rs`:

```rust
use vstd::prelude::*;
#[verifier::verify]
#[verus::internal(const_body)]
pub fn skipped(v: u64) -> u64 {
    verus_builtin::ensures(|r: u64| [
        verus_builtin::constrain_type(r, skipped(v)), false
    ]);
    v
}
```

With the feature on, both modes admit this unit and reach the explicit later inactive error (`functions=1, bodies=2, headers=1`). False predicates alone are legitimate admission inputs; the defect is that the function is marked for omission from subsequent ordinary body checking. To establish that distinction independently, the same frozen runtime with the feature **off** succeeds with `verified=0`, empty `func-details`, and the native inventory reports `body_status="selected_item_without_translated_body"`. The `reveal_fn` variant does too. Removing the skip marker makes the control fail specifically with `postcondition not satisfied`. The native-method witness with the marker also succeeds default-off with zero bodies, while its unmarked control fails the ordinary unsupported-native frontend. These are existing default-off skip semantics, not a newly introduced default-off regression.

Minimal fix: reject both internal skip flags on the item **and every local ancestor** in `checked_item`, without granting source attributes authority or modifying default-off semantics. Add ordinary/Infer F1 negatives for both markers and the ancestor case, asserting an early skip-specific reason, no later-admission message, and zero inventory/export credit. Keep the unmarked false-contract and canonical-header positives to distinguish ordinary proof failure from admission. Reconcile the admitted-function inventory with the actual future translation inventory before any activation. No fix is implemented in this review.

## Source and interface audit

Read all 906 admission lines, the caller and inactive consumer, native preflight/test source, exact constrain-type factoring, relevant builtin declarations and normal skipped-item lowering, the two test files, `F1_REPORT.md`, `F1_RESULTS.json`, `F1_INTERFACE.md`, `f1.py`, `check.py`, and existing execution/guard helpers. Required GOAL/CURRENT/WORKER_BRIEFS, ACCEPTANCE_REFINEMENTS and design IMPLEMENTATION_TARGET/ACCEPTANCE were read. No exact ancestor AGENTS.md was present.

* `verifier.rs:3525` calls `check_request` only under the explicit flag, after selected-output invalidation and before import loading, VIR translation, pruning and Infer reuse. `checked_owned_cell.rs:345` obtains an `AdmittedUnit`; its sole consumer returns the precise later inactive refusal. There is no descriptor, private field permission, checked native Function, proof or export authority in this interface.
* Type admission normalizes aliases, requires actual native `UnsafeCell<u64>` identity, admits bool/u64/unit and recursive scalar tuples, direct C, &mut C and &mut u64, and rejects cell tuples, shared/raw/function/closure/generic/user ADT/container types and needs-drop cases. A separate source-type visitor catches qualified associated projections before normalization can conceal them. Independent positive reexports/chained aliases and negative unused aliases exercise this distinction.
* Whole-crate definitions/body owners/nested bodies are inventoried independently of ordinary selection. Function signatures, source types, params/patterns/locals, expression types and adjustments are visited; unknown expressions and body owners fail closed. ADTs, traits, foreign blocks, statics, opaques, nested anonymous consts and executable closures reject. Shared promotions cannot pass via a scalar-looking binding. Scalar checked consts can pass when written as verified raw items; canonical macro const expansions currently conservatively reject at the external/skipped marker. That limitation is reported, not counted as a grammar escape.
* Direct local/native call resolution checks actual DefId/Instance/substitutions. The local function/const graph rejects cycles, including spec recursion and dead-branch recursion; only the validated header type-only call is omitted. Imported concrete/abstract/dyn/generic/projection controls are rejected, and a minimal imported shared mutator is actually compiled by the original focused test.
* Headers identify the nonlocal loaded builtin crate via the real diagnostic anchor and builtin map. Ensures/requires/constrain_type check applicable safe Rust ABI, arity and result shape. The exact bool-array/result-binding/original-parameter self-call shape is checked before excluding the type-only subtree. Real predicate expressions, scalar helpers and quantifier binders/bodies/captures are scanned, rather than treating an entire closure/span/macro as erased. Adversarial real cell captures, scalar-reference binders, altered self arguments, repeated constrain_type calls and extra closure statements reject before later refusal. Scalar header helpers, quantifiers, recursive scalar tuples, actual builtin reexports and move scalar captures reach the precise later refusal. No admitted real cell predicate was observed.
* `fn_call_to_vir.rs:2587` factoring preserves the old map lookup exactly (`Context::get_verus_item` is that same `id_to_name.get`), the old syntactic path condition, the erasure record and filter polarity. Admission separately uses the actual builtin identity and canonical shape; it does **not** call this factored helper despite the worker report's wording. No semantic dependency on that wording was assumed. Default-off compatibility is checked independently below.

HIR typing/admission does not establish normal mode/lifetime/safety/body queries, or emitted/native correspondence. Those remain future gates, especially for admitted but ordinarily invalid false contracts or arithmetic. A missing frontend skip exclusion cannot be excused by promising later mode checks.

## Independent execution and inventories

`replay.py` uses existing `check.setup`, `infer_experiment.LoggedCommands/replay_receipt`, and `infer_eval_support.ArtifactGuard/digest`. It builds only a small standalone std-based harness with authenticated offline serde_json/tempfile dependencies. Original test bodies/oracles are unchanged; only the unused rustc_private feature, include path and minimal `InferCrates` path/tempdir shim differ. `HARNESS.md` describes this equivalence. Private launcher wrappers retain every compiler argv/stdout/stderr/source using the existing receipt helper. No broad verifier or vstd rebuild.

* All **4 original F1 groups passed: 116 invocations**. This includes 9 positive families in both modes, 2 additional frozen repeats, 35 typed negative families, 8 macro/header/skip negatives and 5 imported negatives in both modes. Frozen trio inventory is exactly 3 functions / 6 bodies including headers / 3 canonical headers. Every original F1 oracle checks precise later-positive versus earlier-negative diagnostics and failure JSON with empty function/helper lists, requested/generated=0, checked exec/proof=0, interface_written=false.
* All **5 inherited groups passed: 32 invocations**. Default-off true/false scalar proofs, full-check/partial-selection refusals, repeated native inactivity, seeded explicit exports, Cargo outputs and parse-failure invalidation are retained. There is also one supporting native rustc compile of the imported fixture, asserted by the original test (not a native proof or separate timed receipt). All 148 verifier calls have separate `runs/invocation-*/receipt.json`; focused test result and outer guarded receipt are in `focused-2/`.
* `independent-2/` contains 42 independent families ×2 modes = **84 invocations**. 80 met their original classification oracle. Two scalar-const cases were overly optimistic positives but correctly refused skipped macro scaffolding; two nested-const fixtures lacked explicit `'static` and failed Rust typing, and are **excluded** from typed-negative credit.
* `followup/` contains **18 invocations**, including the corrected shared-const fixture, raw checked scalar const positive, conservative macro-const refusal and both skip markers/ancestor witness. Its unexpected entries preserve the discovered admitted skip cases and the initially optimistic spec-const classification. These are findings/limitations, not silently relabeled passes.
* `confirmation/` contains **24 invocations**: 6 source families ×2 modes ×feature on/off. It independently demonstrates both skipped false-contract/native bodies and unmarked postcondition/native controls. All feature-on runs have zero proof/helper/interface credit. These runs validate the finding's cause; they do not certify a native body.
* `compatibility/` compares default-off behavior of private authenticated preflight and F1 bundles on 8 families ×2 modes, 32 invocations. All 16 comparisons match after removing only per-function source path labels. The unnormalized receipt preserves those 16 path-only differences; exact normalized fields/results are in `compatibility/normalized.json` and RESULTS.json. It includes original builtin reexports, true/false contracts, scalar spec helpers/requires, nested scalar tuples, scalar quantifiers and arithmetic overflow.

Native preflight checks run on every flagged unit that reaches `check_request`. The R1 mutation harness's **54 MIR +4 signature/region controls were source-inspected but NOT independently rerun here**. Worker execution is historical evidence only. In particular this report claims no new independent MIR-corruption pass, checked native body, native method contract, lifetime proof, state query or original-call erasure check.

## Authentication, costs and exclusions

F1 immutable snapshot `/tmp/owned-cell-F1-2cb432c6-parent-snapshot` receipt SHA256 **51ae6b41380ac4bdd8dd362889a4818df7b352d86aa032d306055377a6b98bf1** authenticated before all eight private copies; rust_verify SHA256 **812e3e94c9994039371b8624dfa9646dd5468242f034be917cd285b6f1c1ad88**. All seven other hashes match the immutable preflight receipt. The private preflight comparator is separately authenticated against receipt **25ccaf84cf2ae40c0e4c84c081ce9924b58ab1eb6257d43828158e1cde1d757b** and all its eight hashes. Complete source/tool/input/receipt SHA256 inventories are in AUTHENTICATION.json, SOURCE_HASHES.json, RESULTS.json and LOG_HASHES.json. Existing Rust1.99.1 and original loaded core/std remain selected. Normal vstd **2257/0 is inherited**, not rebuilt or independently reverified in this review.

Preserved failed attempts: sandbox failed before command execution (`bwrap` loopback permission error); escalated workspace commands were approved. First interface commit lacked git identity and was retried with command-local reviewer identity. `focused/` failed offline because the bootstrap cache contains serde_json 1.0.151, not 1.0.150; `focused-2/` used that authenticated patch version. Initial `independent/` (84 invocations) had an incorrect private VERUS_ROOT and only missing-vstd failures; **none count as admission evidence**. No broad rebuild was used to work around any failure. Cost totals, freeze UTC and conservative active upper bound are in RESULTS.json; nested compiler totals and outer harness wall time are separately reported to avoid double counting. No CPU/dollar meter exists.

F2/F3, native Function/place/read/writeback correspondence, compiler field authority/privacy, final-input-state diagnostics and native erasure support are **ABSENT**, not accepted or tested semantic facts. This is not full stage acceptance. CP1=false, CP2=false, source_coupling_proved=false; full Mutex/atomic Transmute/shared guard/poison/drop, LRUWriteCache, commit/recovery theorem and HMAC/native remain **DEFERRED AND UNACHIEVED** as applicable. Existing finite native traces and the worker's R1 tests are not promoted to source-linked crash proofs.
