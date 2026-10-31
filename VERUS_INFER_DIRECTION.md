# Verus Infer: direction and active roadmap

Working direction, October 31, 2026. This document sets current priorities;
[the feature reference](source/docs/guide/src/infer.md) defines supported behavior.
The [RFC and implementation history](VERUS_INFER.md) preserve earlier designs
and milestone context.

## Goal and division of responsibility

Make existing Rust code easier to verify through reusable, checked automation.
Authors choose intended behavior, supported input domains, abstractions and
dependency assumptions. The tool derives routine implementation contracts and
proposes proof steps that ordinary Verus verification must check.

Three capabilities contribute to that goal:

- **Contract inference** describes implementation behavior. Exact mode requires
  soundness, safe-domain maximality and output exactness within the supported
  observations. Sound mode establishes the obligations it explicitly reports.
- **Proof assistance** proposes annotations, lemma applications and proof settings
  for authored goals. Each proposal remains subject to ordinary verification.
- **Application verification** connects checked implementations to the author's
  intended properties, including coverage of the required input domain.

An exact description of an implementation does not establish its intended
policy. Supplied views, invariants and dependency laws remain visible inputs.
The [adoption guide](source/docs/guide/src/infer_adoption.md) explains this workflow.

## Starting point

The prototype supports bounded scalar, collection and record profiles, checked
cross-crate composition, and selected goal assistance. Native Aleph milestones
cover selected collector and validation paths; native hex milestones extend
from byte-pair semantics through forward iteration to public vector decoding.
Each acceptance record defines its own source, assumptions and finite scope:
[Aleph v3](VERUS_INFER_MECHANICAL_V3_ACCEPTANCE.md),
[Aleph v4](evidence/verus-infer-validation-v4/README.md),
[hex v3](evidence/verus-infer-hex-v3/README.md).

The experiment harness, integration-test support and ordered assistance pipeline
have been consolidated. The next cycle should make those capabilities easier
to diagnose and evaluate on another application.

## Active sequence

1. **Explain assistance decisions.** Extend structured reports with what the
   tool considered, applied or skipped. Give concrete reasons where a profile
   can identify a missing condition, unsupported shape or exhausted budget.
   Distinguish generation observations from checked proof outcomes. Done means
   deterministic reporting, positive and negative regressions, and unchanged
   verification and publication rules for existing inputs.
2. **Measure transfer to an unrelated application.** Select a small existing
   parser or encoder and pin its source and one useful property. First try the
   current engine without changes. Record executable adaptations, authored
   contracts/views/proofs, assumptions, blockers and proof time. Freeze that
   baseline before extending the engine; an unsupported case is a useful result.
3. **Resolve one evidenced blocker.** Use the transfer report to select a bounded
   automation extension or consolidation. Require reuse beyond the motivating
   fixture, semantic negative controls and a fresh application replay. Update
   this sequence from the result before committing to another feature batch.

Keep each step reviewable on its own. Historical milestone documents and frozen
evidence retain their original claims and pins.

## How to judge progress

For each application slice, record the requested property and input domain,
selected original bodies, remaining authored proof work, trusted interfaces and
reproducible cold/warm verification results. Compare changes against that same
slice. Distinguish authored intent from removable implementation proof work;
raw helper or certificate counts alone do not measure reduced author effort.

Preserve executable behavior and document any required adaptation. Exact
candidates retain all three required checks, cached candidates receive fresh
proofs, and failed or incomplete required checks prevent publication. Changes
to trusted library models need an explicit account of the additional assumptions.

## Follow-on candidates

Ordinary-proof-first bounded retries, a convenient expected-input-domain goal,
and shared proof-assistance utilities are candidates for the next decision.
Backward/mixed hex iteration and fixed-array decoding are bounded application
extensions. Richer abstractions and loop families remain research directions.

Formal verification of the production acceptance component is a separately
scoped assurance project; see the
[assurance targets](VERUS_INFER_ASSURANCE.md#later-formal-verification-targets).
General Rust support, arbitrary invariant discovery and whole-protocol
correctness remain beyond the current implementation.
