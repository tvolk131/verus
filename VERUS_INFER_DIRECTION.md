# Verus Infer: direction and active roadmap

Working direction, November 10, 2026. This document sets current priorities;
[the feature reference](source/docs/guide/src/infer.md) defines supported behavior.
The [project entry point](VERUS_INFER.md) links adoption, assurance, application
evidence and the archived RFC; ordinary work need not load the historical journals.

## Active goal: canonical mutable-buffer proving and transfer

The [goal charter](source/tools/research/mutable_buffer_goal/TARGET.md) freezes
the unchanged full-domain varint canonical theorem and a second real buffer
writer. Canonical assistance now verifies cold/warm with no application-local
mechanical proof blocks, theorem calls or snapshots; semantic definitions and
invariants remain authored. The [combined record](source/tools/research/mutable_buffer_goal/parent-combined.json)
binds matching runtime checks and remaining acceptance work. A scoped canonical
review passed; fresh combined review and broader compiler regressions are pending.

The frozen HMAC pad-XOR writer's full XOR-prefix/unchanged-suffix theorem remains
unproved. Shared observation assistance removes snapshots and representation
invariants from its prefix-only proof, but supplies no untouched-tail frame.
Checked reference entry and conditional resource laws have not yet established
native source correspondence. The active implementation task is a checked
single-borrow ownership/refinement bridge through actual native operations.
The writer's body, full domain and property remain unchanged. This goal stays
active until both full properties and the combined acceptance criteria pass.

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

The later [integrated transfer checkpoint](evidence/infer-proof-composition/README.md)
adds bounded native Fedimint deadline and redb codec, page-size and address-range
proofs. Checked caller assistance removes packing, byte-inverse and shift-bound
glue from selected goals. The later [full-request experiment](evidence/transfer-fedimint-full-request/README.md)
retains the actual seven-field Fedimint request through an explicit opaque
external type declaration and narrow readonly eligibility. Combined parent
cold/warm, import and negative-control replays pass; no whole-gateway or database theorem is claimed.
Use the [transfer front door](source/tools/INFER_TRANSFERS.md) for fresh results.

The experiment harness, integration-test support and ordered assistance pipeline
have been consolidated. Growth/range and pointwise clone-transport explanations
and shared annotation utilities are implemented and reviewed. The transfer
changes are integrated locally and have passed fresh
parent replay and [independent review](evidence/production-transfer-review/README.md).

The original [varint baseline](evidence/verus-infer-varint-3b05c191/README.md)
gets past macro syntax using the existing adapter. Exact contract inference
stops at `iter_mut`; its authored annotations prove safety/length. The active
goal above subsequently proves the full canonical property through checked
goal assistance. The [single-word redb transition](source/tools/research/redb_single_word_set/REPORT.md)
proves an authored membership/count/padding invariant with explicit generic
word-law assistance; the frozen literal baseline still fails its semantic goals.
The merged [word-view extension](source/tools/research/word_goal_automation/REPORT.md)
removes local snapshots/proof blocks/theorem calls when equivalent generic
representation symbols are supplied, and transfers to permission/count goals.
The literal old definitions and arbitrary forwarding wrappers remain blockers.
Combined parent replays and an [independent review](evidence/combined-db26-review/README.md)
pass after correcting one diagnostic assertion. These results preserve the
frozen baselines. The later [scalar-pair extension](source/tools/research/scalar_tuple_output/REPORT.md)
infers both original bitmap helpers and passes a
[separate review](evidence/pair-0c00-review/README.md). Its two replay coverage
and classification findings are corrected and checked by fresh parent replays.

## Completed five-step cycle

1. **Completed: validate the integrated transfer work.** Review the production delta and
   trust inventories, build matching compiler/macros/vstd, and replay pinned
   applications with fresh receipts. Historical test counts do not establish
   current acceptance. Keep semantic, safety and auxiliary controls distinct.
2. **Completed: consolidate the current workflow.** Keep status and assurance concise,
   extract genuinely shared assistance utilities, and explain more preparation
   decisions. Preserve profile-specific normalization and independent checks.
   Generation observations must never authorize verification or publication.
3. **Completed baseline: freeze another end-to-end property.** Pin one unfamiliar encoder/parser,
   original bodies, intended domain and authored semantics before extending the
   engine. First measure the existing automation, recording adapters, remaining
   proof annotations, assumptions and the actual first blocker. The fresh varint
   canonical goal was blocked at that checkpoint; its partial length theorem
   did not replace the required property or qualify as transfer acceptance.
4. **Completed: remove evidenced mechanical work and demonstrate transfer.** Select one
   bounded reusable extension from those baselines. Retain original application
   goals/domains, add unrelated positive and negative cases, and replay both the
   motivating and transfer properties. A smaller manual-proof inventory is the
   objective; exact synthesis for every function is not required.
5. **Completed for the frozen single-word domain: repeat for a stateful property.** Supply a meaningful abstraction/invariant
   over a small original transition closure, freeze it before proof attempts,
   and measure generated versus authored frames, bounds and induction work.
   A toy replacement implementation or a silently restricted domain does not
   establish the selected application's property.

The final combined checkpoint passes the complete Infer regression suite, the
four standard transfers, the generic-word stateful reduction with both helpers
inferred, and the full native request transfer. The
[acceptance record](evidence/infer-cycle-20261109/README.md) binds the matching
runtime, receipts, authored inputs and remaining limits. Canonical varint
encoding was outside that checkpoint's accepted application proofs; subsequent
progress belongs to the active goal above.

Independent worktrees may evaluate the encoder and stateful properties in
parallel against a fixed engine. Keep their baselines and evidence separate;
integrate useful improvements only after review and combined validation.

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

## Follow-on priorities

The checked native mutable-iterator ownership/frame bridge is the next coverage
target, measured against the frozen full HMAC writer while retaining the proved
canonical varint fixture. Representation-wrapper bridges, multiword stateful
collection frames and larger proof closures including real application callsites
follow. Ordinary-proof-first
bounded retries and an expected-input-domain goal can improve adoption; backward/mixed
hex iteration and fixed-array decoding remain bounded application extensions.

Formal verification of the production acceptance component is a separately
scoped assurance project; see the
[assurance targets](VERUS_INFER_ASSURANCE.md#later-formal-verification-targets).
General Rust support, arbitrary invariant discovery and whole-protocol
correctness remain beyond the current implementation.
