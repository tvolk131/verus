# Independent bounded scalar-pair review

Input: `0c00b6ecd10eedf9b6e797e3b95edac66e568a6a`, including frozen scope
`cc8aa7a3`, implementation `46630b6f`, merge `0f940711`, word formatting
`2cca4a15` and integration/harness hardening `0c00b6ec`. Compared production
sources with the independently reviewed `db26cfc7` boundary. Child `RESULTS.json`
and reports are historical evidence, not fresh tests performed by this review.

**Recommendation: fix the two small harness validation gaps below, then accept
the frozen pair extension and integration for their stated scope. No engine
correctness, typed-cache acceptance or publication blocker was reproduced.**

## Findings

**P2 — positive pair replay validation does not bind expected inferred functions.**
`source/tools/research/scalar_tuple_output/replay.py:27` calls
`require_exact(info['functions'])` without checking scenario identities or count.
The shared helper deliberately loops over the supplied functions, so an empty
list passes. The portable/architecture cases expect eight inferred functions;
the stateful case expects both original helpers. Dropping either/all report
entries silently removes the check for their three obligations. One arbitrary
passing entry also cannot establish coverage of both original helpers. This is
a report/evidence validation gap, not an observed compiler proof bypass.

Reproduce: `python3 -B evidence/pair-0c00-review/harness_probe.py`, also with
`python3 -B -O`. `positive_empty_inferred_functions` is accepted. Add explicit
scenario-specific identities/counts before calling `require_exact`; allow empty
lists only for the known ordinary-only scenarios. Extend the guard tests with
empty, omitted-helper, duplicate and wrong-helper cases.

**P2 — semantic negatives can be accepted with failed helper proofs or mixed failures.**
`source/tools/research/scalar_tuple_output/replay.py:16` checks the diagnostic
substring and full-crate selection for a semantic negative, but applies no exact
helper-proof check to it. `semantic_negative_failed_helper_obligations` and
`semantic_negative_mixed_precondition_failure` in the same reproduction are
accepted normally and under `-O`. Thus an inference failure can be counted as a
stateful-policy negative, and a postcondition failure accompanied by precondition
failure can pass. The word replay already excludes mixed failures in its
postcondition controls. Check expected helper identities and all mandatory
obligations for ordinary-policy/precondition negatives; reject mixed failure
kinds. Keep the intentionally poisoned-cache negative separate: that case is
supposed to fail helper soundness/exactness/maximality. These checks must live at
the scenario level rather than requiring every negative to prove its helpers.

## Source assessment

- `tuples.rs` restricts the value type to native arity-two tuples of existing
  machine scalars/Boolean. Constructor identity, variant, ordered field names,
  arity and source component types are checked. Projection checks datatype,
  variant, index and variant-check mode. Constructor synthesis clips lifted
  mathematical integers into the source component type through the shared typed
  builder. Clipping is a value operation, not a substitute for safety checking.
- `infer.rs` eagerly computes/conjoins both operands' safety summaries before
  constructing the pair. Local/place projection retains the base safety summary.
  A discarded field cannot discard division/overflow/bounds requirements.
  Exact callees substitute both pair components through existing binder-free
  summary rules, while their domain remains an obligation on the caller.
- `eligibility.rs` admits exact pair returns and immutable locals, explicitly
  excludes pair parameters, and keeps approximate signatures scalar. Nested
  tuples/other arities/enum pair payloads do not acquire an eligibility route.
  No loop, effect, assumption, proof axiom or executable-body rewrite is added.
- `summary.rs`, `simplify.rs` and cache70 decode use the same typed constructors
  and projections. Traversal and expanded-term budgets remain bounded by 4,096;
  projections conservatively retain the constructor term's size. Disk reads
  retain the 2 MiB cap and bounded serde decoding. Malformed candidates may miss
  and regenerate. A valid-shaped cache hit is a candidate, never a proof result.
- `infer_checks.rs` is unchanged by the extension. Maximality drops candidate
  entry preconditions and uses independently lowered original-body safety;
  exactness substitutes a fresh arbitrary typed whole result and compares it
  with the original native tuple return. Missing/weak component clauses cannot
  select the output observations used by the theorem. The new backend precision
  regression uses this actual SST-to-AIR/Z3 route and fixes the return body while
  varying candidate formulas.
- The only production importer change since db26 is policy66 to policy67.
  Schema38 remains the existing representation. Full-body/lifetime/trait checks,
  final publication barrier and conservative standard-library artifact binding
  for ordinary `--infer-goals` providers remain intact. Policy66 imports must be
  rebuilt. Plain no-flag ordinary legacy exports remain outside the inference
  gate and may exist after failed/no verification; that is neither new nor fixed.
- The word proof formatting delta changes whitespace/comments and optional
  trailing commas; it adds no assumptions or proof content. The stateful fixture
  is the exact text delta from frozen `automatic_inferred_mask.rs` enforced by
  fidelity: infer the original tuple helper in place of its authored equations.
  Its two executable helper bodies and stateful domain/semantic goals remain.
  Explicit wf/padding/count/view policies and equivalent canonical word symbols
  are still author supplied. Literal-baseline/forwarding-wrapper failures remain.
- Parent hardening correctly replaces runtime Python assertions, checks fresh
  artifact stability, records commands/failures in a final receipt, requires full
  positive verification scope and checks all three named obligations for every
  reported exact function. The two findings concern coverage and negative
  classification left outside these otherwise useful checks. Word tuple support
  is an explicit opt-in expectation; the default still demands historical
  unsupported rejection, rather than accepting either result.

## Fresh validation

- Own matching compiler/macros/vstd build completes; vstd **2,226 verified,
  zero errors**. All **140 VIR tests** pass, including all **25 precision tests**
  and the new whole-pair fault-injection test. Focused compiler regressions:
  **21/21 pass** (six pair, 12 goal, three word-goal tests). All **seven interface
  tests** pass, including the corrected trusted-model diagnostic assertion and
  old-policy66 rejection. The other 292 tests in the `infer` binary were filtered
  out; this is not a fresh complete compiler regression suite.
- Fresh full pair replay: **48/48 expected outcomes** (18 successes, 30 expected
  rejections). Portable and both conditional architecture fixtures each prove
  25 queries; the stateful two-helper fixture proves **20 queries, zero errors**.
  Its XOR and disabled-goal controls are 19/1 ordinary semantic failures, with
  both helper contracts still fully proved. The separate
  `audit_receipts.py` checks all 48 expected helper identity sets, duplicates,
  every named required obligation for positives and ordinary negatives, and
  excludes mixed failures for the intended semantic controls. Thus this review
  does not depend solely on the validators with the reported gaps.
- The pair replay freshly checks direct and transitive composition, policy66,
  schema/producer/proof-flag/summary/dependency metadata poison, malformed/weak/
  wrong-type/key/dependency cache regeneration, and same-key swapped cache
  rejection through fresh checks. Body/type changes miss the cache. Failing
  ordinary goals and disabled verification remove a seeded physical export.
- Fresh word replay: **18/18 expected outcomes** (nine successes, nine expected
  rejections), explicitly using `--expect-tuple-supported`. The tuple helper is
  now 4/0; inferred-mask stateful proof remains 18/0; the historical baseline and
  forwarding wrapper remain 3/1. Representation equivalence is a separate 1/0
  proof, not an assumption supplied to the application fixture.
- The historical independent combined-boundary probe is freshly repeated with
  policy67 expected and policy66 added to its older-policy attacks: **84 verifier
  invocations**, 46 successes and 38 expected rejections. Its ordinary-only
  zero-candidate providers, full-scope filters, lifetime and trait disabling,
  imported consumers with/without local goal assistance, actual library artifact
  poison, ordinary goal model fingerprints, and stale-export checks all meet
  expectations. Nested declared foreign leaves and combined word/foreign
  providers also remain checked. The original historical probe is unchanged;
  `boundary_probe.py` applies only the two explicit version-list substitutions
  privately and refuses optimized Python. Legacy ordinary early export still
  succeeds without an inference trailer, as explicitly excluded from this gate.
- All eight pair guard tests, eight word guard tests and six shared harness tests
  pass both normally and under `-O` (**44 test executions**). The independent
  three-case validator-gap demonstration is accepted in both modes. Those three
  acceptances are the reproduced findings, not passing evidence checks.
- Fresh native helper execution passes **65,538 cases on this 64-bit host**.
  The word-format token audit confirms identical proof tokens after ignoring
  whitespace, comments and optional trailing commas. Historical source fidelity
  guards pass without changing fixtures or reports.

The final independent extra pair probe completes **37/37 expected invocations**
(26 successes, 11 expected rejections), with unchanged private bundle hashes.
Both false and nonvacuous `x < 254` cache domains are sound/exact but fail
maximality for the original safe `x <= 254` body. An overpermissive domain fails
soundness; swapped results fail soundness and exactness. All rejected runs remove
the physical export, including previously valid seeds. The probes cover every supported scalar width, Boolean
components, mathematical arithmetic lifted back into source types, negative
signed division/remainder, wrapping casts, pair-valued branches and two-level
callee substitution. They also attack discarded fields through calls, branches
and signed MIN/-1, unsupported references/enum payloads/mutation/destructuring,
same-key restrictive and overpermissive domains, swapped outputs, missing fields,
projection-only formulas, wrong types, nested types, invalid field indices,
result-dependent preconditions, oversized/deep JSON and old cache versions.

Two exploratory receipts are retained but excluded from final acceptance. The
first extra probe's helpers all proved, but its two ordinary numerical wrapping
cast assertions failed: ordinary SMT clip axioms provide range/identity facts,
not full modulo arithmetic outside the range. A separate direct ordinary-cast
control reproduces this without inference; the corresponding bit-vector proof
passes. Final caller tests supply proved bit-vector witnesses for those numeric
facts; the inferred helpers are unchanged. Architecture narrowing tests check
the exact source-cast equations, the selected usize maximum and in-range numeric
values, not an unproved numeric wrapping equation. The next exploratory run
correctly rejected a restrictive cache domain but the review script expected the
wrong diagnostic substring; it was corrected to the actual independent
maximality diagnostic. Neither failed trial is an engine finding or a successful
acceptance run. Their raw logs and receipts remain private and hashed.

## Replay and trust limits

Private raw evidence: `/tmp/infer-pair-review-0c00-20261109`, mode 0700. Own
compiler/macros/vstd are built with Rust1.99.1, two cargo jobs, cargo commands from
`source/`, independent build/target paths and a regular copy of standalone Z3.
No parent runtime is a proof input. Production sources, other worktrees and pinned
application checkouts are unchanged; no delegation, push or PR was used.

After a matching private build using `evidence/transfer-fedimint-full-request/build.py`:

```sh
python3 -B evidence/pair-0c00-review/probe.py \
  --verus /private/bundle/verus --solver source/z3 --output /private/new/probes
python3 -B evidence/pair-0c00-review/boundary_probe.py \
  --verus /private/bundle/verus --solver source/z3 --output /private/new/boundaries
python3 -B source/tools/research/scalar_tuple_output/replay.py \
  --verus /private/bundle/verus --output /private/replays
python3 -B source/tools/research/word_goal_automation/replay.py \
  --expect-tuple-supported --verus /private/bundle/verus --output /private/replays
```

Portable proofs, caller preconditions selecting 32/64-bit SMT architectures, and
native 64-bit finite execution are distinct. No global `size_of` declaration,
`--no-lifetime`, `--no-cheating` bypass or architecture assumption supports a
successful proof. Disabled-check flags appear only as expected negative attacks.
The native fixture checks 65,538 cases, not native execution on a 32-bit target.
Imported certificates remain trusted build artifacts with identity/fingerprint
checks, not cryptographic proof objects. Compiler/backend, solver and vstd trust
remain; no whole-compiler, database, crypto or arbitrary-Rust verification is
claimed.
