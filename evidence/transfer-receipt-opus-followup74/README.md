# Opus receipt review and bounded follow-up

Claude Opus 5.5 completed the previously outstanding independent review without
a service restriction. Its original report is immutable at
`/private/tmp/verus-opus-receipt-review74.FXe61X/REPORT.md`.
It confirms all three original receipt corrections, with no regression found.
It reports two P3 follow-ups and one P4 consistency refinement, not a false
theorem, compiler defect, or application vulnerability.

## Follow-up changes

- R1: relational acceptance now checks all five prerequisite commands' exact
  arguments and engine working directories, the before/after application pin
  and clean output, and complete duplicate-free engine enumeration against the
  source ledger. Exit values retain the existing integer-zero requirement.
- R2: certificate tests require the intended diagnostic, keep imported
  fingerprints and corrupted-import fixtures consistent with certificate edits,
  and cover schema integers, integer count errors, and extra guard paths. A
  producer unit test checks the provider guard immediately before and after
  every imported proof and again before final success publication. Its compiler
  subprocesses are synthetic; this test is not an application proof.
- R3: each imported relay must depend on the certificate's address-range helper,
  with the provider's reported candidate identity and exact mode.

The follow-up is parent-authored and validated, not independently reviewed by
Opus. The independent review applies to the preserved pre-follow-up snapshot.
Engine/cache74, policy68, schema38, Rust sources, runtime and authored application
goals remain unchanged. Original handoffs and evidence remain immutable.

## Validation and limits

Fresh follow-up commands, results, source/runtime hashes, mutation coverage and
receipts are recorded under `/private/tmp/verus-opus-followup74.qDmCVH`.

- Python discovery: 225 tests pass normally and 225 under `-O`.
- Targeted check-removal tests: all 22 variants are detected in each mode,
  44 executions total. These include all 14 mutations that survived Opus's
  original coverage check and eight additional prerequisite/relay variants.
  This is not a rerun of all 39 historical mutations.
- The fresh common frontdoor proves all six registered scenarios; the
  dedicated access-prefix replay passes all 39 controls. Neither a controller
  exit nor these extracted helper proofs is a whole-application theorem.
- All 1,554 source entries and nine runtime hashes match the finalized
  follow-up snapshot after replay. Only four source files differ from the
  previous completion snapshot; Rust and runtime inputs remain identical.
- All 3,575 guarded original inputs and recorded local refs match after the
  new transfers. This guard covers the preserved handoff, pinned applications,
  independent review tree and sealed runtime, not the ambient global Cargo
  cache. Its new writable Cargo home is isolated; there is no blanket claim
  about concurrently changing shared Cargo bookkeeping.

Common receipt:
`/private/tmp/verus-opus-followup74.qDmCVH/six-scenarios/transfer-suite-qh2kt409/receipt.json`.
Dedicated access receipt:
`/private/tmp/verus-opus-followup74.qDmCVH/access-prefix/receipt.json`.
Complete reconstruction package:
`/private/tmp/verus-opus-followup74.qDmCVH/handoff/HANDOFF.md`.

The independent Opus review freshly ran 219 Python tests in each mode, a
relational replay, 684 receipt probes per mode, 15 supplementary probes per mode
and 39 check-removal mutations. Its eight unexpected accepts are R1; its 14
surviving mutations motivate R2. These are historical review results, not results
of the repaired snapshot.

Receipt consistency is not cryptographic attestation of arbitrarily fabricated
evidence. Candidate identity binding is consistency with the checked provider
report, not an independently authenticated identity. Existing verifier, solver,
native-library and frontend-correspondence trust remains. No whole-application,
storage, allocator, async-state, cryptography or funding-safety claim follows.
No verifier/vstd rebuild, Rust integration-suite rerun, hosted workflow, online
fetch, commit, PR, merge or branch update is part of this follow-up. A fresh
full-request replay builds only its pinned Bitcoin dependencies offline in an
isolated writable Cargo home copied from the preserved registry inputs.
