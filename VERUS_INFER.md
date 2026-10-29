# Verus Infer
## Automatic Contract Inference and Compositional Verification for Rust

**Status:** Draft RFC / Working prototype for a bounded subset  
**Date:** October 25, 2026  
**Target:** Extension or fork of [Verus](https://github.com/verus-lang/verus)  
**Proposed name:** Verus Infer  
**Implementation language:** Rust  
**Primary verification backend:** Verus + SMT (Z3)

---

## Implementation status — October 28, 2026

The design below includes future work. The current implementation is documented
in [the inference guide](source/docs/guide/src/infer.md); its supported syntax
and restrictions take precedence over illustrative APIs in this RFC.

Implemented and covered by regression tests:

- [x] Opt-in native Rust `#[verus_infer]` and `verus!` executable-function annotations.
- [x] Exact loop-free inference for `bool`, `u8`, `u16`, `u32`, `u64`, and `u128`:
  arithmetic, comparisons, Boolean operations, unsigned casts, immutable locals,
  branches, and short-circuit safety conditions.
- [x] Exact `i8` through `i128` arithmetic, comparisons, negation, and casts,
  including signed enum payloads, fixed arrays, checked arithmetic, and cached
  cross-crate composition. Signed division/remainder preserve Rust's truncation
  semantics and exclude both zero divisors and `MIN` with `-1`.
- [x] Independent soundness, safe-domain maximality, and output-exactness checks
  using the original executable and the existing Verus backend. Exact mode
  requires all three proofs before dependent verification or publication.
- [x] Acyclic composition of inferred functions, including forward references,
  private helpers, shared dependencies, and explicit caller properties derived
  from contracts without inlining implementations.
- [x] Exact scalar newtypes and inherent methods: transparent nongeneric,
  single-variant/single-field scalar types, shared inputs and owned results.
  Candidate-only scalar projection leaves original executable bodies intact;
  type definitions bind caches and exported contracts. An AlephBFT quorum/parent
  comparison replaces one five-clause threshold contract, confirms two existing
  overflow boundaries, and retains the manual collector's state/counting proofs.
- [x] Bounded generic `Vec` length/read templates with all three exact-mode
  proofs, and explicit sound-mode single-slot updates with old/new sequence
  relations and unchanged-slot frames. Guarded updates admit out-of-range indices
  as no-ops. State precision is explicitly not established. Original bodies and
  existing vstd models are retained; collection candidates regenerate and reprove
  on every run, including across certified crate interfaces.
- [x] Initial demand-driven scalar proof strategy behind `--infer-goals`:
  explicit ordinary caller assertions can use bounded exact-summary substitution
  and checked bit-vector theorems, without eager contract enrichment. A pinned,
  source-connected AlephBFT `common_vote` pilot compares manual/baseline/assisted
  proofs and semantic mutations. This bridges equivalent parity representations;
  it does not reproduce the existing stateful/distributed verification effort.
- [x] Supplied counting-invariant assistance behind `--infer-goals`: bounded
  contract/assertion discovery enables four checked generic prefix-count lemmas
  through scoped broadcasts. Aleph's update method verifies without its counting
  proof block; a checked equivalence preserves the original count definition's
  meaning. Invariants, loop annotations and the transition's sequence-equality
  assertion remain supplied.
- [x] Bounded compositional record-state inference in default exact mode:
  guarded scalar/vector field effects, unchanged-field frames, immutable locals,
  Copy generic tuple/Option matching, early unit returns and direct vector
  operations. Closed primitive `From` widenings reuse existing vstd models.
  The supplied Aleph `add` body receives an inferred contract; ordinary caller
  harnesses check the whole valid domain, invariant preservation and the original
  behavior. Independent original-body queries certify maximal input safety and
  exact modeled final state. No vector wrappers or assumed invariants are required.
- [x] Bounded transparent wrapper and helper composition: nested scalar/vector
  leaves, scalar wrapper values, generic static inferred receiver calls and shared
  `Option<&T>` lookups. Unconstrained sized payloads can move into a slot without
  adding a Copy bound. All exact obligations remain required. Aleph's pinned
  NodeMap lookup/insert bodies and a documented constructor-spelled size variant
  compose with the supplied collector invariant. Derived conversion/operator
  inference remains open.
- [x] Supplied direct sequence views for private wrappers around a sole vector
  field. Inferred length/read/update contracts compose across modules and compiled
  crate boundaries without exposing private fields or closed view definitions.
  Compiler-only structural evidence preserves independent exact observations and
  is bound to exported metadata. Aleph's extracted NodeMap API now composes with
  the supplied collector update from a separate crate. The view and a checked
  size observation in each ghost-only caller are supplied; arbitrary abstraction
  discovery, richer representation laws and full Unit/Hasher trait support remain open.
- [x] Bounded consumption of verified trait getter contracts in record inference:
  one shared receiver, no precondition and an exact result/observation equation.
  Shared generic inputs and opaque associated types/equalities compose through
  private collection views while retaining all exact checks. Contracts-only
  providers can certify full verified APIs with `--infer-goals`; failed/default
  implementations, verification bypasses and stale dependencies block publication.
  Aleph's reduced Unit/Hasher adapter retains pinned creator/round default bodies
  and drives an inferred generic unit-to-update bridge. Full original traits,
  cached hashing, operators and collector lifecycle remain outside that experiment.
- [x] Concrete scalar-wrapper `From` and `AddAssign` inference with canonical
  blanket `Into` composition. Ordinary trait verification retains inherited
  requirements; an independent entry-state equivalence check binds those
  requirements to the candidate safe domain. Impl attributes generate checked
  ordinary models for simple public tuple wrappers and preserve erased declaration
  identities. Aleph's optional wrapper checkpoint restores the original size body
  and a NodeCount counter in the supplied collector; native derives/full lifecycle
  remain outside its scope.
- [x] Bounded owned collection construction and round advancement in default exact
  mode: empty/capacity vectors, repeat initialization, fill and indexed Copy loops,
  nested collection-bearing record results and `Option<Self>`. Complete logical
  contents and scalar fields compose through certified private direct views and
  crate interfaces. Pure loop annotations are checked against unchanged executable
  bodies; return exactness independently enumerates the modeled output. Aleph's
  supplied Vec core now infers `new`/`next` without method contracts, loop invariants
  or local proof blocks. A separate private-API lifecycle retains NodeMap's original
  `with_size` body and native derived Clone. Application invariants,
  count equivalence, reduced traits and widened-round/terminal policy remain supplied.
- [x] Bounded borrowed collection queries inside `Result`: early success/error
  returns, Option/tuple matching, canonical primitive widenings of borrowed values,
  complete error tags/payloads and returned collection views. Exact helper summaries
  compose across private crate interfaces. The supplied Aleph selection method now
  infers its contract and ordinary goals check readiness, contents and both error
  policies. Vec replaces the supplied slice borrow; private API spellings and
  checked size/lookup observations remain explicit.
- [x] Checked native derived Clone contracts through supplied direct sequence views.
  `#[verus_verify(clone_via_view)]` retains and verifies Rust's derive body against
  a public relational view contract. Bounded tuple Clone models and an explicit
  verified `CloneIdentity` law let exact clone callers compose across crate
  interfaces. Copy alone never supplies identity. Aleph's lifecycle now uses
  `.clone()` and no `copy_contents` adapter. Native-Clone-only providers require
  complete verification and certified exports.
- [x] Source-connected Unit/Hasher implementation checkpoint: the native Hasher64
  body, record Clone derives, FullUnit construction/clone and all four Unit methods
  verify with explicit contracts. Repeated hash and clone/hash consistency compose
  into a concrete supplied-core collector caller. This checkpoint replaces
  parking_lot guard operations with vstd's verified lock operations and separately
  models SCALE encoding; it does not verify native RAII/Drop or the codec algorithm.
  Generic Data/Hash clone laws and application invariants remain supplied.
- [x] All four original native `UnitsCollector` bodies now infer exact contracts,
  retaining u16 round arithmetic and native branch/error policies. Bounded record
  matches compose guarded effects and early returns; checked scalar comparisons
  compose through concrete PartialEq/PartialOrd implementations. Demand-driven
  selection of an ordinary proved view-length companion removes manual hidden
  length observations from private collection callers. Application invariants and
  goals remain supplied; round/index safety requirements are exposed, not repaired.
- [x] Checked native `CloneView` laws for non-Copy values: primitive, Option, Vec,
  and String implementations preserve their supplied deep view. Aleph's FullUnit
  proof accepts non-Copy Vec<u8> data and checks clone/encoding/hash consistency.
  A verified finite RwLock try-read removes the blocking clone adaptation.
  The explicit codec boundary still requires an encoding-complete payload view;
  the law does not infer the intended abstraction or verify native parking_lot.
- [x] Checked finite sparse iteration, counting, filtered counting, first-match
  search, collection and index-preserving mapping, with no new iterator axioms.
  Exact inference composes supplied trait queries with up to two scalar arguments,
  including complete Result policies for read-only record queries. A bounded
  Aleph experiment checks round projection, parent counts and future-round checks
  behind explicit adapters; the newer native query profile below removes those adapters.
- [x] Bounded data-bearing value enums, including generic/associated hash payloads,
  compose through exact Result and identity `?` propagation. Independent precision
  checks observe every tag and guarded payload. Literal-message runtime assertions
  retain their failure domain. Ordinary checked concrete scalar queries compose
  with immutable private collection receivers. Aleph's original noninitial
  assertion/lookup/propagation sequence now infers exact typed errors and first-error
  priority; the newer native query profile below removes the two collection adapters.
- [x] Bounded native finite iterator queries through supplied checked factory views:
  `count`, pure scalar/field `filter(...).count()` and temporary-iterator `any(...)`.
  Callback precision is checked independently against original executable bodies.
  Empty streams preserve lazy predicate safety. Native count models distinguish
  callback-skipping Map/Enumerate counts from Filter/FilterMap exhaustion and
  exclude arbitrary custom Iterator overrides. These APIs explicitly extend the
  trusted Rust-library model. Aleph's original noninitial validation now infers
  all typed errors with both count/search implementation adapters removed; views,
  factory proofs, policy and goals remain supplied.
- [x] Total native construction folds over supplied finite iterator factories:
  one local sequence accumulator, exact inferred mutable-view updates, generated
  prefix/length/readiness invariants and the unchanged native loop. The initial
  profile requires `true` as its safe domain and rejects unsafe inputs rather than
  restricting them. Ordinary complete-view mutations also compose without a loop.
  Checked generic value queries support associated hash results. Aleph's original
  ControlHash::new now infers its complete fold/hash relation and a supplied caller
  connects it to typed noninitial validation. Partial-domain/arbitrary iterator
  loops remain open; the newer checkpoint below covers initial-round validation.
- [x] Demand-scoped checked observations of indexed-update folds: length, last-write
  reads, sparse per-slot projection and equivalent step functions. Native update
  folds use a checked canonical vocabulary; ordinary caller goals select lemmas
  through `--infer-goals`. Visible unary view-query observations compose with
  constructed collections. Generic same-type equality proposes identity only when
  ordinary verification and independent precision checks establish the supplied
  checked law. Aleph's original initial branch now infers exact errors. Input-map
  bounds and constructor projection goals need no handwritten fold induction.
- [x] Compact composition of complete value-only `Result` equations through calls,
  branches and identity `?` propagation. Collection-bearing results retain guarded
  view observations. Aleph's original public dispatcher now infers its maximal
  safety domain and exact typed output within the unchanged formula budget;
  separate supplied caller goals check the complete application policy.
- [x] A bounded candidate registry and persistent cache with transitive dependency
  invalidation. Source eligibility is always checked, and every reused candidate
  receives fresh proofs; the cache stores no proof outcomes.
- [x] Certified inferred `.vir` interfaces and three-crate `cargo verus build`
  and `verify` examples. Metadata records per-function proofs and dependency/model
  fingerprints; failed full-crate verification prevents inferred API export.
- [x] Explicit `precision = "sound"` scalar interval summaries, compositional
  safety domains, independent optional precision budgets, and honest reporting
  when maximality or output exactness is not established. Exact mode never
  silently falls back. Exact inferred callers require exact-mode dependencies.
- [x] Signed interval approximations for `i8` through `i128`, including negative
  ranges, arithmetic, branch joins, casts, and Rust division/remainder. Exact
  safety predicates are retained when available. Optional precision outcomes, candidate-cache
  reproving, and mixed exact/sound cross-crate interfaces retain the existing gates.
- [x] Bounded safety inference from scalar callee preconditions and result
  ranges, replacing automatic small-input restrictions for signed and unsigned
  functions. Affine normalization handles repeated arithmetic without expanded
  substitutions; guarded operations and callee requirements remain enforced.
  Insufficient information or exhausted analysis budgets produce an incomplete
  diagnostic, with no guessed small argument domain.
- [x] Exact core `Option`/`Result` constructors and simple exhaustive matches with
  flat scalar/unit payloads. Approved checked fixed-width integer arithmetic uses existing
  vstd models, with visible trust dependencies and model fingerprints.
- [x] Exact immutable flat arrays of up to 64 scalar elements: by-value inputs,
  element-list literals, local copies, branches, index-safety preconditions,
  scalar/enum results, cached cross-crate summaries, and a Cargo pricing example.
  `usize` arithmetic and scalar results are also supported; owned scalar array returns are supported.
- [x] Exact early returns in blocks, branches, matches, and supported expressions,
  preserving reachability and operand evaluation order. Return summaries retain
  all three checks, source eligibility, and cached cross-crate composition.
- [x] Exact `?` propagation for flat core `Option`/`Result` with scalar/unit
  payloads and identical or approved losslessly widened integer error types, including nested expressions,
  checked arithmetic, cached summaries, and cross-crate composition.
- [x] Shared `&[T; N]` array inputs and immutable local borrows, `.len()` guards,
  and `[value; N]` repetitions. Approved generic array models remain limited
  to fixed scalar arrays; their non-unwinding guarantees and containing model
  artifacts are recorded and checked through caches and cross-crate exports.
- [x] Shared source capability rules, typed summary builders for cache decoding
  and imported-summary substitution, and one proof-publication requirement used
  by local verification, exports, and imports.
- [x] Exhaustive small-integer comparisons, deliberately incorrect candidates,
  malformed artifact regressions, and tests that incomplete or failed required
  proofs block dependent verification and remove older inferred exports.
- [x] Schema-versioned `--output-json` contract reports with candidate-bound
  proof outcomes, required/optional distinctions, dependency/model fingerprints,
  actual caller/export availability, formula sizes, and generation/cache/SMT
  measurements. Reports are observations, not importable proof certificates.
- [x] Exact read-only shared scalar slices with symbolic lengths, guarded and
  unguarded indexing, aliases, length relationships, scalar/enum results, and
  composition through caches and certified cross-crate interfaces. Existing
  slice models are fingerprinted; no new slice axioms or stronger specs were added.
- [x] Exact `usize` arithmetic, casts to/from supported integers, scalar and flat
  enum results, array/slice elements and index arithmetic, and counted loops.
  Bounds and casts preserve the symbolic 32/64-bit model. Cache identities and
  exported contract fingerprints bind the merged architecture; ordinary Verus
  rejects incompatible imported widths.
- [x] Exact unsigned counted-loop families: constant/immutable parameter steps,
  accumulator addition/subtraction, and ascending/countdown unit counters, for
  `u8` through `u128` and `usize`. Generated prefix invariants and termination
  measures, independent safe-domain maximality, and arbitrary-output exactness
  are checked by Verus. Proof-only decoration preserves executable code through
  a production projection check. See the
  [loop proof design](VERUS_INFER_LOOPS.md).
- [x] Exact `isize` arithmetic, division/remainder edge checks, casts, flat enum
  payloads and arrays/slices, plus all seven existing checked methods for both
  `usize` and `isize`. No new standard models or axioms were added.
- [x] Sound `usize`/`isize` intervals and compositional safety, analyzed separately
  at 32 and 64 bits and merged under a checked symbolic-width selector. Both
  branches are validated in caches/imports; mandatory soundness still gates use.
- [x] Exact bitwise operators and checked shift amounts, wrapping/saturating
  add/subtract/multiply, checked negation, and mixed-sign checked add/subtract
  across supported fixed-width/platform integers. Scalar sound-mode transfers
  remain conservative. Typed summaries, caches, imports, and model fingerprints
  preserve the existing independent proof requirements. The assurance document
  audits 36 added trusted numeric specifications.
- [x] Exact `let-else`, guarded matches, and or-patterns over the supported flat
  values, preserving first-match order, binding scope, and early-exit safety.
  Verus's existing restriction on combining a guard with an or-pattern remains.
- [x] All 35 existing vstd lossless primitive error conversions through `?`.
- [x] Owned fixed scalar array outputs of lengths 0..=64, exact whole-array
  equality, and same/cross-crate composition with arbitrary-output checks.
- [x] Local shared subslices with exclusive/open ranges, nested views, lengths,
  indexing, and inferred-callee substitution. Existing standard Index bounds
  are checked ordinarily and instantiated in maximality queries.
- [x] Branch-sensitive sound ranges through immutable copies and Boolean locals,
  bounded masks/shifts, one-cycle wrapping casts, and useful wrapping/saturating
  cases. Auxiliary scalar lemmas use Verus's checked bit-vector prover; no new
  axioms or standard specifications are added.
- [x] Bounded simplification of the actual installed/verified contracts: typed
  bounds, Boolean redundancy, repeated affine arithmetic, and local-view bounds.
  Exact contracts retain independent soundness/maximality/exactness checks.
- [x] Fifteen representative parsing, indexing and pricing application kernels,
  with successful and failed caller properties and explicit unsupported/resource
  limits. These are examples within the implemented subset, not general Rust coverage.
- [x] Reproducible 47-case evaluation, including expected failures,
  repeated isolated cold/warm pairs, waited-child memory observations, artifact
  stability checks, Cargo body/contract edits, and call-graph depth.
  See [the corpus runner](source/tools/infer_eval/README.md).

- [x] Assurance consolidation: formula-only candidate caches, one installer
  preserving executable code, candidate-bound proof evidence, explicit query-kind
  checks, failure injection, and source mutation tests. The checking and acceptance
  components are not themselves formally verified; see the
  [assurance plan](VERUS_INFER_ASSURANCE.md).

Still planned or outside the implemented subset:

- [ ] Formal verification of the acceptance component and precision
  transformations; see the [assurance plan](VERUS_INFER_ASSURANCE.md).
- [ ] Further numeric methods such as rotates/counts, mixed-sign wrapping or
  saturating methods, and explicit wrapping shifts; the current batch covers
  bitwise operators, normal shifts, wrapping/saturating add/subtract/multiply,
  checked negation, and mixed-sign checked add/subtract.
- [ ] General user-defined or nested enums, broader payloads, reference/@/range
  patterns, user-defined error conversions and `Try` implementations, and enum
  abstractions in sound mode. Guards combined with or-patterns need Verus support.
- [ ] Broader array syntax/patterns, enum array payloads, returned slice
  references, direct fixed-array slicing, stored/inclusive ranges, conditional
  local views, and richer slice element types. Owned array outputs and local
  exclusive/open shared subslices are implemented.
- [ ] General mutation, state relations, and frame-condition inference beyond
  the bounded vector/record subset and counted-loop locals. Standalone-vector
  precision checks, opaque records beyond direct vector views, deeper records,
  broader mutable helper composition and counting
  lemma selection beyond the checked counting catalog remain open.
- [ ] General consumption of handwritten/external contracts by inferred functions,
  beyond exact precondition-free trait queries and bounded ordinary value queries
  with an owned/shared record receiver and shared context; mixed handwritten-and-inferred
  contracts, and broader generic/trait specialization.
- [ ] Broader records, nested/multiple-field datatypes, returned wrapper references,
  and operator-trait inference beyond concrete From/AddAssign through Verus's
  inherited/extension specifications, including derived implementations.
- [ ] Broader loop patterns, generalized invariant discovery, slice traversal,
  search, signed counters, and recursive summaries.
  Checked finite iterator implementations are available as explicit library APIs;
  synthesis of iterator-factory meanings, arbitrary closures/iterator loops, and
  full protocol properties remains open. Original ControlHash construction and both
  validation branches and public dispatcher are inferred and checked against a
  supplied complete policy. Codec implementation proofs and the next outer parent
  validator remain open; its error mapping, richer nested payloads and borrowed
  wrapper accessors need broader inference support.
- [ ] Safe-only mode and the other proposed attribute options; the implemented
  options are `precision` and `precision_rlimit`. Proposed controls include
  safety policy, incomplete-result behavior, budgets, export, and diagnostics.
- [ ] Further relational simplification, dedicated contract-query APIs,
  ordinary-proof-first goal retries, broader demand-driven strengthening, and
  IDE presentation and suggestions. The initial `--infer-goals` strategy is
  includes explicit scalar assertions and a checked bit-vector tactic, plus
  supplied count contracts/assertions and scoped counting theorem selection.
- [ ] Richer abstract domains and refinement, including relational properties
  beyond scalar intervals and Boolean summaries, disjunctive/relational branch
  refinement, and wrapping-cast ranges spanning more than one modular cycle.
- [ ] Investigate broad candidate-cache invalidation after Cargo body edits,
  including independent functions, without weakening provenance or fresh proofs.
- [ ] Broader external application suites, simultaneous process-tree peak-memory
  profiling, and performance studies beyond the implemented repeated curated
  corpus and controlled three-crate Cargo edit scenarios.

| Roadmap phase | Current state |
| --- | --- |
| Phase 0: integration research | Complete for the prototype's VIR insertion point. |
| Phase 1: exact arithmetic | Implemented for fixed-width and platform-sized unsigned/signed integers and Booleans; bitwise/shifts, wrapping/saturating arithmetic, and checked variants are implemented; additional numeric methods remain. |
| Phase 2: modular composition | Implemented within the supported subset, including caches and Cargo imports. Opt-in goal assistance handles selected scalar assertions and supplied counting invariants; general queries, automatic retries and contract strengthening remain future work. |
| Phase 3: partial inference and collections | Partial: scalar intervals, exact flat enums/control flow/arrays/slices, generic Vec views, bounded owned construction/copy, exact record-state composition and transparent generic wrappers are implemented. General heap mutation and standalone-vector state precision remain. |
| Phase 4: loop inference | Exact unsigned constant/parameter-step add/subtract accumulators and ascending/countdown unit counters implemented; signed and collection loops remain. |
| Phase 5: richer verification | Future work. |

The prototype retains Verus's existing trusted-library and standard-model
assumptions. Export metadata records completed proofs and is not a signed or
portable proof object. Application intent remains an explicit specification.

The focused consolidation in the [assurance plan](VERUS_INFER_ASSURANCE.md) is
implemented, as are exact early returns, bounded `Option`/`Result` propagation,
and exact/interval signed integer inference.
Formal verification of the acceptance component and precision
transformations remains separately scoped work.

## Proposed next milestones

The implementation checklist above records completed work and the full backlog.
The current checkpoint includes exact and sound platform integers, explicit
integer policies, unsigned stepped/countdown loops, richer flat control flow,
primitive error widening, owned array outputs, and local shared subslices.
Caches/imports preserve architecture and trusted-model fingerprints. The guide is the source of supported syntax; the original
phases later in this RFC retain their proposed design, including unimplemented
APIs and research directions.

1. **Integer-operation batch implemented.** Exact fixed-width/platform bitwise
   operators, shifts, wrapping/saturating add/subtract/multiply, checked negation,
   and mixed-sign checked add/subtract share typed builders and a closed numeric
   model validator. The guide records their policies and conservative sound-mode
   transfers. The assurance audit records added standard specifications. Broader
   numeric methods remain in the checklist above.
2. **Unsigned counted-loop batch implemented.** Constant/immutable parameter
   steps, addition/subtraction, and ascending/countdown counters share checked
   prefix invariants, termination, independent maximality, and output exactness.
   Parameter products use the existing nonlinear prover with candidate-bound
   proof settings. Signed accumulators/counters, arbitrary counter steps,
   multiple/nested loops, and slice traversal/search remain separate designs.
   See [the loop design](VERUS_INFER_LOOPS.md).
3. **Rust coverage batch implemented.** Exact `let-else`, guards/or-patterns,
   lossless primitive error propagation, owned fixed-array outputs, and local
   shared subslices compose through caches and imported interfaces. Broader enum
   payloads, user conversions, returned references and mutation remain separate
   designs; the guide records the remaining syntax limits.
4. **Precision and practical evaluation implemented.** Branch/copy refinement,
   bounded bitwise and cast ranges, checked proof automation, and bounded
   simplification improve actual verified contracts. Fifteen application kernels
   cover parsing, indexing and pricing. Repeated cold/warm timings, honestly
   scoped memory observations, build-stability checks and Cargo incremental
   verification now have reproducible runners.
5. **Initial bottom-up/top-down pilot implemented.** `--infer-goals` consumes
   explicit ordinary scalar assertions and exact inferred summaries. Bounded
   conditional bit-vector proofs bridge equivalent representations without eager
   contract enrichment. A pinned AlephBFT experiment shares the original
   `common_vote` body with production and records manual, baseline, assisted and
   mutation outcomes. Canonical remainder-based properties already pass; the
   measured benefit is a low-bit parity formulation, not stateful agreement.
6. **Quorum/newtype checkpoint implemented.** The source-connected parent
   experiment preserves the `NodeCount` API while separately applying the real
   overflow-safe threshold correction. Native bounded exploration checks the
   clean-main collector; formal comparison imports the existing manually verified
   collector core. Inference replaces the threshold contract and an experiment
   adapter contract, not the collector's invariant or induction proofs. Removing
   the update lemma call or constructor invariant still fails. Native mutation
   controls detect duplicate/stale counts, future slots and missing own parents.
7. **Bounded collection/state checkpoint implemented.** Generic vector
   length/read helpers receive exact checks; single-slot replacement receives a
   checked sound state relation, including its frame. A supplied counting
   invariant composes with that relation through checked sequence lemmas.
   The AlephBFT experiment extracts three operations from the existing manual
   collector; it replaces experiment wrapper contracts and removes no existing
   collector proof annotations in that initial comparison.
8. **Counting-invariant composition checkpoint implemented.** Reusable checked
   prefix-count proofs cover bounds, updates, zero and full counts. Bounded
   discovery of supplied count contracts/assertions enables those theorems through
   ordinary scoped broadcasts. The Aleph follow-up removes its update proof block
   and all three bespoke counting proof definitions, including initialization
   zero-count calls. A checked reference equivalence
   validates the specification migration. State contracts and loop
   annotations remain explicit; whole-collector inference is not established.
9. **Record-state composition checkpoint implemented.** Bounded symbolic fields,
   pure Copy tuple/Option matching, standard widening and direct vector operations
   infer the supplied collector's `add` contract from its original executable
   body. An ordinary harness proves every valid state can call it and stays valid;
   the old behavioral contract is retained separately as a regression oracle.
   Independent maximality and exactness queries now cover the whole modeled
   final state in default exact mode. Mutation tests distinguish precise candidate
   behavior from requested-domain coverage and invariant preservation.
   This does not establish clean-main collector or arbitrary record inference.
10. **Transparent wrapper composition checkpoint implemented.** Acyclic generic
    receiver calls compose checked domains and final-state equations over nested
    records. Read-only queries include bounded borrowed lookups; exactness walks
    the full nested datatype independently. A source-connected NodeMap experiment
    retains the original get/insert bodies, records one size-expression adaptation
    and verifies the supplied collector through those APIs. Private storage is
    transparent in the harness module; this does not establish opaque cross-crate
    views or the clean-main generic Unit/Hasher implementation.
11. **Direct private sequence-view checkpoint implemented.** A supplied canonical
    View implementation must directly project the wrapper's sole Vec field.
    Inferred APIs compose across module and compiled crate boundaries; independent
    precision checks use certified structural observations after private fields
    are pruned. Aleph's separate API crate preserves private NodeMap storage and
    the original get/insert bodies. Its supplied update core retains the invariant,
    count equivalence and behavior oracle; ghost-only callers observe size to
    expose the hidden length bound. This checkpoint excludes lifecycle methods
    and the clean-main generic Unit/Hasher collector.
12. **Bounded trait getter checkpoint implemented.** Supplied exact observation
    equations describe precondition-free shared trait getters. Generic record
    updates retain trait constraints, normalize bounded associated equalities and
    preserve associated values through private sequence views. Certified trait-only
    providers require full verification and bind their whole exported API/model
    dependencies. A reduced Aleph Unit/Hasher interface retains original default
    creator/round bodies and coordinate getter bodies, while an inferred generic
    add_unit bridge consumes them. The supplied invariant and a separate behavior
    oracle remain explicit, including a wrong-slot control preserving the invariant
    while failing intent. This is not the full original traits or clean-main collector.
13. **Wrapper trait boundary checkpoint implemented.** Concrete nongeneric From
    and AddAssign implementations receive body-derived exact summaries; canonical
    Into calls reuse the existing standard bridge. Generated ordinary extension
    models cover simple public tuple wrappers. Maximality asserts inherited/
    candidate domain equivalence before execution assumptions; false, weak and
    restrictive operator requirements cannot gain certification. A source-connected
    Aleph variant retains all three NodeMap method bodies and uses NodeCount for
    the supplied collector's counter. Explicit verified trait adapters replace
    native derives; the full original collector and cached FullUnit implementation
    remain unverified by this experiment.
14. **Owned collection lifecycle checkpoint implemented.** Pure vector constructors,
    bounded fill/Copy loops and collection-bearing owned/Option results compose in
    exact mode, including across private crate interfaces. An independent checker
    enumerates every output observation rather than deriving coverage from the
    candidate. Ordinary goals check construction, update, advance and another update
    in the supplied Aleph core. Wrong rounds/reset/termination and restrictive-domain
    controls separate exact code descriptions from intended behavior. Generic Clone
    equality is not assumed. The subsequent native Clone checkpoint removes the
    explicit Copy loop using checked relational clone contracts and value laws.
15. **Borrowed selection-query checkpoint implemented.** Complete Result relations
    describe returned collection views and nullary error enums in exact mode.
    Independent precision includes the tag and both guarded payloads. Original
    branch order and bounds policy remain in the supplied Aleph query; caller goals
    reject changed error policies, own-parent acceptance and unguarded indices.
16. **Native Clone and Unit implementation checkpoint implemented.** A supplied
    direct sequence view supports ordinary checked native derives and exact inferred
    callers, including private imported returns. An explicit checked CloneIdentity
    law supports primitive, Option, array and bounded tuple payloads. Aleph's native
    Hasher64 and FullUnit method bodies are now source-connected formal targets,
    with a separately declared codec model and explicit verified-lock adaptation.
    Runtime tests exercise the original parking_lot cold/warm/contended clone paths.
    Native lock/Drop verification, SCALE derivation and general Data clone semantics
    remain open; do not describe the adapted proof as unchanged native verification.
17. **Native collector checkpoint implemented.** The original new_initial,
    from_previous, add_unit and prospective_parents executable bodies retain their
    u16 round, guarded matches and early-return policies. Supplied ordinary goals
    prove invariant preservation, exact changed slot, frames, monotone counts,
    lifecycle and both selection errors. Concrete wrapper comparisons and bounded
    effectful record matches infer all three properties. A checked private-view
    length lemma is selected on demand. Missing round/node bounds and changed
    counter/reset/error behavior fail caller proofs despite exact inference.
18. **Non-Copy payload/finite read checkpoint implemented.** Checked CloneView laws
    preserve supplied deep views for native Clone without requiring Copy or object
    identity. A generic FullUnit proof and concrete Vec<u8> caller check payload,
    model encoding and hash observations. RwLock::try_acquire_read makes one finite
    atomic attempt; copied/fallback helpers release successful handles and preserve
    the configured predicate. Native clone now uses try_read_or(None). Parking_lot
    API substitution and the encoding-completeness assumption remain explicit.
19. **Bounded collection-query checkpoint implemented.** Ordinary verified sparse
    traversal and finite count/filter/search/collect/map APIs connect supplied
    collection meanings to exact parameterized trait-query inference. The separate
    Aleph validation stage proves quorum/future-round success and error policies.
    Source callbacks, model predicates and projection are checked; native and
    adapted runtime corpora cover the same 1,530 cases. Original iterator-chain
    synthesis, complete native ControlHash error payloads and inference of its
    construction loop remain future work, not claims of the adapted proof.
20. **Typed validation checkpoint implemented.** Bounded value enums support up
    to eight variants, four fields per variant and sixteen observations per value.
    Exact Result propagation preserves error payloads and short-circuit domains.
    Aleph's six-variant Error<H>, original own-parent lookup, runtime assertion,
    member count and noninitial `?` orchestration now verify. Ordinary supplied
    goals check success, exact errors and their priority. Native and adapted
    runtime corpora cover the same 7,704 cases, including terminal u16 values,
    with a separate zero-round panic check. Two collection pipelines remain
    adapted; initial-round hash validation and original iterator-chain inference
    remain open.
21. **Native query checkpoint implemented.** Supplied finite iterator views compose
    through exact inferred native count/filter/search consumers. Independent
    callback safety/maximality/Boolean-exactness checks protect the outer precision
    argument. Aleph's original count/search callback bodies and complete noninitial
    error orchestration verify without query implementation adapters. Native
    standard-library specifications are an explicit added trusted boundary; the
    library implementation itself is not verified. Construction-loop inference and
    initial-round hash validation remain separate work.
22. **Native construction checkpoint implemented.** Exact total folds compose
    inferred mutable collection summaries without rewriting the original loop.
    Generated proof nodes retain separate compiler identities and Spec-mode
    constructor metadata; executable borrow/type-invariant checks stay intact.
    Aleph's original ControlHash::new and create_control_hash body are connected
    to the complete typed noninitial validator. The original loop's summary is
    inferred; collection/factory meanings, a checked hash query, codec boundary
    and application goals remain supplied. Native constructors exercise the real
    codec/hasher separately from the proof-only encoding model.
23. **Demanded fold facts and complete validation checkpoint implemented.** Checked
    sparse-update observations derive caller length and per-slot projection facts
    without application-specific induction. Pure view-query specialization handles
    constructed receivers; checked equality laws support generic hash comparisons.
    Aleph's original initial branch is inferred. Factory/view and application
    meanings remain supplied.
24. **Compact result composition checkpoint implemented.** Bounded value-only
    results retain complete equations instead of separate tag/payload formulas;
    same-type identity propagation preserves the error value on its failure path.
    Aleph's original public dispatcher now infers all three exact guarantees
    within the unchanged 4,096-node budget. Separate ordinary caller goals check
    the intended policy and conditional creator bound. General expression sharing
    and arbitrary-depth composition remain separate work.
25. **Parent validation and typed error mapping checkpoint implemented.** The existing
    standard `Result::map_err` specification composes direct immutable callbacks
    with independently checked safety, maximality and complete-output precision.
    Bounded collection-bearing error enums retain every tag and guarded payload
    view. Checked native Clone relations compose through structural records.
    Direct borrowed object accessors retain complete value identity via ordinary
    closed spec definitions, checked by normal Verus well-formedness and modes;
    private representation details remain hidden across imports. Pure summary
    substitution cannot change executable path state. Aleph's original
    `Validator::validate_unit_parents` and supporting accessors now infer in exact
    mode. A supplied caller checks member-count priority, copied PreUnit views,
    exact nested errors, unchanged Signed success and the conditional creator
    bound. The input is already signed; authentication is an explicit boundary.
26. **Checked factory proof assistance implemented.** An opt-in
    `#[verus_verify(iterator_via_view(output_view))]` annotation retains the
    original native body in executable erasure and requests an ordinary checked
    finite-view contract. `--infer-goals` proposes total immutable callback
    equations, including canonical Option propagation. Verified generic sparse
    filter-map and map combinators discharge the factory proof without new
    iterator axioms. Aleph's original NodeMap::iter and ControlHash::parents now
    need no handwritten callback contracts, factory proof blocks or named-local
    adaptations. Their output views remain supplied. Factories are ordinary
    verified interfaces, not exact iterator-object inference candidates; their
    consumers retain all three existing inference checks. More general factory
    chains, partial callbacks and arbitrary iterator implementations remain open.
27. **Outer validation and structured query checkpoint implemented.** Ordinary exact
    value-query contracts can consume an owned record and a shared context and
    return bounded structured values. Canonical Result `?` can compose the concrete
    checked exact generic From implementation selected by Rust. Each conversion
    retains soundness, maximality, output precision and inherited-domain checks;
    partial or unmarked conversions cannot supply this path. Generic trait return
    precision builds its pre-poly observations without inserting Box/Unbox twice.
    Goal-directed view-bound search uses breadth-first traversal and retains
    already discovered checked lemmas at its existing limits.
    Aleph's original outer Validator::validate_unit body is checked directly
    against supplied authentication/session/maximum-round/member/parent policy,
    with no handwritten proof blocks. The constructor and signature-error From
    conversion add two exact summaries (31 total). The outer method is an ordinary
    application proof, not an exact/maximality-certified inference candidate:
    cloned FullUnit errors use supplied logical views, while success preserves the
    complete original Signed value. A supplied authenticated predicate and checked
    adapter model the native signature-check interface; the signature implementation
    and cryptographic security remain outside this experiment.
28. **Next: richer abstractions and reducing the remaining implementation adapters.** General trait contracts,
    external trait proxies, inherited operator/conversion contracts beyond the checked
    concrete subset, arbitrary Clone relations, more loop families and richer mutable
    return values remain.
    More general supplied views, representation-lemma selection and automatic
    abstraction discovery are separate extensions beyond direct projections.
    Broader query control, ordinary-proof-first retries, application invariant
    discovery and a dedicated expected-domain interface remain separate work.
    The existing supplied invariant and regression oracle are
    still visible inputs; do not count their desired semantics as inferred.
    Aleph's outer orchestration is checked against an explicit authentication
    policy. Verifying or more directly connecting the native hash/as_ref/index/
    Keychain::verify chain, reusing signature-library contracts, and inference
    relative to deliberately chosen cache-insensitive views remain separate work.

The [assurance plan](VERUS_INFER_ASSURANCE.md#later-formal-verification-targets)
tracks formal verification of production acceptance and precision transformations,
and possible independent replay. Structural safeguards and mutation tests are
implemented; those formal proofs remain open. They are a separate workstream
alongside these feature milestones.

# 1. Executive Summary

Verus Infer is a proposed extension to Verus that automatically discovers, proves, and composes formal specifications for Rust functions.

Today, Verus primarily operates on developer-supplied specifications. Developers provide preconditions (`requires`), postconditions (`ensures`), loop invariants, and supporting proofs. Verus then checks whether the implementation satisfies them.

Verus Infer would partially reverse this workflow.

Developers could opt individual functions into automated specification inference. The system would analyze the implementation, infer its safe input domain and input/output behavior, and produce reusable, machine-checked contracts.

For sufficiently tractable functions, it would aim to derive:

1. **Maximally permissive preconditions:** The complete set of inputs for which execution satisfies a specified safety policy.
2. **Maximally precise postconditions:** An exact mathematical characterization of possible results and relevant state changes.
3. **Conditional guarantees:** Additional properties that follow when callers establish stronger assumptions about inputs.
4. **Composable specifications:** Contracts that can be consumed by Verus when verifying higher-level functions.
5. **Proof-quality metadata:** Explicit distinctions between exact inference, sound approximation, incomplete analysis, and trusted assumptions.

The project intentionally does not attempt automatic verification of arbitrary Rust programs.

Instead, it focuses initially on pure, deterministic, structurally tractable functions, expanding toward collections, state machines, heap mutation, and selected bounded loops.

**The central design principle is that exact inference should be attempted automatically when practical, while correctness must never depend on the inference algorithm being complete.**

Failure to infer a useful contract should not be confused with verification failure.

# 2. Motivation

Consider the following Rust function:

```rust
fn compute(x: u64) -> u64 {
    (x + 10) * 2
}
```

In traditional Verus, developers write a specification describing its valid input domain and output behavior.

An automatic inference engine could instead derive:

```rust
requires
    x <= 9_223_372_036_854_775_797,

ensures
    result as int == (x as int + 10) * 2,
```

Under the policy that arithmetic overflow is rejected, this precondition is both sufficient and necessary for safe execution.

The postcondition precisely describes the result on that domain.

This is a straightforward example, but the same concept applies to more substantial code:

- Financial amount calculations.
- Fee calculations and rounding.
- Bounds-checked indexing.
- Serialization helpers.
- Deterministic state transitions.
- Validation functions.
- Pure portions of cryptographic implementations.

The value is not merely reducing annotation overhead.

**Automatically inferred contracts can become reusable semantic interfaces between functions**, allowing stronger guarantees to be established compositionally.

For example, if a verified transaction processor invokes several functions whose specifications have been inferred automatically, Verus could reason about the overall transaction using those contracts without having to inspect every implementation in full.

# 3. Goals and Non-Goals

## 3.1 Goals

**G1 — Opt-in, minimally invasive annotation**

Developers should be able to request inference for individual Rust functions without substantially rewriting their implementations.

**G2 — Exact inference for tractable cases**

For supported finite, loop-free computations, infer the precise safe-input predicate and input/output relation, subject to practical solver limits.

**G3 — Partial success**

When exact inference is impractical, infer useful sound properties, report any precision loss, and preserve the distinction between proven and unresolved results.

**G4 — Conditional, demand-driven reasoning**

Enable callers to derive additional guarantees from inferred specifications under caller-specific assumptions.

**G5 — First-class Verus interoperability**

Inferred specifications should participate in normal Verus verification, including modular reasoning and proof composition.

**G6 — Auditable soundness**

Every published guarantee must be justified by a machine-checked proof or explicitly marked as relying on trusted assumptions.

**G7 — Incremental adoption**

Support existing Rust codebases and permit verification to expand function by function.

**G8 — No production runtime overhead**

Inference, specifications, and supporting proofs must not change release executable behavior or require a runtime verifier.

## 3.2 Non-Goals

The initial project will not:

- Automatically prove arbitrary Rust correct.
- Infer developer intent from implementations alone.
- Guarantee inference termination for every supported function.
- Automatically discover every interesting semantic invariant.
- Verify arbitrary external I/O, databases, or concurrent execution.
- Replace explicit developer-written Verus contracts.
- Require an LLM or other probabilistic component for correctness.
- Automatically treat verified descriptive behavior as proof that a function is free from logical design bugs.

# 4. Mathematical Model

For a deterministic pure function:

\[
f:X\rightarrow Y
\]

we define a safety policy specifying which executions are considered acceptable.

Examples include excluding arithmetic overflow, panics, invalid memory accesses, or nontermination.

Let:

\[
Safe_f(x)
\]

mean that evaluating \(f(x)\) satisfies this policy.

## 4.1 Maximally permissive precondition

The ideal precondition is:

\[
P_f(x)\iff Safe_f(x)
\]

This admits every safe input and excludes every unsafe input.

A proposed inferred precondition must satisfy two separate properties.

**Soundness:**

\[
P_f(x)\implies Safe_f(x)
\]

**Maximality:**

\[
Safe_f(x)\implies P_f(x)
\]

A sound but nonmaximal precondition is useful; it simply excludes some safe inputs.

## 4.2 Maximally precise postcondition

For a terminating deterministic function, the ideal postcondition is:

\[
Q_f(x,y)\iff y=f(x)
\]

over the safe input domain.

For stateful, nondeterministic, or otherwise more general computations, this becomes a relational specification:

\[
Q_f(x,y)\iff (x,y)\in Behaviors_f
\]

A precise postcondition represents all and only the permitted observable behaviors.

For mutable functions, the relation must also include relevant pre-state and post-state information.

## 4.3 Sound approximations

When exact inference is unavailable, the system may produce a contract satisfying:

\[
P(x)\land Execute_f(x,y)\implies Q(x,y)
\]

Such a contract is valid even if its precondition is unnecessarily restrictive or its postcondition admits more results than the implementation actually produces.

Importantly, inferring a restrictive precondition does not establish that excluded inputs are unsafe.

## 4.4 Conditional guarantees

A function's exact contract may imply additional properties under specific input assumptions.

For example:

\[
result=x\cdot y
\]

together with:

\[
0\leq x,y\leq100
\]

implies:

\[
0\leq result\leq10{,}000
\]

These conditional guarantees need not all be stored explicitly.

Instead, Verus Infer should retain a reusable base specification and derive specialized consequences as needed.

# 5. User Experience and API

All syntax in this section is proposed and does not represent existing Verus functionality.

## 5.1 Minimal opt-in

```rust
#[verus_infer]
fn calculate_fee(amount: u64) -> u64 {
    amount / 100
}
```

Default behavior:

- Analyze the function.
- Infer safety conditions.
- Infer an input/output relation.
- Verify the generated specification.
- Produce a diagnostic summary.
- Make the inferred contract available to compatible verified callers.

For this function, inference could establish that all `u64` inputs are safe and:

```rust
ensures
    result == amount / 100,
```

## 5.2 Configurable inference

```rust
#[verus_infer(
    safety = "checked",
    precision = "exact",
    on_incomplete = "warn"
)]
fn calculate(x: u64, y: u64) -> u64 {
    (x + y) / 2
}
```

Proposed options:

| Option | Purpose |
|---|---|
| `safety` | Defines the required execution-safety policy |
| `precision` | Requests exact inference or permits approximation |
| `on_incomplete` | Error, warning, or reporting behavior |
| `budget` | Limits solver resources |
| `export` | Controls publication of inferred contracts |
| `diagnostics` | Selects reporting verbosity |

Requested precision is not a guarantee that inference will succeed.

## 5.3 Inference modes

Two primary modes are proposed.

### `infer_safe`

Infers a safe input domain and relevant safety properties.

```rust
#[verus_infer(mode = "safe")]
fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}
```

The system would aim to infer:

```rust
requires
    amount <= balance,
```

### `infer_exact`

Additionally infers the precise mathematical relationship between inputs and outputs.

```rust
#[verus_infer(mode = "exact")]
fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}
```

The generated contract would include:

```rust
requires
    amount <= balance,

ensures
    result as int ==
        balance as int - amount as int,
```

## 5.4 Safety semantics

The system must define safety precisely.

For example, wrapping arithmetic may be intentionally valid for some Rust functions.

Verus Infer must distinguish:

- Arithmetic defined to wrap.
- Checked arithmetic returning an error.
- Arithmetic operations required not to overflow.
- Panicking behavior.
- Aborting behavior.
- Nontermination.
- Memory safety.

The inference policy should be explicit rather than treating every overflow or panic as intrinsically invalid.

The verifier must model the selected Rust execution semantics, including relevant compilation configuration.

# 6. Proposed Architecture

Verus currently processes Rust through rustc integration, VIR-AST, VIR-SST, AIR, and SMT-LIB queries.

Verus Infer should integrate with this infrastructure rather than introduce a completely independent verification language.

Proposed architecture:

```text
             Rust source code
                    |
                    v
             Rust compiler front end
                    |
                    v
               Verus VIR
                    |
                    v
          Eligibility / effect analysis
                    |
                    v
           Contract inference engine
                    |
           +--------+--------+
           |                 |
           v                 v
     Exact symbolic      Abstract / template
       inference             inference
           |                 |
           +--------+--------+
                    |
                    v
          Candidate specification
                    |
                    v
          Verification + precision
                 checking
                    |
                    v
            Verified contract
                    |
           +--------+--------+
           |                 |
           v                 v
       Contract store    Diagnostics / API
           |
           v
      Standard Verus verifier
           |
           v
      Higher-level composition
```

## 6.1 Integration strategy

The preferred long-term architecture is a new inference component operating over Verus's verification intermediate representation.

The initial design should avoid depending on a separate MIR-only analysis pipeline unless required by a specific feature.

**Rationale:** Verus VIR already contains the type information, expressions, specifications, and verification-oriented constructs needed for many inference tasks.

However, MIR may be useful for faithfully modeling particular Rust operations, especially when lowering and borrowing semantics are relevant.

A production implementation may therefore use both:

- **VIR:** Main contract-inference and specification interface.
- **MIR/rustc information:** Supporting semantic information for eligible executable Rust constructs.
- **AIR/SMT:** Query generation and verification.

The precise hook point is an engineering decision for the prototype.

# 7. Core Components

## 7.1 Eligibility Analyzer

Determines whether a function is suitable for the available inference algorithms.

Inputs:

- Function body and signature.
- Rust types.
- Control-flow structure.
- Called functions.
- Mutability and effects.
- Existing Verus annotations.

Example classifications:

```text
ExactCandidate
BoundedCandidate
AbstractCandidate
RequiresExternalContracts
Unsupported
```

Eligibility must not itself imply successful inference.

## 7.2 Symbolic Execution Engine

For eligible functions, constructs a symbolic representation of their execution.

Consider:

```rust
fn abs_diff(a: u64, b: u64) -> u64 {
    if a >= b {
        a - b
    } else {
        b - a
    }
}
```

A symbolic engine may derive:

\[
result=
\begin{cases}
a-b,&a\geq b\\
b-a,&a<b
\end{cases}
\]

It can then establish that the function is safe for all `u64` inputs.

The engine should favor path merging and symbolic conditional expressions over exhaustive enumeration of execution paths.

## 7.3 Safety Precondition Synthesizer

Derives conditions required for execution to satisfy the requested safety policy.

Potential techniques:

- Weakest-precondition calculus.
- Symbolic execution.
- SMT constraint solving.
- Predicate simplification.
- Counterexample-guided refinement.

For functions with multiple potentially failing operations, safety preconditions must account for the conditions under which those operations are actually reached.

A branch containing an overflowing expression does not necessarily make all function inputs unsafe.

## 7.4 Postcondition Synthesizer

Constructs mathematical relationships between:

- Input values.
- Return values.
- Modified state.
- Observable error outcomes.

For simple deterministic functions, an exact postcondition should characterize the result uniquely.

For mutable functions, the specification should include frame conditions describing preserved state.

For example, swapping two array elements should also establish that all other elements are unchanged.

## 7.5 Abstract Interpretation Engine

Provides sound approximations when exact symbolic inference is too expensive.

Initial abstract domains may include:

- Numeric intervals.
- Constant propagation.
- Equality relationships.
- Selected linear relationships.
- Known bits or bit masks.

The implementation should support expanding the set of abstract domains without changing the contract API.

## 7.6 Contract Simplifier

Translates inferred logical formulas into smaller, more comprehensible specifications.

Examples:

- Simplify arithmetic bounds.
- Remove redundant predicates.
- Recognize equivalent expressions.
- Produce readable conditional guarantees.
- Retain exact internal representations when simplification would lose precision.

A human-readable approximation must never silently replace an exact contract while retaining an exactness claim.

## 7.7 Contract Registry

Stores proven specifications for reuse across verification tasks.

Registry entries should include:

- Stable function identity.
- Implementation fingerprint.
- Relevant dependency fingerprints.
- Parameter and return types.
- Safety policy.
- Preconditions.
- Postconditions.
- Precision classification.
- Proof result.
- Trusted assumptions.
- Verus and solver versions.

Cache invalidation should account for modifications to called functions and imported contracts, not just changes to the annotated function's source.

## 7.8 Demand-Driven Guarantee Synthesizer

Allows callers to ask questions about an inferred contract.

For example:

```rust
fn fee(amount: u64) -> u64 {
    amount / 100
}
```

A caller might need to prove:

```text
amount < 100 implies fee(amount) == 0
```

Another caller might need:

```text
fee(amount) <= amount
```

Both follow from the same exact mathematical model.

The system should prove these consequences on demand rather than eagerly enumerate possible guarantees.

This is particularly important because the set of valid consequences of a specification is generally infinite.

# 8. Verification and Trust Model

The inference engine must not be part of the trusted correctness boundary.

It is permitted to generate incorrect candidate contracts.

Those candidates must be rejected unless a trusted verification path establishes their soundness.

## 8.1 Proof pipeline

For each inferred contract:

1. Generate candidate preconditions and postconditions.
2. Construct proof obligations against executable behavior.
3. Submit obligations through Verus.
4. Publish the contract only after successful verification.
5. Record any trusted assumptions.

For exactness claims, additional proof obligations are required.

## 8.2 Optimality checking

A contract being sound does not mean it is optimal.

The tool must distinguish:

**Verified sound**

All included guarantees hold on the admitted input domain.

**Verified exact behavior**

The inferred input/output relation accurately characterizes execution on the admitted domain.

**Verified maximal safe domain**

All excluded inputs violate the selected safety policy.

**Sound approximation**

The contract is correct, but the tool has not established maximality or completeness.

**Incomplete**

The tool could not finish its analysis or verification.

## 8.3 No silent assumptions

The inference engine must not make a candidate pass by inserting unreported assumptions, marking implementations external, or weakening required specifications.

Any reliance on trusted functions, axioms, or external models must be visible.

For higher-level proofs, trust assumptions should be transitively traceable through dependencies.

## 8.4 Verifying exactness

For suitable deterministic finite computations, exactness may be established through a relational equivalence check between implementation semantics and the inferred summary.

For more general code, proving maximality and exactness can be harder than proving the contract sound.

The system must support declining an exactness claim without invalidating already verified sound properties.

# 9. Handling Increasing Program Complexity

## 9.1 Phase-one supported subset

The initial implementation should prioritize:

- Integer and Boolean parameters.
- Integer arithmetic and bitwise operations.
- Comparisons.
- Local variables.
- Immutable computations.
- Conditional branches.
- Pattern matching on simple enums.
- `Option` and `Result`.
- Calls to other functions with approved contracts.
- Loop-free control flow.

Functions using unsupported constructs should receive explicit diagnostics.

## 9.2 Loops

Unbounded loops introduce the need for termination reasoning and invariant discovery.

However, automatically analyzing loops should not be categorically ruled out.

The system should distinguish:

**Statically bounded loops:** Can be unrolled when cost is reasonable.

**Structurally bounded loops:** May be handled through inferred bounds or summarization.

**Template-recognizable loops:** Common patterns such as summation, counting, scanning, and minimum/maximum can be handled with invariant templates.

**General loops:** May require supplied invariants or remain unsupported.

A complete exactness claim cannot be derived from incomplete loop unrolling.

## 9.3 Recursion

Recursive functions require similar handling through:

- Termination measures.
- Inductive summaries.
- Recognized recursive patterns.
- Developer-provided specifications.

General recursive invariant inference is outside initial scope.

## 9.4 Collections

Initial collection support could include:

- Fixed-size arrays.
- Index safety.
- Length relationships.
- Simple element-wise updates.
- Frame conditions.

Later support may include slices, vectors, maps, and dynamic collections.

Quantified properties over arbitrary collection elements may require additional solver guidance.

## 9.5 Nonlinear arithmetic

Nonlinear arithmetic and symbolic multiplication can be computationally expensive.

The system should preserve exact bitvector representations where possible but permit resource-limited solving and sound fallback abstractions.

A solver timeout must not be treated as evidence that a contract is invalid or maximal.

## 9.6 External dependencies

Calls into external or unsupported code require explicit contracts or conservative effect summaries.

The inference engine cannot establish arbitrary guarantees about databases, network operations, filesystems, or external cryptographic libraries without modeled semantics.

## 9.7 Concurrency

General concurrent execution is deferred.

Future extensions may integrate with Verus's state-machine and permission-based verification mechanisms.

Concurrency would require reasoning about interleavings, synchronization, shared state, and environmental interference.

# 10. Composition of Inferred Contracts

A principal goal is enabling higher-level proofs without reanalyzing every callee.

Suppose function \(f\) has contract:

\[
\{P_f\}\ f\ \{Q_f\}
\]

and function \(g\) has contract:

\[
\{P_g\}\ g\ \{Q_g\}
\]

For a caller invoking \(g(f(x))\), Verus must establish:

1. The caller's assumptions imply \(P_f\).
2. The postcondition \(Q_f\) establishes the conditions needed for \(P_g\).
3. The combined postconditions imply the caller's requested guarantee.

The inference engine's job is to make the callee specifications sufficiently precise to support this reasoning.

## 10.1 Demand-driven strengthening

If a caller cannot establish a property from a sound but imprecise callee contract, Verus Infer may attempt to refine that contract.

For example:

```text
Initial inferred guarantee:
    result >= 0

Caller needs:
    result <= amount
```

The tool could reanalyze the callee or consult a previously retained exact symbolic summary.

If successful, it publishes an additional verified lemma or stronger contract.

If unsuccessful, it requests explicit developer assistance.

## 10.2 Compositional state invariants

For a state transition function:

\[
T:S\times Input\rightarrow S
\]

suppose a property \(I(S)\) satisfies:

\[
I(S)\land ValidInput(S,x)
\implies I(T(S,x))
\]

If the initial state satisfies \(I\), induction establishes that every reachable state produced by a sequence of valid transitions satisfies the invariant.

Verus Infer could automate lower-level arithmetic and transition contracts used in this proof.

The higher-level invariant itself would normally still be explicitly supplied.

# 11. Fedimint-Inspired Use Case

A motivating application is financial accounting in a distributed Rust system such as Fedimint.

The initial goal would not be to prove an entire federation correct.

Instead, the tool could infer contracts for pure monetary operations and use them to support explicit higher-level safety properties.

## 11.1 Amount calculation

```rust
#[verus_infer(mode = "exact")]
fn required_funding(outputs: u64, fee: u64)
    -> Option<u64>
{
    outputs.checked_add(fee)
}
```

The exact contract would characterize both success and overflow cases:

\[
result=
\begin{cases}
Some(outputs+fee), & outputs+fee\leq MAX\\
None, & outputs+fee>MAX
\end{cases}
\]

Here the sum is mathematical, not wrapping arithmetic.

No input restriction is necessary, because overflow is handled explicitly.

## 11.2 Transaction validation

```rust
#[verus_infer(mode = "exact")]
fn is_funded(
    inputs: u64,
    outputs: u64,
    fee: u64,
) -> bool {
    outputs
        .checked_add(fee)
        .is_some_and(|required| inputs == required)
}
```

An inferred specification would describe exactly when the function returns true.

A higher-level explicitly written invariant could then use the verified funding predicate.

## 11.3 State transition verification

Eventually, a financial state transition may have an explicitly declared invariant such as:

```text
An accepted transfer preserves the total represented
value and does not produce an invalid balance.
```

Verus Infer would automatically establish relevant arithmetic subproperties.

Verus would then compose those properties into the developer-specified state invariant.

**This illustrates the intended division of responsibility: automatic inference describes how code behaves; explicit system specifications define which behaviors are acceptable.**

# 12. Diagnostics and Developer Workflow

An example command-line workflow:

```bash
cargo verus infer
```

The command is hypothetical.

It could produce:

```text
Analyzing compute(u64) -> u64

Safety precondition:
  x <= 9223372036854775797

Output relation:
  result == (x + 10) * 2

Verification:
  Soundness: PROVED
  Safe-domain maximality: PROVED
  Output exactness: PROVED

Dependencies:
  None

Trusted assumptions:
  Standard arithmetic model

Contract published.
```

For a more complex function:

```text
Analyzing process(Vec<u64>)

Safety:
  PROVED under inferred precondition

Output relation:
  Sound interval approximation

Exactness:
  NOT ESTABLISHED

Reason:
  Unsupported dynamic collection mutation

Suggested next step:
  Provide a loop invariant or enable collection analysis.
```

Diagnostics should distinguish:

- Unsupported syntax.
- Solver timeout.
- Verification counterexample.
- Missing dependency contract.
- Candidate contract too weak for a caller.
- Failed exactness or maximality proof.
- Unresolved environmental assumptions.

Ideally, inferred specifications should also be viewable through editor tooling without requiring developers to permanently insert verbose generated annotations into source files.

# 13. Implementation Plan

## Phase 0 — Integration Research

**Goal:** Validate the best insertion points in the current Verus architecture.

Tasks:

- Study VIR-AST, VIR-SST, and AIR interfaces.
- Investigate existing `dual_spec` support.
- Determine how inferred contracts can be exposed to normal call-site verification.
- Evaluate reusable symbolic execution and SMT infrastructure.
- Establish a minimal experiment using a pure Rust arithmetic function.

**Deliverable:** Architectural prototype and integration decision.

## Phase 1 — Exact Loop-Free Arithmetic

**Goal:** Produce a working end-to-end inference prototype.

Supported features:

- Integer and Boolean inputs.
- Arithmetic and comparisons.
- Branches.
- Simple local variables.
- Pure loop-free functions.

Implement:

- Symbolic execution.
- Safety-predicate extraction.
- Exact relational summaries.
- SMT verification.
- Basic simplification.
- Exactness diagnostics.

**Exit criterion:** The tool correctly infers and proves contracts for a curated set of arithmetic and branching functions without developer-supplied `requires` or `ensures`.

## Phase 2 — Modular Composition

**Goal:** Make inferred contracts useful to callers.

Implement:

- Function-call summaries.
- Contract registry.
- Dependency tracking.
- Cross-function proof obligations.
- Demand-driven conditional guarantees.
- Contract caching.

**Exit criterion:** A higher-level verified function can establish a new property by composing inferred contracts from several callees, without inlining their implementations.

## Phase 3 — Partial Inference and Collections

**Goal:** Expand coverage without sacrificing soundness.

Implement:

- Abstract interpretation.
- Fixed-size arrays.
- Basic slice properties.
- Additional `Option`/`Result` handling.
- Frame-condition inference.
- Sound fallback summaries.

**Exit criterion:** Functions outside the exact subset can still produce useful proven contracts, with accurate precision labels.

## Phase 4 — Loop Inference

**Goal:** Support selected iterative algorithms.

Implement:

- Loop-bound analysis.
- Common invariant templates.
- Candidate invariant generation.
- Inductive invariant verification.
- Counterexample-guided refinement.

**Exit criterion:** Automatically verify selected accumulator, search, and collection-traversal functions without handwritten loop invariants.

## Phase 5 — Richer Verification

Potential subsequent work:

- Mutable heap state.
- Trait and generic specialization.
- More sophisticated refinement relations.
- State-machine summaries.
- Integration with AI-assisted invariant generation.
- Incremental verification across large crates.
- IDE presentation and automated contract suggestions.

These are research directions, not commitments for the initial project.

# 14. Testing and Evaluation

The evaluation plan must measure both correctness and usefulness.

## 14.1 Correctness tests

For simple finite functions:

- Compare inferred predicates against exhaustive enumeration for small integer domains.
- Compare inferred output relations against concrete execution.
- Validate behavior at arithmetic boundaries.
- Test panic and overflow conditions.
- Test branching, pattern matching, and error returns.

Exhaustive small-domain testing provides additional confidence but does not replace mathematical verification.

## 14.2 Soundness regression tests

Every published guarantee must remain independently verifiable through the designated verification backend.

Regression tests should include deliberately buggy inference passes to demonstrate that invalid candidate contracts are rejected.

## 14.3 Precision evaluation

Measure:

- Fraction of functions receiving sound contracts.
- Fraction receiving exact contracts.
- Fraction receiving proven maximal safe domains.
- Number of unsupported operations.
- Solver timeouts.
- Precision loss relative to known reference contracts.

A sound approximation must not be counted as exact inference.

## 14.4 Composition evaluation

Measure how often inferred callee contracts are sufficient to prove real caller properties without manual intervention.

This is arguably a better measure of practical value than simply counting the number of inferred annotations.

## 14.5 Performance evaluation

Track:

- Inference time per function.
- Solver time.
- Formula size.
- Peak memory usage.
- Cache reuse.
- Incremental rebuild costs.
- Growth in verification time as call graphs become deeper.

The tool should expose bounded resource budgets and predictable failure behavior.

# 15. Major Risks

## 15.1 Inference versus verification mismatch

**Risk:** Generated contracts do not accurately reflect actual Rust execution semantics.

**Mitigation:** Reuse Verus/rustc semantics, validate generated contracts through the verifier, and avoid unsupported language constructs.

## 15.2 False claims of maximality

**Risk:** A sound, restrictive precondition is incorrectly described as the weakest possible precondition.

**Mitigation:** Require separate maximality proof obligations and report exactness status independently.

## 15.3 Solver scalability

**Risk:** Path explosion, nonlinear arithmetic, and quantified collection properties make inference expensive.

**Mitigation:** Resource limits, path merging, approximation, modular summaries, and caching.

## 15.4 Unreadable contracts

**Risk:** Exact specifications become too complicated for humans to understand.

**Mitigation:** Separate internal exact representations from simplified presentation contracts, preserving provenance and proof status.

## 15.5 Weak inferred semantics

**Risk:** Functionally correct descriptions are mistaken for meaningful application-level guarantees.

**Mitigation:** Keep system-level intent specifications explicit and provide separate checks against those specifications.

## 15.6 Maintenance burden

**Risk:** Changes in Rust or Verus internals break inference integration.

**Mitigation:** Isolate Verus-specific adapters, minimize invasive changes, and seek upstream collaboration once the architecture is proven.

# 16. Initial Development Decisions

The recommended starting decisions are:

| Decision | Recommendation |
|---|---|
| Project structure | Experimental Verus fork initially |
| Source language | Supported Rust/Verus subset |
| Primary IR | VIR |
| Rust semantic integration | Reuse rustc information when required |
| Solver | Existing Verus Z3 infrastructure |
| First algorithms | Symbolic execution + weakest-precondition analysis |
| First target | Pure loop-free numeric functions |
| Exactness | Explicitly proved, never assumed |
| Fallback behavior | Sound approximation or incomplete result |
| Contract storage | Versioned, dependency-aware registry |
| User interface | Opt-in attribute plus CLI |
| Higher-level verification | Existing Verus contracts and proof mechanisms |
| AI dependency | None required |

A standalone inference prototype may be easier to build initially, but the long-term design should ensure that inferred contracts become natural inputs to the existing Verus verifier.

# 17. Open Questions

The following should be resolved through implementation experiments.

1. **Inference IR:** Should the engine primarily operate on VIR-AST, VIR-SST, or a specialized symbolic IR?
2. **Existing `dual_spec`:** How much functionality can be reused from automatic executable-to-specification translation?
3. **Exactness proof:** What is the cleanest way to prove equivalence between executable behavior and inferred contracts?
4. **Arithmetic semantics:** How should checked, wrapping, saturating, and configuration-dependent behavior be represented?
5. **Function purity:** What is the best method for classifying side effects and determining eligible functions?
6. **Public interfaces:** How should inferred contracts become stable interfaces for external crates?
7. **Conditional inference:** When should additional guarantees be generated eagerly versus on demand?
8. **Contract simplification:** How can human-readable representations preserve exact logical meaning?
9. **Inference budgets:** What resource limits produce predictable performance in developer workflows?
10. **Upstream strategy:** Which components should eventually become part of Verus, and which should remain optional?

# 18. Long-Term Vision

The desired end state is a verification workflow in which developers can progressively opt code into stronger guarantees without writing a complete formal specification for every function.

For simple code, contracts are inferred and proved automatically.

For more complicated code, the system generates the strongest sound information it can establish within its analysis budget and requests human guidance where necessary.

For higher-level code, inferred guarantees are composed with explicitly written semantic invariants.

Over time, the system becomes a reusable collection of machine-checked facts about a Rust codebase.

Rather than requiring developers to formally specify every low-level operation, they can focus on meaningful application-level properties:

- Value cannot be created by a transfer.
- A note cannot be redeemed twice.
- An unauthorized transition cannot succeed.
- A parser cannot access memory outside its input.
- A successful state transition preserves required invariants.

Verus Infer does not attempt to eliminate the need for human specifications.

**It attempts to make the mechanical construction of those specifications—and the proofs supporting them—as automatic as possible.**

---

# 19. References and Related Work

- [Verus repository](https://github.com/verus-lang/verus)
- [Verus architecture overview](https://github.com/verus-lang/verus/blob/main/source/CODE.md)
- [Verus tutorial and reference](https://verus-lang.github.io/verus/guide/)
- [Verus automatic exec-to-spec generation](https://verus-lang.github.io/verus/guide/exec_to_spec.html)
- [Verus specification and proof modes](https://verus-lang.github.io/verus/guide/modes.html)
- [Microsoft Verus proof synthesis research](https://github.com/microsoft/verus-proof-synthesis)

Related techniques include weakest-precondition calculus, symbolic execution, abstract interpretation, counterexample-guided abstraction refinement, inductive invariant synthesis, and SMT-based deductive verification.

**End of draft RFC.**
