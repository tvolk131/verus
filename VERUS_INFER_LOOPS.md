# Exact unsigned counted-loop inference

Supported functions are exact, nongeneric executable functions over two or three
immutable unsigned inputs of the same type (`usize` or `u8` through `u128`), with
that same return type. They initialize a mutable accumulator from input `x`, run
one of the following loops, and return the accumulator:

| Counter | Initialization | Guard | Counter update |
| --- | --- | --- | --- |
| Ascending | `i = 0` | `i < n` | `i += 1` |
| Countdown | `i = n` | `i > 0` | `i -= 1` |

The accumulator update is `r += step` or `r -= step`, before the counter update.
`step` is an unsigned literal or immutable same-width parameter; `usize::MAX` is
also supported as a symbolic platform constant. It may alias `x` or `n`. Every
parameter must be used. Both assignments may use `v = v + operand` / `v = v - operand`.
Local names, parameter order, and the order of the two declarations may differ.
The recognizer checks the entire typed body, arithmetic policies, and absence of
user proof annotations. Signed arithmetic, arbitrary counter steps/guards, extra
statements, calls, multiple/nested loops, breaks/continues, and general mutation
remain outside this family. Loop inference requires exact mode.

For inputs `x`, `n`, and `step`, the mathematical candidate predicates are:

| Accumulator update | Requires | Ensures |
| --- | --- | --- |
| Addition | `x + n * step <= MAX` | `result == x + n * step` |
| Subtraction | `n * step <= x` | `result == x - n * step` |

Products use unbounded mathematical integers, even when the executable's type is
`u128`. A zero step or count admits the full range of `x`; there is no artificial
step/count bound. The original unit-step pattern keeps its simpler `x + n`
formula. Synthesis is untrusted and grants no proof evidence.

## Proof annotations and executable preservation

Both counter directions receive these prefix invariants:

- `i <= n`.
- `r == x + completed * step` (addition), or `r == x - completed * step` (subtraction).

For ascending counters, `completed = i` and the decreasing measure is `n - i`.
For countdown counters, `completed = n - i` and the measure is `i`.
Loop isolation is disabled, so the ordinary query can retain its input
precondition without adding that precondition to the invariant set. The
invariants must be inductive without the candidate safe-domain predicate.

The body installer adds only invariants, a decreasing measure, and the loop
isolation proof setting. A production projection check removes exactly those
annotations and compares every remaining serialized executable field with the
original typed body. Annotation expressions must belong to the restricted pure
summary language. Loop-free installation keeps the original body pointer.

A parameter step (or symbolic platform constant) selects Verus's existing
nonlinear arithmetic prover for the function's ordinary and precision checks.
Literal-step loops do not require that selection. This is a proof setting, not an
assumption, helper contract, or executable rewrite. Installation preserves other
function fields and attributes. Candidate identity version 3 binds the original
function, installed proof annotations, actual contract, actual nonlinear-prover
setting, configuration, and dependency provenance. Cache version 17 stores only
typed formulas; annotations and prover selection are regenerated from source on
every hit. Metadata schema 12 requires inferred dependents to be rebuilt.

## Three independent checks

1. **Soundness and termination.** Ordinary Verus checks invariant initialization
   and preservation, accumulator overflow/underflow, counter safety,
   decreasing-measure obligations, and the result relation under the proposed
   precondition. Termination assumptions and options suppressing termination
   checks are ineligible for these patterns.
2. **Safe-domain maximality.** Remove the proposed precondition and use the
   original executable's built-in safety conditions to select successful
   executions. Invariant and decreasing-measure obligations remain checked by
   normal loop lowering. The prefix invariants must initialize and preserve
   without assuming the proposed precondition. At a successful exit,
   `completed == n` and the accumulator's unsigned type bounds establish
   `x + n * step <= MAX` or `n * step <= x`. Neither the candidate precondition
   nor a circular invariant may be assumed here.
3. **Output exactness.** Under the proposed precondition, the existing arbitrary
   result-value query proves the relation equivalent to the original executable's
   result, using the independently checked prefix invariants.

The publication barrier requires all three successful candidate-bound outcomes.
Every required subquery must succeed. Required failures, timeouts, skips, or
missing evidence block dependent verification and exports. No new trusted loop
axioms, helper specifications, or termination assumptions are introduced.

## Validation

Independent concrete checked `u8` execution covers every starting value and
count, for both directions and accumulator operations, across constant steps
`0`, `2`, `255` and parameter steps `0`, `1`, `2`, `3`, `7`, `127`, `128`, `255`.
Backend fault injection constructs SST separately from synthesis and rejects
incorrect invariants, nondecreasing measures, permissive/restrictive domains,
circular preconditions, weak output relations, and wrong outputs. The same
ordinary SST-to-AIR translator and nonlinear prover options are exercised.

Compiler tests cover all six unsigned types, both pointer models, parameter and
literal steps, zero counts/steps, full-range boundaries, aliased step parameters,
and symbolic `usize::MAX`. Source/projection tests reject changed executable
updates, wrong counter directions/steps, dynamic step expressions, extra
statements, user proof annotations, and termination suppression. Cache poisoning,
precision timeouts, fresh three-crate composition, Cargo imports, structured
reports, and debug/release evaluation exercise the production publication paths.
The mutation runner tests critical source, proof, and acceptance boundaries.

`usize` uses symbolic bounds; synthesis never specializes to the host machine.
Verus's ordinary architecture model and cross-crate architecture merge determine
32/64-bit bounds. Verification at an alternate width is distinct from native
cross-target code generation.

This describes the checking strategy, not a formal proof of its implementation.
The broader assurance targets remain in
[VERUS_INFER_ASSURANCE.md](VERUS_INFER_ASSURANCE.md). Signed loops, arbitrary
counter steps, multiple/nested loops, slice traversal/search, general mutation,
and handwritten loop proofs remain separate work.
