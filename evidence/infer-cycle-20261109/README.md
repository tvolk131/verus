# Infer five-milestone cycle

This record covers the locally integrated transfer, consolidation, fresh-property,
automation and stateful milestones requested on `verus-infer`. It does not claim
general Rust coverage or a whole-database, gateway or consensus proof.

**The five scoped milestones are complete.** The final parent regression and
replay batches pass. [manifest.json](manifest.json) binds the exact runtime,
source revisions, tool hashes, commands/logs and raw receipts. Nothing was pushed.

The runtime was built at `0c00b6ec` (policy 67/cache 70/schema 38). Subsequent
changes through `04f53748` strengthen Python evidence validation; the Rust
production/library sources are unchanged. Matching compiler/macros/vstd and
providers were rebuilt together.

## Changes and evidence

- [Current entry points](../../VERUS_INFER.md) replace the accumulated RFC/status journals with concise
  adoption, direction and assurance pages. Immutable Git versions preserve the
  original text. The capability map points readers to applicable profiles and
  their remaining authored inputs; historical milestone evidence is unchanged.
  The three active entry documents remove 4,078 lines from the pre-consolidation
  checkpoint; this is a context reduction, not deletion of frozen evidence.
- Shared annotation utilities replace duplicated assistance helpers. Pointwise
  clone transport now explains applied, skipped and preparation-error outcomes.
  Diagnostics describe proposals; they never authorize proof acceptance.
- Complete body, lifetime and trait checking are required for inference-aware
  certification, including ordinary-only `--infer-goals` providers. Failed or
  incomplete checking removes stale inference-aware exports. Goal-assisted
  providers bind library model artifacts. Policy 67/cache 70 require matching
  rebuilt providers and dependents; the metadata schema remains 38.
- Checked word-view assistance selects reusable ordinary vstd laws for supplied
  generic representation vocabulary. The original single-word bitmap transition
  proves its authored invariant without local ghost snapshots, proof blocks or
  theorem calls. Exact scalar-pair output inference also removes both original
  helper contracts without changing their executable bodies. Unrelated
  permission/count goals verify with the same word-law selection.
- The full Fedimint request transfer retains all seven fields and the real
  SHA-256 type through an explicit opaque declaration. Its unchanged deadline
  and circuit helpers support the authored request properties. No crypto laws,
  funding/settlement semantics or application callsite wiring are proved.

The independent review records are
[original transfers](../../evidence/production-transfer-review/README.md),
[consolidation/publication](../../evidence/parent-commits-review/README.md),
[combined word/foreign/checking](../../evidence/combined-db26-review/README.md),
and [scalar pairs](../../evidence/pair-0c00-review/README.md).
The latter two reviews requested one diagnostic assertion correction and two
replay coverage/classification fixes. All are implemented. The pair reviewer
also audited all 48 raw replay outcomes independently of the reported harness
gaps. Fresh parent replays use the strengthened checks.

## Final parent validation

- Full Infer compiler regression suite: **590 passed across all 30 binaries**,
  zero failures or ignored tests. VIR library: **140 passed**. Matching full
  vstd build: **2,226 verified, zero errors**. Rust and Verus formatting pass.
- Strict optimized-Python pair replay: **48 expected outcomes**, separately
  audited against every helper identity set and named obligation. The stateful
  fixture proves **20 queries, zero errors**, with both original helpers inferred.
  Portable/conditional architecture tests, unrelated arithmetic, eager safety,
  cache poisoning, direct/transitive imports and failed exports are covered.
- Strict optimized-Python word replay: **18 expected outcomes**, including the
  new pair support, representation equivalence, permission/count transfer and
  preserved literal/forwarding baseline failures.
- Full seven-field Fedimint: **nine expected verifier outcomes**. Cold/warm
  application proofs are 10/0 each; imports are 3/0 each. Wrong identity, guard
  deletion and false policy controls reject. All **8,448 native erasure
  comparisons** pass as supporting evidence.
- The common front door accepts all **four standard transfers**: Fedimint
  deadlines and redb codec, page-size and address-range properties.
- The varint checker validates all **66 expected outcomes** and native
  debug/release controls. Full canonical status remains **blocked**; the
  explicitly partial authored length proof remains 8/0 cold/warm with and
  without goal assistance. Semantic mutants and seeded inference-aware
  publication controls reject.
- General Python tests: **138 passed**; optimized shared-evidence and transfer
  tests: **8 and 12 passed**. Current pair/word/shared guard suites also pass
  normally and under `-O`. Native pair execution passes **65,538 host-64-bit
  cases**, separately from the portable formal proofs.

Setup failures and the strict-harness status-name trial are retained and
excluded from accepted receipts. The manifest identifies them; successful
reruns are new receipts. Native tests are finite correspondence checks, not
substitutes for formal proofs.

## Remaining authored work and limits

The bitmap domain was frozen before proof attempts: one physical u64 word,
1–64 logical pages, allocated padding, arbitrary membership, and repeated
allocation. Its state invariant, allocation/count meaning, padding policy,
input domain and transition postconditions remain authored. Explicit canonical
word representation symbols are supplied, with equivalence to the original
definitions proved separately. The historical literal definitions and arbitrary
forwarding wrappers remain blockers. Multiword/database properties are outside
this experiment.

The unfamiliar unsigned-varint property remains **blocked**. Every u64 and
arbitrary initialized ten-byte buffer remain in scope. Authored loop annotations
and bit-vector work prove safety and returned length 1–10; they do not prove
canonical bytes, continuation bits and minimality. Exact inference still stops
at mutable iteration, and the authored canonical attempt still fails its
postcondition. The weaker length theorem is never substituted for that goal.

Exact pair support covers two scalar outputs and immutable locals. Tuple inputs,
nested tuples, other arities and sound-only pairs remain unsupported. Portable
proofs, conditional 32/64-bit caller checks and finite native host tests are
recorded separately.

Legacy ordinary Verus exports without inference or `--infer-goals` remain
outside the certification gate and can physically exist after failed checking.
A file alone is not proof evidence. Compiler, macros, SMT, existing vstd models
and explicitly declared external-type correspondence remain trust boundaries.

## Assessment and next priorities

The intended division of work is demonstrated on bounded application slices:
authors supply meaning, domains and invariants; checked automation discharges
routine helper contracts and selected proof bridges. General whole-project
adoption still needs substantial coverage work. There is no measured percentage
of arbitrary Rust's mechanical proofs that the engine can handle.

The strongest next coverage target is mutable-iterator/fixed-buffer effect
reasoning and written-prefix invariants, measured against the unchanged varint
canonical goal. Other useful targets are representation-wrapper bridges,
multiword/stateful collection frames, and larger real proof closures that include
application callsites. High-level protocol/database specifications, environmental
assumptions and refinement arguments remain authored. Formal verification of the
production acceptance component is a separate assurance project.

## Reproduction

Build matching compiler/macros/vstd from `source/` with Rust 1.99.1 and Z3 4.16.0.
The [transfer front door](../../source/tools/INFER_TRANSFERS.md) documents the four
standard pinned transfers. The new experiment controllers are:

```sh
python3 -B -O source/tools/research/scalar_tuple_output/replay.py --verus /matching/verus --output /private/pairs
python3 -B -O source/tools/research/word_goal_automation/replay.py --verus /matching/verus --expect-tuple-supported --output /private/word
python3 -B evidence/transfer-fedimint-full-request/run.py --application /pinned/fedimint --verus /matching/verus --solver /matching/z3 --output /private/full-request
python3 -B evidence/verus-infer-varint-3b05c191/replay.py --verus /matching/verus --solver /matching/z3 --stage all --output-root /private/varint
python3 -B evidence/verus-infer-varint-3b05c191/check_results.py --raw-root /private/varint
```

Create the word output parent first. Keep the tool bundle and source inputs fixed
during guarded replays. No push or PR is part of this cycle.
