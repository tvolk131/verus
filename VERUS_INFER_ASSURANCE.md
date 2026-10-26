# Verus Infer assurance plan

This tracks assurance work for the implemented prototype. It is a development
plan, not a claim that the new verification machinery is formally verified or
that the potential failure modes below are known bugs. Supported behavior is
documented in the [inference guide](source/docs/guide/src/infer.md).

## Recommended sequence

The focused assurance consolidation, exact early returns, and bounded
`Option`/`Result` propagation, and exact/interval signed integer inference are implemented. Retain the formal verification
targets below as separate work.
Formalization of the entire precision checker is not a prerequisite for every
feature.
The feature backlog and proposed milestone order are maintained in
[the main roadmap](VERUS_INFER.md#proposed-next-milestones).

## Current protections and trust boundary

- Contract installation preserves original executable code. Loop-free bodies
  retain their original pointer. Counted loops receive only pure proof
  annotations, with a production source-projection check; ordinary Verus checks
  the proposed contract, invariants, and termination against that code.
- Exact mode requires soundness, safe-domain maximality, and output exactness.
  Sound mode always requires soundness; optional precision failures do not
  establish those precision guarantees.
- Proof outcomes start unproved. A recorded success requires a query to have
  been checked without unproved subqueries or timeouts.
- Candidate caches store typed formulas, not proof results. Reused candidates
  receive fresh proofs, and source eligibility is checked independently.
- Inferred functions are checked in dependency order before dependent callers.
  Full-crate verification is required for local inferred definitions. Exports
  are withheld on failure, and previous inferred exports are invalidated during
  a rebuild.
- Imports validate per-function guarantees, contract fingerprints, dependency
  fingerprints, and standard-model fingerprints. Imported bodies are not
  reverified. This retains Verus's existing trusted-library model; the new
  metadata checks are additional enforcement, not independently checkable proofs.
- Two existing external-body array helpers additionally declare `no_unwind`.
  Those declarations are trusted specification additions. No new array axioms
  or external-body functions were introduced for this milestone.

Candidate synthesis need not be correct for a properly checked contract to be
sound. Correctness of contract installation, query construction, outcome
recording, publication, and trusted specifications still matters. A defect
confined to precision checking can mislabel a sound contract as maximal or
exact; that alone does not invalidate its separate ordinary soundness proof.
Early-return support adds only candidate-generation logic. The original body,
precision transformations, and acceptance policy remain unchanged by that
feature; its guarded formulas receive the same independent proofs.
Propagation additionally recognizes a closed set of existing core/vstd helpers
and their complete lowered expression. It retains the original executable and
all proof gates. Source eligibility runs before resolution expansion; subsequent
verification ordering scans every call in the expanded body for inferred dependencies.
See the propagation-model audit below for the existing `Result` trust assumptions.
Checking a caller before its callee is not inherently unsound: accepting the
overall verification without the required callee proof would be the error.

## Completed consolidation: structural safeguards and targeted tests

These safeguards and tests are implemented. They do not constitute formal
verification of the checker or its integration.

- [x] Have synthesis and cache decoding return contract data through a narrow
  interface, with one installer responsible for preserving executable code,
  function identity, signature, and unrelated attributes.
- [x] Make the separation between the ordinary query and transformed precision
  queries explicit. Test that constructing or running precision probes cannot
  suppress ordinary overflow or array-bounds failures.
- [x] Isolate proof-outcome aggregation and acceptance rules in a small component.
  Associate outcomes with the function, candidate identity, and obligation kind;
  prevent absent or unsuccessful outcomes from authorizing use or export.
- [x] Extend fault injection for failed, canceled, timed-out, skipped, duplicate,
  reordered, and mismatched outcomes, including mixed exact/sound dependencies.
  Cover both caller acceptance and export suppression.
- [x] Mutation-test critical precision and publication rules: deliberately weaken
  a rule and require the corresponding regression test to detect the change.
- [x] Review the two added array `no_unwind` declarations and record their
  justification. A test that itself relies on these declarations is not an
  independent proof of them.

The dependency-free production policy is in
`source/vir/src/infer_proof/policy.rs`; its compiler-facing evidence adapter is
`source/vir/src/infer_proof.rs`. The source mutation runner is
`source/tools/test_infer_assurance_mutations.py`. It tests nine acceptance-core
mutations, four precision-check mutations, and four counted-loop boundary
mutations in temporary copies. Baselines
must pass; each mutant must run tests and fail them. Compilation failures and
empty test runs do not count as detected mutations. This tests selected failure
mechanisms, not all possible defects in the pipeline.

Candidate identities include the original typed function, actual formulas,
configuration, and transitive dependencies. Outcomes are bound to candidate and
obligation; duplicate, mismatched, and unknown records invalidate acceptance.
Diagnostic/profile reruns supply no evidence. Ordinary parallel workers cannot
overwrite inference outcomes. Query kinds must match ordinary/precision bodies.

The assurance milestone introduced metadata schema 6 and cache version 6.
Compositional safety inference uses cache version 11 and retains metadata schema 6.
Shared-slice inference extends the summary language with existing slice length
and index operations, using cache version 12 and metadata schema 7. Older inferred
interfaces must be rebuilt. This extension adds no serialized AST fields.
Counted-loop inference uses cache version 13, candidate identity version 2, and
metadata schema 8. It binds generated proof annotations into candidate identities
and also adds no serialized AST fields.
The serialized identity introduced with the assurance milestone also requires
rebuilding `vstd`, other `.vir` libraries, and dependents, including ordinary
libraries built with older compiler binaries.

## Trusted array-helper audit

The additions are `no_unwind` on two pre-existing external-body functions in
`source/vstd/array.rs`:

- `array_as_slice<T, N>` returns a shared array reference through Rust's built-in
  array-to-slice coercion. Its body performs no indexing, arithmetic, allocation,
  or user-defined call that could unwind.
- `array_fill_for_copy_types<T: Copy, N>` uses built-in `[t; N]` repetition.
  Copying a `Copy` value invokes no user-defined `Clone`, and a `Copy` type has no
  destructor. Its body performs no checked arithmetic, indexing, or heap
  allocation. This reasoning also covers `N = 0`.

These arguments justify the declarations by review of the bodies and Rust
operation semantics; they are not formal implementation proofs. Evaluating the
operand passed to a helper remains the caller's responsibility, even for a
zero-length repetition. Inference regressions retain operand-overflow checks.
The `no_unwind` integration test checks use of the trusted model; it is not an
independent proof of those declarations.

## Propagation-model audit

`?` uses the existing `Option`/`Result` `branch` and `from_residual` specifications
in `source/vstd/std_specs/control_flow.rs`. Recognition checks their core identity,
vstd proxy, generic signature and bounds, then the concrete operand types,
ControlFlow arms, bindings, return, and residual call. The internal `Infallible`
and `ControlFlow` types are not generally eligible source values. Operands still
pass ordinary source eligibility, including checks for proof assumptions.
Generic helpers outside this complete lowering are rejected.

`Result` propagation requires identical scalar/unit error types and the core
conversion implementation. Rust's blanket `From<T> for T` returns its input;
the existing trusted `spec_from_blanket_identity` axiom models that identity.
The generic Result residual specification has no `no_unwind` clause, so this
feature does not add or export that guarantee. A `no_unwind` inferred wrapper
propagating an arbitrary Result input fails ordinary verification with that
existing model; a regression test checks this. The absence of runtime unwinding in the restricted identity
case rests on reviewed standard-library semantics, not a new formal proof.
The two branch models and Option residual model already declare `no_unwind`.
No specification, axiom, or external-body declaration was added or strengthened
for propagation. Declaration and containing-library fingerprints cover these
models, including their supporting axioms, through caches and exports.

## Signed-arithmetic integration

Signed support uses existing VIR operations and Verus's original executable
overflow checks, including the separate `MIN`/`-1` checks for both signed division
and remainder. Candidate generation derives bounds using mathematical integers.
The closed literal-negation lowering also admits signed `MIN` literals without
generally allowing unchecked executable arithmetic. Casts retain the existing
signed/unsigned truncation operations through synthesis, cache decoding, and
imported-summary substitution.

Verus lowers signed `/` and `%` to the existing inline vstd `rust_div` and
`rust_rem` definitions. These implicit dependencies are recorded alongside
explicit standard-model calls in candidate keys and exported model fingerprints.
Missing models receive an unsupported diagnostic before verification. Signed
checked methods use the pre-existing vstd numeric specifications, with distinct
Rust and Euclidean division/remainder summaries. No standard specification or
axiom was added or strengthened. The original body, precision transformations,
and proof-acceptance machinery remain unchanged by this feature.

Exhaustive i8 comparisons cover ordinary and checked arithmetic, including both
division conventions; cast comparisons cover complete i16/u16 input domains.
Native tests cover all widths and i128 boundaries. Cache corruption tests retain
the independent maximality and soundness gates, including rejection of a cached
remainder domain that admits `MIN % -1`. Cross-crate tests check signed summary
composition and invalidation when the containing vstd artifact changes.

## Signed interval approximation

Sound mode now supports signed scalars and intermediate values using the same
mandatory ordinary soundness proof as unsigned intervals. Arithmetic and branch
joins operate on mathematical integer bounds. Division splits denominator ranges
around zero; remainder bounds account for the dividend's sign. Wrapping casts
conservatively widen to the full destination range. The compiler's special
signed `MIN` literal lowering is recognized before restricting intermediate
values to machine ranges.

When exact safety extraction exceeds its budget or a sound-mode dependency
prevents it, bounded compositional synthesis derives a sufficient domain from
scalar contracts and result ranges. The previous signed `[-1, 1]` and unsigned
`[0, 1]` argument heuristics have been removed. Ordinary verification must still
prove safety under the generated domain. Maximality probing remains skipped
for sound-mode dependencies, and optional precision failures do not establish
the missing guarantees.

Exhaustive tests compare signed arithmetic and casts with concrete executions
and check division/remainder bounds across sign-changing intervals. Native tests
check useful ranges, all widths, expansion-budget fallback, and optional precision
timeouts. Cache poisoning tests reject ranges that omit valid negative results
and domains that admit unsafe negation. Cross-crate tests preserve mixed
exact/sound guarantees, require soundness evidence, and invalidate stale vstd
model dependencies. No standard specifications, precision transformations, or
proof-acceptance rules were changed for this feature.

## Compositional safety synthesis

The untrusted analyzer in `source/vir/src/infer/abstract_infer/safety.rs` carries
scalar value bounds and sufficient truth/falsehood conditions for predicates.
It instantiates callee preconditions over argument values; approximate returns
use their published intervals, while exact scalar relations can retain symbolic
values. Arithmetic checks are derived from those values and machine ranges.
Affine normalization prevents repeated addition from duplicating formulas.
Non-affine formulas and coefficients remain bounded; exhaustion reports an
incomplete analysis rather than selecting arbitrary small inputs.

Conditional and short-circuit requirements retain their execution guards.
Simple comparisons refine local values and copied aliases within a branch;
facts are restored at the join. This is candidate-generation logic, not a new
assumption in the executable or verifier query. Incomplete or obviously empty
derived domains produce diagnostics. The original executable, precision
transformations, candidate-bound evidence, and publication gates are unchanged.

Regressions check full signed/unsigned domains for safe range-based composition,
operation-derived offset limits, retained callee requirements, scoped branch
facts, affine expansion, and explicit budget exhaustion. Exhaustive small-width
tests check admitted domains against every result allowed by the callee range.
Poisoned caches must still fail the original ordinary overflow checks and
suppress exports. Cross-crate tests preserve honest optional precision metadata
while accepting the broader generated domains.

## Structured reporting and evaluation

`--output-json` reports actual candidate formulas and reads proof outcomes from
the production evidence tracker using the same identity and poison checks as
acceptance. The accessor returns a copy and cannot create or modify evidence.
Generation observations, presentation strings, timers, and cache-hit records
never participate in proof acceptance or imported-interface validation. The JSON
report is not a certificate and has no import path.

Aggregated resource-limit outcomes now retain `Incomplete` instead of collapsing
to a generic rejected outcome. This changes presentation categories, not the
acceptance condition: only `Valid` passes. Exhaustive acceptance/order tests and
the existing source mutations still check that failures, skips, and timeouts
cannot be repaired by later successes. New tests check resource-limit
classification, read-only observations, worker freshness, and poisoned evidence.

Integration regressions check cold/warm reproving, optional timeouts and skipped
precision, valid inference followed by failed caller verification, actual export
state, imported identities and model fingerprints, and a corrupted cached
precondition with fresh failed soundness and stale-export removal. The curated
[evaluation corpus](source/tools/infer_eval/README.md) checks structured outcomes
and publication, including deliberate frontend/unsupported/budget/proof failures.
The corpus supplements the existing semantic and assurance tests; it does not
prove the implementation or estimate general Rust coverage.

## Shared-slice inference

Dynamic slice candidates use the existing Verus slice length/index IR. The
original executable retains ordinary bounds checks, arithmetic checks, and its
control flow. Exact mode still requires soundness, maximality, and exactness;
no precision-check or acceptance rules were added for slices.

The canonical vstd slice length and index spec declarations are implicit
dependencies, with closed signature checks and fingerprints covering their
entire artifacts and supporting axioms. The existing executable `.len()` model
retains its closed recognition and `no_unwind` requirement. Shared reborrows
are stripped only as immutable value representations; mutable references remain
ineligible. No new axioms, trusted helpers, or stronger specifications were added.

Array-to-slice substitution retains the array's precise symbolic value and
receiver safety. Length/index operators specialize to the array, while reverse
slice-to-array substitution is rejected. Conditional summaries require compatible
branch representations, so differing-length array coercions cannot install an
ill-typed summary. Summary decoding and imported-summary substitution use the
same typed builders. Source eligibility is rechecked on cache hits.

Tests compare signed/unsigned slice domains and outputs against concrete Rust,
check empty/large slices and eager receiver evaluation, poison cached bounds
and reads, reject restrictive domains via maximality, and verify cache/import
proofs and stale-export removal. Extending collection support does not formally
verify the inference implementation; the formal targets below remain open.

## Counted-loop proof boundary

The first loop recipe and its checking strategy are specified in
[VERUS_INFER_LOOPS.md](VERUS_INFER_LOOPS.md). The recognizer admits the whole
typed two-local, unit-step unsigned accumulator body, not an arbitrary loop with
matching fragments. Source proof annotations and termination-suppression settings
are ineligible. Loop-free inference retains its prior source policy.

The installer adds only binder-free, typed invariant/measure expressions and the
non-isolation proof setting. It projects those fields away and compares the
remaining serialized body with the original. This prevents candidate generation
or cache reuse from changing any executable operation. Candidate identities bind
the installed proof body as well as the original source, actual contract,
configuration, and dependencies. Cache entries still contain formulas only;
annotations are regenerated through the same checked boundary on every hit.

Ordinary soundness includes invariant initialization/preservation, overflow,
termination, and the result relation. Maximality removes the candidate precondition
and keeps the original checked prefix invariants; their initialization,
preservation, and decreasing-measure checks are emitted by normal loop lowering
and are not converted into assumptions. The SST transformation preserves the
exit branch introduced by non-isolated `while` lowering. Query-kind validation
rejects missing loop invariants/measures or incompatible loop proof settings.
No new loop axioms or trusted helper specifications were introduced.

Concrete `u8` iteration tests cover every input pair. Backend fault injection
rejects bad invariants, nondecreasing measures, restrictive domains, and circular
use of a candidate precondition as an invariant. Tests additionally check source
projection, assumption smuggling, termination suppression, cache/output poisoning,
fresh cross-crate proofs, timeout publication barriers, and ordinary caller
enforcement. Source mutations exercise the new projection, recipe, and
termination guards. These checks supplement, rather than replace, the formal
verification targets below.

## Later: formal verification targets

1. **Acceptance component.** Prove that an accepted or exported contract has all
   required successful outcomes for that exact candidate and dependency set.
   Prefer verifying the component used by production. A separate state-machine
   model needs an explicit connection to the implementation before its theorem
   can be claimed for the implementation. State assumptions about solver results,
   fingerprinting, serialization, and filesystem behavior explicitly.
2. **Precision transformations.** Define semantics for the supported intermediate
   language and prove that the generated obligations establish maximality and
   exactness. Include branch reachability, arbitrary valid outputs, and modular
   calls. This is a larger project than checking the acceptance rules.
3. **Independent replay.** Evaluate emitting original bodies, inferred contracts,
   and explicit precision obligations for checking through a separate verifier
   path. Replay can reduce reliance on our driver, but its translation and build
   correspondence still need justification.

The objective is to reduce and justify the added trust boundary. Existing
Verus/rustc semantics, solver correctness, and trusted library specifications
remain assumptions; these tasks do not establish the entire toolchain's
correctness.
