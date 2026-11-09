# Infer integration status — November 7, 2026

This is a working integration map, not an acceptance certificate. Consolidation
started at HEAD `b32846b61dcf724618f7345e219e5007a0c79643` with substantial inherited,
uncommitted implementation, tests and evidence. This summary does not imply that
those changes are committed, merged, reproduced on hosted CI or accepted as a whole.

## Capabilities and limits

- Infer supports documented bounded scalar, collection and record profiles,
  checked cross-crate composition and selected proof assistance. Exact inference
  checks soundness, safe-domain maximality and output exactness; authors still
  choose application intent, domains and abstractions. See the
  [adoption guide](source/docs/guide/src/infer_adoption.md).
- Aleph and hex have separate finite native milestones. Pinned redb and Fedimint
  transfers retain selected original functions or disclosed contiguous blocks.
  Their individual scope/source/trust ledgers determine the claims, not helper
  counts. The [common frontdoor](source/tools/INFER_TRANSFERS.md) has six scenarios;
  later standalone pilots are not silently additional hosted-CI acceptance.
- The default-OFF [resource experiment](source/vir/NATIVE_RESOURCE_IR.md) checks
  hypothetical capability transport through VIR/SST/AIR. It proves neither
  original Rust ownership correspondence nor an exportable native resource
  theorem. Its recorded 204 feature-enabled VIR tests are compiler regressions,
  not application proof coverage.
- All eighteen final [redb property/history cells](evidence/redb-crash-consistency94/GOAL.md)
  remain OPEN. Bounded recovery, encoding and helper proofs do not close native
  ownership, I/O, persistence, errors or logical key/value refinement.

## Current hardening

The newly authorized [consolidation ledger](evidence/infer-consolidation95/README.md)
separates this work from exhausted historical cycles. No refused task is
restarted, primitive axiom installed or certification policy activated.

New regressions exercise actual bincode artifact import/export, private/bodyless
markers, and the actual compiler's marked-import rejection under ordinary,
filtered, no-verify, no-lifetime and build-SST options. The driver checks require
an intended diagnostic rather than a panic and removal of a seeded stale direct
export. The ordinary-import positive control is plumbing evidence, not a native
application proof. These tests do not inject a marked local Rust body, test every
Cargo driver path, or establish universal internal-VIR exclusion.

Reproduce both feature profiles with a new output directory:

```sh
python3 -B source/tools/check_native_resource_boundaries.py \
  --output /private/tmp/new-native-resource-boundary-replay
```

The runner builds a matching driver and builtin library offline with a locked
dependency graph, then runs the named tests including the normally ignored
driver test. It uses a private build directory, two Cargo jobs, raw logs and
selected-input hashes; another command is refused below 16 GiB free. Dependencies
and the pinned Rust components must already be installed. The receipt is not
complete compiler/runtime provenance or an application proof certificate. This
matrix is not yet a hosted-CI acceptance gate.

Two fresh-target replays each passed all five boundary tests in both profiles,
including the actual driver matrix. The broader Python tooling suite ran
622 tests: 621 passed and one was skipped, after replacing a vanished-checkout unit dependency
with a clearly synthetic extraction fixture. Production Fedimint source guards
remain strict. Fresh [independent implementation review](evidence/infer-consolidation95/INDEPENDENT-REVIEW.md)
gives qualified support with no blocking defect in the narrow patch; it does not
accept the inherited dirty tree wholesale. The sealed reviewed inputs and raw
logs are retained with the consolidation ledger.

A later [test-only boundary follow-up](evidence/infer-boundary-followup96/README.md)
strengthens paired driver positives, leading ordinary siblings and real
malformed-metadata precedence controls. All five tests pass in both feature
profiles; its own [independent review](evidence/infer-boundary-followup96/INDEPENDENT-REVIEW.md)
gives qualified support with no blocking defect. The controls match options,
not marker-only identical artifact structure; some stronger observations remain
log-only, and runner/provenance limits persist. Its exact 273-file snapshot,
review and raw logs are retained separately from hardening95. Neither review
closes full F1 or native correspondence.

A subsequent [runner-oracle correction](evidence/infer-boundary-oracle97/README.md)
requires each exact named test to finish successfully instead of accepting name
mentions plus aggregate counts. Eight Python tests pass; re-evaluating ten
retained logs preserves all seven successes and rejects all three failures.
No new Cargo replay is claimed. This small parent correction is separate from
the frozen95/96 reviews; remaining structural-pairing and provenance limits
are unchanged.

## Next gates

1. Retain the reviewed narrow patch and its failed runs, remaining
   local/lowerer/Cargo-driver gaps and historical reports. Strengthen those seams
   when an application-led need justifies another bounded implementation slice.
2. Use parallel Fedimint/hex assessments to select source-connected goals, freeze
   a current-engine baseline and remove one reusable blocker with matched
   semantic negatives. Do not shrink requested input coverage to obtain success.
   The [frozen-runtime hex encoding experiment](evidence/hex-canonical-encoding95/README.md)
   stopped at an opaque-Table visibility error before any body proof. The user
   subsequently authorized iterative visibility-respecting specification repair,
   completed in a [separate candidate cycle](evidence/hex-privacy-repair96/README.md)
   without assuming a table equation or changing executable privacy. Positives
   verify 48 obligations and retain all 32 historical executable bodies; the
   Lower-to-UPPER mutant fails semantically. Its [independent review](evidence/hex-privacy-repair96/INDEPENDENT-REVIEW.md)
   supports that bounded evidence and individually confirms all 32 historical
   bodies, while finding the name-set preservation oracle too weak for future
   reuse. A [separate identity/span-multiset audit](evidence/hex-body-preservation98/README.md)
   now checks all 96 body matches across three positives and passes 23 metadata
   tests; the parent replay agrees. This is not new native verification or an
   independently reviewed product integration.
   Same-directory replay rechecks proofs but does not show a persisted cache hit;
   the strict warm-cache gate remains unmet, not waived. The [source diagnosis](evidence/hex-cache-diagnosis99/README.md)
   confirms that owned summaries deliberately bypass scalar persistence; a new
   restricted codec/key design is required before any qualified cache experiment.
   No cache implementation is adopted. The original failure remains unchanged,
   and no parent application source is integrated.
   Fedimint clean-source recovery remains a separate prerequisite.
3. For redb, target actual native correspondence/caller premises, not merely more
   abstract helpers. New primitive-law adoption, certification-policy activation
   and changes to the final proof scope require separate explicit decisions.
   The newly provider-blocked padding-boundary task is stopped without retry or
   rerouting; no usable result or new proof credit is claimed.
   The [leaf partition validation follow-up](evidence/redb-leaf-validation100/README.md)
   now has an independently reviewed 14-line ghost-only repair. All three original
   functions pass release and a restored repeat with both partition assertions
   unchanged. The fresh partition-specific comparator negative fails only its
   intended assertion without export, between passing identical-source positives.
   Native bodies, input domain and inherited conditional trust are unchanged.
   Original metadata-exception tracebacks were not retained; the review's scratch
   reproductions are explicitly not historical logs. The failed original run is
   not reset or relabelled, and no final cell closes.
   A separate [native admission101 candidate](evidence/redb-leaf-admission101/README.md)
   checks actual leaf count/capacity predicates against the existing conditional
   builder/lookup premises. Both native helper bodies are byte-identical; an
   explicitly authored caller derives count/capacity instead of assuming them.
   Full and identical restored positives report 8 verified and 6 checked
   executable bodies; three intended scalar negatives fail locally. Nine fresh
   applications include three ordinary correction attempts. Fresh source/evidence
   [review](evidence/redb-leaf-admission101/REVIEW-STATUS.md) finds no new blocking
   mismatch for the conditional bridge; it is not proof replay or native/final
   acceptance. Original native caller, allocator, ownership and I/O remain open.
   The [branch-to-leaf composition102 task](evidence/redb-branch-leaf-composition102/README.md)
   checks a generic completeness/exact-value theorem over supplied child leaves:
   three verified bodies (one executable, two proof), not new native accessor
   coverage. Its sealed handoff is PARTIAL/NOT ACCEPTED: fixed/variable witnesses
   remain unverified, and semantic negatives/restoration were not run. Fresh
   [completion103](evidence/redb-branch-leaf-completion103/PLAN.md) targets those
   gaps without resetting the nine charged attempts/seven launches of102. Native
   child-range establishment and fetched-page identity remain missing dependencies.
   Completion103 has now passed fresh independent conditional review/rechecks:
   both positives38/0, each fixed-fixture negative38/1 only at its intended
   assertion, and seven original generic entities byte-identical. Scope is19
   authored exec plus19 proof bodies, not new imported native-body coverage.
   Its [review](evidence/redb-branch-leaf-completion103/INDEPENDENT-REVIEW.md)
   keeps both native correspondence gaps high severity. The source-only
   [fetch-image assessment104](evidence/redb-native-fetch-design104/README.md)
   selects actual Arc-backed PageImpl view binding rather than another supplied
   array theorem; [gate105](evidence/redb-native-view-gate105/PLAN.md) checks only
   whether the required justified bindings already exist, without adding trust
   or implementing a resource engine. All18 cells remain OPEN.
   Gate105's returned source audit finds native Rust's unsized Arc as_ref body
   but only Sized direct Arc View impls in matched-vstd; no checked native
   resource producer or nominal codec binding is established. No frontend or
   native execution occurred. The [trace106](evidence/redb-native-binding-trace106/PLAN.md)
   checks existing frozen build/source correspondence before making any claim
   about instance resolution. The fetch probe remains unexecuted.

No permission grant, model theorem, generated obligation or test count closes a
final database cell without the required proof and review evidence.
