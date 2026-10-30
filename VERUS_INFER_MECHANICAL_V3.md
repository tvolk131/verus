# Semantic-only adoption across native implementation patterns

Status: active. Verus baseline `f8318b9c5`; original Aleph baseline
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034`. V2 acceptance remains complete.

## Objective and stopping rule

Remove three concrete mechanical adoption workarounds through general checked
inference and goal assistance. Authors choose mathematical views, validity,
semantic trait/library interfaces, application behavior/error policies and input
coverage. They do not supply local supporting proof blocks, executable-loop
annotations, trigger hints, mechanical getter/selector frame clauses, or source
refactors made solely to fit these three supported patterns.

The acceptance targets are three distinct native modules from pinned Aleph:

1. **Scalar-wrapper state updates (`crypto/src/node.rs`).** Check the real
   derive_more AddAssign implementation by opt-in, generate its proposed extension
   model, require all exact obligations, and compose the checked method into
   record-state inference. Restore `direct_parents += NodeCount(1)` in the v2
   collector observer. No hand-authored arithmetic model or primitive-field
   rewrite may satisfy this target. Generic unsigned/signed wrappers and nested
   state consumers must exercise the same mechanism.
2. **Collection construction within a larger function
   (`consensus/src/units/control_hash.rs`).** Verify original `ControlHash::new`:
   initialize a local map, traverse sparse parents into it, then construct the
   record including an opaque hash call. No extracted projection helper. Prove
   exact positional round projection for arbitrary hash payloads and all member
   counts. The opaque hash result remains arbitrary: this ordinary semantic
   contract does not claim exact whole-result inference or deterministic hashing.
3. **Growth followed by a mutable return
   (`consensus/src/creation/creator.rs`).** Verify the original
   `get_or_initialize_collector_for_round` while loop followed by the indexed
   mutable return. No extracted unit-returning growth helper. Preserve valid
   history, configuration and earlier collectors, establish requested target and
   membership, and generate borrow identity/frame/restoration support. Keep the
   full raw `round < u16::MAX` domain and existing checked terminal rejection.

These targets remove specified refactors from the completed v2 implementation;
they do not require reversing unrelated documented v2 changes (opaque getter
observation capture or concrete ConstraintError). Native function bodies are
compared to pinned original source for the selected constructor/selector, and
derive expansions are checked against their actual Cargo implementations.
Existing v2 semantic coverage is retained as a regression requirement.

## Generality and boundaries

Recognition and generation depend on typed operations, verified contracts,
authored views and function structure, never application names, paths, particular
constants or Aleph-specific invariants. Each engine extension needs unrelated
generic positive and semantic negative cases. Exact mode retains ordinary body
soundness, safe-domain maximality and output-exactness; assistance on ordinary
functions is checked proof scaffolding, not an exact certificate.

Retain v2's actual quorum domain, initial/zero-member coverage, opaque hash/codec/
lock strengths, generic clone relation, complete error ordering and actual
creation-output eligibility. No assumed first-party implementation, narrower
domain, stronger dependency law or silent sound-mode fallback is authorized.
Signed/UncheckedSigned serializer exclusions remain unauthorized. Protocol-wide
agreement/liveness, cryptography and arbitrary loops/recursion are outside this
finite milestone.

## Required evidence

- Generic proof/certificate regressions, wrong implementation/domain/goal cases,
  cross-crate use and withheld failed/no-verify/filtered publication.
- All three source-connected native targets, retained v2 semantic goals and
  same-source native tests under a matching recorded toolchain.
- At least one deliberate implementation mutation for each native pattern,
  ordinary application proof rejection, and restored full verification.
- Authored-source and compiled-scope inventories with semantic review of author
  inputs, unchanged trust, original executable bodies and input coverage.
- A fresh-context acceptance audit using a fixed engine, general guidance and
  frozen semantic requirements. Preserve its independent review; no prior proof
  solution or application-specific hints as adoption instructions.

Record unsupported cases as unfinished requirements rather than weakening this
stopping rule. Commit cohesive validated work locally; do not push. Broader
mutation, traversal, recursion and unrelated codebases remain future milestones.

## Implementation checkpoint

The parent development fixture restores all three native patterns. Parsed body
tokens for ControlHash::new and the Creator selector equal pinned original main;
the collector observer uses its original NodeCount += expression. The extracted
projection/growth helpers and authored AddAssign extension model are removed.
The existing semantic coverage passes: 154 proof results, 35 exact helpers/all
105 required checks, 83 checked executable bodies and one generated proof body.
The lower body/result counts reflect removed helpers rather than weaker coverage.
Same-source native checking passes 192 unit tests plus one doctest. Three actual
implementation mutations fail ordinary proofs with healthy helper certificates,
no-verify/filtered publication is rejected and restored full verification passes.
The authored inventory retains no local supporting proof blocks, executable-loop
annotations or manual trigger hints. The fresh-context acceptance audit remains
required before completion; these are parent results, not independent acceptance.
