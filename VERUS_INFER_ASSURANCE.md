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
  a rebuild. This includes crates whose inferred helpers are all private: their
  successful exports retain an inference-aware marker after public-API pruning.
  Early rebuild failures remove marked artifacts. Once local inference is known,
  generation failures also remove an older unmarked ordinary artifact; a legacy
  unmarked artifact cannot be identified as inference-aware before that point.
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

## Goal-directed ordinary caller proofs

The opt-in `--infer-goals` strategy decorates ordinary caller SST only. It never
runs on inferred soundness, maximality or arbitrary-output exactness bodies.
The base contract, input domain, precision policy and import/export gates remain
the existing ones. No serialized VIR fields, trusted models or project axioms
are added by this strategy.

Definition tracking and exact-summary substitution are untrusted proposals.
For a requested scalar assertion, a restricted translator proposes a pure
conditional theorem containing finite-type bounds, nonzero-divisor conditions
and supported caller/branch assumptions. The existing bit-vector backend checks
the complete theorem independently; the same immutable expression is used in
the ordinary query through the existing production check/use pair validator.
The original assertion is retained, so even a stale or wrong substitution still
has to connect to the actual executable values through normal verification.
No guessed equality for a call result is installed as an assumption.

With `--infer-goals`, ordinary exports also wait behind the existing complete
post-verification publication barrier, even when the consumer contains no local
inferred function. Direct export paths are invalidated before parsing; Cargo
paths are invalidated once known. A frontend error at those boundaries or a
failed auxiliary proof cannot leave a stale assisted API.
Legacy ordinary exports without the flag retain their existing behavior.

Auxiliary failure or timeout fails the ordinary caller, including when a later
ordinary query succeeds. Unsupported expressions, exceeded expansion/visit/
request budgets, proof scopes and loop bodies receive no generated theorem.
Definitions are discarded at unsupported calls/control flow and branch joins.
Sound-only contracts are not treated as exact. Successful theorems are local to
this proof; this milestone does not persist proof evidence or cached lemmas.

Compiler regressions cover mask bounds and modulo consequences, multi-layer
composition, conditional requirements, branch-local context, signed variables,
false goals, exhausted request budgets, stale definitions after mutation,
callee safety requirements, sound-only interfaces, and cross-crate consumption
with failed-proof/frontend-error stale-export cleanup. The pinned AlephBFT
evaluation checks identical shared production executable code, unrestricted
input domain, all three exact checks, manual and assisted formulations, and
five semantic mutations. Canonical remainder-based schedule proofs already
pass in the baseline; the observed assistance benefit is an equivalent
bitwise parity formulation, not distributed consensus automation. The pinned
no-default-features build failure is compared to unmodified source and reported
separately from passing native tests.

The new proposal traversal is not formally verified. Conditional-theorem
construction/check-use binding, BV-to-integer correspondence and complete
ordinary query outcome accounting remain implementation obligations. The
unchanged original assertion supplies an additional check on substitutions;
it does not eliminate the need to safeguard that proof plumbing.

## Precision, simplification and application evaluation

The practical-precision milestone uses candidate cache version 19 and inferred
metadata schema 14. Candidate identity version 3 still binds the actual installed
formulas, original executable, proof settings and dependency provenance. This
milestone adds no serialized VIR fields, trusted specifications, axioms or source
proof statements. Its cache/interface policy requires rebuilding inferred crates.

Path refinement, immutable-copy/Boolean facts, modular cast intervals and bounded
bitwise transfers are untrusted proposals. Both pointer widths are analyzed
explicitly. Simplification runs after executable safety extraction and before
contract installation. It uses typed structural comparisons, bounded affine
normalization and pure Boolean reductions; it does not rewrite source operations.
The simplified candidate receives the same independent proofs. A wrong reduction
must fail soundness, maximality or arbitrary-output exactness in exact mode.
Sound mode still claims only the independently proved guarantees and can remain
restrictive or imprecise. Unit comparisons cover concrete small-width arithmetic,
cast cycles, masks, shifts and preservation of earlier overflow checks.

Ordinary integer SMT encoding does not automatically establish all bitwise or
truncating-cast range facts. An SST-only pass therefore proposes bounded pure
conditional lemmas, checks them with the existing bit-vector backend, and uses
that identical complete theorem in the ordinary query. Guessed operand intervals,
valid shift counts, explicit finite-type bounds for every free variable and the
selected architecture are premises. A guessed local/callee range is never added
as an unchecked fact. Unsupported expressions or exhausted hint budgets are
skipped; the ordinary proof must still establish the proposed contract.

Hints are inserted after the independent precision transformation, so maximality
cannot turn a hint assertion into an unchecked assumption. The proof-check and
proof-use nodes share the same immutable expression allocation. A production
pair gate rejects an absent/empty auxiliary check, extra auxiliary assumptions,
or a disconnected used theorem. Preexisting bit-vector proof statements in an
inferred SST body are rejected. Each auxiliary query belongs to its function and
obligation and participates in the existing all-subqueries outcome aggregation;
failure, timeout or missing required evidence prevents publication. The ordinary
query retains every original safety assertion and executable assignment/call.

Generating these hints is untrusted, but the coupling of checked theorems to
normal-query facts, BV/integer translation correspondence and complete outcome
aggregation are part of the verification boundary. This is an additional proof
construction path to safeguard, not a claim that all new code is outside the
trusted implementation. Check/use fault injection and source mutation tests
cover missing auxiliary checks, disconnected facts and identity-gate bypasses.
Compiler tests poison a valid mask interval, force resource exhaustion, check
all supported integer types at both pointer widths with fresh/warm candidates,
and confirm caller/export suppression. Existing acceptance tests exercise a
failed subquery followed by success and every receipt ordering.

Canonical path/reference/proxy helpers are shared across numeric, array, slice
and residual model validators. The helper consolidation retains each model's
existing module, signature, arity, generic, mode and non-unwinding restrictions;
it does not authorize additional models.

The fifteen application kernels and their caller proofs supplement synthetic
feature tests. The 47-case corpus includes successful examples, incorrect caller
properties, unsupported code and intentionally incomplete proofs. Repetition
caches are isolated, warm candidates get fresh proofs, and failed harness runs
are excluded from timing statistics. Cargo's no-edit reuse is reported separately
from actual verification. Body-only edits preserve displayed contracts; a
semantic contract edit rejects the old caller, removes its stale interface, and
passes after the caller is updated. Compiler/model artifact hashes are checked
before and after each invocation, with persistent failure reports on instability.
These are observed-boundary checks, not continuous monitoring or an authenticated
build guarantee. Memory reports the OS waited-child maximum RSS; simultaneous
process-tree peak memory remains future profiling work. Neither measurements nor
mutation detection formally prove this implementation or establish coverage of
arbitrary Rust applications.


The practical-precision checkpoint passed 523 distinct targeted Rust tests (with
2 existing loop tests ignored), both debug/release builds, 11 focused precision
regressions in each profile, the debug three-crate Cargo regression, and native
application execution in both profiles. All 95 runs of the 47-case corpus passed
in each profile. Eight evaluator tests cover repetitions, malformed reports,
measured memory, timeouts, statistics and artifact replacement; all 25 deliberate
source mutations were detected after passing baselines. Repeated release kernel
runs passed 18/18 samples. The controlled Cargo edits passed 15/15 release
scenarios across three repetitions and 5/5 debug scenarios. These counts are
selected regression evidence, not a
formal proof or an estimate of arbitrary Rust coverage.

## Scalar newtype projection and the parent-selection checkpoint

Exact inference now admits available transparent, nongeneric datatypes with one
variant and one executable scalar field, excluding destructors, proxies and
user-defined type invariants. Candidate analysis projects their constructors,
field reads and immutable/shared inputs to scalars. Only lifted, source-typed
formulas are installed on original functions; original bodies, signatures,
function kinds and other noncontract fields remain. Scalar-only functions keep
their existing analysis/installation path. The installer binds actual lifted
formulas and original bodies to candidate identity, and all three ordinary
proof obligations are still required. Projection/lifting is untrusted synthesis;
the unchanged precision construction and publication plumbing remain part of
the verification boundary.

Cache version 20 stores scalar formula proposals, not executable projections or
proof outcomes. Keys bind original typed functions, available scalar-wrapper
definitions and transitive dependencies. Imported contracts are validated after
all imported datatype definitions are available. Metadata schema 15 fingerprints
exported datatype definitions as well as inferred functions and architecture;
dependency fingerprints cover imported definitions. Old interfaces require a
rebuild. Compiler tests cover named/tuple wrappers, shared inputs, signed/Boolean
payloads, full-input quorum callers, unsafe calls, cross-crate composition,
cold/warm proof checks, definition changes, poisoned safety predicates and stale
export removal. Unsupported trait inference reports a diagnostic instead of
installing an invalid implementation-specific precondition.

The AlephBFT parent experiment keeps two source origins explicit. Native probes
execute clean-main production types/operators/collector at the pinned revision
in an isolated clone. A separately recorded runtime correction preserves the
NodeCount API and avoids threshold multiplication overflow. Formal comparison
imports the existing manually verified production collector from the input
working tree through an ordinarily verified scalar adapter. It does not verify
the clean-main generic collector or synthesize its state invariant. Existing
counting induction, frame contracts and loop annotations remain. Removing the
update hint or constructor invariant produces proof failures; missing model/type
support is separately classified. Bounded native exploration and mutation tests
provide concrete runtime evidence, not universal state-space coverage or a
new distributed-protocol theorem.

## Bounded vector view/effect checkpoint

The collection path recognizes closed, direct `Vec` length/read/slot-assignment
templates. Its model catalog validates canonical core/vstd identities, signatures,
generic bounds, modes and contract shapes. Exact read maximality can assume an
index operation's precondition only when that condition is the canonical
`i < vec@.len()` on the same parameters; stronger or different conditions are
rejected by targeted regressions. Ordinary soundness still checks every bound.
Candidate formulas instantiate the existing vstd contracts, and the original
body/signature are installed unchanged. No new trusted specification is added.

Standalone mutable vector templates require sound mode. Their full sequence update relation
supports ordinary frame and counting proofs, but state maximality/exactness are
explicitly not established. The return-only checker rejects mutable-reference
parameters outside the supported record path, and the driver does not submit
precision queries for these standalone templates. The record follow-up below
adds a separate state-aware transformation.

Cache version 21 and metadata schema 16 cover the new summary forms and model
dependencies. Collection candidates bypass scalar cache serialization: templates
regenerate each time and all required proofs run. Imported summaries are checked
against the closed template schema using the full imported model catalog.
Cross-crate tests check reproof, honest precision labels, caller rejection and
failed-export cleanup. Existing metadata remains trusted compiler-generated
evidence, not an independently replayable proof certificate.

A generic counting regression verifies a supplied invariant through the inferred
update contract using checked recursive update/bound lemmas. The AlephBFT runner
uses the same approach in the supplied manual collector and distinguishes its
source from clean main. This checkpoint does not synthesize invariants, prove
induction automatically, or infer general record/collection mutation.

## Supplied counting-invariant assistance

`Seq::count_prefix` and four generic broadcast lemmas are ordinary vstd
specification/proof code. Full vstd verification checks their recursion,
termination, bounds, update accounting, zero and full-count implications. They
introduce no axiom or external-body declaration. A full count implies all prefix
elements match; together with a known nonmatching slot and the vector length
bound this can establish counter-increment safety without an extra assumption.

The `--infer-goals` catalog validates canonical vstd identities, generic types,
modes, broadcast attributes and signatures. Discovery starts from supplied
contracts or explicit assertions, follows at most 32 spec definitions and visits
at most 4,096 source nodes per function. Selection inserts four scoped broadcast
activations into the existing body block. Original executable statements, block
identity, child expressions, goals, contracts and safety assertions remain.
Oversized/unavailable context produces no selection. Inferred functions and vstd
itself are excluded, including while proving the catalog's own lemmas.

This reuses Verus's existing conditional theorem instantiation and trusted-library
boundary. Library proof results are not independently replayed by consumers.
Selection is not evidence: `goal_assistance` is observational JSON, and the
existing complete-proof/export barrier still decides acceptance. No new query
kind, precision claim, serialized AST field or cached proof outcome is added.
Source/model artifact fingerprints already invalidate candidates after a vstd
change; the scalar candidate-cache and interface policy versions stay unchanged.

Tests cover an occupied-slot invariant, captured predicates, prefix-external
updates, zero/full counts, false goals, duplicate occupancy, missing invariants,
wrong updates, scoped source preservation, catalog/budget gates, cross-crate
invariant consumption and failed-export cleanup. Aleph's source-connected
comparison removes its update proof block, all three counting proof definitions
and four counting calls across update/initialization/transition methods.
A separate checked equivalence preserves the original count specification's
meaning; changing that definition must fail. Its reference count and equivalence
proof are experiment-only, while state contracts, loop annotations and the
transition's sequence-equality assertion remain supplied. These checks do not establish
general invariant discovery, arbitrary proof search or new protocol properties.

## Bounded record-state composition and requested-domain checks

The record synthesizer tracks one exclusive mutable root, Copy parameters/locals
and bounded transparent fields. Each assignment changes a symbolic field under
its active path; branches merge field values, early returns disable later effects,
and vector writes use existing Seq updates. Scalar arithmetic contributes guarded
range requirements. Immutable aliases, additional reference parameters, proof
statements, loops, unsupported calls and effectful match/value expressions are
rejected. Source and actual generated formula nodes are bounded independently.
The shared installer preserves the original executable body and binds its typed
signature, actual predicates, datatype definitions and dependencies to identity.

Every generated postcondition contains one final equality per record field.
Vector equalities describe the full sequence; scalar/opaque Copy fields describe
their final values, including unchanged frames. Ordinary soundness checks that
relation against the original body. This initial checkpoint did not establish
state precision; the follow-up below adds independent precision queries.
An ordinary harness supplies the input invariant and desired output invariant;
its call must establish the synthesized safety predicate for every allowed input.
That coverage check is distinct from soundness of a potentially restrictive
candidate. Invariants are never assumed by the synthesizer.

Closed primitive From recognition reuses existing vstd conversion extension
specifications, validating the canonical trait/concrete implementation, proxy,
primitive argument/result types and the approved 35 lossless pairs. User-defined
conversions are excluded. It adds no external-body/no-unwind declarations or
conversion axioms. All supported exact scalar conversions receive the existing
three independent checks; record callers use the same existing library models.

Cache version 22 regenerates record formulas instead of using scalar formula
serialization. Metadata schema 17 validates the closed imported expression
language and complete field coverage, alongside existing candidate-bound proof,
datatype and model/dependency fingerprints. Invalid imported summary diagnostics
are made span-free at the import boundary, where source spans are unavailable;
they reject the interface rather than triggering the driver's span assertion.

Compiler regressions cover the unrelated occupied-slot record, read-after-write,
early returns, signed arithmetic, frame changes, missing accounting, duplicate
counting, source/expansion budgets, unsupported aliases/proof code, all primitive
widenings, cross-crate use, honest precision labels and failed-export cleanup.
The Aleph experiment checks the direct supplied `add` body without wrappers.
A minimal invariant harness and a separate copy of the original behavioral
contract check the inferred relation. An added overflow operation yields a sound
restricted contract but fails whole-valid-domain coverage and a native test.
These are mutation controls, not claims of a newly discovered production bug.
The supplied manual core, its count-equivalence evidence and loop annotations
remain distinct from clean main's generic collector and protocol verification.

Cargo edit measurements observe broad candidate invalidation, including an
independent function, after changing the source. Such source-span/build-provenance
sensitivity is a cache-efficiency follow-up; it does not reuse failed evidence or
weaken proof requirements. The runner records these hits/misses instead of
assuming that a body-only edit must preserve cache identities.


## Independent bounded record-state precision

The record path now supports default exact precision, requiring ordinary
soundness, safe-domain maximality and modeled-output exactness before publication.
Explicit sound mode still requires soundness and reports optional precision
probes honestly, including resource exhaustion. Standalone mutable-vector
summaries retain their previous sound-only policy.

Maximality uses the original verifier-generated executable SST and safety checks,
without assuming the proposed precondition. It evaluates the proposed domain on
parameter entry snapshots, including the initial value behind the mutable root.
Only signature-validated canonical Vec index/index_mut bounds and the already
approved exact callee models may constrain successful calls. Ordinary soundness
continues asserting every safety condition independently.

Exactness introduces an arbitrary legal typed final record and substitutes that
record only for the proposed relation's final value behind the input borrow.
The original body, initial snapshots, arguments and borrow-expiration constraints
remain fixed. The independent comparison enumerates every field from the datatype,
rather than relying on candidate clause coverage. It proves the substituted
relation iff every final observation equals the original execution's observation.
Copy fields use value equality; Vec fields use existing canonical views and
extensional sequence equality. Allocation identity and capacity are outside this
observation model. The return is unit in this subset, so unit equality never
stands in for state exactness. No application invariant, guessed frame axiom,
extra external specification or synthesized safety assumption is introduced.

Candidate-cache version 23 and metadata schema 18 bind this policy change. Record
candidates still regenerate and reprove on every run. Imported exact summaries
require all three candidate-bound outcomes; incomplete required precision blocks
same-crate callers and removes stale exports. No precision-transformed query can
be recorded as ordinary soundness.

Independent SST/backend fault injection rejects overly restrictive preconditions,
loose output ranges, omitted updated fields, omitted unchanged frame fields and
wrong outputs while preserving the ordinary safety query. A resetting-body
control shows why maximality must test entry state rather than updated state.
Opaque record fields are rejected before contract lowering. Compiler coverage
includes integer widths on both pointer models, generic Copy fields, multiple Vec
fields, early returns, read-after-write, cold/warm cross-crate use and optional
versus required precision exhaustion. The Aleph runner uses default exact mode
for the original supplied add body and requires all three checked outcomes in
its semantic controls. Its ordinary invariant harness remains necessary: exact
behavior alone does not assert that the behavior preserves an application goal.

Validation for this checkpoint: 386 targeted Rust tests and 32 evaluation-tool
Python tests pass. The direct record experiment passes all 13 formal scenarios
and five native runs; earlier parents, counting, collection and common-vote
experiments pass 11, 11, 11 and nine formal scenarios respectively. Experiment
artifacts preserve source hashes, raw diagnostics and actual proof outcomes;
the supplied manual collector remains distinct from the clean-main generic API.


## Transparent wrapper calls and bounded borrowed queries

The composition checkpoint extends record observation paths to four projections
and at most sixteen leaves, with up to eight fields per datatype. Eligibility
checks every traversed datatype for availability, transparency at the function's
module, executable fields and absence of custom invariants/destructors/proxies.
Scalar/value-record assignments cover every affected leaf. The independent SST
exactness checker traverses the datatype definitions recursively rather than
using candidate clause coverage as its frame. Fault injection removes changed
and unchanged nested leaves and must fail output exactness.

Static acyclic inferred methods may borrow the current root or a direct nested
record, with bounded value arguments and up to two type parameters. Candidate
composition consumes the callee's candidate-bound precondition and complete
relation. It substitutes initial observations from a single snapshot of the
current symbolic state, computes every new leaf before applying updates and
retains unaffected siblings. Generic substitutions replace parameters once;
callee T -> (caller T, u16) must not substitute that caller T again in enclosing
Option/reference types. Formula and source budgets still apply after expansion.
Source bodies, calls and borrow-resolution operations are retained unchanged.

Exact callers require exact-mode callees. The publication ordering examines all
calls in resolution-expanded bodies, including compiler-generated post-argument
scaffolding, while source eligibility continues rejecting embedded proof code.
The original verifier checks each call. Maximality assumes only the already
checked exact callee requirements at correctly instantiated types; it does not
assume the caller's proposed domain. Imported interfaces and failed/incomplete
callee obligations retain the common publication barrier.

Read-only queries produce typed value equations, including Option<&T>. The only
new library recognition is canonical core Option::as_ref with its existing vstd
proxy, typed signature, bounds and unwind policy. No new trusted specification,
external body, unchecked proof call or executable copy is introduced. Generic
payloads may move without a Copy bound; ordinary Rust/Verus ownership and existing
Drop/unwind contracts govern those operations. Shared references are interpreted
by the existing Verus value model; exactness does not establish pointer addresses.

Cache version 24 and metadata schema 19 cover the new closed summary forms.
Record updates and queries regenerate and reprove, including on warm runs.
Their report-only template flag preserves honest cache/kind labels without
changing proof evidence. Import checks validate complete mutable leaf equations
or a single typed read-only result equation. Cross-crate tests reprove nested
callers after a certified lookup changes behavior and remove stale dependent
exports; precision exhaustion in a generic helper blocks publication and callers.

Aleph's extracted NodeMap lookup and insertion retain their original bodies and
unrestricted generic signatures. Its size probe explicitly replaces derived Into
syntax with a NodeCount constructor; this is an adaptation, not inferred trait
semantics. The supplied collector's storage is wrapped and its update uses the
checked API under the same invariant and separate behavioral oracle. The API is
placed in the harness parent module to make private storage transparent. Those
module/storage adaptations, retained loop annotations and specification evidence
remain explicit; opaque views and a clean-main Unit/Hasher proof are still open.
Native checks compare the adapted API against the original pinned method bodies,
including the original size conversion. Semantic mutations require successful
exact checks for all helpers and the composed update before rejecting caller goals.

Checkpoint validation passes 399 targeted Rust tests, 38 evaluation-tool Python
tests and the standalone wrapper example. Aleph's wrapper suite passes twelve
formal scenarios and seven native runs; prior record, parent, counting, collection
and common-vote suites pass 13, 11, 11, 11 and nine formal scenarios respectively.
The assembled tool/library bundle was rebuilt after missing artifacts prevented
one sweep from starting; those setup failures were not counted as semantic
outcomes. Its standard library then verified 2,107 obligations with zero errors,
and the full guarded experiment sweep was rerun successfully.

## Certified direct sequence views across private boundaries

The direct-view checkpoint recognizes a supplied canonical vstd View trait
implementation with one shared spec parameter. Its type must have exactly one
executable field, a default-allocator Vec, and satisfy the existing bounded record
restrictions. The spec body must consist only of the direct canonical Vec view
call on that field, optionally surrounded by empty blocks. Slices, filters,
additional statements/fields, custom invariants/destructors and external-body
definitions do not receive structural evidence. No view is inferred from names.

The compiler installs `infer_sequence_view` internally after inspecting local
definitions. There is no source attribute for this marker. The record generator
may then express the same full-vector-content equations through the public view.
Integer result/constructor equations preserve mathematical values instead of
introducing implicit modular casts; source casts retain their original semantics,
and original-body proofs still establish the range and exact result.

Independent SST precision selection consults structural view evidence, never the
candidate equations. Normal module pruning removes private datatype fields; the
evidence retains which direct sequence observation represents that entire sole
vector field. Thus an unmodified sibling remains an independent observation, and
a view cannot silently hide another modeled field. This preserves the existing
extensional content model, not allocation addresses or capacity. Maximality and
soundness continue using original executable calls, borrow constraints and the
same proof/publication ordering.

Metadata schema 20 fingerprints marked view declarations, inferred contracts,
datatype definitions and architecture. Import requires the certificate even when
all inferred executable methods were private and only public view evidence remains.
It validates the view signature/shape and any retained public definition. Closed
definitions remain stripped, and clients cannot reveal them. Publication requires
full successful local verification. Cache version 25 includes direct-view evidence
in dependencies; templates regenerate and all required proofs run freshly.
This adds acceptance/observation code requiring the same engineering safeguards
as existing inference, but adds no axiom or trusted standard-library model. The
structural checker and its evidence transport are not themselves formally proven.

Ordinary caller goals may need a checked length/size observation to expose the
representation's length bound hidden by a closed view. The examples and private
Aleph harness state that step explicitly; no new input-domain assumption replaces
it. General representation-lemma discovery is not implemented.

Regressions cover private tuple/named wrappers, unconstrained moved payloads,
multiple updates, untouched sibling collections, application and domain mutations,
lossy views, hidden extra state, closed-body privacy and metadata removal including
view-only exports. The separate-crate Aleph experiment preserves original NodeMap
lookup/insert bodies, records the existing size adaptation, retains the supplied
invariant/count equivalence/oracle, and excludes other lifecycle methods from its
formal scope. Its controls distinguish successful exact helper certification from
failed application goals and unsupported abstract views.

Checkpoint validation passes 408 targeted Rust tests (100 VIR inference/precision
tests and 308 integration tests), 42 inference-tool Python tests and the private-view
example (13 verified obligations). The guarded Aleph sweep matches all 85 expected
formal scenarios, including eighteen separate provider/consumer runs for private
views, and all seven native wrapper controls. The standard library was rebuilt
for the changed VIR layout and verified 2,107 obligations with zero errors. Final
privacy checks require an actual failed assertion for a hidden representation fact
and an explicit private-field access diagnostic, rather than accepting unrelated
frontend errors as evidence.

## Exact ordinary trait getter contracts

The bounded trait profile consumes an authored `result == observation(self)`
equation from a public trait declaration. Both signatures have one shared receiver
and only the implicit Self type parameter. Explicit preconditions, recommendations,
prophetic queries, external proxies/bodies, reference/unit observations and weaker
or additional postconditions are rejected. Default executable bodies are permitted
and verified through ordinary Verus rules, as are implementations. The profile
does not infer trait intent, inspect an implementation to guess a generic result,
or introduce a trusted standard-library model.

The untrusted record generator substitutes the matching specification call for a
getter result, with original shared inputs and bounded type substitutions. Associated
values remain opaque. Only explicit type-equality bounds are normalized; trait/name
identity is preserved, cycles fail within eight rewrites, and runtime payload depth
is separate from the four-level projection-syntax bound. Generic constraints remain
in the original body's proof context. Candidate installation does not change calls,
borrows, drops or executable bodies. No arbitrary precondition is assumed as a
maximal safe domain: eligible getters have none. Soundness, maximality and final-state
exactness still run on the original enclosing executable through the existing backend.

Getter/observation markers are compiler-only attributes. They are checked again on
import, included in dependency keys and never accepted as source annotations.
Metadata schema 21 fingerprints the entire exported VIR API, including ordinary
spec implementations and associated-type/trait definitions. This closes the gap
where a generic observation could resolve to an unmarked ordinary spec definition.
Providers with no local inferred executable can opt into certification via
`--infer-goals`; no-verify and function/module filters are rejected when marked
contracts exist, and the existing post-verification export barrier waits for all
ordinary implementation/default-body obligations. Approved model artifacts are
conservatively bound for these providers too. Existing Verus library and explicit
trusted-specification assumptions remain; these metadata files are not signed proof
objects. Cache version 26 regenerates state templates and rechecks every obligation.

Regressions cover unrelated traits, concrete correct/incorrect implementations,
nested associated equalities, cross-trait name collisions, cyclic equality budgets,
private borrowed tuple lookups, weak/precondition-bearing/recommended/borrowed
profiles, three-crate interface changes, stale exports, missing metadata, legacy
uncertified interfaces and verification filters. Aleph's experiment uses an explicit
reduced Unit/Hasher interface, original creator/round default and coordinate getter
bodies, supplied coordinate/getter specs and a generic bridge to the supplied
collector core. A wrong adapter slot must preserve the minimal invariant while
failing its independent behavior oracle. Other controls check failed provider bodies,
unsupported weak contracts, duplicate accounting, frames and valid-domain coverage.
The structural/profile checker and metadata handling remain engineering components
whose own formal verification is still separately scoped.

Checkpoint validation passes 421 targeted Rust tests (103 VIR inference/precision
tests and 318 integration tests), 48 inference-tool Python tests and the standalone
trait example (four verified obligations). All 101 guarded Aleph scenarios match
their expected outcomes, including sixteen provider/consumer runs for the reduced
trait adapter and the previous eighty-five scenarios. The seven native wrapper
controls also pass. The rebuilt verifier's standard library verifies 2,107
obligations with zero errors. No new native behavior or production-bug discovery
is claimed for the reduced trait adapter.

## Concrete wrapper trait boundaries

Concrete nongeneric From and AddAssign implementations now enter the existing
scalar-wrapper/record synthesis paths. The compiler recognizes canonical core
trait and method identities, concrete argument/result types and the exact Rust
implementation selection. The wrapper must have one supported scalar field,
without a destructor, user invariant, proxy or type parameters. The original
executable retains its trait kind and body. Only candidate analysis rewrites a
resolved call to the selected implementation; installation restores the source
body and binds that body, signature, formulas and dependencies into its identity.
Scalar-projected callee formulas are lifted before record composition.

The implementation's stored candidate predicate does not become a new callable
trait precondition. Well-formedness permits it only on eligible inferred
implementations with compiler-generated candidate identities. SST declaration
lowering retains it as proof input without declaring an implementation requires
predicate. Ordinary body verification continues to use the inherited trait
requirement. In the maximality query, a new assertion proves inherited-requires
iff candidate-requires for unrestricted typed entry states. It precedes all
successful-execution assumptions and is inserted after the original safety
assertions have been transformed. The query then establishes the proposed
domain's necessity from the original body. This joins ordinary soundness with
maximality without silently changing the trait contract or assuming a handwritten
requirement is maximal. From's inherited requirement is unrestricted, so a partial
implementation cannot invent a restricted domain. All three outcomes remain
required; sound mode is outside this profile.

The canonical blanket Into bridge uses the existing vstd specification forwarding
to From. Recognition checks the core/vstd identities, signature, selected blanket
implementation and the concrete checked From implementation in its impl paths.
User Into implementations do not acquire conversion semantics. Resolved wrapper
callee requirements can constrain successful executions in maximality only after
the ordinary dependency/publication barrier has certified that callee's exact
checks, including the new boundary assertion. Existing standard-library model
assumptions remain; the new feature adds no assumed application model. Approved
model artifacts are conservatively bound for wrapper providers as well as getter
providers. Metadata schema 22 and candidate-cache version 27 reject older inference
policies, and the whole exported VIR API continues to bind ordinary extension
definitions and their dependencies.

The impl-level native annotation generates ordinary extension definitions for a
direct constructor/projection conversion or one tuple-field addition assignment.
The shortcut uses public open definitions and requires a public wrapper/field;
arbitrary derives and more complex implementations need separate support or
supplied ordinary models. Generated declarations also remain in stub erasure:
omitting their spec impls shifts later Rust implementation identities in compiled
interfaces. A compiled native-provider/consumer/application regression checks this
alongside the ordinary method-level three-crate path. Stable Rust erases ghost
definitions, and the example executes unchanged native conversions/increments.

Backend fault injection checks that false, weaker and restrictive trait boundaries
cannot pass the entry equivalence assertion. Integration controls separate
ordinary soundness from maximality, reject wrong models and partial conversions,
check signed/unsigned ranges and primitive rhs inputs, and verify that consistently
changing operator behavior can pass exact checks while failing application intent.
Cross-crate controls reject stale dependencies, missing certificates and failed
provider publication. The new Aleph variant retains original NodeMap size/get/
insert bodies and uses a NodeCount counter in the supplied generic collector. It
verifies explicit wrapper adapters, not the native derives or the full original
collector. Its supplied models/invariant/oracles and the prior source adaptations
remain recorded. The new compiler recognition and transformation code remains
within the separately documented engineering assurance/formalization workstream.

Checkpoint validation passes 437 targeted Rust tests (331 integration, 104 VIR
inference/precision, and two macro tests) and 52 inference-tool Python tests.
All 122 guarded Aleph formal scenarios match their expected outcomes, including
21 wrapper-operator provider/consumer runs and the previous 101 scenarios. The
seven native Aleph wrapper controls pass. The new standalone example verifies 18
obligations with five exact inferred functions, compiles, and executes its native
conversion/increment checks. The rebuilt standard library verifies 2,107
obligations with zero errors.
