# Independent review of annotation consolidation and complete-scope publication

Review target: `be3b1365e8133c00845aa74c450894c7e3e6fccb` and
`d0d12c6f3297439352c1f837a31c8759642a2bb2`, based on the isolated checkout at
the latter commit. Earlier transfer acceptance and parent runtime results are
context only, not fresh validation here. No production code, parent artifacts,
or other checkout was changed. No delegation, push or PR was used.

## Findings

**Recommendation: accept both commits for the stated scope. No blocking correctness or publication defect was found.**

**P3, documentation only:** `source/docs/guide/src/infer.md:2606` still calls
policy 63/cache 69/schema 38 the “current reconciled boundary,” whereas the
updated opening at line 9 correctly requires policy 64. The opening's historical
version caveat limits the ambiguity, but this later use of “current” should say
“that checkpoint” or be updated to policy 64. Reproduce by searching the guide
for `current reconciled`. This is not an enforcement defect or acceptance
blocker. Parent should make the small text correction; production was left intact.

No new soundness/publication defect was reproduced. The pre-existing ordinary-only
bypass is rejected under policy 64. The known legacy ordinary early-export behavior
is outside this fix; the changed gate does not label those files inference-aware.

## Reviewed boundaries

- `source/rust_verify/src/config.rs:129` includes enabled proof checking and
  every existing selection flag. `verifier.rs:2927` applies that predicate to
  every goal-assisted invocation before setting up pending publication,
  regardless of candidate/profile count. The independent check in
  `import_export.rs:327` guards every call with `Some(verified_inference)`.
  This is a selection predicate, not a claim that verification succeeded;
  `verifier.rs:3599` still requires zero proof/driver/frontend errors before
  calling that exporter.
- `import_export.rs:154` checks policy before the empty-proof ordinary-only
  case can be accepted. Policy 64 invalidates both old inferred providers and
  formerly unchecked ordinary goal-assisted interfaces. Schema 38 still
  describes the unchanged trailer representation. Cache 69 can remain because
  the fix changes publication eligibility, not candidate formulas; candidates
  remain untrusted and undergo fresh mandatory checks. This deliberately
  requires rebuilding inference-aware providers/dependents.
- `verifier.rs:2936` preserves complete checking for consumers of any certified
  dependency, even without `--infer-goals` and without local candidates. Such
  consumers still enter the delayed exporter at line 2992 and carry dependency
  fingerprints transitively. Legacy ordinary imports/early exports retain
  their existing trust and publication behavior; their physical presence is
  not an inference certificate.
- `driver.rs:35` now uses the same predicate. A successful `--no-verify` command
  no longer gets the top-level full-crate `success` key, consistently with
  filtered commands. `infer_report.rs:83` separately retains conservative
  selected-body accounting. This reporting compatibility change is intentional
  and prevents disabled checking from looking like a successful full proof.
- All seven moved bodies (`clauses`, `future`, `seq_len`, `current`, `pure`,
  `place_pure`, `fresh`) were compared byte-for-byte with `be3b1365^`; all are
  identical. Hashes are in the manifest. Growth/range/selector callers only
  change helper paths. Recognizers, normalization, precision checking and the
  source-preserving decoration validators remain separate.
- `pointwise_goals.rs:173` refines the same early exits into reporting outcomes.
  The bounded formulas, order of preparation, fresh-ID allocation and checked
  assertions are preserved. `validate_decoration` at line 342 still projects
  out only one to four pure assertions and compares the complete original
  function, including source statements, tail and contracts.
- `report.rs:258` records the actual attempt, including its error, before
  propagating failure. Earlier decisions survive and later attempts are absent.
  `infer_report.rs:259` sorts the presentation by profile/function; that list
  is not execution order. `EXPLAINED_PROFILES` is shared reporting coverage,
  never an acceptance registry. `applied` is not a proof result and observations
  do not enter proof/cache/publication decisions.

## Fresh validation

Fresh results, all on the fixed d0d12c6f compiler:

- Matching vstd: **2,221 verified, zero errors**.
- **134 VIR library tests** pass, zero failures/ignored; this includes the
  source/contract/assumption/projection attack controls.
- **81 focused integration tests** pass, zero failures/ignored: 14 clone,
  12 goals, 12 growth, 9 range, 8 relational goals, 9 selection, 5 interfaces,
  8 reporting and 4 scope tests. The three `infer` filters are disjoint; the
  other 280 tests in that binary were not run.
- The independent runner completes **84 compiler invocations**, with all
  expectations satisfied: 46 successful commands and 38 expected rejections.
  These include repeated seeds, not 84 distinct application theorems. Every
  rejected command physically removes/withholds its export. The three legacy
  reporting controls intentionally succeed after disabled/filtered checking;
  their scope is false, their top-level full-verification `success` key is
  absent, and their physical legacy exports have no inference trailer.

The scope controls include all selection flags, `--no-verify --build-sst`,
no-export invocations, and empty/spec-only crates. Valid ordinary-only checked
providers have zero inference candidates and an empty proof list in their
policy-64 trailer. Their false-postcondition full rebuild and partial rebuilds
reject. Direct consumers pass with and without `--infer-goals`; partial and
false-property consumers reject, as does a partial transitive consumer. Editing
only `policy` to v63 (also v62), while retaining the same producer/schema/API,
rejects both kinds of consumer and removes seeded consumer interfaces. Thus this
checks policy invalidation independently of an incidental producer mismatch.

Pointwise positive and missing-goal variants prove; false transition, false
output equation and unconditional-invariant variants fail ordinary proof
obligations and never publish. The first two false variants report `applied`;
the unconditional-invariant variant reports `skipped`. `choose` in the annotation
reports a preparation error. In a separate multi-function fixture,
`aaa_before` retains its skip row, `middle_error` records its error, and
`zzz_after` has no invented attempt. No growth/range rows appear after that
pointwise error. Repeated transport explanations compare identically, and the
integration cache tests separately establish fresh required candidate proofs on
cold/warm runs. A frontend type error leaves preparation unstarted, with no
decisions or export; this is an error-path control, not a semantic negative.

All raw output is private and persistent under
`/tmp/verus-parent-review-d0-20261109` (directory mode 0700). Compact source
fixtures and the runner are committed here; binaries and large logs are not.

The environment has no `cargo make`, so the build uses the exact Cargo tasks
from `source/Makefile.toml`, with the in-tree `cargo-verus` driver instead of
installing it globally. All Cargo commands run from `source/`, with
`PATH=/home/dev/.cargo/bin:$PATH` and `CARGO_BUILD_JOBS=2`. The standalone Z3
4.16.0 executable was copied from the specifically authorized path into this
checkout's ignored `source/z3`, never symlinked. Compiler/macros/vstd are built
from this checkout; no changing parent runtime is used. To avoid Cargo's configured
build-directory lock, vstd uses the private `vstd-target` directory with both
`--target-dir` and `CARGO_BUILD_BUILD_DIR` set there. Its five matching library/
macro artifacts are copied back to this checkout's release bundle. Two queued
vstd attempts were stopped before compilation; neither is counted as validation.
Bundle hashes remained unchanged through the final test/probe verification.

To replay the independent compiler probes after building:

```sh
export PATH=/home/dev/.cargo/bin:$PATH
python3 -B evidence/parent-commits-review/probe.py \
  --verus source/target-verus/release/verus --solver source/z3 \
  --output /path/to/new/private/directory
```

## Limits

This review covers these two production commits and their immediate import,
export, annotation and reporting interactions. It does not reaccept historical
transfers, extend the frozen research scope, prove whole applications or rerun
the entire workspace/doctest suite. The ordinary Verus backend, explicitly
assumed specifications, trusted library models and compiler remain trust inputs.
No compile error or timeout is classified as a semantic counterexample. Export
absence is checked physically in addition to report fields.
