# Adopting Verus Infer in an existing codebase

Verus Infer is useful for deriving implementation contracts and composing them
into application proofs. An author still chooses what the program should mean
and which properties matter. Start with a useful property and a small slice of
code, then expand as checked interfaces become available.

This chapter describes the working prototype on this branch. Use a matching
Verus compiler, macros and vstd; released tools do not necessarily include these
extensions. The [feature reference](infer.md) records supported syntax and limits.
General Rust support, arbitrary loops, automatic application-invariant discovery
and automatic discovery of private-data abstractions are not implemented.

## Start with a property and its inputs

Choose a concrete goal: a counter accurately describes a collection, an update
preserves unrelated entries, a parser reports truncation, or validation rejects
invalid inputs in a specified order. Define the intended input domain as well.
For each dependency in that slice, identify the contract the proof needs and
whether its implementation is verified or modeled.

Keep expected behavior independent of the inferred description of the code.
Proving that a function satisfies its inferred contract establishes a checked
description of its implementation; a separate application goal checks whether
that description meets your requirements.

For example, this ordinary Rust helper needs one annotation:

```rust
{{#include ../../../../examples/verus_infer_adoption.rs:debit}}
```

Inference derives a safety requirement equivalent to `amount <= balance` and the
mathematical subtraction equation. Supply the intended coverage and behavior in
an ordinary verified caller:

```rust
{{#include ../../../../examples/verus_infer_adoption.rs:coverage}}
```

The caller checks both that the intended inputs satisfy the inferred requirement
and that the result has the desired meaning. If the helper changes to addition,
these goals fail even if inference correctly describes the changed implementation.

An inferred function cannot also have handwritten `requires`/`ensures`. Use an
ordinary function with an explicit application contract over inferred helpers,
as above, or verify the implementation directly with an authored contract.

Do not narrow the intended domain merely to make a proof pass. If arbitrary
inputs should be accepted, implement runtime handling with guards, `Option` or
`Result`, and verify that policy. Ordinary Verus preconditions are obligations on
verified callers; they do not automatically become runtime checks for ordinary
Rust callers. See [calling verified code from unverified code](call-from-unverified-code.md).

## Infer supporting behavior, then check the application goal

Work outward from small helpers: arithmetic, accessors, supported collection
updates, constructors and error-handling paths. Use `#[verus_infer]` on ordinary
Rust functions, or `#[verifier::infer]` inside `verus!`. Inference composes checked
helper summaries across the supported call graph; you need not inline every
implementation into the application proof.

Default exact mode checks soundness, safe-domain maximality and output exactness
against the original executable body, using the supported logical observations.
Those are precision guarantees about implementation behavior. They do not choose
your application policy or prove that an external model matches its real library.

Sound mode is an explicit option for a sufficient contract without both precision
guarantees. Use it when that weaker result meets your needs or a documented
profile requires it; inspect which obligations were actually established. An
unsupported exact-mode operation does not automatically become supported by
selecting sound mode.

Top-level application proofs need not themselves be inference candidates. An
ordinary `requires`/`ensures` contract can compose inferred helpers and existing
verified library APIs. It can express the property clients actually need even
when exact output inference is unavailable for that function's representation.

Concrete domain error enums make error payloads and precedence available to
checked callers. Type-erased library errors can introduce a separate integration
boundary. Where appropriate, retain a concrete error in the checked core and
perform erasure at an outer API boundary. Treat a signature or error-representation
change as an explicit API refactor: inspect all callers, preserve the intended
diagnostics/behavior, run native tests and state exactly which interface is proved.

## Supply views and invariants at abstraction boundaries

A view describes the abstract value exposed by a data structure. Choose whether
positions, missing entries, order, multiplicity, cache state and object identity
are relevant to the properties you intend to prove. A view need not expose every
field, but it must retain distinctions required by those properties.

The example uses a private record with an author-defined view and validity
predicate:

```rust
{{#include ../../../../examples/verus_infer_adoption.rs:view}}
```

Inside the module, inference derives the field effects of a supported update.
An ordinary public method connects those effects to the intended abstract API:

```rust
{{#include ../../../../examples/verus_infer_adoption.rs:state}}
```

The public contract states the balance change, capacity preservation and validity
preservation. Its body needs no additional proof statements. This example's
`valid()` is an ordinary predicate: callers establish it, the constructor proves
it, and `spend` proves preservation. It is not automatically assumed as a type
invariant, and inference does not discover it.

A closed invariant can hide its meaning from clients as well as its physical
representation. If callers need to derive facts such as a count bound or parent
eligibility, expose those facts in the public semantic contract, or define an
open predicate using the public view. The view's implementation can remain
closed. Choose this interface deliberately: proving that an opaque predicate is
preserved does not by itself give clients its underlying field equations.

For a private sole-vector wrapper, a supplied direct sequence view also enables
supported collection APIs to compose without exposing storage. The
[private collection example](../../../../examples/verus_infer_private_views.rs)
defines slot contents and a counting invariant; the
[trait example](../../../../examples/verus_infer_traits.rs) supplies logical
getter contracts for generic events. These examples include supporting contracts
and, where needed, explicit queries that expose representation bounds.

Providing a view does not make every operation inferable. General view-relative
inference remains limited. In particular, objects with caches or interior
mutability can require ordinary verified APIs and representation invariants.
For Aleph, cloned `FullUnit` error values preserve the authored logical view;
the outer validator is checked directly against that policy rather than claiming
an exact inferred contract for every cache-bearing output.

Public inferred constructor contracts must respect the same abstraction boundary
as authored contracts. Supported private records can use a supplied tuple view of
their scalar state, complete forwarded inputs and collection contents. Inference
checks that the view retains the modeled output; it does not assume injectivity.
Reconstructing another private record inside that view is a limited profile:
use a flat view of supported components or an ordinary verified semantic API.
For public iterator predicates over private elements, name their supplied public
observations in application contracts as well.

## Use explicit contracts and proofs where they add meaning or bridge a gap

Two bounded assistance profiles also avoid extracting implementation helpers.
An ordinary immutable-input function can construct a local collection with one
supported native fold and then return a larger record or call an opaque API.
`--infer-goals` proposes only the fold's proof annotations; the complete original
function still proves its semantic contract. The opaque suffix receives no
functional equation and the ordinary constructor receives no exact certificate.
The report lists this separately as `local_fold_assistance`.

An append-only growth loop followed directly by a mutable vector-element return
can receive generated induction and selector contracts. Authors supply validity
and the selected element's semantic requirements. The compiler proposes bounded
growth, unchanged fields/earlier positions, initial selected-value identity and
conditional validity restoration when the caller finishes the borrow. All these
facts must verify against the original body; neither an extracted preparation
method nor a handwritten borrow-frame contract is required. Other algorithms
remain outside these bounded profiles.

Explicit application contracts are useful even when inference succeeds: they
record the intended invariant, input coverage, error policy and frame conditions.
Trait contracts similarly specify what generic code may rely on. Rust's `Clone`
or `Eq` alone does not establish every logical identity an application might need;
use appropriate checked laws or library interfaces.

Explicit proofs are useful when a required fact is outside the current
assistance profiles. Examples include an induction connecting a recursive
reference count to a library counting operation, a representation-equivalence
lemma, a custom Clone law, or a protocol-specific invariant. Put reusable facts
in checked lemmas instead of repeating proof steps at each call site.

Try `--infer-goals` for the documented assistance strategies. It can select
checked counting, fold and view-bound lemmas and propose certain scalar proofs.
Bounded growth/range profiles can supply loop induction and internal index
triggers, and a checked cloned-field transition can receive intermediate
pointwise transport assertions. The source contracts and chosen invariants
remain ordinary obligations.
Checked sparse-count lemmas also connect positional `count_prefix` accounting
to sparse iterator length after mapping and filtering. Their pointwise
compatibility and equal-length conditions must follow from the supplied views
and contracts. Selecting the lemmas does not assume those conditions or an
arbitrary relationship between two counting expressions.
Direct mutable element selectors following a checked preparation call can also
receive generated identity/frame contracts and conditional invariant-restoration
facts. The authored contract still defines the selected value's application meaning.
It does not provide an unrestricted proof search or automatically discover all
necessary invariants. For comparison, these two callers establish the same
bound:

```rust
{{#include ../../../../examples/verus_infer_adoption.rs:proof}}
```

Here `bucket(key)` is inferred from `(key & 15) as u8`. The first caller uses
`--infer-goals`; the second demonstrates an explicit ordinary bit-vector proof.
This particular fact already has automatic assistance. The explicit form shows
how a checked intermediate fact can connect a precise inferred equation to a
caller goal when an analogous automatic strategy is unavailable.

For unsupported loops or Rust APIs, use an ordinary contract and explicit
loop invariants/proofs, a checked adapter, or a semantics-preserving refactor.
Do not assume that a supported fill/fold pattern means arbitrary loops are
automatically inferable. Preserve executable behavior and run native tests when
introducing adapters or refactoring.

## Make dependency contracts and trust visible

Prefer verified dependency implementations and reusable library specifications.
Application developers should be able to reuse a container or serialization
library's verification work rather than rebuilding its model for each project.

When only a model is available, document its contract, the implementation/revision
it stands for, and what remains assumed. An externally assumed encoding equation
is a trust boundary; a proved lemma relating two specification definitions is
checked proof work. Both may appear in a project, but they provide different
evidence. Verification remains conditional on the stated assumptions.

Check integration logic around a dependency too. A signature-checking contract
does not by itself prove that the caller passes the correct message, signature
and creator. Conversely, verifying that integration does not establish the
cryptographic primitive's security. Model boundaries should name that distinction.
See [assumptions and trusted components](tcb.md) and
[calling unverified code](calling-unverified-from-verified.md).

Keep the connection to production code inspectable. If verification uses a
reduced trait, replacement lock API or authentication adapter, state which bodies
are original and which are adaptations. Native tests are useful evidence about
those connections; they do not replace a missing implementation proof.

## Opting an existing Cargo workspace into verification

Follow the [Cargo setup](cargo_verus.md) for every first-party crate containing
types, traits, or implementations that the proof uses. Opting in only the caller
does not make dependency declarations available to Verus. Keep a private
container's authored `View` in its defining crate, where its fields are visible;
public contracts can use the view without exposing the representation.

For this prototype, use the matching fork's `vstd` and macros. One portable
arrangement is an exact `vstd` version in the workspace manifest and a
command-local override to the built fork:

```sh
export PATH="$VERUS_SOURCE/target-verus/release:$PATH"
export VERUS_Z3_PATH="$VERUS_SOURCE/z3"
cargo verus verify \
  --config "patch.crates-io.vstd.path=\"$VERUS_SOURCE/vstd\"" \
  -p storage -p domain -p application \
  --fwd-verus-args-to roots -- --infer-goals --no-cheating --output-json
```

Use the fork's pinned Rust toolchain for verification. A project's normal Rust
toolchain can still compile erased annotations; check that separately. The same
`--config` override is needed for ordinary Cargo builds using the local fork.
Select all first-party proof crates as roots so their checks receive the flags.
The `roots` option avoids forwarding `--no-cheating` into vstd or explicit trusted
library-specification dependencies. Such dependencies remain trust boundaries.

Annotating a datatype can activate its derived implementations too. Use
[`external_derive`](exec_attr.md#selecting-derived-implementations) for unused
codec, arithmetic, or other derives. Excluded implementations cannot be called
from verified code; they acquire no trusted contract. Prove the selected
operations separately. External trait declarations can make otherwise missing
marker bounds available without asserting behavior; their bounds must match the
actual trait, including associated-type and supertrait bounds.

For example, an arithmetic wrapper can keep its real `Sum` derive for ordinary
Rust callers while checking only the operations needed by a proof:

```rust
#[verus_verify(infer_derive(Add, From, Into))]
#[verus_verify(external_derive(Sum))]
#[derive(Clone, Copy, derive_more::Add, derive_more::From,
         derive_more::Into, derive_more::Sum)]
pub struct Quantity(pub usize);

#[verus_infer]
pub fn unwrap(value: Quantity) -> usize { value.into() }
```

The reverse owned conversion generated by `Into` is checked against the actual
`From<Quantity> for usize` body. Its generated extension definition is a candidate,
not evidence. The same normal three exact checks and inherited trait-domain
checks apply. `external_derive(Sum)` neither changes Rust execution nor provides
a usable verified `sum` contract. A verified call to that excluded implementation
is rejected. Codec derives unused by the selected proof can be treated the same
way; an operation that relies on encoding behavior needs an adequate checked or
explicitly inventoried library interface instead.

The [native integration example](../../../../source/tools/native_adoption/README.md)
uses real arithmetic/conversion/Sum/codec derives, tests ordinary Rust behavior,
and rejects verified use of the excluded methods. Its runner records commands,
source/binary identities and full proof reports. This is a general example, not
an application proof template or permission to assume a useful dependency law.

For a trait with unverified implementations, consider an `obeys_*` predicate
that defaults to false, as in [vstd's trait specifications](external_trait_specifications.md#the-obeys_-pattern-in-vstd).
Guard useful getter guarantees with that predicate. A checked implementation can
define it to true and prove its getter bodies. A concrete coverage wrapper should
discharge this premise for the real application type.

Choose library guarantees according to the theorem. A slot-counting invariant
may hold for every hash value returned by a dependency, without any equation for
that hash or any cache invariant. Keep those results distinct: accounting does
not establish hash-cache consistency. An ordinary wrapper can call an opaque
dependency and pass its result to an exact inferred state-update helper.

`--no-cheating` and the JSON field `is-verifying-entire-crate` do not mean every
Rust item was opted into verification. Review the annotated scope, unverified
trait implementations, and imported library specifications too. Use fresh
verification and candidate-cache outputs for final evidence; repeat native tests
and make deliberate implementation mutations fail the independent application
contracts.

The JSON `verus-infer.verification_scope` object helps inspect compiled source
scope, including for crates with zero inference candidates. It lists local native
functions with source locations and distinguishes checked executable/proof
bodies, specification definitions, trait interfaces, external specifications,
unverified bodies with contracts, and items outside the selected scope. A filtered,
failed or `--no-verify` run does not claim successful full-scope body verification.

Read this alongside the existing per-candidate certificates and the authored
property/coverage inventory. `complete_selected_scope_verified` refers to the
selected Verus items, not every Rust function. `checked_exec_bodies = 0` is not a
successful proof of unannotated application implementations. The native list
contains only compiled cfg-enabled functions and includes generated methods;
outside scope can mean unannotated, excluded or automatically ignored code. This
presentation is not a certificate, does not change verification/publication, and
does not establish that unverified Rust callers satisfy a method's preconditions.

For the authored side of that inventory, the standalone
[source parser](../../../../source/tools/adoption_inventory/README.md) lists
source contracts, views, attributes, assumptions and proof syntax before macro
expansion. It preserves opaque macros for review and includes cfg-disabled code.
Reconcile its explicitly supplied files with the compiled scope; it does not
decide whether a contract is the right application property or replace proof.

## Diagnose failures before adding hints

Different failures call for different next steps:

| Outcome | What to investigate |
| --- | --- |
| Rust/frontend or unsupported-source error | Check the feature reference and dependency interfaces; simplify, adapt, or use ordinary verification. |
| Inferred precondition fails at a caller | Compare the intended input domain with the actual arithmetic, indexing or panic conditions. |
| Application postcondition fails after inference succeeds | Check for an implementation/policy mismatch, then missing bridge facts or inadequate helper contracts. |
| Candidate soundness or precision check fails | Treat inference as unsuccessful; investigate the candidate/profile rather than using an uncertified result. |
| Timeout or resource limit | Inspect proof complexity and budgets; this is not a counterexample or proof of a program bug. |

An explicit proof can supply a missing argument. An `assume` or unchecked
external contract introduces an assumption instead. Changing verification scope
or bypassing required checks is not a substitute for establishing the property.

## Run, inspect and maintain the proof

From this repository's root, with the prototype already built, run the complete
[adoption example](../../../../examples/verus_infer_adoption.rs):

```sh
source/target-verus/release/verus examples/verus_infer_adoption.rs \
  --infer-goals --no-cheating --output-json > /tmp/infer-adoption.json
```

For a Cargo project, follow [Cargo setup](cargo_verus.md) using the matching fork's
tools and dependencies. Keep both native build/tests and full verification in
your development workflow. Inferred and assisted interfaces require complete
verification before certification; partial/unchecked runs do not certify them.

Inspect the process exit status and the structured report. Check crate success,
the inferred domain, each function's required proof results, and interface export
when publishing one. A helper can have successful inference proofs while a later
application proof fails. A cached candidate is reverified; it is not a cached
proof. The [report reference](infer.md#structured-contract-reports) explains the
fields and reasons for unestablished obligations.

Keep independent policy and input-coverage goals alongside the code. Useful
regressions deliberately change an error order, drop an update, or remove a caller
bound and check that verification fails for the intended reason. Such mutation
checks test whether the proof exercises the property; they supplement the proof.

The [Aleph experiments](../../../tools/infer_alephbft/README.md) show this workflow
at a larger scale: authored collection views and invariants, inferred helper
behavior, generated proofs for supported iterator factories, and an ordinary
outer validation policy proof. Their pinned sources, supplied authentication and
codec interfaces, lock adaptation and quorum correction are documented explicitly.
They are selected source-connected experiments, not verification of the whole
crate or consensus protocol.

## Ownership of the specification

Authors control abstraction choices, application requirements and the input
domain clients must support. Library providers own reusable contracts and their
evidence. Tools should derive implementation safety conditions, construct routine
proofs, apply checked library facts and report gaps. Suggestions and inferred
supporting invariants can reduce author effort while leaving those decisions
visible and independently checkable.
