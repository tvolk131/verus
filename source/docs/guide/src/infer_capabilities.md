# Inference capabilities and authored inputs

This map locates the implemented profiles. A listed area is not general Rust
support: follow its reference for signature, shape, observation and budget
restrictions. Use a matching compiler/macros/vstd bundle and the current version
identities at the top of the [feature reference](infer.md).

The author chooses the application property and required input domain.
Inference describes implementation behavior; assistance proposes ordinary proof
steps. Neither chooses the intended policy. An application function can prove
an authored contract over inferred helpers without itself being an exact
inference candidate.

| Area | Implemented automation | Inputs that remain authored |
| --- | --- | --- |
| [Scalar arithmetic and control flow](infer.md#supported-code-and-safety-policy) | Exact bounded integer/Boolean summaries, supported branches, casts and arithmetic policies; explicit bounded sound approximations | Required input coverage and intended behavior; handling of inputs outside the checked safe domain |
| [Flat scalar-pair outputs](infer.md#exact-flat-scalar-pairs) | Whole-pair exact contracts, immutable pair locals/projections and eager safety for both operands | The meaning and required domain of the two outputs; tuple inputs and nested tuples remain unsupported |
| [Scalar wrappers and operators](infer.md#scalar-wrappers-and-inherent-methods) | Supported projections, conversions, assignment/binary operators and explicit native derives, checked against actual implementations | Public abstraction and trait meaning beyond supported structural models; required domain |
| [Arrays and shared slices](infer.md#supported-code-and-safety-policy) | Supported finite values, indices, subslices and complete output observations | Semantic meaning of contents; goals connecting representation to that meaning |
| [Option/Result and callbacks](infer.md#shared-native-option-calls-in-exact-readonly-values) | Bounded tagged values, shared projections, readonly native Option callbacks and selected typed error-mapping assistance | Success/error policy, error priority and dependency/clone laws needed by the application |
| [Vectors and mutable records](infer.md#bounded-mutable-records) | Supported read/effect summaries, exact modeled record state and unchanged-state frames; explicitly sound single-slot vector updates | Views, validity predicates and intended transition properties |
| [Private representations](infer.md#private-vector-wrappers-with-supplied-views) | Composition through supported supplied sequence/tuple views and checked observation contracts | The abstraction, distinctions it retains and semantic interface laws |
| [Opaque foreign fields](infer.md#declared-opaque-foreign-fields-in-readonly-queries) | Exact shared-reference queries retain eligible declared foreign types as whole nested leaves | An explicit external type declaration and dependency trust; no representation or crypto law is discovered |
| [Owned construction and finite folds](infer.md#owned-collection-construction-and-round-advancement) | Bounded construction/advancement profiles; checked fold decorations inside selected larger ordinary functions | Abstract result meaning and contracts for opaque suffixes or external operations |
| [Numeric counted loops](infer.md#counted-numeric-loops) | Exact supported unsigned accumulator patterns, inductive annotations and termination measures | The caller's intended arithmetic property and expected domain; arbitrary loops are outside this profile |
| [Growth/range loops and selectors](infer.md#checked-growth-and-transition-loops) | Supported prefix induction, frames, bounded history growth and mutable-selector contracts | Invariants, final-state/length/selection goals and checked transition interfaces |
| [Clone transport](infer.md#checked-pointwise-transport-through-cloned-fields) | Checked pointwise bridges for supported invariant-preservation goals through cloned fields | The supplied invariant and checked clone relation; Rust `Clone` alone does not imply arbitrary identity |
| [Ordinary scalar caller goals](infer.md#demand-driven-scalar-proof-assistance) | Bounded checked bit-vector proposals from exact published relations, including supported records, arrays, casts and relational premises | The assertions/contracts to prove; unsupported or failed proposals do not establish them |
| [Word-view goals](infer.md#bounded-u64-word-view-assistance) | Scoped selection of checked membership, sequence-update/count and padding laws for supplied generic word views | The semantic invariant, domain and explicit representation vocabulary; arbitrary wrapper bridges remain unsupported |
| [Cross-crate use](infer.md#cross-crate-inference-and-cargo) | Certified interface composition, dependency/model fingerprints and candidate reuse with fresh proofs | Dependency specifications and trust assumptions; unverified callers must respect preconditions |

## Diagnose the boundary

Use `--infer-goals --output-json` to inspect proposals and required proof
outcomes separately. [Assistance explanations](infer.md#assistance-explanations)
currently cover growth loops, range loops and pointwise clone transport. Other
strategies retain their separate assistance fields; an absent decision does not
mean a strategy was attempted or failed.

Keep these outcomes distinct:

- Unsupported syntax or profile shape needs an adapter, another profile, an
  authored proof or a bounded engine extension.
- A generation budget limit is not a solver timeout or a counterexample.
- A proposed annotation or exact helper contract can coexist with a failed
  application goal. Inspect the original goal and required domain.
- Successful inference does not establish every required input is admitted.
  An ordinary application caller can check that the expected domain implies
  the inferred precondition.
- An imported assumed model is a trust input, not a newly verified native body.

General recursion, arbitrary invariant/abstraction discovery, general mutable
iteration and general unsafe Rust are not implemented by these bounded profiles.
The macro-loop adapter addresses syntax processing; it does not establish
iterator semantics, invariants or application properties.

## Evaluate an application slice

Follow the [adoption workflow](infer_adoption.md). Before extending the engine,
freeze original source bodies, one useful property, its domain and the semantic
definitions supplied by the author. Record executable adapters, remaining
implementation proof annotations, exclusions and trusted interfaces separately.
An honestly blocked baseline is useful evidence.

The [pinned transfer front door](https://github.com/tvolk131/verus/blob/verus-infer/source/tools/INFER_TRANSFERS.md)
replays selected Fedimint/redb helpers with source guards and fresh proof,
semantic/safety/auxiliary-negative and publication controls. Its acceptance
does not imply whole-gateway fund safety or whole-database correctness.
Progress means less manual implementation proof work for unchanged meaningful
properties and domains, with successful transfer beyond the motivating fixture.
