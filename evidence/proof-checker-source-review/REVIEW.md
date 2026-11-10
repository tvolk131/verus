# Proof checker source-only review

Recommendation: **changes requested for source-entry snapshot completeness**.
The completed corrected model replay is authenticated and compatible with the
original model results. It does not establish full controller acceptance or a
native theorem. One incomplete fix and two inherited concerns follow, in priority
order. These findings are source reasoning, not executed substitution results.

Reviewed checkout: `d9ad548b5a2ba5255d674a25046c31054e2b965c`.
Comparison: `bb5be3f0` -> `e6ce22cf` -> `aea14e56`. The five requested Python
files at this checkout equal their `aea14e56` versions. The four mathematical
`candidates/loan-v12/*.v` files are unchanged across the comparison.

## Findings

### F1 — P2: authenticate retained model source identities in the inspector

Location: `source/tools/research/native_source_entry/entry_audit.py:137-139`;
related lines 106-115 and 118-126.

The inspector checks each model `.v` against the live loan-v12 candidate at
108-109, before creating `ArtifactGuard` at 115. The new snapshot validation
checks the model `.vo` hashes at 137-139 but never rechecks the model `.v`
hashes against an authenticated source identity in `guard.initial`. Neither
the four model source files nor the loan-v12 candidate sources occur in the
current inspection freeze's `files`/`private_inputs` maps. The candidate sources
are not independently required or guarded either.

Consequently, a persistent model-source change after the equality check and
before the initial snapshot can become the accepted starting source identity.
Subsequent guard checks compare with that changed identity. This leaves the
same validation-to-snapshot defect for these inputs that the refactor fixes for
foundation sources, objects and tools. Normal compilation of `EntryInventory.v`
and recursive checking use the already compiled, separately pinned model
objects, so they need not detect a comment-only model-source change. This is an
inspection provenance defect; it does not demonstrate a false proof term,
new native theorem, or changed completed receipt.

Bind every retained model `.v` SHA256 to the authenticated model receipt's
recorded source identities (with exact module coverage), and compare those
expected identities with this same guard's `initial` entries. Alternatively,
require and authenticate the exact source set in the inspection pretrial
metadata. A live source-to-source comparison before snapshotting is insufficient.
No experiment was performed to exercise this ordering.

### F2 — P2, inherited: require the proof checker's shared support inputs

Location: `source/tools/research/native_semantic_bridge/check.py:41-46`;
related lines 83-93, `infer_eval_support.py:34-60`.

The required pretrial set includes the checker, proof parser, binding files,
foundation metadata and selected proof sources, but omits both shared support
files returned by `support_artifacts()`. They are added to `ArtifactGuard` at
83-84 regardless of freeze contents. The snapshot-to-pretrial comparison at
91-93 only iterates supplied freeze entries. A supplied freeze that omits either
support file therefore gets no pretrial identity authentication for that file;
guarding it only establishes stability from a freshly measured starting state.
These imports implement `ArtifactGuard`, `require`, `LoggedCommands`, and
`replay_receipt`, so they are part of the checker's trust inputs.

Require `*support_artifacts()` independently in the checker’s mandatory set.
The active `full-snapshot-freeze.json` already includes both files, and their
hashes and recorded replay identities match. This is not evidence against that
completed replay. The omission also exists at `bb5be3f0`; it is an inherited
completeness concern explicitly covered by this review, not a new regression.
No incomplete-freeze or file-change test was executed.

### F3 — P2, inherited: finalize proof status after the final guard check

Location: `source/tools/research/native_semantic_bridge/check.py:162-169`;
related `source/tools/infer_experiment.py:32-45`.

The checker sets `model_proofs_kernel_checked=True` at 162, then calls the
input guard at 169. If that final check raises `ArtifactChanged`,
`replay_receipt` persists `success=False` and the failure but does not reset the
already true proof-status flag. This produces an inconsistent failed receipt
for a rejected input-identity evaluation. Publication and native proof status
remain false, and the inspector also requires `success=True`, so its consumer
does not accept this failed receipt. Nevertheless, the requested invariant that
error paths retain false proof status is not satisfied.

Run every final validation before setting the proof-status flag, or explicitly
clear it on failed receipt finalization. This ordering already exists at
`bb5be3f0`; the new copied-source guard checks correctly precede the update,
but the original final input guard remains after it. No late-failure experiment
was executed.

## What the refactor correctly establishes by inspection

- `source_map.py:36-46` retains the loaded freeze bytes, creates one input guard,
  authenticates its freeze hash and mandatory seven-file source-audit set using
  that guard's initial identities, and returns that same guard. Both source-map
  and audit entry points retain it through their final check. Their extraction
  authentication occurs after guard creation; persistent later changes are
  rejected by final stability checking. Source eligibility is not promoted to
  semantic proof.
- `check.py:89-114` authenticates the retained snapshot against loaded freeze
  bytes, all supplied frozen files, pinned ready/baseline receipts, the baseline
  source inventory, foundation loadpaths, the 284-object map and four tool
  identities. `aea14e56` adds **every** copied source, including documentation,
  to the guard at 82. This resolves the earlier missing-entry setup failure
  caused by retaining only `.v`/`.vo` paths.
- `check.py:132-161` authenticates each copied candidate `.v` immediately after
  copying, records a source guard before compilation, and checks all these
  guards after recursive kernel checking. The generated assumption inventory
  is similarly bound to the generated text. The inspector’s own copied
  `EntryInventory.v` is bound before compilation and checked after the kernel
  command at `entry_audit.py:159-164,190`.
- `freeze.py` includes the mandatory source-audit inputs and foundation pin.
  The inspector independently requires public/private input coverage and
  authenticates loaded freeze bytes, supplied public/private identities,
  baseline source/object/tool identities and foundation loadpaths against the
  retained guard. F1 is the missing model-source comparison in that work.
- Explicit `require` checks survive optimized Python. Command/setup failures
  propagate to failed receipts. `audit.py` labels source controls as eligibility
  rejections and keeps `semantic_negative_count=0`. No reviewed code publishes
  native facts. F3 is the specific remaining model-status exception.

## Independently authenticated completed evidence

Read only the completed model result under
`/tmp/verus-infer-buffer-cycle-20261109/full-snapshot-validation-aea14e56/proofs/`.
Its `receipt.json` SHA256 is exactly
`46e35308c297fd6c5eb9ce0fe94b80f86b094d4c6f59493962dee5eb8e34d562`,
matching the task's independent expected identity.

The receipt records version, all four normal compilation commands, assumption
inventory compilation and recursive kernel checking with exit 0. Its success
and model-kernel status are true; publication, native bridge, source connection
and original-array future import are false, and discharged native effects are
zero. All ten completed `.v`/`.vo` artifact hashes match their recorded values.
The four model source hashes match this checkout and the original source hashes
in `parent-semantic-foundation.json`. The four proof-object hashes match that
record's original accepted objects exactly.

Independently parsed the completed assumption stdout and kernel stderr with
the unchanged pure parsers. Both conclusions equal the authenticated receipt:
25 constants covering 18 named theorems, six byte obligations and `byte_ty`;
only dependent functional extensionality for the permitted specialized byte
statements; recursive summary lists functional extensionality and Eq_rect_eq
with no unsafe fixpoints, assumed positivity, type-in-type or rewrite rules.
The generated inventory source matches `inventory_source()` exactly.

All 23 public file hashes in the current model freeze and all 15 public file
hashes in the current inspection freeze match this checkout. The completed model
receipt's recorded public input identities match the current model freeze.
Recorded parent checkout paths were treated as receipt data only; that checkout
was not opened or inspected. Command-output hashes and detailed conclusions are
retained in [checks.json](checks.json). The logs were read from completed model
output files; their exit codes are authenticated receipt claims, not commands
rerun by this reviewer.

## Checks actually executed and review limits

Executed only five in-memory `ast.parse` syntax checks, immutable file/hash and
pure parser comparisons, and the six explicitly named `Assumptions` tests from
`test_proof_checks.py` in each of normal and optimized Python: **6/6 + 6/6 pass**.
See [normal](assumptions-normal.log), [optimized](assumptions-optimized.log), and
[checks.json](checks.json) for exact test names, commands and hashes. `-B` was
used; no bytecode or source-under-test changes were made.

Read both test source files and the source/guard/gate/race-controller sources.
Did not run `SourceBindings`, `test_source_freeze.py`, `audit.py`, source/race
controllers, or any filesystem concurrency, mutation, substitution or race
experiment. Did not rerun Coq/Rust builds, install/rebuild/copy/execute pinned
tools, inspect other worker checkouts, poll/read parent job handles, or read the
enclosing batch's incomplete inspection/control results. The previous broader
review's automatic rejection supplies no acceptance; its action was not retried.
Only this new evidence directory was written. No implementation, shared
toolchain, push or PR changes were made. This is **source-only review**, not
dynamic validation of the reported ordering scenarios or full-batch acceptance.

Read the current bridge/source-entry reports, goal charter, all three requested
parent records and dependency pins. Historical receipt claims outside the
completed model result above were read as documentation and are not newly
validated tests. The active model metadata is `full-snapshot-freeze.json` and
the active inspection metadata is `entry-full-snapshot-freeze.json`; older
files remain historical records. No fresh inspection receipt was assumed.

## Preserved goal limits

Canonical varint remains the inherited **6/0** result for every `u64` and
arbitrary initialized `[u8;10]`, with authored semantic definitions/invariants
only and no local mechanical proof blocks, calls or snapshots. The complete
frozen bitcoin_hashes 0.14 pad-XOR **prefix AND untouched suffix** remains
**1/1, unproved**, for arbitrary initialized `[u8;128]` and all valid keys,
including length 0 and >128. The observer is redundant for canonical encoding
and necessary as a group for prefix only; no genuine common checked step
across the full properties is confirmed.

The checked 18+6 model obligations do not establish actual native input/source
typing, provenance or C/F correspondence. A 42-constant existing-rule inspection
introduces no theorem. All **44 minimal native simulations**, three additional
phase obligations and **101 full-writer effects** remain open. Publication and
goal completion remain **false**. No new contracts, premises, source/domain
changes or native facts are authorized or supplied. The separate model-heap
initialization work was not inspected.
