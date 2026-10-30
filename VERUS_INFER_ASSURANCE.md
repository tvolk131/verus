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
  in the scalar path retain their original pointer; owned-value bodies are checked
  for structural identity. Numeric and collection loops receive only pure proof
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

## Pointwise mutation traversal checkpoint

The borrowed-record prefix profile uses metadata schema **36** and candidate
cache version **42**. Rebuild inferred providers and dependents after upgrading.
It uses the existing `Seq::new`, indexing, update and vector specifications;
there are no new standard-library axioms or external-body contracts.

Generated loop annotations refer to entry state with explicit `VarAt(Pre)`
snapshots, rather than the bookkeeping-only `Old` node. Inserted expression and
place identifiers are fresh. The shared body projection gate retains every
executable operation and admits only pure annotations and proof settings.
Constructor metadata is extended only in generated spec positions; original
executable constructor checks remain intact. Ordinary checking still proves
induction, source bounds/arithmetic safety and termination. Function- and
loop-level termination bypasses are rejected.

The maximality query retains the same loop-lowering rules. Its visited-prefix
safety witness starts empty and must be inductive after the proposed precondition
is removed. At successful exit it establishes the entire visited domain; it is
not an assumed copy of the candidate precondition. Full-state output exactness
and all existing publication gates remain mandatory. Compiler regressions cover
empty prefixes, signed arithmetic, index-dependent values, aliased slot updates,
checked error preservation, unsupported shapes and cross-crate composition.
False callers and native API mutations are rejected with healthy helper proofs.
These checks are evidence, not a formal proof of the inference implementation.

## Checked binary wrapper and derive boundaries

Policy 39 and candidate version 45 add exact concrete scalar-wrapper binary
operators and explicit opt-in checking of native derived implementations. Generated
extension definitions are untrusted candidates. The real executable remains the
ordinary body; all three exact checks and inherited-domain equality are required.
The inferred arithmetic follows the actual body, including nonstandard Add bodies.
Primitive UFCS arithmetic uses canonical core identities and existing vstd
operator contracts, concrete equal integer types and matching call signatures.
The successful-execution precision rule is limited to that closed primitive
boundary. No vstd axiom or new assumed function contract is added.

Every generated derive request must match a same-owner, same-signature exact
implementation before inference can return, including zero-target crates. This
prevents a generated extension from becoming usable when its derive failed to
provide a checked implementation. Request markers grant no evidence. Regressions
reject missing targets, wrong bodies, vacuous/changed inherited domains, unsafe
calls and false application goals. Additional checks cover all integer widths,
shared copied receivers, native erasure, repeated provider/client builds and
withheld failed interfaces. These are safeguards, not a formal implementation
proof. The broader Aleph adoption v1 charter was still active at this checkpoint;
its completed acceptance audit is recorded at the end of this document.

Policy 40 and candidate version 46 extend explicit derive requests to PartialEq,
PartialOrd and Ord. The native eq body is an exact target. Ordering-enum methods
are ordinary checked implementations of complete generated trait definitions;
they receive no soundness/maximality/exactness certificate of their own. The
request validator requires a real, same-owner and same-signature body before
the zero-inference early return. An Ord-only provider is still an inference
interface: filters/no-verify cannot publish it, and failed ordinary bodies cannot
publish it. Imported interfaces bind the entire API and library-model artifacts.

Candidate projection of inherited Boolean comparisons is limited to the
canonical core/vstd boundary, concrete primitive-integer wrappers and a matching
validated derive request. It preserves custom explicit operator meanings.
Ordinary executable calls and all precision obligations remain unchanged.
Verification dependency collection retains inferred comparison providers and
computed argument calls, including inherited ne. Regressions cover actual Rust
derives with and without Ord delegation, copied shared receivers, private record
fields, reversed/missing/external bodies, false client goals, zero-target provider
metadata, repeated builds and stale/withheld exports. No axiom or assumed function
contract is added. These safeguards do not formally prove the implementation.

## Demanded map projections and native fold triggers

Fold observation assistance also recognizes an authored sequence-map equation as
an elementwise projection request. It enables the existing checked sparse/fold
lemmas in that caller's scope; it adds no facts to inferred contracts and no new
axioms. Unrequested callers remain unassisted. Native fold readiness quantifiers
now carry a pure iterator-entry trigger so entry-independent safety conditions do
not require author trigger hints. The annotation purity gate recursively checks
trigger terms and rejects hidden assumptions. Mutated projections and false
mapping goals must still fail ordinary verification with healthy helper proofs.

## Complete tagged shared returns

Policy 41 and candidate version 47 strengthen shallow Option/Result shared
projections. A bounded scan can propose one pure immutable input/field projection
shared by all direct successful constructors. Its private spec definition is
ordinary checked vocabulary, never an assumed accessor. Public field projections
can have public bodies; private fields retain restricted body visibility. The
original executable must prove the resulting identity equation.

Returned payload candidates preserve both complete logical-value identity and
the existing content equations. Summary decoding checks both and composes both
across forwarding and imports. Cloning/mutation does not inherit the original
object-identity equation. Independently, precision enumerates shared payloads from
the return datatype and compares complete values, including through tag guards;
it does not select an observation from the candidate's clauses or helper marker.
Equal collection views cannot substitute a different returned object. Existing
Rust borrow checking, ordinary body proofs, all exact obligations, full-crate
publication and dependency metadata remain mandatory. No new axioms are added.

## Ordinary native range-loop proof assistance

Metadata policy **38** and candidate cache version **44** accompany this
checkpoint. Rebuild inference providers and dependents. No library axiom or
assumed helper contract is added.

The untrusted pass proposes pure current-state/prefix invariants from authored
postconditions for one native unsigned integer range. It retains the complete
iterator lowering, its invariants and its original decreasing measure. Its
projection validator independently removes only the inserted invariant prefix
and restores loop isolation, then compares every serialized remaining body field
with the original. It rejects changed source code, source invariant deletion,
changed termination, termination bypasses and effectful/assumed annotations.
Ordinary Verus must prove each generated invariant, source safety condition,
helper precondition and original postcondition. Ordinary functions receive no
maximal-safe-domain or output-exactness certification.

Compiler regressions cover exclusive/inclusive ranges, empty and singleton
cases, inclusive machine maximum, frames, changed values/slots/range endpoints,
skipped work, early breaks and insufficient input contracts. Cross-crate tests
repeat provider builds, check ordinary caller composition, reject verification
filters and withhold failed-provider interfaces. Native Aleph ingestion composes
checked mutable-return framing with collector updates and history growth;
source mutants must fail despite healthy exact helper certificates. A trigger
on the collector-index observation supports the authored quantified progress
contract; the pass does not invent its meaning.

A separate frontend regression preserves Rust disambiguators in unnamed-const
paths. Codec derive wrappers previously collapsed unrelated method identities.
Both independent implementations are checked, and a wrong body in either is
rejected. This fix does not authorize serializer assumptions or derive exclusions.
The native Signed getter proposal remains unapplied pending that separate
boundary decision. These checks are evidence, not a formal proof of the compiler
or transformer.

## Ordinary growth-loop proof assistance

Metadata schema **37** and candidate cache version **43** accompany the growth
checkpoint. This adds no serialized AST fields, library axioms or assumed helper
contracts. Providers and dependents with inference metadata must be rebuilt.

Proposals come from the ordinary function's supplied postconditions, including
an exact final-length goal. They are obligations, not new entry assumptions.
The original preconditions, postconditions, executable operations and source
checks are preserved. Generated state predicates, frames, prefix facts, length
bounds and the decreasing measure receive ordinary induction and termination
checks. A production projection gate checks every remaining serialized body
field, and a pure-expression gate rejects effects and assumptions in inserted
annotations. Fresh spec identifiers and constructor bookkeeping do not alter
executable constructor checks.

Assisted ordinary functions are reported separately from exact inference
candidates. They receive no maximality/output-exactness claim. Full-crate checks
are required; no-verify/filter attempts and failed providers cannot publish an
assisted interface. Backend negative cases cover wrong transitions, overwritten
prefixes/frames, insufficient round bounds, non-progressing loops and incorrect
mutable-borrow selection. Existing handwritten loop proofs are preserved.
Application invariant meanings, public abstraction contracts and input policies
remain authored. These safeguards are not a formal proof of the new transformer.

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

## Native Clone and implementation-model checkpoint

The subsequent native collector checkpoint uses metadata schema **26** and
candidate cache version **31**. Rebuild providers after upgrading. It adds concrete
checked PartialEq/PartialOrd adapters and bounded effectful record matches. Ordered
guards control each arm's safety requirements, field updates, and surviving paths;
unselected arms cannot impose their effects or overflow restrictions. Inherited
trait input requirements are still checked independently by maximality.

`clone_via_view` also emits an ordinary broadcast proof of the supplied sequence
view's machine-length bound. This theorem is proved from the actual Vec field;
changing the view to an oversized sequence fails verification. Goal assistance
selects the theorem only after observing a certified direct view in caller goals
or called preconditions/specifications. It enables a checked lemma and inserts no
assumption. The traversal has fixed budgets and produces no selection on exhaustion.
Recognized view-bound-only providers require full-crate verification and proof
metadata before export. A same-named theorem with a precondition is not selected.

The new native collector experiment retains all four pinned executable bodies,
including u16 round arithmetic, guarded matches and original error policy. Supplied
application invariants and behavior goals prove lifecycle, counts, exact slot
updates, frames and selection. All four candidates pass soundness, maximality and
exactness. Changed counters/resets/errors and omitted round/node requirements must
fail named ordinary caller goals while candidate precision remains proved. It
does not silently fix or claim discovery of the previously known u16 terminal-round
overflow. Unit/Hasher codec and lock boundaries remain explicit.

Native collector validation: **8 expected formal outcomes**, including five
semantic negative controls, and **15 original native collector tests** passed.
The new cross-crate bound test and the 21 record / 16 wrapper integration tests
passed. These checkpoint counts supplement the earlier regression evidence below.

Native derived Clone through a supplied direct view introduces a proposed ordinary
postcondition, not an axiom. Rust's original derive body remains executable and
must prove it. A generated public spec relation avoids exposing private fields.
An exact inferred caller may propose contents equality only as a candidate; its
three independent obligations remain mandatory. `CloneIdentity` is an explicit
proof trait whose implementations establish identity from vstd's reflexive
`cloned` relation. Copy is never interpreted as a Clone identity law. Negative
tests cover insufficient Copy bounds, nonidentity component clones, false supplied
relations, malformed helper signatures and dishonest value-law implementations.

Rust's builtin tuple Clone instances (arity 1–4) map to bounded vstd models with
one `cloned` clause per field. The helper bodies themselves verify by fieldwise
cloning. Frontend validation restricts their signatures, generic bounds and exact
weak contract shapes, preventing a stronger helper specification from silently
describing the builtin shim. The mapping remains part of the trusted compiler
translation. The pinned Rust 1.99.1 compiler's `rustc_mir_transform/src/shim.rs`
implements tuple Clone through `build_clone_shim`, `tuple_like_shim` and field
clone calls. The reflexive relation also permits copy specializations. Custom
nonidentity Clone and function-pointer regressions exercise the weak relation.

Native-Clone-only providers also require full-crate verification and delayed
certified export. Certificates bind the whole exported API and the vstd artifact,
including tuple relations and CloneIdentity laws. Tests reject partial/no-verify,
false relations, unrelated failed goals and missing proof metadata. Opaque boxed
returns use independently certified view observations after SST representation
lowering; they do not fall back to private object identity.

This checkpoint uses metadata schema 25 and candidate cache version 30. Rebuild
vstd and dependent `.vir` artifacts. Templates never store proof outcomes.

The u64 `to_ne_bytes` specification is a new trusted standard-library declaration,
expressed using the existing little-endian byte model and reversal for big-endian
targets. It records the compile target's endian choice in the vstd artifact; it
does not assert a platform-independent encoding. External-signature checking
accepts differing closed array-length expressions only when Rust's const
normalizer evaluates both to the same usize. Unequal lengths remain rejected.

The new RwLock `read_copy` and `replace` methods are ordinary verified bodies:
they acquire and explicitly release handles, and preserve the predicate configured
at lock construction. Reads promise predicate membership for the observed value,
not a stable snapshot after release. No lock assumption or Drop contract is added.

The source-connected Aleph Unit experiment isolates SCALE in a separately
compiled trusted codec model. The target implementation and concrete collector
caller verify with `--no-cheating`; the codec model deliberately declares
deterministic encoding of a supplied complete immutable view and omission of
the cache. Its source/artifact hashes are recorded and guarded. Neither SCALE's
algorithm nor that bridge's correspondence to the native derive is formally
proved. Native runtime tests exercise the real codec/hash/cache implementation.
Parking_lot operations are explicitly adapted to the verified RwLock operations.
The original checkpoint substituted a blocking read for nonblocking clone; the
subsequent finite-read implementation below removes that substitution. This is
not native RAII/Drop verification or a proof of parking_lot equivalence. Permissive cache,
wrong hasher input, wrong session, and dropped-clone-payload controls must fail
semantically, and failed formal providers must not export certified interfaces.

The non-Copy checkpoint adds the ordinary proof trait `CloneView`, separate from
CloneIdentity. Implementations prove equality of their DeepView from the native
reflexive cloned relation. Primitive, Option, Vec and String implementations verify;
the Vec proof composes checked element laws and existing clone/sequence models.
An abstractly unchanged value may have a different representation. Tests accept
that case and reject dishonest laws or an attempted law from plain Clone alone.
No Clone identity or view-preservation axiom is added.

The FullUnit interface now uses Option<D::V> with D:CloneView and no Copy bound.
Generic and concrete Vec<u8> goals prove clone/encoding/hash consistency. The
trusted codec bridge must factor through an **encoding-complete** deep view:
CloneView by itself does not establish that arbitrary application abstractions
retain all encoded data. This remains a declared model boundary.

RwLock::try_acquire_read extracts one iteration of the verified read protocol:
it returns None after failed contention/CAS/capacity checks, and abandons any
pending reader token before failure. It has no loop or termination suppression.
Blocking acquire_read retries that checked operation. try_read_copy releases
successful handles; try_read_or requires a predicate-compatible fallback. No
new lock axiom is added, and the contract does not promise success, fairness or
a stable snapshot. Formal fallback/false-success controls and an erased runtime
test cover writer contention, successful reads and balanced failure/release paths.

Validation: vstd **2,155 verified / 0 errors**; **11 Clone integration tests** and
**3 try-read integration tests** passed. The Unit runner passed its **7 formal
outcomes** and **3 native runs**, including the non-Copy Vec payload and contended
clone cases. The native wrong-clone control still fails and restoration passes.

## Bounded finite collection queries

The collection-query checkpoint uses metadata schema **27** and candidate cache
version **32**. Rebuild providers and dependent interfaces. Public precondition-free
trait observations may now receive the shared receiver and up to two scalar
arguments. Certification requires identical arity/types and unchanged positional
arguments in the single result equation. Changed or permuted arguments are rejected.
Source argument arithmetic is evaluated with its original safety checks, and each
dynamic observation remains tied to the checked trait implementation and complete
provider certificate. This is composition of supplied meanings, not inferred
application semantics or a name-based library model.

Read-only record Result queries without a collection payload use the existing
structured-value checker. It observes the complete tag and both guarded payloads;
unit success and scalar errors remain exact. The actual method body receives
ordinary soundness and independent maximality/exactness checks. Wrong error and
threshold controls remain precisely inferred but fail ordinary application goals.

`vstd::iter` adds ordinary checked implementations of sparse slice traversal,
finite count, filtered count, first-match search, collection, and sparse mapping.
No Iterator::count/filter_map axiom or external body is added. Generic consumers
call next directly, using its existing Iterator contract. A **nonprophetic** metric
proves termination; prophetic remaining sequences describe contents and counting
bounds only. Predicates are supplied and connected to executable callbacks by
ordinary checked contracts. Search returns an earliest witness or complete absence.
Sparse traversal preserves indices and mapping preserves holes. Allocation and
standard-library assumptions remain the existing Verus models.

The Aleph query experiment deliberately adapts the native iterator chains to these
checked APIs and uses a reduced Result<(),u16> policy for the quorum/future-round
stage. This earlier adapter does not establish the full native ControlHash
validator, payload error enums, original enumerate/filter_map/count chains or
construction-loop inference. The typed-validation checkpoint below advances
noninitial errors and propagation. Application observations and
callback specifications remain visible inputs. Provider callback/projection
mutations must fail verification and remove exports; changed consumer policies
retain exact candidates but fail caller goals. Native and adapted corpora exercise
the same 1,530 cases; runtime completion markers prevent empty binaries from
counting as evidence. Runtime-only entrypoints use verus_keep_ghost_body rather
than verus_keep_ghost because the compiled library retains ghost stubs.

Collection-query checkpoint validation (2026-10-28): release compiler/macros rebuilt;
vstd **2,171 verified / 0 errors**, and an isolated no_std/no_alloc vstd build
**1,981 verified / 0 errors**. **615 Rust tests** passed: 378 inference integration
tests, 6 finite-iterator/try-read tests, 118 standard/iterator/native-model tests,
107 VIR tests, 5 macro tests and the tuple-shim validator. **64 Python checks**
passed. All Aleph runners passed **174 expected formal outcomes** and **14 native
experiment runs**; the two adapted query runs each completed all 1,530 cases.
The finite-read example separately verified, compiled and emitted its runtime
completion marker after contention/release checks. Expected negative controls
are included in outcome counts; they are semantic rejections, not successful
proofs of the deliberately wrong implementations.

## Data-bearing errors and native validation ordering

The typed-validation checkpoint uses metadata schema **28** and candidate cache
version **33**. Rebuild providers and dependent interfaces. Bounded executable
value enums admit two to eight variants, at most four fields per variant, two
type parameters, bounded value nesting and at most sixteen observations. Generic
and associated payload values retain their existing opaque value semantics.
Known heap and recursive payload shapes are excluded. Independent precision
construction enumerates every variant tag and every field under its own tag;
it does not take the candidate's equations as its observation list. A direct
checker fault test rejects missing tags and missing guarded payloads.

Result `?` recognition still checks the complete compiler lowering, including
the residual conversion and early return. This extension accepts bounded value
payloads with identity error conversion, alongside the existing primitive
lossless conversions; it does not accept arbitrary From implementations. Error
exits retain their original reachability, so an earlier Err bypasses later
overflow requirements. Complete constructor values and enum matches compose
through checked imported summaries. Candidate-only tag simplification and removal
of duplicated match tests keep Aleph's chain inside the unchanged 4,096-node
formula budget. Original executable bodies remain authoritative for all proofs.

Ordinary nongeneric shared-receiver scalar queries can supply the exact equation
`result == observation(receiver, scalar_args...)`, with up to two scalar args.
They require no preconditions, unchanged positional arguments, matching types,
ordinary verified bodies and full provider certification. Immutable original
objects are tracked separately from their sequence views for these observations;
mutable locals and newly constructed collections cannot reuse an old object's
identity. A private direct View remains supplied. Cold/warm cross-crate tests
exercise composition, false query implementations and failed-provider export
gates. Closed observations remain opaque outside their defining module.

Literal-message Rust assertions are recognized only through the complete
canonical panic_fmt(from_str(literal)) capsule. Both the candidate builder and
the independent maximality transformer recognize the existing core model's
requires-false contract. No panic or formatting axiom is added. Normal soundness
must prove the failing branch unreachable under the inferred safety predicate.
Dynamic formatting and proof-only assertions remain outside this profile.

`eval_verus_infer_alephbft_validation.py` retains the pinned six-variant Error<H>
declaration, own-parent lookup, n_members body, runtime assertion and three-stage
noninitial `?` ordering. Only the filtered-count and future-parent pipelines are
replaced with explicit ordinary checked collection queries. Supplied goals prove
success equivalence and the complete first-error policy, including exact payloads.
Wrong payload/tag/priority, ignored errors, a wrong query callback and missing
round/index bounds fail ordinary proofs while inferred candidates retain all
three proved properties. Failed providers must remove their exported interface.
The same 7,704-case corpus executes against adapted and untouched native methods;
an explicit marker and a separate caught zero-round panic prevent empty-runtime
or ghost-only runs from counting as evidence.

This does not verify the initial-round encoding/hash checks, the complete native
ControlHash API or native iterator chains. In particular, the isolated native
future-parent leaf can return Ok at round zero on an empty collection, whereas
the adapter eagerly computes round - 1. The complete noninitial path asserts a
positive round before this call, so its checked scope agrees. Native enumerate,
filter_map and count need separate contracts and overflow/termination treatment;
arbitrary Iterator implementations can override these methods. Do not introduce
a blanket count equation based only on next. Those implementation adapters remain
the next collection milestone, rather than enlarging the trusted model set here.

Typed-validation checkpoint validation (2026-10-28): **627 targeted Rust tests**
passed (389 inference integration, 6 finite-iterator/try-read, 118 standard/native
model, 108 VIR, 5 macro unit and 1 tuple-shim checks), plus **66 Python checks**.
All Aleph experiments passed **184 expected formal outcomes** and **16 native
experiment runs**, including the two 7,704-case typed-validation corpora. The
library rebuild passed **2,171 verified / 0 errors** and the isolated no_std/no_alloc
build passed **1,981 verified / 0 errors**. Plain cargo macro doctests retain the
pre-existing 23 failures from Verus snippets and partial code examples under
rustdoc; the five applicable macro unit tests pass with --lib.

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
Legacy ordinary exports without the flag or certified inference dependencies
retain their existing behavior. Certified consumers now use the publication
barrier described under checked mutable selectors below.

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

## Owned collection construction and lifecycle

Candidate-cache version 28 and inference metadata schema 23 cover compiler-only
owned-value markers, the new closed summary forms and model fingerprints. Owned
construction templates regenerate every time; they never load proof outcomes or
owned formulas from the persistent candidate cache.

The untrusted interpreter proposes complete contents equations for vectors and
certified sole-vector wrappers, every scalar/transparent-record field, and Option
tags with guarded payload equations. It composes exact helper interfaces rather
than executing their bodies. Bounded fill and indexed Copy loops receive two pure
invariants and one decreasing measure. Source validation rejects user annotations
and termination bypasses before traversing their specification calls. The
installer restores the actual original executable body, including trait syntax
that candidate analysis temporarily normalized, and reapplies only checked loop
annotations. A serialized source-projection comparison rejects changed executable
steps or proof assumptions; candidate identity binds the installed annotations.

Ordinary verification checks the proposed contract, loop invariant initialization
and preservation, bounds, overflow and termination. Safe-domain maximality retains
the existing independent SST transformation. Fill expressions that add partial
operations inside the repeated body are explicitly unsupported: the current
transformation cannot recover a necessary first-iteration safety condition at the
loop exit. Computing checked arithmetic before the loop is supported. Independent
output exactness traverses the declared returned type rather than the candidate
clauses, introduces an arbitrary typed output, and checks the relation iff every
modeled observation equals the actual return. Observations cover full sequence
contents, all scalar fields, Option tag and the present payload; capacity and
allocation identity are not observed. No application invariant is assumed.

Imported summaries must reconstruct the same complete output tree, conjunction
shape and Option guards. Their clauses are limited to pure values and recognized
specification/view models; executable helper calls, proof blocks and assumptions
are rejected. The existing proof receipts, full-provider API/model fingerprints,
verification order, architecture matching and full-crate export gates also apply.
Generic owned callees enter the existing certified-precondition safety barrier;
they cannot supply a maximal domain without their independent required proofs.

The new model recognition reuses existing vstd declarations for Vec new/capacity,
push, repeat and clone, and Seq empty/new/take/push. Canonical identities, arity,
type arguments, parameter/result types and generic library bounds are checked.
There are **no new library axioms, external bodies or no-unwind declarations**.
These ordinary vstd Vec specifications remain trusted Rust-library models.
Generic Clone is not assumed to preserve value, even for a Copy type. Repeat of
`None` uses the existing Option clone model; opaque payload copies use an explicit
Copy loop. Structural primitive/Option/tuple clone candidates still require the
backend to establish their contents equation; unsupported clone laws fail closed.

Fault injection bypasses synthesis and checks arbitrary owned records against an
independent executable return: missing siblings, loose relations and wrong fields
cannot gain output exactness. Additional tests cover source-projection changes,
assumptions, observation budgets, unsupported loops/clones, all three exact proof
receipts, warm cross-crate composition through private views, changed providers,
missing inference metadata and failed-provider/consumer export invalidation.
The standalone example also compiles and runs natively.

The Aleph lifecycle experiment retains supplied direct Vec new/next executable
bodies while removing method contracts, loop annotations and local proof blocks.
Its separate private provider retains NodeMap's original with_size/size/get/insert
bodies and adds an explicit checked Copy adapter; this does not verify the native
generic Clone derive. Both variants prove construction/add/advance/add using the
supplied application invariant and counting model. Semantic controls retain every
exact proof while wrong initial/advanced rounds, reset and terminal policies fail
the ordinary behavior goals; an unnecessary executable overflow restriction fails
the unrestricted construction goal. Reduced traits, wrapper adapters, threshold
correction and the widened-round/Option terminal design remain explicit inputs.

Validation on 2026-10-27: release compiler build; vstd **2,107 verified / 0 errors**;
**348 integration tests** across ten inference targets, **106 VIR tests** and
**2 builtin macro tests** (**456 Rust tests** total); **57 Python checks**.
The lifecycle runner passed **17 expected formal outcomes** and all nine earlier
Aleph runners passed their **122 expected outcomes**, for **139** total. The earlier
wrapper native target passed its seven tests. The new owned-value example had
**11 verified / 0 errors**, compiled, and passed its native construction/copy/fill/
terminal checks. Expected semantic rejections are included in the outcome counts;
they are not successful proofs of those deliberately incorrect implementations.

## Borrowed Result selection queries

Candidate version 29 and metadata schema 24 bind the expanded closed language.
The structured interpreter represents Result tags and both guarded payloads,
including scalar values and bounded nullary error enums. Shared collection results
use complete sequence observations. Immutable Option/tuple matching and early
value returns preserve branch order and guard every proposed safety condition.
The existing canonical Option::as_ref and primitive From models are reused; shared
reference decorations left after source dereference are normalized only after
Rust's original type/borrow checking and model signature checks.

The independent precision traversal enumerates the Result tag and both guarded
payloads from the declared type. A direct SST/Z3 fault test bypasses synthesis:
removing the tag, success payload or error payload is sound but cannot pass
exactness. Wrong application errors/contents and missing bounds remain separate
ordinary caller failures despite exact candidates. Imports validate the full
relation/guards and pure model language; warm cross-private-crate queries reprove
all required checks, and changed error policies invalidate downstream publication.
No new library assumptions or external bodies are introduced by this checkpoint.

The supplied Aleph query now infers its contract. Its direct version substitutes
a Vec borrow for a slice; the private version spells storage operations through
NodeMap, projects the wrapper counter and widens the borrowed round. Branch order
and explicit bounds-error policy are retained. The private ghost-only caller
observes size and conditional get so their checked facts about hidden contents
and payload types are available. Application invariants, counting equivalence,
Copy adapter and reduced Unit/Hasher interface remain explicit inputs. Native
Clone and FullUnit/cache integration are subsequent stages, not claimed here.

Selection checkpoint validation (2026-10-28): 355 integration tests across eleven
inference targets and 107 backend tests passed. The 61 inference-tool Python
checks passed. The direct/private Aleph selection runner passed all 11 expected
formal outcomes; its negative cases retained all three exact checks. The selection
example verified 5 obligations, compiled and passed its native success/error/index
checks. Broader Clone/cache work continues as a separate cohesive change.


### Native iterator query and callback checkpoint (October 28, 2026)

Native finite iterator views are ordinary verified provider contracts, with full
crate verification and artifact binding required before export. Candidate-only
cache/import preparation rechecks their bounded signatures. A supplied view
connects a lazy native prefix, successful exhaustion and the implementation's
count relation to a supplied full sequence; it does not invent collection meaning.

Callback synthesis is restricted to pure scalar/field Boolean expressions. Every
generated callback retains its original executable body. Lowering separately
records its original safety statements without candidate requirements or
postconditions. Independent maximality converts those safety checks to
successful-execution assumptions, then ASSERTS the candidate precondition, the
actual output equation, and exclusion of the opposite Boolean output. These checks
run without the enclosing function's precondition. Fault injection rejects a
restrictive callback domain, a false actual equation and an equation admitting
both Boolean outputs. Closure requirement equivalence for generated callbacks is
supported by these independent checks; handwritten closure semantics is unchanged.

Lazy any rejects item-dependent partial callbacks and earlier partial filters.
Uniform callback safety applies only to a nonempty full sequence. This avoids
claiming maximality for an all-items safety requirement when native search might
stop before an unsafe later item. Semantic controls include empty round-zero
search, where subtraction is never evaluated, and nonempty round-zero search,
where it panics.

Unlike the earlier checked vstd::iter implementations, this checkpoint ADDS
trusted native-library models. Native count support is closed over known slice,
Map, Filter, Enumerate and FilterMap types. Map/Enumerate delegate to inner count;
Filter/FilterMap count consumes callbacks. Counting therefore does not
unconditionally imply stream completion. Custom Iterator count/try_fold overrides
receive no new law. Any's added completion fact is guarded by the native-family
marker. The precompiled Rust-library implementations are not formally verified.

Two core verifier fixes retain a constructed generic closure's Rust type-validity
fact and preserve iterator-law state across generated for-loop invariants. Neither
changes source executable code. These facts are needed to use existing standard
iterator contracts under generic and opaque iterator types.


### Total native construction fold checkpoint (October 28, 2026)

This bounded profile composes a checked complete sequence-view mutation into one
local owned accumulator and summarizes the original native for loop with a fold.
Only pure proof invariants are added. The decoration gate compares the complete
original body after removing the three generated invariants and restoring its
original isolation setting. Original iterator invariants, measures, patterns and
runtime statements must match exactly.

The profile always proposes an unrestricted domain. The precision-query gate
rejects a restricted native-loop candidate, missing loop annotations or missing
decreasing measures. Its maximality query retains original safety checks and
callee requirement assertions; it does not erase them as successful-execution
assumptions. Ordinary soundness still proves each original write safe, invariants
initialized/preserved, and the loop terminating. The independent arbitrary-output
query checks the complete returned view. Backend fault injection confirms that an
unsafe body fails both ordinary verification and this maximality path. A shifted
index fails semantic verification and exports no contract.

Generated proof nodes use separate AstIds, so they cannot receive an original
runtime node's borrow-resolution assumptions. The resolution pass extends
constructor-mode metadata only within loop specifications, where constructors
have Spec mode. Existing executable constructor checks and borrow-resolution data
are preserved. A unit regression checks this separation. The source body and its
runtime erasure are not rewritten.

Mutation composition accepts a direct single-vector wrapper and exactly one
complete final-view equation, a local mutable receiver, unit return, and value
arguments. Its original inferred helper requires all three proofs. The query
interface additionally accepts bounded generic type/associated-value results,
with unchanged type arguments and input arguments and full provider verification.
No new mutation axiom or application-name rule is introduced.

Aleph's construction comparison has sixteen exact inferred methods and twelve
formal outcomes. Wrong projected rounds and a wrong hash input reach ordinary
semantic proof failures; the latter also checks that a failing ordinary query
implementation blocks publication. The real native constructor/codec/hasher runs
378 maps and an empty map, then the same 7,704 typed validation cases. The proof
runtime uses supplied states because the declared codec bridge is proof-only.
Initial-round validation, arbitrary/partial-domain folds, codec implementation
verification and automatic abstraction discovery remain outside this checkpoint.


Validation for these checkpoints: 615 Rust unit/integration tests and 68 Python
tests pass. Standard vstd verifies 2,177 obligations with zero errors; no_std/no_alloc
verifies 1,987 with zero errors. The native-construction experiment records twelve
expected formal outcomes, sixteen complete exact contracts, both 7,704-case runtime
corpora and 378 native constructed maps plus an empty map. `cargo test -p vir --lib`
passes all 111 unit tests. The broader command's existing instructional pseudocode
doctests do not compile and were not counted as passing tests.

### Demanded fold observations and full validation checkpoint (October 28, 2026)

Indexed-update steps whose index and value do not depend on the accumulator use
the ordinary checked `seq_update::scatter` definition. Original runtime loops are
unchanged; their synthesized prefix invariants and all three original-body checks
prove this equation. The library proves length, last-write reads, sparse projection,
empty sparse views and fold congruence by induction. These are proof functions,
not new axioms. General folds retain the previous representation and restrictions.

With `--infer-goals`, a bounded deterministic traversal of requested contracts and
assertions selects scoped checked theorems from that catalog. Callee contracts can
expose a fold equation but cannot create an observation request. No contract is
strengthened eagerly, no original goal or safety check is replaced, and no theorem
conclusion is assumed. Budget exhaustion or an unavailable theorem leaves ordinary
verification responsible for the unchanged goal. Reports identify selected lemmas.
Private collection view bounds and sparse factory meanings remain supplied checked
interfaces; the catalog does not discover abstractions or prove arbitrary loops.

An exact unary collection query can specialize its visible observation body with
a constructed receiver's certified sequence view. Other receiver uses and hidden
observation bodies are rejected. Its runtime implementation is not inlined or
rewritten. Pure nonprophetic spec vocabulary is retained and included in candidate
keys, but never approved as executable source calls. The original query call and
both precision obligations must prove the proposed specialization. Existing exact
query provider certification and full-verification publication requirements remain.

Generic same-type eq/ne calls use the existing standard PartialEq specification.
Logical identity is only a candidate. The checked `PartialEqIdentity` law supplies
both enabled equality semantics and the per-value identity theorem; Rust's Eq marker
does not imply either. Library implementations for booleans, integers and arrays are
verified, including zero-length arrays. A weak generic Eq bound fails semantic proof
and publishes no interface. No comparison or hash-identity axiom is added.

Aleph now infers the original initial-round branch as well as construction and
noninitial validation. The original public dispatcher is an ordinary top-down goal
with a supplied conditional index bound and complete typed-error policy. Fully
expanding its inferred typed-result equation still exceeds the existing formula
budget; that budget was not increased. The caller proves input-map size, occupied
and absent slot projection, and initial acceptance without handwritten fold
induction. Application intent is supplied, not inferred. Original constructor,
factory, callback, hash-helper and validation executable operations are retained.

The declared codec model, verified-lock adaptation, corrected quorum arithmetic,
and existing trusted native iterator models remain explicit boundaries. Formal
initial hash behavior depends on the declared encoding model; the separate native
corpus executes the real SCALE codec and hasher. No claim about encoding correctness,
hash collision resistance, the full production crate or protocol correctness is
introduced. This checkpoint uses metadata schema/policy 31 and candidate cache 37.

Validation: 515 Rust integration tests, 111 VIR unit tests and 69 Python tests
pass. Standard vstd verifies 2,204 obligations; no_std/no_alloc verifies 2,014,
both with zero errors. The final pinned Aleph run records sixteen expected formal
outcomes, seventeen complete exact contracts, both 7,704-case noninitial runtime
corpora, 378 native constructed maps plus an empty map, and 254 additional native
initial-validation cases. Cold/warm runs enable fold assistance in five callers.

### Compact exact result composition checkpoint (October 28, 2026)

Bounded value-only `Result` summaries now retain a complete typed value equation.
Branches and helper calls compose that equation directly. For identity `?`
propagation with the same complete return type, the failure exit preserves the
input value under its error tag. Different success types retain ordinary guarded
error projections and reconstruction. Results containing collections still use
complete guarded view observations; this change does not grant a collection an
unproved value abstraction.

This changes only untrusted candidate representation. Original executable bodies,
safety checks, candidate-bound evidence, independent maximality/output-exactness
queries and publication barriers are unchanged. No new axiom, trusted executable
model, proof-status shortcut or formula-budget increase is introduced. General
expression sharing, arbitrary-depth composition and broader `map_err`/conversion
support remain separate work. Metadata schema/policy is 32 and candidate cache is
38; rebuild inferred providers and their dependents after upgrading.

Aleph's original public `ControlHash::validate` dispatcher now infers its safety
domain and complete typed output without a handwritten method contract. Its cold
and warm reports contain 516 requirement nodes and 573 output-equation nodes,
within the unchanged 4,096-node budget. Separate supplied caller goals still check
the complete intended policy and conditional creator-index bound. A new missing
public caller-bound control fails ordinary precondition verification. Existing
wrong errors, payloads, priority, callback and constructor controls still fail
semantically and publish no interface. These are deliberate mutations, not newly
discovered production bugs.

Cross-crate cold/warm tests cover complete generic errors, first-error priority,
non-unit success values, success-type changes and error paths bypassing a later
overflow. A successful path without its overflow bound fails. Reordering an
independently verified provider's errors fails the consumer's supplied policy and
blocks consumer publication. A collection-bearing result regression retains the
exact success sequence view and guarded error value.

Validation: the rebuilt verifier passes 517 integration tests across twenty
binaries, 111 VIR unit tests and 69 inference-tool Python tests. Rebuilt standard
vstd verifies 2,204 obligations with zero errors. The guarded pinned Aleph run
passes seventeen expected formal outcomes, eighteen complete exact contracts,
both 7,704-case noninitial runtime corpora, 378 native constructor maps plus an
empty map, and 254 native initial-validation cases. Supplied factory/view and
application meanings, checked equality law, codec/lock/native-library boundaries
and corrected quorum arithmetic remain as documented in the previous checkpoint.
The next outer parent validator, full crate, encoding implementation and protocol
properties are not claimed by this checkpoint.

### Typed error mapping and parent-validation checkpoint (October 28, 2026)

The existing standard `Result::map_err` specification now composes direct immutable
callbacks with complete bounded output relations. The original executable closure
is checked independently for safety, necessity of its inferred precondition, and
`Q(other) <==> observations(other, actual)` for an arbitrary complete output.
Observation selection comes from the datatype, independently of the candidate.
Callback requirements constrain the outer function only under the Err tag. No
new standard-library axiom or executable model is added for `map_err`; its existing
specification remains the library boundary. Mutation, arbitrary indirect closures,
recursive enums and general conversion-trait inference remain outside this profile.

Collection-bearing enums retain every tag and every guarded payload observation.
The bounded profile admits up to eight variants, four fields per variant, and four
type parameters; the nested profile has a fixed depth and 64-observation cap. The
expanded formula budget remains 4,096. Native structural Clone calls propose
field/view preservation only when paired with a checked ordinary Clone relation.
A weak relation cannot justify the mapped callback's equation and fails proof.
Private sole-vector wrappers keep their independently certified direct View;
no arbitrary Clone implementation is treated as identity.

Direct shared object projections now use generated ordinary closed spec definitions
for complete value identity. Each definition retains a validated source projection,
passes normal Verus well-formedness and mode checks, and has its body restricted to
the source owner. The executable getter still proves soundness, maximality and
complete-value exactness. Imported clients use the closed declaration rather than
private representation selectors. Invariant-bearing objects are atomic complete
values; preserving one does not infer its invariant, cache behavior or Clone law.
The normal exported-crate fingerprint binds the generated declarations, alongside
the existing candidate-bound evidence and dependency checks.

Generated callback contracts can contain constructors. Resolution bookkeeping marks
only those pure specification constructors as Spec; pre-existing executable
constructor checks and borrow metadata are preserved. A unit regression covers
both generated regions and conflicting existing constructor entries. Pure summary
substitution now takes immutable engine/path state, preventing a contract field or
conditional from changing the executable path condition. This is a compiler
safeguard, not a formal verification of the transformation itself.

Aleph's original `Validator::validate_unit_parents` and supporting source accessors
now infer in exact mode. A separate supplied caller checks member-count error
priority, complete PreUnit views in both error forms, the exact wrapped
ControlHash error, complete Signed identity on success, and the conditional creator
bound. The input is already checked Signed; reduced unused marker bounds and a
supplied Keychain count interface are explicit boundaries. Signing/authentication,
the outer session/max-round validator, codec correctness and full protocol
correctness are not claimed. No additional production bug is claimed from the
synthetic negative controls.

Metadata schema/policy is 33 and candidate cache is 39. Rebuild inferred providers
and dependents after upgrading. Cross-crate regressions cover cold/warm typed
mapping, captured structural clones and collection payloads, conditional arithmetic
safety, exact nested enum tags, changed providers, private borrowed chains and
opaque complete-object forwarding. Failed providers and consumers publish no
interface.

Validation for this checkpoint: 522 Rust integration tests across twenty binaries,
111 VIR unit tests and 70 inference-tool Python tests pass. Rebuilt standard vstd
verifies 2,204 obligations with zero errors. The guarded pinned Aleph run records
22 expected formal outcomes and 29 complete exact contracts, both 7,704-case
noninitial runtime corpora, 378 native constructed maps plus an empty map, 254
native initial-validation cases, and all six original validator tests. The parent
summary contains 715 requirement nodes and 1,707 output-equation nodes in both
cold and warm runs, within the unchanged 4,096-node formula limit.

### Ordinary native factory proof assistance (October 28, 2026)

`#[verus_verify(iterator_via_view(output_observation))]` requests an ordinary
finite-native-view contract for a supported shared factory. The output observation
is an explicit author choice. This is not maximal-domain or exact whole-iterator
inference. Consumers continue to use the existing independent soundness,
maximality and output-exactness checks.

The frontend scaffold evaluates the original receiver once, retains the original
adapter call and direct closure argument, and emits a normal proof call. Keeping
the closure in its original argument position preserves Rust's contextual type
inference. Executable erasure returns the original item. The generic proof calls
receive ordinary Rust typing, Verus mode, precondition and body checks; the
inference engine does not directly install a typed theorem instance into AIR.
Structural macro tests cover evaluation multiplicity, adapter retention, generic
where clauses and unsupported source shapes. This source transformation remains
part of the compiler/macro implementation; it has not itself been formally proved.

The new iterator_view module contains five ordinary verified proofs. Sparse
filter-map and map combinators connect existing native constructor postconditions
to a supplied output sequence. Their premises connect the actual callback to its
complete output equation, and require callback safety over the relevant full
input. The sparse-entry origin lemma exposes index provenance for downstream
bounds. These proofs use the existing native iterator specifications and add no
axiom or external body. Both standard and allocation-free vstd builds check them.

Behind --infer-goals, bounded immutable callback candidates are regenerated for
certified factory interfaces. Their original bodies prove the proposed equations
with an unrestricted callback precondition. Partial or mutating callback candidates
fail closed. Canonical value-only Option propagation is supported, including the
original sparse iterator's as_ref()? expression. Callback candidate evaluation now
uses a separate function return type, exit list and path state, preventing an
inner propagation from being interpreted as an enclosing function return. Generated
specification-constructor bookkeeping extends the existing protected region to
ordinary factory callback contracts, preserving executable checks.

Factory assistance reports callback proposals separately from inference receipts.
The existing full-crate verification and export barrier applies to factory
interfaces. Cross-crate cold/warm tests cover non-Copy borrowed elements, mapped
records, count/search consumers, wrong indices, wrong mapped values, an incorrect
supplied view, missing assistance, unchecked builds, and partial/mutating callbacks.
No failed factory or dependent consumer may publish an interface.

The guarded Aleph --factory-proofs run retains the complete original NodeMap::iter
and ControlHash::parents signatures and bodies. Each has one view annotation;
there are no handwritten callback contracts, factory proof blocks or named-local
adaptations in this fixture. Both ordinary factory proofs succeed, along with the
same 29 exact inferred contracts. Twenty-four expected formal outcomes include
named factory failures for deliberately wrong indices and mapped rounds. Existing
application policies, collection/output views, value laws, corrected quorum,
codec/lock/native-library boundaries and checked-Signed input remain explicit.

Validation: 525 Rust integration tests across twenty binaries, 111 VIR unit tests,
seven builtin-macro unit tests and 71 inference-tool Python tests pass. Standard
vstd verifies 2,209 obligations; no_std/no_alloc verifies 2,019, both with zero
errors. Aleph retains both 7,704-case noninitial corpora, 378 native constructor
maps plus an empty map, 254 initial-validation cases, and six original validator
tests. The broad Cargo macro command additionally attempts existing documentation
fragments as plain Rust doctests: 23 fail on Verus syntax and five are ignored.
Their documentation text is unchanged from the preceding commit; the focused
macro --lib test command passes. These doctest failures are not counted as passing
verification checks.

Metadata schema/policy is 34 and candidate cache is 40. Rebuild inferred providers
and dependents after upgrading. Richer chains, partial-domain factory assistance,
arbitrary iterators and automatic abstraction discovery remain separate work.

## Outer validator and structured query checkpoint (2026-10-28)

Bounded ordinary exact static queries now admit an owned/shared datatype receiver,
shared context, up to four type parameters and structured returned values. The
single result equation must call a matching pure observation with unchanged
parameters. Implementations remain ordinarily verified; external bodies,
preconditions, mutable parameters and prophetic observations are excluded from
this recognition rule. Complete immutable value provenance is retained separately
from collection observations. Resolved trait queries carry the selected impl paths
into their pure observation calls. Certified exports bind these ordinary APIs and
their observation definitions to the existing full-verification barrier.

Canonical Result residual recognition now accepts a bounded exact inferred From
implementation only when Rust selected that concrete impl. Bounded type matching
instantiates its actual input/output types; the conversion becomes an explicit
inference dependency. The owned interpreter applies its checked summary on the
error path without evaluating the operand twice. All three ordinary checks and
the independent inherited-trait-domain equivalence remain required. Partial,
unmarked and unchecked conversions cannot supply this path. The generic standard
residual's existing unwind limitation is unchanged. No new standard-library
model, axiom or external body is introduced.

Generic trait returns also exposed a precision-lowering panic: boxed output
observations were explicitly unboxed before being passed through ordinary poly
lowering, which inserted boxing operations again. The observation builder now
keeps the pre-poly expressions and lets the existing lowering handle each
projection. It still derives every tag and guarded payload independently of the
candidate. The new generic collection-error tests cover this path and rejected
changed payloads. Optional vstd FromSpec whole-object laws remain ordinary
obligations. The fixture sets obeys_from_spec to false for its structured
conversion, using the independently checked inferred contract instead of adding
a stronger whole-vector/object-identity law.

View-bound assistance traverses demands breadth first and preserves already
discovered checked lemmas at the existing 32-function/4096-expression limits.
This only enables ordinary verified theorems; it introduces no assumed bound.
A cross-crate regression uses a private view and a longer unrelated spec chain
to check that the bound theorem remains selected and that the caller fails
without assistance.

Aleph --outer-validation retains the complete original validate_unit executable
body, with no handwritten proof blocks. It checks an authored contract directly:
authentication rejection precedes session rejection, which precedes a strictly
greater-than-maximum round rejection, followed by the existing member/parent
policy. Signature errors retain the complete original unchecked value, cloned
FullUnit errors retain their supplied logical views, and success returns the
complete original Signed value. The constructor and original signature-error
From body add two exact summaries (31 total); the two factories and outer policy
are ordinary checked proofs. The outer method has no inferred exactness receipt
or maximal-domain claim. Cache-insensitive abstract output inference remains open.

The authentication predicate, Keychain method contract and ordinary checked
UncheckedSigned::check adapter are explicit supplied interfaces. They bind the
logical unit, creator and original signature. They do not verify the native
hash/as_ref/index/verify chain or the signature implementation's security.
Representation drift checks, codec/lock/native-iterator boundaries, corrected
quorum arithmetic and supplied application meanings remain in place. Nine new
formal controls target authentication, signature conversion, session/payload,
maximum round, outer priority, ignored parent errors and input coverage. These
are deliberate mutations, not discovered production bugs.

The final guarded Aleph run has 33 expected formal outcomes, with no exports from
failed inference-enabled providers. Both 7,704-case noninitial corpora, 378 native
constructor maps plus empty, 254 initial-validation cases and the six original
validator tests remain green. An added native corpus checks 95 outer-validation
combinations and one expected out-of-domain creator panic. The mock signature
checker does not imply a creator-range check, so no such law is assumed. The
native corpus runs the original SCALE/hash/signature wrappers, separately from
the supplied formal authentication adapter.

Validation: 531 Rust integration tests across twenty binaries, 111 VIR unit tests
and 72 inference-tool Python tests pass. Standard vstd verifies 2,209 obligations;
no_std/no_alloc verifies 2,019, both with zero errors. Final guarded evidence is
recorded in /tmp/verus-infer-outer/final-delivery/results.json.

Metadata schema/policy is now 35 and candidate cache is 41. Rebuild inferred
providers and dependents after upgrading.

### Native Cargo collector accounting checkpoint (October 29)

The follow-up to the independent clean Aleph pilot verifies the collector inside
its actual Cargo workspace on branch `verus-infer-clean-pilot`, based on local
main f35c7bba6d628bcafdf0d65dd471cc6f868ec034. This parent-led integration is not
part of the blind trial. The Aleph worktree's
verification/units_collector/verify_native.py checks native source directly;
NATIVE.md documents reproduction, scope and explicit boundaries.

The actual NodeMap methods, two native NodeCount operation implementations,
collector constructor and private state-update helper receive seven exact
contracts with all 21 required obligations proved. Ordinary contracts compose
these with checked Unit defaults and the native FullUnit coordinate law. The
selected first-party crates report 41 verification results and zero errors. A
concrete coverage wrapper discharges the observation-law premise for FullUnit;
a composed native-input proof covers replay and advancement. Other unverified
Unit implementations default to a false observation-law predicate.

The accounting theorem does not assume a functional hash equation. Native hash
outputs are opaque. A separate ghost-only library-specification crate declares
trusted parking_lot read/write/dereference signatures and the codec callback
interface, with no cache invariant, read/write snapshot identity, encoded-fields
equation or hash equation. The public wrapper preserves existing candidates on
ignored arrivals and frames other slots, while the exact state helper records
the hash it is passed. The adapter pilot's stronger stable hash observation has
not been established for the native cache. Native SignedUnit observation laws,
cache consistency, generic clone hash identity, prospective_parents and the rest
of the consensus implementation remain outside this checkpoint.

Five actual-source controls (increment, equal-round replacement, count reset,
coordinate getter, and observation-law premise) and a no-assistance control fail
ordinary application verification after all inferred helpers pass. Failed root
crates publish no inference interface. Each first-party proof crate is selected
as a --no-cheating root; vstd and the explicit trusted library boundary are not.
This does not opt every Rust item in those crates into verification.

The source audit checks retained NodeMap and FullUnit getter bodies and the
collector split's original getter order/state-update statements. Native Rust
1.85.1 passes all 180 consensus tests, 10 crypto tests, and one crypto doc test;
Verus uses its own pinned Rust 1.99.1. Root regressions pass 281 trait/stdlib tests
plus an ordinary Cargo erasure test. This pass fixed the primitive-target derive
scanner panic, multiple-name external_derive parsing, and iterator proof imports
that failed to erase in normal Rust builds. No candidate, precision, or metadata
policy was relaxed, and no new inference axiom was added.
## Opaque record components and authored input projections

Readonly record inference now retains an unrelated private first-party component
as a complete logical value. A constructor can preserve an original immutable
by-value input moved directly into such a field. The independent SST precision
checker selects whole-value equality for opaque components from module/type
information, without consulting the candidate's clauses. Existing certified
direct sequence views retain their previous content observations. Opaque-root
mutation remains unsupported without a supported state interface.

A bounded untrusted projection pass can express input equations through an
authored public scalar observation or tuple-valued View. Generated borrow
definitions are expanded only to match such an authored projection; they do not
become abstraction choices. Original-body soundness and both independent
precision checks remain required. The pass does not rewrite output schemas or
callback-bound predicates, assume View injectivity, or create executable models.
Unavailable private projections fail before module pruning. Generic type
substitution, complete-object arguments and imported pure vocabulary remain
validated and bound to the published candidate/dependency evidence.

Cross-crate regressions exercise composite private payloads, scalar and shared
queries, whole moved inputs, repeated provider/caller builds, wrong input/getter
mutants, missing observations, and full-verification/export gates. Independent
backend fault injection removes all opaque representation fields and rejects a
missing return relation and an equality to the wrong input. The matching native
PreUnit checkpoint verifies all five original bodies and retains the opaque
Hasher/codec boundary. Policy 42 / candidate cache 48 invalidates older profiles.

The checkpoint passes 462 compiler regressions across all 18 inference suites
and 114 backend assurance tests. Native evidence covers 140 proof results,
31 exact helpers, all 93 certificates, three rejected implementation mutations
and 40 pinned creation runtime tests. The runtime sources are unchanged by
this inference checkpoint. General constructor-result abstraction and the
remaining adoption-v1 scaffold removal/audit/replay remain open.

## Constructor tuple views and complete opaque results

A bounded record containing an opaque component can propose its constructor
result through an authored tuple View. The inference generator additionally
proposes checked uniqueness at the actual result: a peer with the same view must
equal that result as a complete logical value. This clause is an ordinary
verified postcondition, not a trusted View-injectivity axiom. It lets exact
compiled callers recover complete output identity without seeing private fields.
Original SST soundness/maximality/exactness checking is unchanged.

Selection uses the datatype's owner rather than the caller's module, so local
and imported schemas agree. Other fields must fit the value-record profile;
direct sequence-wrapper profiles keep their previous semantics. Projection
expansion and output-view evaluation are bounded candidate generation. Imported
summaries retain canonical coverage checks, pure vocabulary and candidate-bound
evidence. Trigger-bearing imported formulas are traversed fully, including their
trigger terms; the fault test rejects an assumption hidden inside a trigger.

Repeated provider/caller tests verify exact forwarding across private interfaces.
A wrong downstream payload requirement fails after healthy helper proofs. A
lossy tuple view fails required constructor proofs before callers run and removes
any stale interface. The checkpoint passes 463 compiler tests in all 18 inference
suites and 114 backend assurance tests. Policy 43 / candidate cache 49 invalidates
older output schemas. The native Creator policy has 145 proof results, 31 exact
helpers and 93 certificates; five real-source mutants are rejected and 41 pinned
creation runtime tests pass. No new library specification or hash law was added.

## Checked cloned-field transport and hidden range predicates

Behind --infer-goals, a bounded pass proposes pointwise invariant-transport facts
for an ordinary record transition using existing checked native Clone relations.
It matches an authored unary-predicate implication and clones of input fields,
expands only visible pure spec definitions within fixed budgets, and retains
source guards when proposing the corresponding indexed property on cloned
fields. These are ordinary assertions, never assumptions or replacement contracts.
The original function's executable code, return and all contract/metadata fields
must survive an independent production projection check byte-for-byte.

Assertion predicates are independently checked for pure spec vocabulary, including
trigger terms and temporary-place children. A backend fault test rejects assumed
assertions, removed/reordered executable statements, changed code or return,
changed input contracts, and an assumption hidden in a trigger. Fresh expression
and place IDs keep generated assertions distinct from the source expressions.

For generated range-prefix invariants, one unambiguous final-state sequence-length
bound can supply an internal sequence-index trigger when the authored pointwise
predicate hides the access. Existing source triggers and native iterator
scaffolding are preserved. The existing production range projection checker
continues to enforce executable code, source invariants and termination measures.

No public contract schema, independent precision checker, library axiom or trust
boundary changed; policy 43 / candidate cache 49 remain valid. Reports expose
pointwise_assistance separately from exact inference. Aleph's remaining local
collector proof block and all collector/Creator/NodeMap trigger hints are removed;
the mutable-selector helper contract and final adoption audit/replay remain open.

This checkpoint passes all 466 compiler regressions in the 18 inference suites
and 115 backend tests. Native evidence retains 145 proof results, 31 exact helpers,
93 certificates and 41 pinned creation runtime tests. Five actual-source mutants
(advancement round, invalid parent round, retained count, skipped ingestion and
truncated ingestion range) fail with healthy exact helpers and withheld exports.

## Checked mutable selectors and ordinary dependency consumers

With --infer-goals, a bounded ordinary helper consisting of one checked
unit-returning preparation call followed by a direct mutable vector-field element
borrow can receive generated identity/frame/length postconditions. The template
uses the existing canonical vector borrow model, stable preparation clauses and
its checked preserved-prefix relation. A receiver predicate established by
preparation and required on entry can be proposed after borrow expiration,
conditional on the future returned value satisfying the original semantic return
clauses. No predicate, borrow law or restoration premise is assumed.

All proposed postconditions undergo ordinary original-body verification. An
independent production projection check permits only appended pure postconditions
and an internal publication marker; it preserves code, input conditions, original
output goals, bounds, modes and termination metadata byte-for-byte. Its fault test
rejects modified inputs/body/goals, forged markers, assumptions (including hidden
trigger effects), and undeclared variables. Formula/clause budgets and the direct
record/vector/call shape bound generation. Contract Old snapshots retain normal
Verus semantics; loop-only entry variables are not emitted into global contracts.

The new internal marker requires full verification and inference-aware imported
interface validation, even in a provider with zero exact-inference candidates.
Public contracts and imported model artifacts remain covered by the existing
whole-API fingerprint. The generator is rerun rather than caching proof evidence;
ordinary assisted functions do not claim maximality or exactness.

Certified dependencies now carry the same full-verification/publication barrier
through ordinary consumers without --infer-goals. An inference-aware artifact
header requires the proof/dependency trailer after private-helper pruning or an
ordinary intermediate API. Failed imports, failed callers and filtered rebuilds
remove old certified artifacts and publish no replacement. A three-crate test
checks ordinary forwarding, full-check gates and removal of an intermediate
trailer. Ordinary unmarked library payloads retain legacy trailing-data behavior;
trusted-model fingerprints still bind every artifact byte and changed model
artifacts still require independent reproof.

Metadata schema 38 / policy 44 and candidate cache 50 invalidate older formats.
Libraries must be rebuilt for the changed artifact header and internal function
marker. Aleph's six authored mutable-selector mechanical clauses are removed;
the original helper body and semantic input/return contract are unchanged. No
library axiom, hash/codec assumption or Signed serializer exclusion was added.

The selector checkpoint passes 470 compiler regressions across all 18 inference
suites, 116 backend tests and the driver metadata test. The matching library
format was rebuilt with 2,209 verified vstd results and no errors. Native evidence
retains 145 proof results, all 31 exact helpers and 93 certificates, 41 pinned
creation runtime tests, six rejected actual-source mutants and a rejected
no-assistance control. Full input-inventory audit and fresh adoption replay
remain required before declaring adoption v1 complete.

## Readonly vector queries over opaque first-party elements

Readonly record eligibility can retain an otherwise supported vector's hidden
first-party elements as complete logical values. Private collection fields inside
an element are not expanded. Mutable record eligibility is unchanged; no field,
clone or mutation equation follows from the new element eligibility. Length
queries use canonical sequence-view vocabulary so the existing checked input
projection pass can express them through an authored public view rather than leak
a private physical vector field into an imported contract.

Independent SST precision checking is unchanged. Soundness, maximality and
exactness must prove the generated scalar query against its original body. The
generic cross-crate regression retains private collection-bearing elements and
private outer storage, repeats provider builds, checks a caller over the supplied
view and a wider cast domain, rejects wrong output and empty-input callers, and
rejects the corresponding mutable opaque-element record profile.

Aleph's original current_round body now uses exact inference; its manual getter
domain/equation was removed during the acceptance audit. Inference recovers the
nonempty-vector safety condition and the actual u16 cast without assuming the
Creator invariant. The native application proves its stronger untruncated round
meaning from the authored validity predicate. Policy 45 / candidate cache 51
invalidates older query templates; metadata schema remains 38.

## Completed Aleph adoption v1 acceptance

The finite charter in VERUS_INFER_ADOPTION_V1.md is complete. This is a bounded
native adoption result, not a claim of inference completeness or a formal proof
of the inference implementation. The final audit inventories 51 fixed source
implementations, all authored semantic inputs and existing boundaries, including
outside-slice methods in the same files. It finds no local supporting proof
blocks, executable-loop annotations, manual trigger hints or mechanical getter /
mutable-selector identity/frame contracts in the fixed implementation.

A source audit using the actual Rust/Verus parser rejects five deliberate
annotation/exclusion regressions. Compiler reports reconcile the checked source
closure and required generated derives. Source hashes show the native dependency
boundary, signature module and Cargo.lock are unchanged from the frozen baseline.
Only the explicitly approved ConstraintError generated Display and Error methods
are newly unverified; the enum, selector and error ordering are checked. The
Signed serializer proposal remains unapplied and no assumed contract is added.

Fresh pinned baseline trees plus audited overlays build a new release engine,
verify vstd (2,209 results / zero errors), and verify the native Cargo roots with
147 results, 32 exact helpers and all 96 soundness/maximality/exactness
certificates. The final engine passes 471 compiler regressions across 18 suites,
116 backend tests and the driver metadata test. The native pinned toolchain
passes 197 unit tests plus one doctest, including the 41 creation tests.

All 50 native source mutations and the no-assistance control fail as expected
with healthy exact-helper checks and failed interfaces withheld. The restored
source verifies again after the negatives. Repeated setup reproduces both source
trees and upgrade patch hashes; rustc dependency files exclude old extracted proof
adapters. The replay explicitly audits the authored semantic inputs already in
the pinned adoption baseline and removes its old scaffolding before verification;
it does not claim a pristine upstream-master adoption experiment.

The Aleph pilot retains ACCEPTANCE.md, EVIDENCE.json, SOURCE_INVENTORY.json,
fixed-closure.json, compilation-inputs.json and portable audit/replay tools under
verification/units_collector. They record commands, source/artifact identities,
positive receipts, mutation results and manual-input classifications. Opaque
hash/codec/lock limitations remain explicit, including the frozen restriction on
initial constructor/validator hash agreement. No requirement is weakened to close
the milestone.

## Native adoption integration checkpoint

Policy 46 / candidate cache 52 adds opt-in checked reverse scalar conversion
derives. `infer_derive(Into)` proposes a projection specification for the actual
`From<Wrapper> for Integer` emitted by the derive. Canonical trait identity,
concrete source/destination types, the local nongeneric wrapper owner and its
single primitive-integer field bind the request to its implementation. The
original executable body, inherited domain and all three exact obligations must
verify. A request without its checked target fails even when there are no other
inference candidates. A forward request alone neither checks nor infers an
otherwise ignored reverse derive. Explicit `external_derive(Into)` excludes the
reverse source derive and grants no callable specification.

Primitive identity conversion composition recognizes only canonical core From
dispatch with equal concrete primitive integer input/output types and the existing
vstd From model. It proposes an identity expression for candidate generation;
ordinary verification still checks the original call. No user-defined type or
trait gains an identity law, and no vstd axiom or trusted contract is added.
Persistent metadata schema remains 38; the policy/cache changes invalidate older
candidate and interface evidence.

The new compiled native-function scope snapshot is presentation-only. It is not
serialized into proof interfaces or consulted by candidate acceptance, dependency
checks or publication. Reports distinguish checked bodies, specifications,
interfaces, external bodies and outside-scope items. Failed, partial and
no-verification runs never report a fully checked selected scope. This does not
certify unannotated Rust, that callers satisfy preconditions, or that an authored
application property covers its intended domain.

Compiler regressions exercise repeated/cross-crate reverse conversion, all twelve
primitive integer identity conversions, missing/wrong/restrictive implementations,
unrequested reverse derives and failed/filtered/no-verification publication.
Scope regressions cover ordinary requirements, external bodies and frontend
failure. The general Cargo fixture uses real derive_more and codec expansions,
retains excluded Sum/codec implementations for native execution, and rejects
verified use of those exclusions and exclusion of a requested conversion. Its
runner records commands, source/tool identities, complete reports and native
test outcomes. These checks support the integration change; the independent
clean-main Aleph acceptance in VERUS_INFER_ADOPTION_V2.md remains a separate,
unfinished requirement.

## Checked positional/sparse count bridges

The collection library now proves positional count congruence and the connection
to sparse observations after mapping/filtering by induction. Equal prefix/slot
lengths and pointwise predicate compatibility are premises, not synthesized or
assumed facts. Occupied-slot counting is a checked specialization. Private helper
proofs establish prefix traversal and identity filtering; no external-body,
axiom, native implementation or standard-library assumption is added.

The bounded counting selector enables these canonical checked broadcasts only
when a supplied count goal also reaches sparse/projection vocabulary. Original
contracts, bodies and goals are preserved. Selection alone supplies no proof of
the conditional premises and grants no exactness claim to ordinary functions.
Existing exact precision checking and interface publication are unchanged.

The generic regression needs the assistance, and rejects wrong counts, a wrong
projection and incompatible lengths. The original independent occupied-slot
minimization and the actual Aleph count/eligibility goals pass without local
proofs. All 481 inference regressions and 116 backend tests pass; matching vstd
verifies 2,215 results with no errors. Policy 46, cache 52 and metadata schema 38
are unchanged for this ordinary theorem-selection checkpoint; changed library
artifact fingerprints require rebuilding providers and dependents normally.
