# Verus Infer assurance plan

This tracks assurance work for the implemented prototype. It is a development
plan, not a claim that the new verification machinery is formally verified or
that the potential failure modes below are known bugs. Supported behavior is
documented in the [inference guide](source/docs/guide/src/infer.md).

## Recommended sequence

The focused assurance consolidation, early returns, bounded `Option`/`Result`
propagation, shared slices, unsigned stepped/countdown loops, and exact/interval
fixed-width and platform-sized integer inference are implemented. Retain the
formal verification targets below as separate work. Formalization of the entire
precision checker is not a prerequisite for every feature.

The integer-operation batch adds bitwise/shifts and explicit arithmetic policies.
Every extension must keep source eligibility, typed summaries, fresh ordinary
proofs, and publication gates aligned. Any added standard-library specification is an explicit trusted-model
change requiring a documented audit and dependency fingerprints. The feature
backlog and proposed milestone order are maintained in
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
  or external-body functions were introduced for that array milestone.
- The integer-operation batch adds 36 explicit trusted primitive-method
  specifications for missing checked and saturating methods. The model audit
  below records their semantics, `no_unwind` justification, and limits; ordinary
  vstd verification does not prove these assumed library implementations.

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
mutations, four precision-check mutations, five counted-loop boundary mutations,
two guarded-pattern mutations, and two local-view mutations in temporary copies. Baselines
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
Counted-loop inference introduced cache version 13, candidate identity version 2, and
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

`Result` propagation admits identical scalar/unit error types and a closed set
of 35 lossless primitive integer widenings. Recognition checks the concrete core
conversion implementation and types. Rust's blanket `From<T> for T` returns its
input; the existing trusted `spec_from_blanket_identity` axiom models identity.
For widening, existing `FromSpecImpl` declarations in `std_specs/convert.rs`
characterize the cast; the residual specification connects the returned error to
that conversion through `call_ensures`. Arbitrary user-defined `From` and `Try`
implementations remain ineligible.
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

The unsigned loop families and their checking strategy are specified in
[VERUS_INFER_LOOPS.md](VERUS_INFER_LOOPS.md). The recognizer checks the whole
typed two-local body: constant/immutable parameter accumulator steps, checked
addition/subtraction, and ascending/countdown unit counters. Source proof
annotations and termination-suppression settings are ineligible. Loop-free
inference retains its prior source policy.

The installer adds only binder-free, typed invariant/measure expressions and the
non-isolation proof setting. It projects those fields away and compares the
remaining serialized body with the original. This prevents candidate generation
or cache reuse from changing any executable operation. Candidate identities bind
the installed proof body as well as the original source, actual contract,
configuration, and dependencies. Parameter products and symbolic platform
constants also select Verus's existing nonlinear prover. This proof-only setting
is bound into candidate identity version 3; ordinary and precision queries still
must succeed. Tests verify all other attributes and fields are preserved. Cache
version 17 contains formulas only; annotations and prover choice are regenerated
through the same source-checked boundary on every hit. Metadata schema 12 rejects
older inferred interfaces. No serialized VIR fields change in this extension.

Ordinary soundness includes invariant initialization/preservation, overflow,
termination, and the result relation. Maximality removes the candidate precondition
and keeps the original checked prefix invariants; their initialization,
preservation, and decreasing-measure checks are emitted by normal loop lowering
and are not converted into assumptions. The SST transformation preserves the
exit branch introduced by non-isolated `while` lowering. Query-kind validation
rejects missing loop invariants/measures or incompatible loop proof settings.
No new loop axioms or trusted helper specifications were introduced.

Concrete checked `u8` iteration covers every starting value/count for both
counter directions and accumulator operations, with literal steps 0/2/255 and
parameter steps 0/1/2/3/7/127/128/255. Independent backend fault injection rejects
bad invariants, nondecreasing measures, permissive/restrictive domains, circular
preconditions, weak output relations, and incorrect outputs. Compiler matrices
check all unsigned widths and both pointer models, including zero/full-range
boundaries, aliased step parameters, and symbolic platform constants. Tests
additionally check source projection, assumption smuggling, termination
suppression, cache/output poisoning, fresh cross-crate proofs, timeout publication
barriers, and ordinary caller
enforcement. Source mutations exercise the new projection, recipe, and
termination guards. These checks supplement, rather than replace, the formal
verification targets below.

The unsigned loop-family checkpoint passed 490 targeted Rust tests (two existing
loop tests remain ignored), debug/release verifier builds, all required
proofs for the unsigned-width matrices at both pointer sizes, three-crate Cargo
verification, and compiled native example runs. The 36-case corpus passed all
73 cold/warm/incremental runs in each profile. All 18 deliberate source mutations
were detected by failing tests after their baselines passed. These results are
regression evidence for this checkpoint, not a formal implementation proof.

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

## Platform-sized integers

Exact `usize`/`isize` candidates retain Verus's existing symbolic architecture operations
for `usize::MAX`, `isize::MIN/MAX`, pointer width, and truncating casts. Cache decoding and imported
summary validation admit only closed architecture constants, not arbitrary bound
operators or new axioms. Candidate-cache configuration includes the merged local
and imported architecture. The platform milestone introduced schema 10, which
fingerprints the architecture
together with exported functions; ordinary Verus rejects incompatible fixed widths.
Tests check 32/64-bit arithmetic/casts, portable and width-constrained imports,
cache reproving, and rejection of a poisoned 32-bit candidate with a 64-bit bound.
These are regression safeguards; the new synthesis and plumbing are not formally
verified. The existing seven checked methods use the same closed model allowlist
and fingerprints for both platform types.

Sound intervals run separately at 32 and 64 bits; differing ranges and safety
domains are merged under `arch_word_bits() == 32`. Merging retypes both formulas
and enforces the expanded formula budget. Imported sound-summary validation checks
the closed selector and both interval branches. Numeric endpoints never use the
host width. A conservative 64-bit envelope is used only for optional affine
simplification; executable safety checks and interval transfer use the selected
model. Tests include boundary execution comparisons, cross-crate composition,
malformed selectors/branches, and a well-typed poisoned range that ordinary
soundness rejects. The platform milestone used cache version 15 and metadata
schema 10; the integer-operation batch below updates these versions.

## Integer-operation model audit

The added trusted declarations in `source/vstd/std_specs/num.rs` are generated
for the six unsigned/signed pairs, including `usize`/`isize`:

| Method | Added declarations | Specification justification |
| --- | ---: | --- |
| Unsigned `checked_neg` | 6 | Only zero has a representable unsigned mathematical negation. |
| Signed `checked_neg` | 6 | Every input except `MIN` has a representable negation. |
| Unsigned `checked_sub_signed` | 6 | Compute mathematical `x - y`; return `Some` exactly when it lies in the receiver's unsigned range. This avoids negating a signed `MIN` intermediate. |
| Signed `saturating_add`, `saturating_sub`, `saturating_mul` | 18 | Clamp the mathematical operation to the receiver's `MIN..=MAX` range. |

These are 36 explicit trusted specification additions. They describe total Rust
primitive methods, so they declare `no_unwind`. Their justification relies on
Rust's primitive-method semantics, not on proving the compiler or standard
library implementation. Rebuilding/verifying vstd does not independently prove
an `assume_specification`. Independent small-width execution comparisons and
boundary tests supplement this audit; model and containing-artifact fingerprints
bind their use through caches and inferred interfaces. The declarations retain
the existing `not(verus_verify_core)` guard to avoid duplicate core specifications.

Wrapping add/subtract/multiply, unsigned saturation, unsigned checked addition
of signed values, and signed checked addition/subtraction of unsigned values
reuse existing vstd models. Numeric-model recognition checks the core identity,
receiver-specific vstd proxy module, arity, argument signedness/width, return type,
empty precondition, and non-unwinding policy. No bitwise axioms are added. Bitwise
operators and shifts use Verus's existing AST semantics; normal shifts require
`0 <= amount < bit_width` and left shifts discard high bits. Wrapping summaries
use bounded corrections or modular products matching the existing models;
saturation and checked results retain their distinct policies.


Integer-operation candidates use cache version 16 and metadata schema 11. Typed
summary substitution re-establishes finite operand types before bitwise evaluation;
cache decoding rejects mismatched widths, signedness, operand types, and executable
shift checks in pure formulas. Source shifts require the checked amount policy.
Independent exhaustive `u8`/`i8` tests compare results and safe shift domains with
Rust, including wrap/saturation/mixed-sign/negation boundaries. Compiler matrix
coverage checks all supported widths and architecture tests exercise both pointer
models. Wrong but well-typed cache formulas must fail fresh ordinary or maximality
proofs and suppress caller use and export. Source bodies and the precision/evidence
acceptance machinery retain the existing preservation and publication checks.


## Rust control flow and immutable data transformations

This extension implements exact `let-else`, guarded matches, or-patterns,
primitive integer error widening, owned scalar array outputs, and local shared
slice views. Cache version 18 and metadata schema 13 require inferred libraries
and their dependents to be rebuilt. Candidate identity version 3 is retained;
no serialized VIR fields change. New local view metadata exists only during
candidate generation and never enters installed formulas or exported bodies.

Pattern synthesis evaluates an initializer or match scrutinee once, restores
bindings between arms, preserves first-match order, and joins alternative
bindings under their actual pattern tests. Guards run only after matching and
failed earlier arms. A returning guard cannot fall through; a returning else
branch cannot execute continuation code. Independent concrete checked execution
covers both variant directions and all `u8` values. Unsupported code and proof
assumptions are rejected by whole-source traversal even on unreachable paths.
Verus's existing prohibition on combining an or-pattern and guard is preserved.

Owned arrays retain the existing 0..=64 scalar-element bound. Contracts describe
whole array values, and precision queries still quantify over arbitrary valid
outputs. Ordinary ensures lowering uses extensional array equality; the
arbitrary-output comparison now uses that same equality with the normal boxed
SST encoding, instead of opaque SMT function identity. This includes empty
arrays. The existing `vstd::array::axiom_array_ext_equal` characterizes equality
over every element. Its canonical declaration and containing artifact are
recorded as an implicit trusted dependency of array-valued interfaces. No new
array axiom, external-body declaration, or stronger standard specification is
introduced. Independent SST fault injection rejects weak and wrong array
relations; compiler matrices cover all scalar types, lengths 0/1/2/64, and both
pointer widths.

Local views store a base, start, and end during synthesis. Lengths reduce to
`end - start`, and indexing recursively adds offsets to reads of original
inputs. Callee substitution uses the same typed builders, so views cannot escape
through a scalar operator, cached formula, or returned reference. Range operands
are evaluated in their source order; bounds and subsequent reads remain guarded
by actual reachability. Exhaustive small-array comparisons cover empty, reversed,
out-of-range, nested, and indexed views.

Source recognition validates the complete concrete `[T] as Index<I>` lowering:
canonical core/vstd identities, generic signature and bounds, resolved trait
identity, concrete scalar element/range types, implementation paths, constructor
fields, and call attributes. Only exclusive/open ranges over shared slices are
admitted. It does not authorize arbitrary Index implementations or range values.
The existing standard Index/range/sequence models are trusted in Verus; their
containing vstd artifact is fingerprinted. Source proofs still assert their
ordinary requirements. In maximality queries, only the validated canonical
shared-range calls may use their existing exact bounds requirements as
successful-execution constraints. Both generic types and actual values are
substituted in lowered requirements. Other uninferred callee requirements are
rejected. This expands reliance on these existing standard models to maximality;
it does not formally prove their correspondence with Rust implementations.

The existing Result residual specification still has no `no_unwind` guarantee.
Primitive widening does not strengthen it or introduce a new axiom. As with
identity propagation, ordinary verification cannot derive an unconditional
`no_unwind` promise from that model. Conversion assumptions and supporting
specifications are covered by the residual model's complete vstd artifact hash.

Cross-feature compiler tests combine arrays, patterns, widening, and views
through fresh/warm caches and three crates at both pointer widths. Poisoned
array elements and permissive/restrictive bounds must fail their independent
proofs and remove stale exports. Malformed recipes are discarded and re-inferred.
Precision timeouts must block callers and publication even after ordinary
soundness passes. The runnable example, Cargo workspace and evaluation corpus
exercise these same production paths. This regression evidence supplements the
formal acceptance/precision targets above; it is not a formal implementation proof.

The completed Rust coverage checkpoint passed 512 targeted Rust tests, with two
pre-existing loop tests still ignored; debug/release builds; compiled native
example runs; and debug/release three-crate Cargo verification. Conversion and
array matrices prove all three obligations for 87 functions at each pointer
width in both profiles. All 81 runs across the 40-case evaluation corpus passed
in each profile, and all 22 deliberate mutations were detected after their
baselines passed. The corpus includes expected failures and incomplete proofs;
these counts describe selected regression coverage, not general Rust coverage.
