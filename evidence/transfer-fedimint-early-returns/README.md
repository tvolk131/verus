# Readonly record early returns: Fedimint transfer extension

This extends the frozen gateway transfer experiment at engine base
`55bd67ad980e90e92c77efe4453a927f097c3386`. It does not modify that experiment,
its receipts, its application clone, or its original observations. No commit or
PR is created. The assigned branch is `t3/infer-record-early-returns`, based on
`verus-infer`; HEAD, origin/verus-infer and the common repository's FETCH_HEAD
all name that base. The worktree was clean before editing. No local Verus
AGENTS.md was present; the inference guide, direction, architecture and
contribution guidance were read. The application clone's AGENTS.md and its three
relevant project skills were read; the frozen report records the missing
linked-specs/multipart-review resources. No governed Fedimint code is edited.

## Production change

The record interpreter accumulates each executed return's active guard and
value. Branches and match arms merge normal values only from continuing paths;
the final result merges those normal values with the captured returns. Returned
paths become inactive before later requirements are generated. Requirements of
the return expression itself and of match guards remain guarded by the path on
which they execute. A never-to-any coercion carries control flow through this
traversal without inventing an executable value.

The complete application replay also needs the existing canonical primitive
saturating-arithmetic model in the readonly interpreter. This reuses the engine's
identity/type/proxy checks and value formulas; no arithmetic or byte model or vstd
specification is changed. Readonly value types, mutable unit results, source and
formula budgets, loop/effect restrictions, dependency keys, import validation,
ordinary verification, all three exact checks and publication gating remain.
The executable body stays unchanged; no unchecked summary, new axiom, assumed
precondition or application identity is introduced. A single empty accumulator
initializer in `source/vir/src/infer/owned.rs` is the only shared-file overlap.

## Application and intent

Application pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
The already frozen source is cloned into a separate persistent scratch
directory. The runner extracts the same request projection and methods and preserves the
four previously audited combinator/Option/tuple-constant desugarings. It requires
the complete inferred application candidate to equal the frozen
`record_infer-proof.rs` byte for byte, and the desugared executable slice to
equal the frozen `lowered.rs` byte for byte.

Both complete methods are inference candidates. An ordinary `deadline_policy`
wrapper supplies the same three policy/coverage postconditions, without any
precondition. Its domain remains **every u64 channel identifier, every u64 HTLC
identifier, every u32 expiry, and None or every u32 current height**:

1. Either nonzero circuit identifier implies zero budget.
2. Unknown height implies zero; budget at least two with known height implies
   mathematical `expiry >= height + 2`.
3. Zero circuit identifiers and a known height with that mathematical margin
   imply budget at least two.

The implementation relation is inferred; the intended policy remains authored.
The ordinary deadline implementation contract from the frozen mixed proof is
removed. The authored wrapper also checks that inferred call requirements hold
over the entire intended input domain.

The faithful mutant deletes only the intercepted-forward early-return guard.
An exact inferred description of that mutant is allowed. Semantic detection
requires a failed **authored policy postcondition**, successful required exact
checks for both helpers, no precondition failure, no frontend error, no timeout,
no published interface and deletion of a seeded stale export. False and malformed
goals are separate controls; malformed syntax/types are never counted as
semantic detection.

## Observed results

Authoritative replay:
`/private/tmp/verus-fedimint-early-returns-jyfO2M/replays/run-qbyeu5vi/receipt.json`.
The frozen fully inferred candidate that failed at the early return now verifies:
**8 verified, 0 errors**, with fresh soundness/maximality/exactness for both
methods. Its SHA256 remains
`0b53afe22496c2aaed368e2b7a075ae268bd90ef18af51d7eb32cff87510604a`.
The classifier precondition displays as true; the deadline precondition contains
exhaustive Option-tag tautologies. The unrestricted policy wrapper proves those
requirements for the entire stated input domain. No authored input restriction
is added to make inference succeed.

Cold and warm runs both pass all required proofs and write fresh interfaces.
Both candidates report `miss`, `regenerated_state`; there is no proof-cache hit.
The guard-deletion mutant has **7 verified, 1 postcondition error**, with both
helper relations still exactly proved. Its sole error names the first authored
policy clause. The false-goal control also has one semantic postcondition error.
Both remove seeded stale exports. The malformed-goal control compares u32 with
bool, fails type checking, and also removes its stale export; this is separately
classified as frontend rejection. Original and partially desugared bodies retain
their original frontend/profile blockers.

Native execution compares **2,112 inputs** successfully. The preserved witness
has channel 101, HTLC 0, expiry 1000 and height 990: fixed budget 0, guard-deletion
budget 10. The independent native checked-u8 oracle covers **196,608 cases**.

Validation passes: 7 focused integration tests; 82 adjacent integration tests
(`infer_records` 21, `infer_wrappers` 18, `infer_selection` 9,
`infer_error_values` 10, `infer_wrapper_traits` 16, `infer_traversals` 8);
85 inference unit tests; 4 replay unit tests normally and under Python `-O`;
6 shared harness tests; rustfmt and `git diff --check`. The full verified vstd
rebuild has 2,219 verified, zero errors. Raw commands and outputs, including
development-only failures corrected before these final runs, remain in scratch.

## Build, replay and validation

Persistent scratch: `/private/tmp/verus-fedimint-early-returns-jyfO2M`.
Its `build.py` records bounded builds into its own `target-verus`, using two
jobs. It rebuilds the matching compiler/driver, explicitly cleans and rebuilds
builtin/macros in that target, and verifies/rebuilds vstd. The source ledger is
guarded during the build. Its `test.py` records bounded compiler/unit/integration
commands. Large binaries, native controls, caches, clones and raw output stay
in scratch. The copied prior target is an independent copy-on-write seed, with
no shared mutable target or candidate cache.

The replay command, from this worktree, is:

```sh
python3 source/tools/check_fedimint_early_returns.py \
  --application /private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint \
  --verus /private/tmp/verus-fedimint-early-returns-jyfO2M/target-verus/release/verus \
  --solver /private/tmp/verus-fedimint-early-returns-jyfO2M/z3 \
  --output /private/tmp/verus-fedimint-early-returns-jyfO2M/replays \
  --baseline /Users/tommyvolk/.t3/worktrees/verus/t3-f129db18/evidence/transfer-fedimint-gateway \
  --original-receipt /private/tmp/verus-fedimint-gateway-vq13a0t6/replays/run-calw54ea/receipt.json
```

Each verifier command uses `--no-cheating --output-json --num-threads 1
--export`, a dedicated candidate cache and a 120-second process-group bound.
Cold/warm runs must rerun soundness, maximality and exactness; regenerated record
candidates are reported as such. Source/tool/artifact fingerprints are checked
around commands. Concurrent timings are observations, not benchmarks.

`manifest.json` records actual final outcomes, exact source and artifact identities,
commands, retained receipts, the original-versus-updated comparison and limitations.
The original source SHA alone does not identify this dirty build: the production
patch, per-file source hashes and built-artifact hashes are recorded separately.

The new standalone `infer_record_returns` suite checks nested and multiple
returns, match and normal tail paths, scalar and Option values, generic shared
Option payloads, the existing borrowed Result profile, unit-return effects,
unreachable arithmetic/index/call requirements, required match-guard/return/tail
safety, imported summaries and false callers. A native Rust unit oracle compares
the record interpreter's domain and value formulas with checked-u8 execution over
all 65,536 input pairs for each of three control-flow shapes. It does not consult
production acceptance logic. The application native differential control compares
the untouched and desugared methods over the original finite input grid.

## Trust and remaining limits

Trust remains ordinary Rust/Verus/SMT and the bundled library specifications,
including canonical unsigned saturating subtraction. The new record use of an
existing arithmetic model is an explicit dependency, not a newly supplied law.
Source auditing and finite native tests connect the desugarings to std
combinators; this is not a formal proof of those unsupported original APIs.

Untouched bodies remain blocked by then_some/map_or, Option::is_some and tuple
constants. Returns embedded inside otherwise readonly operands/constructor
arguments and unsupported effects, mutation, loops and value types remain outside
this bounded profile. The Result regression uses the already supported borrowed
result path; this does not add arbitrary Result/value-type support to records.

**Untouched native gateway and whole-flow safety remain unproved.** No DB,
asynchronous execution, payment authenticity, ownership/exactly-once funding,
LND sentinel validity, consensus, crypto, recovery or live service behavior is
proved here. The result concerns the same extracted/refactored stateless decision
boundary as the frozen experiment. It is not a whole-protocol security claim.
