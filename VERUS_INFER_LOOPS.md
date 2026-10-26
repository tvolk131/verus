# First counted-loop inference milestone

The first supported pattern is an exact, nongeneric executable function over two
immutable fixed-width unsigned inputs of the same type. It initializes a mutable
accumulator from one input and a mutable counter to zero, then executes
`while counter < bound { accumulator += 1; counter += 1; }` and returns the
accumulator. Local names are irrelevant. The recognizer checks the entire typed
body, arithmetic policies, assignments, and absence of user proof annotations.
Additional loop shapes remain unsupported.

For inputs `x` and `n`, mathematical candidate predicates are:

- Requires: `x + n <= MAX` for the unsigned result type.
- Ensures: `result == x + n`.

This is untrusted synthesis. It never grants proof evidence.

## Proof annotations and executable preservation

The loop receives these prefix invariants:

- `counter <= n`.
- `accumulator == x + counter`, using mathematical arithmetic.

The decreasing measure is `n - counter`. Loop isolation is disabled for this
small pattern, so the ordinary query can retain its input precondition without
adding that precondition to the invariant set. In particular, the invariants
must not include `x + n <= MAX`.

The installer may add only loop invariants, a decreasing measure, and the loop
isolation proof setting. A projection check removes exactly those annotations
and compares every remaining executable field with the original typed body.
Loop-free installation keeps the original body pointer. Candidate identities
bind the original body, installed proof annotations, actual contract, configuration,
and dependency provenance. Cached formulas cannot supply executable bodies or
proof annotations; annotations are regenerated and rechecked on every use.

## Three independent checks

1. **Soundness and termination.** Ordinary Verus checks invariant initialization
   and preservation, both integer additions, decreasing-measure obligations, and
   the inferred result relation under the proposed precondition. Termination
   assumptions and options suppressing termination checks are ineligible for
   this pattern.
2. **Safe-domain maximality.** Remove the proposed precondition and use the
   original executable's built-in safety conditions to select successful
   executions. Loop invariant and decreasing-measure obligations remain checked
   by the normal loop lowering. The prefix invariants are initialized and
   preserved without assuming the proposed precondition. At a successful exit,
   `counter == n` and the accumulator's unsigned type bounds establish
   `x + n <= MAX`. Neither the candidate precondition nor an invariant justified
   only by that precondition may be assumed here.
3. **Output exactness.** Under the proposed precondition, the existing arbitrary
   result-value query proves the relation equivalent to the original executable's
   result, using the independently checked prefix invariants.

The publication barrier still requires all three successful candidate-bound
outcomes. Every required subquery must succeed. Required failures, timeouts,
skips, or missing evidence block dependent verification and exports.

## Validation

Semantic tests compare the proposed domain and result with concrete checked
`u8` iteration, including zero iterations and overflow boundaries. Backend fault
injection must reject incorrect invariants, nondecreasing measures, permissive
and restrictive preconditions, and incorrect outputs. Maximality must fail for
a restrictive cached predicate even when soundness and output exactness pass.
Source and projection tests reject changed increments, wrong update order,
extra statements, nested loops, breaks, mutation outside the recognized locals,
user proof annotations, and termination-suppression options. Cache, import,
report, and Cargo tests exercise ordinary acceptance and publication paths.

This document describes the checking strategy, not a formal proof of its
implementation. The broader assurance targets remain in
[VERUS_INFER_ASSURANCE.md](VERUS_INFER_ASSURANCE.md).
