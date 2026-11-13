# Adopting inference in an existing codebase

Start with a bounded, useful source slice and its actual dependency path.
[Capabilities](infer_capabilities.md) are examples of supported forms, not a reason
to rewrite the original algorithm to match a recognizer.

## Freeze the task

Record the upstream commit, original complete bodies, executable adaptations,
semantic property and full input domain. Include normal error behavior, repeat
operations, boundary inputs and relevant state/frame conditions. Separate:

- semantic definitions, views and validity invariants;
- manual mechanical proofs and their locations;
- executable/frontend adaptations and unverified dependencies;
- environmental premises and imported trust.

A spec correction is legitimate when disclosed with the old failed attempt.
An adapter or narrower domain must not be counted as an unchanged-source proof.

## Establish a baseline

Use normal lifetime, trait, well-formedness, mode, safety and body checking.
Run ordinary verification and optional `--infer-goals` separately. Inspect actual
named bodies and required helper proofs; zero helpers is a real expected inventory.
Record unsupported/frontend, resource, semantic and safety failures separately.
A test-controller success flag alone does not establish a verified application.

An initial manual ordinary proof often identifies whether the remaining burden
is arithmetic, set transport, mutation framing or a genuinely missing model.
Before engine work, measure an actual necessary obligation and its annotation
removal target. Existing automatic arithmetic or redundant annotations do not
count as a new assistance gain.

## Choose the inference boundary

Inference is cheapest on small helper functions whose exact contracts stay
short. When a run notes that a contract is large, callers usually do better
with a hand-written contract that states a compact invariant for that function,
calling inferred helpers inside it. If you keep it inferred, refer to its
precondition and postcondition through `f_requires`/`f_ensures` in lemmas rather
than proving them inline in a large function body.

## Validate a reduction

Keep the original body, property and domain. Compare manual and reduced cases
using the same verifier/runtime, with fresh outputs and appropriate cold/warm
runs. Check imports and stale-output suppression for exported capabilities.

Retain at least one meaningful false-property control and separate safety controls.
Validate the intended failing function and query kind with healthy dependencies.
Deleted stores/frames, wrong returns and weak domains are different controls;
structural rejection is not an ordinary solver-level state proof.

Independent review should inspect source fidelity, required checks, new compiler
trust, complete bodies and manual cost. Small review fixes belong with the author;
a new integrated capability needs fresh review of that exact source revision.

## Keep the result maintainable

Promote accepted source examples into canonical fixtures, preserve licensing and
compact provenance, and use the shared test harness. Keep expected outcomes in
the tests; full generated reports go to temporary output or CI artifacts.
Historical replays run at their archived revision. A hash does not preserve a
private binary or make a machine-specific replay portable.

Stop at a concrete value/scope checkpoint if the proof requires unsupported
semantics. No automatic chain of native ownership/private-representation work is
implied by an application proof attempt. In particular, model-only resource laws,
source-body matching and finite native traces cannot replace the missing source
link in a database crash or cryptographic theorem.

See [implementation and testing](infer_design.md) for the maintained commands and
[application status](infer_capabilities.md#application-status) for current limits.
