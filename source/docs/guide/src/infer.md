# Experimental contract inference

For a practical workflow, including authored views, application contracts,
explicit proofs and dependency boundaries, start with
[adopting inference in an existing codebase](infer_adoption.md).

Verus Infer generates and checks contracts for a small subset of executable Rust.
Opt a function in with `#[verus_infer]`, imported by `vstd::prelude::*`:

```rust
use vstd::prelude::*;

#[verus_infer]
fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}
```

No `requires` or `ensures` is needed on this function. Inference generates a
precondition equivalent to `amount <= balance` and a postcondition stating that
the returned value is the mathematical difference. Callers must prove that
precondition, and can use the postcondition in ordinary Verus proofs.

Inside `verus!`, use `#[verifier::infer]` on an `exec` function instead. The
equivalent attribute spelling for native Rust is `#[verus_verify(infer)]`.
Exact inference is the default. `#[verus_infer(precision = "exact")]` requests
it explicitly. Opt into sound summaries with
`#[verus_infer(precision = "sound")]`; inside `verus!`, the equivalent spellings
are `#[verifier::infer]` and `#[verifier::infer(sound)]`. Native attribute
rewriting also supports `#[verus_verify(infer(sound))]`.

## Scalar wrappers and inherent methods

Exact inference also supports transparent, nongeneric datatypes with one variant
and one executable scalar field, including tuple and named-field structs:

```rust
use vstd::prelude::*;

verus! {
pub struct NodeCount(pub usize);

impl NodeCount {
    #[verifier::infer]
    pub fn consensus_threshold(&self) -> NodeCount {
        NodeCount((self.0 / 3) * 2 + ((self.0 % 3) * 2) / 3 + 1)
    }
}

fn threshold_property(n: NodeCount) -> (q: NodeCount)
    ensures q.0 as int == (2 * n.0 as int) / 3 + 1,
{ n.consensus_threshold() }
}
```

The caller has no input restriction. The decomposed calculation avoids the
overflow in `n * 2 / 3 + 1`; inference does not repair that original expression.
If the multiplication is retained, its safety restriction is retained too.
Native `#[verus_infer]` methods can be used inside an impl annotated with
`#[verus_verify]`. Immutable wrapper locals, shared wrapper inputs and owned
wrapper results compose with other inferred functions and imported contracts.

The type definition must be available and transparent, with no type parameters,
destructor, user-defined type invariant, proxy or tracked/ghost field. Its payload
must be a supported Boolean or machine integer. Nested/multiple-field wrappers,
returned wrapper references and sound-mode value projection remain unsupported.
Mutable unit-returning methods can use the bounded sound record-state path below.
Concrete `From` and `AddAssign` implementations now have a bounded exact-mode
path described below. Other operator implementations remain unsupported. Verus
inherits implementation preconditions from the trait; the wrapper path checks
that inherited requirement rather than introducing a different callable domain.
No arithmetic meaning is assigned to an overloaded operator by its name.

Candidate generation projects eligible wrappers to scalars and lifts the formulas
back to the original field/constructor types. The original bodies and signatures
receive the independent soundness, maximality and exactness checks. Projected
functions never replace executable code. Type definitions participate in cache
provenance and exported-contract fingerprints; warm candidates still get fresh
proofs.

## Wrapper conversions and assignment operators

The native impl-level attribute handles simple public scalar tuple wrappers:

```rust
use vstd::prelude::*;

#[verus_verify]
#[derive(Copy, Clone)]
pub struct Count(pub usize);

#[verus_infer]
impl From<usize> for Count {
    fn from(value: usize) -> Self { Count(value) }
}

#[verus_infer]
impl core::ops::AddAssign for Count {
    fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
}
```

The attribute retains the executable method and generates ordinary
`FromSpecImpl` or `AddAssignSpecImpl` definitions using the existing vstd extension
traits. The original method receives all three exact-mode checks. No assumed
contract is generated. Spec impl declarations remain in the stub-erasure pass so
compiled Rust implementation identities agree with exported Verus interfaces.
Normal Rust builds erase the annotation and its ghost definitions.

This shortcut accepts a nongeneric single-method impl with a direct conversion
constructor/projection, or one `self.0 += rhs.0` / `self.0 += rhs` statement. It
generates public open models, so the wrapper and its field must be publicly
visible. Only default exact precision/budget is supported on the impl shortcut.
It does not annotate arbitrary derive-macro output or infer general conversion
algorithms. The separate method-level `#[verifier::infer]` / `#[verus_infer]` path
can analyze the existing bounded body subset with supplied ordinary extension
models; those models must satisfy Verus's normal visibility and verification rules.

Concrete `Count::from(value)` and the canonical core blanket `.into()` through
that checked `From` implementation compose into scalar and record summaries.
The Into bridge uses its existing vstd model and binds that dependency; custom
Into implementations are unsupported. Assignment operators compose into nested
record effects, preserving all unchanged fields. Implementations must have no
type parameters/bounds, involve a transparent single-scalar wrapper, and retain
the existing source-eligibility restrictions. Generic trait dispatch, arbitrary
external specifications, `TryFrom`, and other arithmetic/comparison operators
remain separate extensions.

For a trait implementation, ordinary soundness keeps the trait's inherited
precondition. Maximality additionally proves, on unrestricted typed entry states,
that this condition equals the candidate safe domain. This equivalence is asserted
before the original execution's safety conditions become assumptions. A condition
that permits overflow fails verification; an unnecessarily restrictive or false
condition fails the boundary/maximality check. A partial `From` implementation
cannot gain an implementation-specific precondition: From's inherited domain is
unrestricted. Concrete dependent inference waits for the required proof outcomes.

See [`examples/verus_infer_wrapper_traits.rs`](../../../../examples/verus_infer_wrapper_traits.rs)
for checked conversions in both directions, `.into()` round trips, nested counter
updates, ordinary caller goals and an erased native smoke test.

The Aleph `--wrapper-ops` experiment retains the original `NodeMap::size` body
`self.0.len().into()` and restores a `NodeCount` counter with the production
`+= NodeCount(1)` expression in the supplied collector. Its explicit verified
wrapper implementations replace native derives in the formal adapter. It still
uses the reduced Unit/Hasher interface, supplied view/invariant, corrected
threshold and supplied widened-round collector core. It does not verify the full
original collector, native derive implementations or cached FullUnit hashing.

The [AlephBFT parent-selection experiment](../../../tools/infer_alephbft/README.md)
compares a handwritten threshold contract with this inference path, supplies
native boundary/state-sequence probes, and records the remaining collector proof
requirements. Its formal collector is the existing manually verified production
core, not the generic collector from clean main.

## Bounded vector views and updates

Three small templates support ordinary `Vec` references, including generic
payloads. These native Rust bodies need no handwritten contracts:

```rust
use vstd::prelude::*;

#[verus_infer]
fn length<T>(v: &Vec<T>) -> usize { v.len() }

#[verus_infer]
fn read<T: Copy>(v: &Vec<T>, i: usize) -> T { v[i] }

#[verus_infer(precision = "sound")]
fn replace<T: Copy>(v: &mut Vec<T>, i: usize, value: T) { v[i] = value; }
```

`length` gets `result == v@.len()`. `read` gets the safety requirement
`i < v@.len()` and `result == v@[i as int]`. Both default to exact mode and
receive independent soundness, safe-domain maximality and output-exactness proofs.

`replace` gets `i < old(v)@.len()` and the state relation
`final(v)@ == old(v)@.update(i as int, value)`. This describes the complete final
sequence: its length and every other slot stay the same. Ordinary callers may
use that relation with their own invariants and checked sequence lemmas.

Standalone mutable vector templates require explicit sound mode. Their precision
checks remain unimplemented: reports mark maximality and exactness
`not_established`, with reason `state_precision_not_supported`, even with a
precision budget. The bounded mutable-record path below supports default exact
mode and independently checked state precision.
The original executable body must still pass ordinary soundness verification.
An informative state equation does not by itself establish these two precision
guarantees.

The mutator also accepts exactly `if i < v.len() { v[i] = value; }`, or
`if i >= v.len() { return; } v[i] = value;`. These guarded forms have no input
restriction; their postcondition describes an update in bounds and no change
otherwise. All templates use the default allocator. Generic read/write require
`T: Copy`; nongeneric scalar vectors are also supported. Additional effects,
aliases/locals, pushes, loops, arbitrary mutable records, custom allocators,
owned vector arguments and generic helper composition remain unsupported.
Handwritten contracts and proof statements cannot be added to an inferred helper.

This path copies and instantiates existing checked/trusted vstd vector and
sequence models; it adds no axioms or external bodies. Original function bodies
and signatures remain intact. Candidate identities and exported metadata bind
the state formulas and model dependencies. Collection templates regenerate and
are reverified on every run (`metrics.cache_policy = "regenerated_template"`),
including warm runs; their formulas do not enter the scalar candidate cache.
Ordinary callers can use certified collection summaries across crate boundaries.

The initial AlephBFT collection experiment extracts these three operations from the
existing manually verified collector. Its counting invariant, induction lemmas,
loop annotations and proof calls remain explicit in that comparison. The
counting-assistance checkpoint below removes the update proof block by reusing
generic library lemmas. Whole-collector inference remains unsupported.

## Supplied counting invariants

`Seq::count_prefix(predicate, n)` counts matching elements in a valid prefix.
The library provides four checked broadcast proofs: an upper bound of `n`, the
change caused by a slot update, zero when no slot matches, and the fact that a
full count means every slot matches. The last fact also proves there is room to
increment a counter when a known slot does not yet match.

With `--infer-goals`, ordinary function contracts and explicit assertions can
select this small theorem catalog automatically. For example, this supplied
accounting invariant needs no application-specific counting proof block:

```rust
use vstd::prelude::*;

verus! {
#[verifier::infer(sound)]
fn replace<T: Copy>(v: &mut Vec<T>, i: usize, value: T) { v[i] = value; }

pub open spec fn used(s: Seq<bool>) -> nat {
    s.count_prefix(|occupied: bool| occupied, s.len())
}

fn occupy(v: &mut Vec<bool>, total: &mut usize, i: usize)
    requires
        i < old(v)@.len(), !old(v)@[i as int],
        *old(total) == used(old(v)@),
    ensures *final(total) == used(final(v)@),
{
    let n = v.len();
    replace(v, i, true);
    *total += 1;
}
}
```

The length observation exposes the vector's `usize` length bound; no separate
counter-overflow assumption is needed. Real code often already reads the length
for its bounds guard, as Aleph does. The invariant and predicate remain supplied
by the developer. Incorrect updates, missing accounting assumptions and attempts
to count an already occupied slot still fail ordinary verification.

Selection follows at most 32 spec-function definitions and visits at most 4,096
source nodes per function, looking for the canonical count operation. It enables
only the four named library theorems in the function's existing body, using
Verus's scoped `broadcast use` mechanism. The SMT solver instantiates their
conditional conclusions as needed. There is no new axiom, assumed assertion,
rewritten goal or generated executable operation. Inferred definitions and vstd
itself are excluded from this decoration; their proof obligations are unchanged.
Missing models or exhausted discovery budgets leave ordinary verification in
charge. This is bounded theorem selection, not arbitrary induction or a general
ordinary-proof-first retry planner.

The JSON report's `goal_assistance` entries show the selected strategy and lemmas;
selection is not a proof receipt. Goal-assisted exports still wait for complete
ordinary verification, including when the invariant and mutator are imported.
No inferred precondition, contract precision claim or candidate cache format is
changed by this strategy.

The Aleph counting runner removes the `add` proof block, all three application
counting proof definitions and four calls across `add`/`new`/`next`.
Its count delegates to this library recurrence,
and a separate checked equivalence compares it with the original recursive count.
Loop annotations, state contracts and the transition's sequence-equality assertion remain.
The checked reference/equivalence definitions are experiment-only evidence.
Reproduce it with `source/tools/eval_verus_infer_alephbft_counting.py` as documented
in the [experiment README](../../../tools/infer_alephbft/README.md).

## Bounded mutable records

Default exact mode supports loop-free updates to one exclusively borrowed,
transparent record. Inference tracks each field through branches and early unit
returns, deriving guarded safety requirements and a final relation for every
field. For vector fields the relation describes the complete sequence; unchanged
fields and slots remain available to callers without handwritten frame clauses.

```rust
use vstd::prelude::*;
verus! {
pub struct Pool<T: Copy> {
    pub slots: Vec<Option<T>>,
    pub used: usize,
    pub epoch: u32,
}
impl<T: Copy> Pool<T> {
    pub closed spec fn valid(&self) -> bool {
        self.used == self.slots@.count_prefix(
            |slot: Option<T>| slot.is_some(), self.slots@.len())
    }

    #[verifier::infer]
    pub fn insert(&mut self, i: usize, value: T) {
        if i >= self.slots.len() { return; }
        let empty = match self.slots[i] { None => true, Some(_) => false };
        if empty { self.slots[i] = Some(value); self.used += 1; }
    }

    pub fn check_insert(&mut self, i: usize, value: T)
        requires old(self).valid(),
        ensures final(self).valid(), final(self).epoch == old(self).epoch,
    { self.insert(i, value); }
}
}
```

With `--infer-goals`, the ordinary caller proves both the counting invariant and
the unchanged epoch through the inferred contract. Its arguments are unrestricted:
every state satisfying `valid()` must establish the callee's safety requirements.
This checks the requested input domain as well as preservation. The invariant is
an explicit developer goal, not an assumption used by synthesis. Additional input
restrictions cannot silently replace that goal. No new expected-domain attribute
or mixed handwritten/inferred-contract syntax is introduced; this uses an
ordinary verification harness.

The subset permits up to eight fields per record, four field-projection levels,
sixteen scalar/vector leaves, two type parameters and eight function parameters.
Records must be available and transparent in the inferred function's module,
or have the certified direct sequence view described below,
with one variant, no custom destructor, proxy, tracked/ghost fields or user-defined
type invariant. Nested transparent records and scalar newtype fields are supported.
Payloads may be machine integers, Booleans, opaque generic values, tuples of up to
four elements, and bounded `Option`/tuple/value-record combinations (depth four).
Type parameters require `Sized`. Bounded trait and associated-type equality
constraints may be retained, and associated values remain opaque; this does not
interpret arbitrary trait operations. The getter profile below supports selected
generic calls. Vector fields use the default allocator. Normal Rust/Verus ownership,
borrowing and drop rules still apply: candidate generation does not insert copies
or invent a `Copy` bound for moved generic payloads.

Bodies support immutable locals, pure `Option`/tuple matches with at most four
arms, scalar assignments, checked add/subtract/multiply, comparisons, integer
casts, direct vector operations and closed primitive `From` widenings. Assigning
an entire scalar/value record updates all its leaves. Reads after writes use the
updated state, including through helper calls. Supplied predicates such as
`valid()` remain ordinary spec functions; inference does not assume them.

One mutable receiver with unit return may call acyclic, static inferred methods
on that receiver or a direct nested field. Their checked safety conditions and
complete final-state relations are instantiated at the call site, including
unchanged leaves. Generic type substitutions are simultaneous and bounded;
exact callers require exact-mode inferred callees. The verifier still checks the
original call and body. Compiler-generated borrow-resolution scaffolding is
preserved and inspected for dependencies; source proof code remains forbidden.

Read-only receiver methods can infer value results and bounded `Option<&T>`
lookups, including shared tuple patterns and `Option::as_ref` through its existing
vstd specification. These use return-value exactness and report
`summary_kind = "record_view"`. Returned references are characterized by their
logical values; lifetimes/ownership use the original Rust/Verus checks. This does
not establish pointer-address identity. Unit-valued queries and non-unit early
returns are currently unsupported. Mutable methods still require unit return.

Aliases, additional mutable/structured reference parameters, general mutable locals, whole-vector
replacement, general loops, division/remainder and general trait/operator calls remain
unsupported by this path. The bounded concrete From/AddAssign path above is an
exception. Arbitrary `Into` and derived operator implementations are not interpreted
as constructors or arithmetic.

`u32::from(round)` and the other 34 approved primitive widenings reuse existing
vstd conversion extension specifications. Recognition checks the canonical core
trait, concrete primitive implementation, proxy, argument/result types and
lossless pair. Ordinary exact scalar helpers may also use them and retain all
three independent proofs. User-defined conversions/operators do not receive
guessed semantics; supported wrapper implementations instead supply independently
checked body-derived summaries.

Record synthesis visits at most 4,096 source nodes and bounds generated formulas
at 4,096 nodes. It installs only contract data on the original function body.
Default exact mode requires three independent proofs before callers or exports
can use it:

- **Soundness:** the original body executes safely under the inferred precondition
  and satisfies its final-state relation.
- **Maximal input safety:** successful executions of the original body imply the
  inferred precondition, evaluated on the entry snapshot. The query does not
  assume that precondition; it uses the verifier-generated safety conditions.
- **Output exactness:** for any legal typed candidate final record, the inferred
  relation holds if and only if every modeled field matches the actual execution.
  Observations are enumerated from the datatype independently of the inferred
  clauses, so omitted changed or unchanged fields cannot escape this check.

Vector observations compare full sequence contents extensionally. This does not
claim exact allocation addresses or capacity. Copy fields use value equality.
The actual body and its borrow-expiration constraints remain unchanged in the
exactness query; only the proposed relation's final-state value is substituted.
No application invariant is assumed by these checks.

Explicit `precision = "sound"` remains available: soundness is required and both
state-precision probes are optional, with their actual outcomes reported. An
incomplete required precision proof blocks publication and dependent verification;
there is no automatic fallback to sound mode.
Record summaries regenerate and reprove on every run
(`summary_kind = "record_state"`, `metrics.cache_policy = "regenerated_state"`);
their state expressions never enter the scalar persistent cache. Identities bind
the original function, visible datatype definitions and model dependencies.
Imports validate a closed formula language, complete leaf coverage for updates,
and a typed result equation for read-only queries.

The Aleph record experiment infers the supplied collector's direct `add` body,
removing its handwritten production contract and all three vector wrappers from
the preceding experiment. A minimal harness checks only `valid()` preservation;
an experiment-only regression oracle separately retains the original behavioral
clauses. The counting equivalence, other state contracts, loop annotations and
transition equality assertion remain supplied. This is the existing manual
collector core, not clean main's `NodeMap`/`Unit`/`Hasher` implementation.

### Transparent wrapper composition

The [wrapper example](../../../../examples/verus_infer_wrappers.rs) defines a
`SlotTable<T>` with inferred length, borrowed lookup and insertion methods, then
uses those methods from a containing registry:

```rust
#[verus_infer]
pub fn insert_if_empty(&mut self, i: SlotIndex, value: T) {
    if i.0 >= self.entries.len() { return; }
    let empty = match self.entries.get(i) { None => true, Some(_) => false };
    if empty { self.entries.insert(i, value); self.occupied += 1; }
}
```

An ordinary caller supplies the counting invariant and proves its preservation
with `--infer-goals`. No application-specific container name is recognized by the
compiler. Transparent interfaces also compose across crates, with regenerated
summaries and fresh proofs. Private storage can instead use the supplied direct
sequence views below. Automatic discovery of arbitrary views is not implemented.

The Aleph wrapper experiment checks the pinned `NodeMap::get` and `insert` bodies
unchanged and an explicitly constructor-adapted `size()` method. The supplied
collector core uses these helpers under its existing invariant. API extraction,
module placement and the `Into` adaptation are recorded separately from proof
annotation removal; this is not a proof of the clean-main `Unit`/`Hasher` collector.

### Private vector wrappers with supplied views

The [private-view example](../../../../examples/verus_infer_private_views.rs)
retains collection privacy. Supply an ordinary Verus view inside the defining
module, then annotate the supported executable methods as usual:

```rust
pub struct Slots<T>(Vec<Option<T>>);
impl<T> View for Slots<T> {
    type V = Seq<Option<T>>;
    closed spec fn view(&self) -> Self::V { self.0@ }
}
```

Inference recognizes only the canonical `vstd::view::View` implementation whose
body directly projects the type's **sole executable vector field** through the
existing Vec view. Named and tuple fields, ordinary scalar elements and the
existing bounded generic payloads are supported. The wrapper retains the record
restrictions on destructors, invariants, fields and generic bounds. A projection
with filtering, slicing, transformation, extra statements or additional fields
does not qualify. The compiler does not invent a view or assume its completeness.

The inferred contracts use `self@`, preserving private fields and the closed view
definition across modules and compiled crate interfaces. Shared length/lookup
queries and unit-returning slot updates compose with enclosing inferred records,
including unchanged sibling collections. No source program operation is inserted.
Default exact mode still requires soundness, maximal input safety and exact
modeled final state. A structurally certified direct view re-expresses the same
whole-vector-content observation used by the independent precision checker;
it cannot discard other modeled fields. Capacity and addresses remain outside
that observation, as with existing vector inference.

A closed view also hides facts such as the physical vector length bound. The
example's ordinary caller uses `let _ = r.entries.len();` before the update to
obtain that bound from the checked query contract. It adds no input restriction.
With `clone_via_view`, a companion length theorem is now generated and proved from
the supplied view. `--infer-goals` can select it when caller goals or called
preconditions observe that private view, removing this manual observation. A false
view bound fails the ordinary proof; no bound is assumed. Other representation
lemmas remain future work. Integer length
equations remain mathematical through scalar wrapper returns, so a checked
`NodeCount`-valued size query can serve the same purpose.

Compiler-only direct-view evidence is fingerprinted with inferred interfaces and
the datatype, validated on import and published only after complete verification.
Closed bodies remain unavailable to clients. View/state templates regenerate and
receive fresh proofs; this introduces no new trusted library model or axiom.

The private Aleph checkpoint builds an extracted `NodeMap` API in a separate
crate and checks the supplied collector's update through it. It retains original
get/insert bodies, the documented size-constructor adaptation, supplied invariant,
checked counting equivalence and separate behavior oracle. Size observations are
explicit in the ghost-only callers. Construction, transition and selection methods
are excluded from this checkpoint; the earlier experiments cover their manually
specified supplied-core versions. This does not establish clean-main `Unit`/`Hasher`
trait inference or automatic abstract-view discovery.

### Verified trait query contracts and associated types

The [trait example](../../../../examples/verus_infer_traits.rs) uses supplied
ordinary Verus contracts to describe getters, then infers a generic record update:

```rust
pub trait Event {
    type Payload: Copy;
    spec fn slot_value(&self) -> usize;
    spec fn payload_value(&self) -> Self::Payload;
    fn slot(&self) -> (result: usize)
        ensures result == self.slot_value();
    fn payload(&self) -> (result: Self::Payload)
        ensures result == self.payload_value();
}
```

The supported profile is a public trait method with one shared receiver, up to two
boolean or machine-integer value arguments, no precondition and one exact
postcondition equating the result to a same-trait specification observation.
The observation receives the receiver and every argument unchanged, in the same
order and with matching types. For example, `count_in(round: u16)` can ensure
`result == self.count_in_value(round)`. Observation and return
types must match. The observation has no recommendations, prophetic dependency or
explicit borrowed/unit result. External specification proxies, method/trait type
arguments beyond Self, trait objects, weaker relations and precondition-bearing
contracts are outside this profile. A default executable getter body may be present;
ordinary Verus verifies that body and every local implementation against the
supplied contract. Inference does not infer the contract or inline the body.

Supported record methods can accept shared generic inputs and call these getters.
Opaque associated values compose through tuples, vector slots and private direct
sequence views. Explicit associated-type equalities are normalized with at most
eight rewrite steps; projection syntax has a separate depth-four budget so
`Option<&(H::Hash, u16)>` retains the existing payload-depth limit. Record signatures
allow at most 24 bounded trait/equality constraints and four trait type arguments
per constraint; the existing two type-parameter limit remains. No semantics for
hashing, conversions, operators or arbitrary traits is guessed from their names.

Default exact mode retains all three original-body checks. A getter with no
precondition introduces no additional safety-domain restriction, and its singleton
result equation supplies the proposed value. The verifier still checks the original
call and its enclosing state update. Application invariants and desired behavior
remain explicit goals.

Across crates, the getter provider must publish a certified interface. Providers
with local inferred functions do this automatically. A provider containing only
ordinary getter contracts can use `--infer-goals`; that enables recognition and the
full-verification publication barrier without requiring an inferred executable.
No-verify and function/module filters are rejected for these interfaces. Ordinary
legacy trait interfaces cannot silently stand in for certified getter contracts.

The certificate binds the entire exported API, including ordinary specification
implementations, trait definitions, associated-type implementations and architecture.
Provider model artifacts are conservatively fingerprinted even without local
inferred functions. Normal Verus library/external-specification assumptions remain.
There is no new axiom or external body, and these contracts are not classified as
trusted standard-library models.

The Aleph trait checkpoint retains pinned creator/round default bodies and coordinate
getter bodies in an explicit **reduced Unit/Hasher adapter**. Supplied observations,
getter contracts and a closed coordinate-parts definition support ordinary provider
verification. A supplied `Collector<H>` core stores `H::Hash`; inferred `add_unit`
reads the three getters and calls the checked direct update. A minimal invariant
goal and a separate unit-behavior oracle check the result. This does not verify the
full original traits, cached FullUnit hashing, unrelated native bounds or the
clean-main collector's operator/lifecycle code.

## Pointwise mutable prefix loops

Default exact mode also supports one zero-based `usize` traversal of a vector
field in a borrowed record:

```rust
#[verifier::infer]
fn bump(&mut self, n: usize) {
    let mut i = 0usize;
    while i < n {
        self.values[i] += 1;
        i += 1;
    }
}
```

For `Vec<u8>`, inference derives `n <= old(self).values@.len()` and that every
visited element is below 255. It describes the complete final sequence: each
index below `n` increases by one, the suffix is unchanged, length is unchanged,
and every other record field is framed. A zero count permits every initial
value. Signed element arithmetic retains its ordinary checked bounds too.
Loop-independent partial arithmetic is required only when the prefix is nonempty.
Direct slot compound assignments outside loops also retain sequential aliasing:
in `balances[src] -= amount; balances[dst] += amount`, the second operation reads
the already updated sequence, including when `src == dst`.

The loop body must contain one pointwise update followed by `i += 1` or
`i = i + 1`. The update may use an exact inferred record method, including a
private sequence wrapper's insertion API. Reads of the changed collection must
use the same index. Other record fields remain unchanged. Bounds may be immutable
parameters, wrapper fields or the collection length. Native Rust uses the usual
`#[verus_infer]` spelling. Source contracts and loop annotations are unnecessary.

The candidate adds a prefix-content invariant, visited-prefix safety facts,
bounds and frame facts, and the decreasing measure `bound - i`. Function-entry
observations use explicit entry snapshots, so loop mutation cannot change their
meaning. Ordinary Verus proves initialization, preservation, source safety and
termination. The maximality query removes the candidate precondition and still
checks the induction; visited safety facts must be established by completed safe
iterations. Output exactness observes the complete final modeled record state.
The shared projection gate removes only proof annotations and compares the
remaining executable with the original typed body. No loop axioms are added.

General iterator mutation, neighboring-element dependencies, growing/shrinking
collections, multiple/nested loops, non-unit counter steps, early exits, mutable
returns and sound-mode traversal summaries remain unsupported. Such algorithms
can still use ordinary Verus contracts and loop invariants.

A public checked `Result` API can validate inputs before calling an inferred
unit-returning worker. Its error choices and preservation policy remain authored
and receive ordinary verification; mutable `Result` inference is not introduced
here. For a transfer, check indices, available source funds, and destination
capacity **before** subtracting funds if errors must preserve the state. Returning
`Err` after the subtraction does not establish that property. See
[`verus_infer_traversals.rs`](../../../../examples/verus_infer_traversals.rs) for a
checked prefix-fill wrapper and signed increment example.

## Checked growth and transition loops

`--infer-goals` can propose loop invariants and a decreasing measure for an
**ordinary authored growth contract**. This complements `#[verus_infer]` on
supported bottom-up helpers. It does not infer application policies, choose a
state-machine invariant, or certify the whole application contract as maximal
and output-exact.

The bounded profile accepts a unit-returning function with a mutable receiver,
one unannotated `while` loop and one recognized vector `push` in its body. The
contract must supply one final sequence-length equation with an entry-state or
immutable target, for example `final_len == max(old_len, target)`. Supplied final
state predicates, unchanged-field equations, prefix relations and quantified
properties of appended elements are proposed at intermediate states too. Those
clauses must be inductive; unsupported algorithms still need explicit proofs.

For current length `len` and supplied final length `target`, the proposals include
`old_len <= len <= target` and measure `target - len`. The length equation is
weakened to its upper bound inside the loop, while the original exact equation
remains a required postcondition. Function-entry references use explicit entry
snapshots. Loop isolation is disabled so the ordinary query retains the supplied
input contract. No precondition or output goal is weakened, and every invariant
must initialize and survive every iteration. Ordinary Verus also checks source
bounds, arithmetic, helper preconditions and termination.

Elements can be records containing their own collections. Their application
meaning and transition laws come from authored views and ordinary checked
contracts, rather than from guessing nested representations or assuming helper
semantics. Existing elements retain their full values through the supplied prefix
relation; an abstract view is used only where the author explicitly chooses it.
The [growth example](../../../../examples/verus_infer_growth.rs) grows a vector
of snapshots by calling a checked transition that copies each snapshot's vector.
It contains no loop annotations or local proof blocks.

Handwritten loop annotations are preserved. Multiple/nested loops, native `for`
loops, termination bypasses, non-unit return functions and loops without the
required length goal receive no growth proposal. Other algorithms can keep using
ordinary Verus. Finite traversal and formula budgets bound candidate generation;
exhaustion does not establish a proof. Failed proposals fail verification.
Reports list `growth_loop_assistance` separately from exact inference candidates.
Assisted providers require full-crate checks, and failed assisted exports are
withheld. Imported ordinary contracts retain Verus's normal trusted-library
boundary.

Aleph's native Creator uses this path for its collector-growth worker. The author
supplies the Creator invariant, final length, prefix preservation and round bound.
An ordinary mutable-return wrapper checks the selected collector and its unchanged
other slots; preservation of the Creator invariant after the borrow is conditional
on the borrower returning a valid, correctly indexed and sized collector. A
checked `try_initialize_round` API accepts every `u16` round and rejects `u16::MAX`
without mutation, because that collector would need to store `round + 1` in `u16`.
Existing `add_unit`/`create_unit` orchestration remains outside this native proof.


## Owned collection construction and round advancement

Ordinary public methods may also expose a total value query with a single
`ensures result == self.observation(context)` equation. The bounded static profile
accepts an owned or shared datatype receiver, up to four type parameters, and up
to two scalar or shared datatype/type-parameter context arguments. The matching
pure observation has the same parameters and types. Structured results are
eligible only when their complete values/collection observations fit the owned
inference profile. Preconditions, mutable arguments, prophetic observations and
external-body implementations are not accepted by this recognition rule.
Ordinary verification checks the method; cross-crate use requires its certified
export, including the observation's meaning and dependency fingerprints.

Canonical Result `?` in this profile can use a bounded generic `From` method
marked `#[verifier::infer]`. Rust must select that concrete implementation;
inference neither guesses a converter nor assigns meaning from the trait name.
The converter proves all three exact-mode obligations and equivalence to its
inherited trait domain. Its ordinary vstd `FromSpecImpl` obligations still apply.
For a collection-bearing conversion whose optional whole-object FromSpec law is
unused, `obeys_from_spec()` can be false; the actual inferred contract and every
verification obligation remain checked. General conversion/Into inference and
arbitrary user Try implementations are separate work.

For example, the Aleph outer validator is checked directly against an authored
application policy over inferred helpers. Its error conversion is inferred; its
authentication decision has a supplied contract. Cloned FullUnit error payloads
are compared through the authored view, which omits the hash cache. This ordinary
caller proof does not claim exact whole-object output inference or maximality
for the outer method. Neither the authentication contract nor a supplied codec
model proves the underlying cryptographic or serialization implementation.

Default exact mode supports bounded functions that return a `Vec`, a transparent
record containing vectors, a private sole-vector wrapper with a certified direct
view, or `Option` of such a value. For example, these methods need no handwritten
contracts or loop annotations:

```rust
#[verus_verify]
pub struct Snapshot<T: Copy> { slots: Vec<Option<T>>, epoch: u16 }

#[verus_verify]
impl<T: Copy> Snapshot<T> {
    #[verus_infer]
    pub fn new(size: usize) -> Self {
        Self { slots: vec![None; size], epoch: 0 }
    }

    #[verus_infer]
    pub fn next(&self) -> Option<Self> {
        if self.epoch == u16::MAX { return None; }
        let mut slots = Vec::with_capacity(self.slots.len());
        let mut index = 0usize;
        while index < self.slots.len() {
            slots.push(self.slots[index]);
            index += 1;
        }
        Some(Self { slots, epoch: self.epoch + 1 })
    }
}
```

`new` accepts every `usize` size in the verifier's existing allocation model and
returns exactly that many `None` slots with epoch zero. `next` has no overflow
restriction: it returns `None` exactly at the maximum epoch, otherwise preserves
all slot contents and increments the epoch. Ordinary caller goals can prove
these facts using the inferred interfaces. The
[complete example](../../../../examples/verus_infer_owned.rs) also verifies and
runs natively, including empty construction and the terminal epoch.

The source subset includes `Vec::new`, `Vec::with_capacity`, `vec![value; size]`,
primitive/structural clones whose equality the existing vstd model can prove,
immutable locals, field projections, branches, early value returns, checked
scalar arithmetic and exact inferred helpers. Repeat initialization accepts
primitive identity-clone values or `None`. A generic `T: Copy` bound does **not**
justify equating arbitrary `Clone` results: a custom Clone can differ from a
Copy. An explicit indexed copy loop remains useful for opaque Copy payloads.
Alternatively, a checked `CloneIdentity` bound supplies the needed law, as below.
A clone candidate is withheld unless ordinary Verus proves its contents equation.

One loop is supported per function, in either of these forms:

- Fill an initially empty local vector with the same total, loop-independent
  value: `while out.len() < size { out.push(value); }`.
- Copy a vector prefix into an initially empty local vector using a zero-based
  `usize` counter, `out.push(source[index])`, and `index += 1`.

Inference proposes a length bound, a complete contents/prefix invariant, and a
strictly decreasing length difference. Ordinary Verus checks their initialization,
preservation and termination on the original loop. The source-projection gate
permits only these proof annotations and the loop-isolation setting to change.
Handwritten loop annotations, extra effects, other indices, multiple/nested loops
and termination bypasses are rejected. Partial expressions inside a fill loop,
such as `push(x + 1)`, are currently rejected because the independent maximality
transform cannot recover their first-iteration necessity. Computing `let value =
x + 1;` before the loop gives an ordinary checked input-domain restriction.

Results have at most 16 independently observed leaves, type depth five, eight
fields per transparent record, eight parameters, two type parameters and 24
bounded constraints. Private wrappers retain the supplied direct-view requirement.
Shared inputs are supported; mutable-reference parameters and sound mode are
outside this owned-return profile. Generic helper composition is limited to the
supported certified summary forms; it does not infer arbitrary trait semantics.

All three proof obligations remain mandatory. Output exactness independently
traverses the output type and compares every scalar field, complete vector/view
contents, and Option tag and present payload. Exactness is with respect to these
logical observations, not vector capacity or allocation identity. No application
invariant or desired round policy is assumed by synthesis. A wrong initial epoch
can receive a perfectly exact contract and still fail the developer's separate
initial-state goal. Adding unnecessary executable overflow can similarly produce
an exact but unwanted input restriction, caught by an unrestricted caller goal.

Reports classify these summaries as `owned_value`, list loop proof annotations,
and report fresh soundness, maximality and exactness evidence. Templates regenerate
on every run. Imported interfaces validate complete observation coverage, guards,
a closed pure model language and the existing candidate/dependency/model evidence.

The [Aleph lifecycle runner](../../../tools/eval_verus_infer_alephbft_lifecycle.py)
removes supplied constructor/advance contracts, loop annotations and local proof
blocks from the supplied Vec core, preserving executable bodies. A separate crate
variant retains original NodeMap API bodies including `with_size`, and supplies
native derived Clone through a checked public view relation. Ordinary goals prove construction → add → next →
add invariant preservation and the original new/next behavior. The application
invariant, count-equivalence proof, reduced Unit/Hasher adapter, threshold correction
and widened-round/Option terminal policy remain supplied. This does not verify
arbitrary generic Clone equality or the full clean-main collector. The separate
Unit implementation checkpoint below covers cached hashing with explicit library
boundaries and a verified-lock adaptation.

## Native derived Clone through a supplied view

For a sole-Vec wrapper with a supplied direct sequence View, place
`#[verus_verify(clone_via_view)]` **before** `#[derive(Clone)]`. Rust's native derive
is retained. The adapter generates a public spec relation saying that lengths are
equal and corresponding elements satisfy vstd's reflexive `cloned` relation.
Ordinary Verus verifies the actual derive body against that relation; the derive
itself is not an inferred function with three precision claims.

An inferred `.clone()` caller can propose complete logical contents equality.
That candidate still needs soundness, safe-domain maximality and output exactness.
For generic payloads, use `vstd::std_specs::clone::CloneIdentity` and enable its
checked broadcast lemma `lemma_cloned_identity` where the caller is proved. The
trait proves `cloned(a,b) ==> a == b`; vstd verifies implementations for primitive
integers/bool/char, Option, arrays and tuples of arity one through four. Copy alone
does not establish this law. Custom Clone implementations can change their value.
The tuple Clone model deliberately permits cloning or copying each component.

The [native Clone example](../../../../examples/verus_infer_clone.rs) demonstrates
the declarations and a generic exact caller. Current source limits are one unnamed
Vec field and at most two type parameters, with no lifetime or const parameters.
The supplied View must be the checked direct projection. Complete logical contents
are observed; capacity, allocation identity and arbitrary representation laws are
outside the claim. Imported providers, including providers with no inferred
functions of their own, must pass full-crate verification before export.

## Source-connected Unit implementations

The [Aleph Unit runner](../../../tools/eval_verus_infer_alephbft_units.py) checks
the native Hasher64 hash body against existing DefaultHasher models and a
target-endian u64 byte model. It also checks native UnitCoord/PreUnit/ControlHash
methods and Clone derives, FullUnit construction/clone, all four Unit methods,
and the original creator/round defaults. Ordinary goals establish repeated-hash
equality and clone/hash consistency, and instantiate the supplied collector with
`FullUnit<Hasher64,u32>`.

Two boundaries remain explicit. A separate trusted codec model says that SCALE
encoding depends deterministically on the immutable fields and skips the cache;
the codec algorithm and derive are not proved. Parking_lot read/write/try_read
expressions are adapted to vstd's verified RwLock copied read/replace operations,
with a fixed per-lock predicate and ghost/type-invariant annotations. Clone now
uses `try_read_or(None)`: one verified finite attempt, followed by a compatible
fallback on failure. This is a functional proof of the adapted implementation;
parking_lot's protocol and RAII/Drop implementation remain outside the proof.
Native runtime tests separately exercise the original lock and codec paths.

The checked `CloneView` proof trait in `vstd::std_specs::clone` supports native
Clone preservation of DeepView without requiring Copy or object equality. Primitive,
Option, Vec and String laws verify, and `lemma_cloned_view` exposes them to callers.
FullUnit accepts such data and has a concrete `Vec<u8>` proof/runtime case. The
trusted codec model must still retain every encoded distinction in that deep view;
CloneView alone does not establish encoding completeness.

## Native finite iterator queries

Inference can compose a supplied finite native iterator factory with `count`,
`filter(...).count()`, and temporary-iterator `any(...)`. A library author supplies
an ordinary closed sequence observation and proves the factory's
`vstd::std_specs::iter_adapters::finite_native_view(result, observation)` contract.
The observation records the intended full sequence; the native stream may be a
prefix until successful exhaustion. Its count relation is separate: Rust's
`Map::count` can skip the mapper, so counting does not generally prove that the
mapper ran or that its `next` stream completed.

For a collection with such a checked `iter` factory, these consumers need only
inference annotations:

```rust
#[verus_infer]
fn occupied(slots: &Slots<u16>) -> usize { slots.iter().count() }

#[verus_infer]
fn matching(slots: &Slots<u16>, round: u16) -> usize {
    slots.iter().filter(|&(_, value)| *value == round - 1).count()
}

#[verus_infer]
fn future(slots: &Slots<u16>, round: u16) -> bool {
    slots.iter().any(|(_, value)| *value > round - 1)
}
```

The inferred output equations use the supplied sequence's length and filtered
sequence. Callback contracts are synthesized for bounded pure scalar/field
expressions, including irrefutable tuple/reference patterns and captured scalar
inputs. The original callbacks remain executable. Ordinary closure verification
checks soundness; an independent query checks each callback's maximal safe domain
and both Boolean outputs against its original lowered body, without assuming its
candidate requirements or the enclosing function's requirements. Generated closure
requirements are equated with this checked domain; handwritten closures retain
Verus's existing one-way rule.

A uniform partial predicate such as `round - 1` requires a positive round only
when the supplied sequence is nonempty. Fully consumed filters also support
item-dependent scalar safety. Short-circuit `any` currently rejects item-dependent
partial safety and preceding partial filters: requiring safety for unvisited items
would be too restrictive for exact mode. Callback mutation, executable helper
calls, stored mutable iterators, arbitrary iterator methods, factory synthesis and
general iterator-loop inference remain outside this profile.

These native APIs **extend the trusted Rust-library model** with count,
enumerate and filter_map specifications. They do not verify precompiled Rust core.
Count support is restricted to native slice iterators and their supported native
adapter families; no blanket count law is assumed for custom Iterator overrides.
The separate checked `vstd::iter` implementations below still add no iterator
axioms. See the [native factory example](../../../tools/infer_alephbft/native_pipeline_models.rs)
and its [source-connected Aleph experiment](../../../tools/infer_alephbft/README.md).

## Checked native iterator factories

For supported native factories, an authored output observation can now request a
generated ordinary factory proof:

```rust,ignore
impl<T> NodeMap<T> {
    #[verus_verify(iterator_via_view(self.entries_value()))]
    pub fn iter(&self) -> impl Iterator<Item = (NodeIndex, &T)> {
        self.0.iter().enumerate().filter_map(|(idx, value)| {
            Some((NodeIndex(idx), value.as_ref()?))
        })
    }
}
```

Place this attribute on an ordinary Rust method outside `verus!`, and enable
`--infer-goals`. The output observation remains authored: the sparse profile uses
the supplied `sparse_entries` vocabulary. The attribute generates a
`finite_native_view` contract and normal proof calls. Callback equations are
proposed from the original closure and checked against its executable body;
their exits and safety requirements are scoped to that closure. No handwritten
callback contract, type annotation or factory proof block is needed in this
example. A native `map` over a certified finite factory is supported similarly.

The initial profile requires a shared receiver, a single returned adapter call,
a direct immutable callback and a total callback candidate. The sparse family
starts with native slice iteration and enumeration. More general chains, partial
callbacks and custom iterator implementations remain separate work. The macro
keeps the original receiver/closure evaluation order for verification and returns
the original item unchanged during executable erasure. Generated generic proof
calls undergo normal Rust and Verus checking. All factory proofs must succeed
before publishing an interface; `--no-verify` cannot certify them.

These are ordinary verified factory interfaces, not exact inference of the whole
iterator object or its maximal construction domain. Consuming inferred methods
still prove soundness, maximality and output exactness. The checked combinators
add no library axioms and retain the existing native iterator model boundary.
The `factory_assistance` JSON entries report regenerated callback proposals;
they are not independent precision receipts.

The Aleph validation runner's `--factory-proofs` mode exercises both original
factory bodies. Supplied collection/output views and application policies remain
explicit, while their factory proof blocks and callback contracts are removed.

## Total native construction loops

A bounded owned constructor can now use one native `for` loop over a supplied
finite factory. The loop updates one local collection through an already inferred
mutable method whose contract describes the complete sequence view. For example,
a `Slots<T>` collection with inferred `with_size`, `size` and `insert` methods and
a checked native `iter` factory can use:

```rust
#[verus_infer]
fn copied<T: Copy>(source: &Slots<T>) -> Slots<T> {
    let mut out = Slots::with_size(source.size());
    for (index, value) in source.iter() {
        out.insert(index, *value);
    }
    out
}
```

Inference composes the checked mutation equation and proposes a sequence fold over
the supplied full iterator view. Generated invariants relate the accumulator to
the visited prefix, preserve its length, and establish readiness for each update.
The original native loop and update calls remain unchanged. Existing iterator
scaffolding, invariant checks and decreasing measures are retained.

This first loop profile requires an **unrestricted input domain**. It always
proposes `requires true`; Verus must prove the original body safe and terminating
for every input of its Rust types. A partial callback or unsafe index shift causes
verification to fail rather than producing a restrictive domain. Maximality is
trivial for `true`, and its independent query retains the original safety checks.
Output exactness still compares arbitrary complete output views with the actual
return value. Partial-domain folds, early exits, nested loops, multiple mutable
accumulators and arbitrary iterator loops remain unsupported.

Complete inferred sequence-view mutations also compose into owned constructors
without loops, with their ordinary exact safety requirements. This profile
currently requires a local receiver, unit return, value arguments and one full
final-view equation on a direct single-vector wrapper. Generic precondition-free
ordinary exact queries may return a type parameter or associated value type;
these retain supplied meanings and require full provider verification.

Aleph's original `ControlHash::new` now infers its fold and combined-hash equations.
Its original `create_control_hash` body is an ordinary checked generic query through
the existing declared codec model. A supplied caller connects the newly constructed
value to the original noninitial validator and its exact typed error policy. The
fold is a precise implementation equation. Indexed updates independent of the
accumulator are represented by the checked `vstd::seq_update::scatter` vocabulary.
Other steps retain a general fold. With `--infer-goals`, ordinary callers requesting
length, element, or equivalent-fold facts receive scoped checked library lemmas.
For supplied factory views expressed using `sparse_entries`, the checked sparse
projection theorem derives the resulting per-slot values, including absent slots.
No function contract gains eagerly synthesized application facts. General invariant
or abstraction discovery remains separate from this bounded lemma catalog.

Ordinary exact unary queries can also observe a newly constructed collection when
their visible specification body uses that receiver only through its certified
sequence view. The query implementation remains checked against its supplied exact
contract. Pure nonprophetic specification terms are vocabulary for proposed
equations; they do not become executable inference models. Hidden observation
bodies and queries inspecting other representation details are rejected.

Generic same-type `PartialEq::eq`/`ne` calls can propose value-identity comparisons.
The original call and all three checks must prove that proposal. The explicit
`vstd::std_specs::cmp::PartialEqIdentity` law, enabled with `group_eq_identity`, is
verified for booleans, machine integers and arrays of lawful elements. Rust's `Eq`
marker alone supplies no such proof; unsupported or weaker equality fails closed.

Value-only `Result` summaries retain a complete typed conditional value through
branches and helper calls. Identity `?` propagation can preserve that value on the
error path when the complete return type matches. This avoids repeated tag/payload
reconstruction. Collection-bearing results retain guarded complete view observations.
All original safety and precision checks still run, and the 4,096-node formula
budget remains unchanged. This is a bounded representation improvement, not general
expression sharing or support for arbitrary-depth call chains.

Aleph's full-validation mode infers both original validation branches and the
unchanged public dispatcher. Separate ordinary caller goals check the complete
supplied error policy and conditional creator-index bound. Constructor callers
state input-map bounds and per-slot projection goals without fold induction.
Collection views, factory proofs, equality law, application policy and the codec
boundary remain explicit supplied inputs.

The existing standard `Result::map_err` contract also composes a direct immutable
closure with bounded scalar, record or collection-bearing enum output:

```rust,ignore
#[verus_infer]
fn attach_context(input: Result<u8, u16>, context: &Snapshot) -> Result<u8, Error> {
    input.map_err(|code| Error::WithContext(context.clone(), code))
}
```

`Snapshot` must have a checked Clone relation for the proposed complete field
observations; `Copy` or `Clone` alone supplies no identity law. Every variant tag
and guarded payload observation participates in output exactness. The closure's
original body separately proves safety, maximality and output precision. A partial
callback restricts the outer domain only on its error path. Captured mutation,
arbitrary callbacks, conversion traits and unbounded recursive enums remain outside
this profile. No new standard-library assumption is added for `map_err`.

Direct shared getters returning an object can infer complete logical value
identity. For `fn inner(&self) -> &Inner { &self.inner }`, inference creates an
ordinary closed spec projection and proves the executable getter's equation.
Generated definitions undergo the normal well-formedness and mode checks; their
bodies stay private to the owner. This supports accessor chains through opaque
objects, including records with type invariants, without exporting private field
selectors. Preserving an opaque object does not infer its representation, invariant
or Clone behavior. Pure contract substitution is read-only and cannot add an
execution-path condition.

Aleph's `--parent-validation` experiment uses these features on the unchanged
`Validator::validate_unit_parents` body. Its supplied policy checks member-count
priority, complete PreUnit views in errors, exact nested ControlHash errors, and
complete Signed value identity on success. Signed authentication and Keychain's
count equation remain explicit interfaces; the outer signature/session/round
validator is separate work.

## Checked finite collection queries

`vstd::iter` provides ordinary verified implementations of `SparseIter` for
`&[Option<T>]`, `count`, `count_where`, `find`, `collect`, and `map_sparse`.
Sparse traversal returns occupied entries with their original indices; mapping
preserves empty positions. Count/search predicates have checked executable callback
contracts connecting them to supplied mathematical predicates. Generic iteration
requires the existing iterator laws and a nonprophetic termination metric. Counting
and collection also require a machine-length bound. The implementations call
`next` directly and add no axiom about overridable `count` or `try_fold` methods.

These library bodies have explicit contracts and loop invariants. Inference can
compose a checked collection query through the parameterized trait-query interface
above. Read-only record methods returning bounded `Result` values retain complete
tag/payload exactness, including scalar error policies. This does not infer arbitrary
iterator loops or synthesize callback meanings.

The [bounded Aleph query model](../../../tools/infer_alephbft/parent_query_models.rs)
uses these helpers for parent round projection, counting in a round range, and
finding future-round parents. A separate inferred validator composes quorum and
future-round checks. That older comparison adapts the native iterator chains. The newer native-pipeline
validation mode above removes both count/search implementation adapters and checks
the original typed noninitial validator. The experiment retains supplied meanings and checks both
formal negative controls and native/adapted runtime corpora.

## Borrowed collection queries inside Result

The structured-value path also supports `Result<&Vec<T>, E>` and
`Result<&Wrapper<T>, E>` with a certified private direct sequence view. Error
values may be ordinary supported scalar/value leaves or a local, nongeneric enum
with up to four nullary variants. A query can use early value returns and up to
four guarded Option/tuple match arms. Immutable shared payloads support the
existing Option::as_ref and primitive From widening models. General error payloads,
mutable references, arbitrary aliases and returned slice unsizing remain outside
this profile.

The [selection example](../../../../examples/verus_infer_selection.rs) infers a
quorum/own-parent query and proves separate success, contents and error goals.
No application invariant or intended policy is assumed. Returned references are
compared by complete logical contents, not allocation or address identity;
ordinary Rust/Verus lifetime checking still checks the executable. Reports retain
the `owned_value` structured-summary classification for these borrowed results.

Exactness independently observes the Result tag and both guarded payloads,
including the precise error variant. Imported summaries must reconstruct the same
complete shape. Safe-domain maximality uses the original body: removing a bounds
check can produce a necessary input restriction that an unrestricted caller goal
rejects. Canonical primitive widening recognizes the shared-reference decorations
retained by VIR after Rust has checked the original dereference.

The Aleph selection runner removes the supplied method contract and checks
readiness, returned contents, and both error policies in ordinary wrappers. It
preserves the supplied core's branch order and explicit out-of-range error policy.
It returns a Vec borrow rather than the supplied slice borrow. The private variant
uses NodeMap's checked APIs; its ghost-only caller observes size and conditionally
get to expose their checked representation/type facts. The invariant/counting
model, Copy helper, reduced Unit/Hasher adapter and round policy remain supplied.
This checkpoint does not yet verify the unchanged native collector or cached hash.

## Demand-driven scalar proof assistance

The experimental `--infer-goals` command-line flag asks an additional bounded
proof strategy to handle eligible ordinary caller assertions. For example:

```rust
use vstd::prelude::*;

#[verus_infer]
fn bucket(key: u64) -> u8 { (key & 15) as u8 }

verus! {
fn caller(key: u64) {
    let index = bucket(key);
    assert(index < 16);
}
}
```

Run `verus example.rs --crate-type=lib --infer-goals`. Exact inference still
publishes the equation `result == ((key & 15) as u8)`. The additional strategy
starts from the assertion, substitutes relevant exact summaries and local
definitions, and proposes a conditional theorem for the existing bit-vector
prover. Ordinary verification then uses that checked theorem to establish the
unchanged assertion. It does not eagerly add ranges or alternative equations
to every function's contract.

The first strategy handles assertions whose expanded goals depend on an exact
inferred call and contain bitwise `&`, `|`, or `^`. Free variables must be Booleans
or fixed-width integers. Caller requirements and branch conditions can supply
explicit premises. Type bounds and nonzero divisors are premises too, never
unchecked facts. Each complete theorem is checked and used verbatim, and all
original calls, safety checks and assertions remain. A failed or timed-out
auxiliary proof fails verification, as with a handwritten `by(bit_vector)` proof.
Goal-assisted exports wait for complete verification, including for consumers
with only imported inferred contracts. Previous outputs are invalidated as soon
as their paths are known (before parsing for direct `--export` paths).

This is an opt-in strategy, not yet an ordinary-proof-first retry planner.
Omitted unsupported context can make it less effective than a manual proof.
It handles at most eight requests per caller, with 2,048 expanded nodes and
32 substitution levels per request, 8,192 statement visits and a 4,096-bit
intermediate-width limit. Unsupported/exhausted requests get no generated fact;
the original verifier still must prove them. It does not strengthen sound-only
contracts, infer loop invariants, directly discharge postconditions or implicit
index obligations, or cache successful lemmas. An explicit proved assertion can
provide the fact needed by a subsequent array access.

The source-connected AlephBFT pilot can be reproduced without network access:

```sh
python3 source/tools/eval_verus_infer_alephbft.py \
    --alephbft /path/to/AlephBFT \
    --output /tmp/aleph-infer-pilot --native-tests
```

Build this fork first. The runner makes a local clone at commit
`f35c7bba6d628bcafdf0d65dd471cc6f868ec034`, ignoring the input checkout's uncommitted
changes. It mechanically moves the original `common_vote` function to a shared
production module, checks executable-body identity, and verifies that exact
source file. The canonical remainder schedule and round alternation already
verify without assistance. An equivalent low-bit parity formulation needs two
manual local lemma invocations or the new strategy. This is a representation
bridge, not a new voting-agreement or recovery theorem.

Results include ordinary handwritten-contract/manual-proof, inferred baseline,
inferred/manual-proof and inferred/assisted variants; five semantic negative
controls; the source integration patch; tool fingerprints; proof reports; and
raw diagnostics. Timing observations are not a performance benchmark. Six
production election tests pass with default features. The pinned original
`--no-default-features` build fails in `collection/mod.rs`; the runner compares
the patched and unmodified error signatures and reports this as a baseline
limitation, not as passing tests. Quorum wrappers, generic state mutation and
the larger distributed proofs remain outside this pilot.

## Sound approximations and partial success

```rust
#[verus_infer(precision = "sound")]
fn fee(amount: u8) -> u8 { amount / 100 }
```

This mode proposes a contract equivalent to `requires true; ensures 0 <= result
&& result <= 2`. An ordinary caller can prove the range for every input. It
cannot conclude `fee(255) == 2` from that summary alone: the exact relationship
between the argument and result has deliberately been discarded.

Signed integers from `i8` through `i128` are supported too:

```rust
#[verus_infer(precision = "sound")]
fn scaled(x: i8) -> i8 { x / 10 }
```

This produces a range equivalent to `-12 <= result && result <= 12` for every
input. Signed remainder by 10 produces `-9 <= result && result <= 9`. These
ranges do not establish `scaled(30) == 3`; use an exact-mode contract when a
caller needs that relationship. `examples/verus_infer_partial.rs` demonstrates
both unsigned and signed range properties.

Every published summary still needs a successful ordinary soundness proof
against the original executable. Maximality and output exactness are attempted
separately when justified; counterexamples or timeouts in these optional probes
produce warnings and honest precision labels, while the sound contract remains
usable. A soundness failure or timeout still blocks dependent verification and
inference export. Exact mode continues to require all three proofs and never
silently falls back.

The first abstract domain tracks integer intervals and known/unknown Boolean
values. It handles the supported scalar expressions, refines ranges in reachable
branches and short-circuit expressions, follows immutable copies and Boolean
local guards, joins branch outputs, and composes inferred interfaces. When exact symbolic safety extraction fits
the budget and uses only exact-mode callees, sound mode retains that safety
predicate and widens the output relation to a range. If symbolic expansion
exceeds its budget, or a sound-mode callee prevents exact substitution, a
bounded compositional analysis derives safety conditions from callee
preconditions, result ranges, and the caller's operations. It keeps affine
arithmetic compact, so repeated addition can retain meaningful overflow bounds
without exponentially expanding local substitutions. Arguments start with
their full Rust type ranges; there is no automatic `[0, 1]` or `[-1, 1]` input
restriction.

For example, a sound `scaled(x: i8)` summary with `requires true` and result
range `[-12, 12]` lets `scaled(x) + 1` accept every `i8` input, since its result
lies in `[-11, 13]`. For `scaled(x) + delta`, a sufficient derived condition is
`-116 <= delta && delta <= 115`, with `x` unrestricted. Callee requirements are
preserved, including when arguments are approximate values. Conditional and
short-circuit guards scope safety conditions to the paths that execute; simple
guards on local values and their copies also refine the available facts.

The verifier must prove safety and the result range under the proposed domain.
These domains can still be conservative when contracts lose correlations. If
analysis exhausts its bounds or cannot express a usable sufficient condition,
it reports incomplete instead of guessing small input values. Ordinary proof
failure still blocks publication. Nonlinear arithmetic can require Verus's
existing solver guidance. Larger admitted domains can also produce wider output
intervals; a sound summary does not preserve caller-specific input/output
relationships unless they are present in the published contract.

Intervals use mathematical integer bounds for arithmetic and join branch
ranges. Signed division accounts for both negative and positive nonzero
divisors, including intervals spanning zero. Safety still excludes zero and
`(MIN, -1)` through the ordinary executable checks. Casts retain a tight interval
when its endpoints remain in one destination modular cycle, including wrapping
intervals such as `128..=130` cast to `i8`, which gives `-128..=-126`.
An interval crossing a modular boundary widens to the full destination range. Signed `MIN` literals are recognized before
their special compiler lowering is analyzed. Enum values, arrays, and early
returns still require exact mode.

Exact inferred callers conservatively require exact-mode callees, including
across crate boundaries and cache hits. Consume a sound-mode interface from an
ordinary verified caller or another sound-mode inferred function. Even when a
sound-mode callee happens to pass its precision probes, this initial composition
policy retains that restriction. Maximality probing is skipped for inferred
callers of sound-mode dependencies: assuming a potentially restrictive callee
precondition would not characterize all safe executions. Boolean top summaries
may also omit an exactness probe when no result relation is retained. Skipped
obligations remain **NOT ESTABLISHED**, never **PROVED**.

Use `precision_rlimit` to limit precision probes independently of the mandatory
soundness proof, in the same units as Verus's `rlimit`:

```rust
#[verus_infer(precision = "sound", precision_rlimit = 0.1)]
fn fee(amount: u8) -> u8 { amount / 100 }
```

Inside `verus!`, combine `#[verifier::infer(sound)]` with
`#[verifier::infer_precision_rlimit(0.1)]`. Without this override, precision
probes inherit the normal function/global solver budget. The override must be
finite and positive (tiny values round up to one solver resource unit); ordinary `#[verifier::rlimit(...)]` still controls soundness.
Other proposed RFC options, safe-only modes, and inference of unsupported data
structures remain unimplemented.

## Running the prototype

Build Verus as described in [BUILD.md](https://github.com/verus-lang/verus/blob/main/BUILD.md),
including `vstd`, then run from the repository root:

When invoking a debug compiler outside `source/`, also set
`RUST_MIN_STACK=20971520`, matching `source/.cargo/config.toml`. Cargo commands
launched inside `source/` receive that setting from the project configuration.

```sh
export VERUS_Z3_PATH="$PWD/source/z3"
source/target-verus/release/verus examples/verus_infer.rs
```

To compile and run the verified example:

```sh
source/target-verus/release/verus examples/verus_infer.rs --compile -o /tmp/verus-infer-example
/tmp/verus-infer-example
```

If building manually with Cargo, the equivalent development setup from `source/` is:

```sh
cargo build --release -p rust_verify -p verus -p cargo-verus
PATH="$PWD/target-verus/release:$PATH" target-verus/release/cargo-verus verus build --release --manifest-path vstd/Cargo.toml
```

The driver prints each candidate formula and separate results for:

- **Soundness:** every admitted input executes safely and satisfies the result relation.
- **Safe-domain maximality:** every safe execution starts from an admitted input.
- **Output exactness:** the result relation admits exactly the executable's returned value.

Inferred functions are checked in callee-first order. A failed or incomplete
callee stops verification before any dependent inferred function is checked.
After all required checks succeed for every inferred function, the driver makes
those contracts available to ordinary caller verification. Exact mode requires
all three checks; sound mode requires soundness and records the precision probes
independently. A required check failure or timeout prevents dependent verification
and publication. Individual proof results remain visible in diagnostics.

The example automatically infers two more layers over `compute` and `debit`,
then proves a new application property using the top-level summary. It also uses
`fee` to prove that inputs below 100 incur no fee. These proofs use callee
contracts without inlining the executable implementations.

## Structured contract reports

`--output-json` includes a `verus-infer` object with schema version 1. Save it
with the existing timing breakdown using:

```sh
source/target-verus/release/verus examples/verus_infer_partial.rs \
  --output-json --time > /tmp/infer-report.json
```

Stdout contains the JSON; source and proof diagnostics remain on stderr. Check
the process exit status as well as the report. Normal compiler and verification
failures still produce a report when control returns to the driver; an invalid
command line, process crash, or external interruption may produce none.

Each generated function records its name, source location, parameter/result
types, requested `mode`, actual `candidate_id`, separate `contract.requires` and
`contract.ensures` strings, and three `proofs` entries. These predicates come
from the actual installed candidate. They use mathematical integer arithmetic,
explicit machine casts, and `rust_div`/`rust_rem` for signed truncating arithmetic.
`result` names the return value. Bounded simplification happens before installation:
redundant typed bounds and Boolean conditions can disappear, affine terms can
cancel, and simple local-view bounds can reduce to a direct index bound. The
simplified formulas themselves receive all required fresh proofs. For example,
`(x + 1) - x` on `u8` has `requires x <= 254; ensures result == 1`; cancellation
retains the prior addition's safety condition. There is no separate display-only
predicate substituted for the checked formula.

| Proof status | Meaning |
| --- | --- |
| `proved` | Candidate-bound evidence records successful verification of every required subquery for that obligation. |
| `not_proved` | The query did not establish the obligation. This does not by itself prove the candidate false. |
| `incomplete` | At least one subquery exhausted its solver resource limit. |
| `not_established` | No successful check is recorded; `reason` explains a skip, a check not run, or unavailable matching evidence. |

Every proof entry also has `required` and `reason` fields. Sound mode requires
soundness; exact mode requires all three obligations. A sound-mode dependency
explains a skipped maximality probe with `sound_mode_dependency`. An unconstrained
Boolean result can skip exactness with `unconstrained_result`. Neither skip
becomes a proved precision claim. Optional failed or incomplete probes can
coexist with successful sound-mode verification.

`required_checks_passed` uses the same candidate-identity and poisoned-evidence
checks as acceptance. `available_to_same_crate_callers` records whether all
inferred functions passed the required-check barrier. `exported` additionally
requires successful interface writing and public visibility. A later ordinary
caller failure can leave valid inference proofs visible while preventing
interface export. The crate-level `interface_written` reports the actual export
action; an invocation without an export path leaves it false.

Crate-level `status` distinguishes `verified`, `no_inference`, `unsupported`,
`analysis_incomplete`, `generation_failed`, `required_proof_failed`,
`required_proof_incomplete`, `caller_proof_failed`, and `verification_failed`.
`generation_failure` retains the synthesis diagnostic. Other frontend, import,
and verifier errors use `verification_failed` with details on stderr. An early
frontend failure can precede discovery of annotations; zero requested or
generated functions in such a failed invocation does not mean it had no
inference annotations. Partial generation reports only completed candidates,
without granting them proof status or publication.

Per-function dependencies record inferred callee identities and modes. The
crate report also lists validated imported inference fingerprints and trusted
standard-model fingerprints, including transitive assumptions. These preserve
the existing Verus trust policy. This JSON is a read-only report; it cannot be
imported as a certificate or used to bypass verification. Candidate IDs identify
actual typed formulas and provenance, including source spans; they are not stable
semantic hashes for comparing cold and warm runs.

Counted-loop functions also report `loop_proofs`: the installed invariant
predicates, decreasing measure, loop-isolation setting, and arithmetic prover.
These are proposed proof annotations and settings. Successful ordinary soundness establishes invariant
and termination obligations under the reported precondition; the separate
precision queries retain their own checks. The report does not mint an additional
termination receipt or treat an unchecked annotation as a theorem.

`metrics` records generation time, candidate-cache hits/misses, candidate SMT
time, and expanded predicate AST sizes. Generation and solver results remain
separate: a cache hit still requires fresh proofs. Optional precision queries
and diagnostic reruns contribute to SMT time. The whole-generation timer also
includes dependency ordering and failed work. See the
[evaluation corpus](../../../tools/infer_eval/README.md) for measurement scope.

## Supported code and safety policy

Exact mode supports `bool`, `usize`, `isize`, fixed-width unsigned integers (`u8`, `u16`, `u32`,
`u64`, `u128`), and fixed-width signed integers (`i8`, `i16`, `i32`, `i64`,
`i128`), immutable parameters and locals, scalar return values, `+`, `-`, `*`,
`/`, `%`, signed negation, comparisons, Boolean operations, signed/unsigned integer
casts, bitwise `&`, `|`, `^`, `!`, shifts `<<`/`>>`, and `if` expressions.
Short-circuit conditions guard the safety checks of
the expressions that are actually evaluated. Casts retain Rust's truncation
semantics; inferred arithmetic formulas otherwise use mathematical integers.
As in ordinary Verus proofs, callers may need a `by(bit_vector)` assertion to
derive a concrete value from a narrowing cast outside its destination range.

`usize`/`isize` arithmetic and casts use Verus's existing symbolic pointer-width model.
The default verifies both 32-bit and 64-bit architectures. Existing
`global size_of usize == 4;` or `== 8;` declarations select one width. Native
compilation also checks that the declaration matches the actual Rust target.
The explicit-width regression probes test verification with `--no-erasure-check`;
portable examples retain erasure checks and run on the host. For example:

```rust
#[verus_infer]
fn advance(index: usize, count: usize) -> usize { index + count }
```

Its contract requires the mathematical sum to lie in `0..=usize::MAX` and ensures
that the result equals that sum. `usize::MAX` remains symbolic in formulas,
candidate caches, and exported interfaces. Casts to `usize` truncate at the modeled
width: `4294967297u64 as usize` is `1` on 32-bit and `4294967297` on 64-bit.
Both platform types work as flat enum payloads, array/slice elements and scalar
results. Signed division excludes zero divisors and `isize::MIN` with `-1`, with
Rust's truncation semantics. The seven checked methods listed below also support
both types through existing vstd models. Counted loops remain unsigned, including
`usize`; signed counted loops require further work. See
`examples/verus_infer_usize.rs` and `examples/verus_infer_platform.rs`.

For ordinary arithmetic, the safety policy excludes overflow, underflow, and
division/remainder by zero, consistently with Verus's existing executable
arithmetic checks. This policy applies even when a release Rust executable would
wrap on overflow. Inference adds no runtime checks, changes no executable body,
and imposes no verification overhead on the compiled program. Unverified Rust
callers are responsible for respecting inferred preconditions.

Signed arithmetic excludes results below `MIN` or above `MAX`; negation excludes
`MIN`. Signed `/` rounds toward zero, and `%` retains the dividend's sign. Both
operations exclude a zero divisor and the pair `(MIN, -1)`, including remainder
even though its mathematical result would be zero. For example:

```rust
#[verus_infer]
fn reverse(delta: i64) -> i64 { -delta }

#[verus_infer]
fn quotient(total: i64, count: i64) -> i64 { total / count }
```

`reverse` requires `delta != i64::MIN`. `quotient` requires `count != 0` and
`total != i64::MIN || count != -1`; it returns `-2` for `(-7, 3)`. Inferred
formulas display Rust division/remainder as `rust_div`/`rust_rem`. They use
Verus's existing inline vstd definitions, whose declaration and artifact
fingerprints are recorded even though the source contains an operator rather
than an explicit call. Import `vstd::prelude::*` when using signed `/` or `%`;
missing models produce an unsupported diagnostic. Signed casts preserve Rust's
truncation, sign extension, and reinterpretation behavior. No new standard
specifications or axioms were added.

See `examples/verus_infer_signed.rs` for a runnable example covering negative
division, overflow-safe adjustments, conservation, and signed casts. Signed
values are also supported in flat enum payloads and fixed arrays in exact mode.
Sound interval mode supports fixed-width and platform-sized unsigned/signed
scalar integers and Booleans, including signed intermediate expressions.
Platform-sized analysis computes independent 32-bit and 64-bit ranges and guards
differing results with `usize::BITS == 32`. For example, `x: usize` divided by
`10` yields `0..=429496729` on 32-bit and `0..=1844674407370955161` on 64-bit.
The matching safety predicates and ranges are preserved through composition,
caches, and imported interfaces. Mandatory soundness proves these claims;
optional precision checks can still report that exact output is not established.

Exact mode additionally supports core `Option<T>` and `Result<T, E>` as inputs,
locals, and return values. Each payload must be a supported scalar or `()`;
nested enums and general user-defined enums are not supported. The single-variant
scalar-wrapper case is described above. Constructors and simple
exhaustive `match` expressions may use by-value bindings, scalar literal
patterns, unit patterns, and wildcards. Arms retain Rust's first-match priority,
and only the selected arm's operations contribute safety conditions. Match
guards and or-patterns are supported. Guards execute only after their pattern
matches and earlier arms fail; their arithmetic and inferred calls retain safety
requirements. Or-pattern alternatives must bind the same immutable by-value
names and types. Verus currently rejects combining an or-pattern with a guard in
one arm. Reference patterns and mutable bindings remain unsupported.
Enum expressions currently require exact mode; sound interval inference remains
scalar-only.

The fixed-width and platform-sized unsigned and signed methods `checked_add`, `checked_sub`, `checked_mul`,
`checked_div`, `checked_div_euclid`, `checked_rem`, and `checked_rem_euclid` are
also supported in exact mode. Inference recognizes their core identities and
existing vstd specifications, rather than accepting arbitrary methods with the
same name. An unsuccessful operation returns `None`; it does not impose an
overflow or zero-divisor precondition. Evaluating the arguments must still be
safe. The generated postcondition describes both the enum variant and its
payload exactly.

For signed values, `checked_div`/`checked_rem` use Rust's rounding/sign rules;
`checked_div_euclid`/`checked_rem_euclid` use Euclidean division with a nonnegative
remainder. All four return `None` for a zero divisor or `(MIN, -1)`. Mixed-sign
methods are also supported: unsigned receivers accept `checked_add_signed` and
`checked_sub_signed`; signed receivers accept `checked_add_unsigned` and
`checked_sub_unsigned`, using the corresponding signed/unsigned width. Both
signed and unsigned types support `checked_neg`: signed `MIN` returns `None`,
and unsigned negation returns `Some(0)` only for zero. These checked methods
remain exact-mode enum operations.

For example:

```rust
#[verus_infer]
fn checked_debit(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

#[verus_infer]
fn validated_debit(balance: u64, amount: u64) -> Result<u64, ()> {
    match checked_debit(balance, amount) {
        Some(remaining) => Ok(remaining),
        None => Err(()),
    }
}
```

Both functions admit every pair of `u64` inputs. A successful result contains
`balance - amount`; the error variant characterizes insufficient funds. The
standalone `examples/verus_infer_enums.rs` composes checked debit, a fee, and
validation, then proves conservation for successful transfers and runs concrete
success, insufficient-funds, overflow, and zero cases. Run it with the same
`--compile` workflow shown above.

Exact mode supports explicit early `return` values in blocks, branches, match
arms, and expressions using the supported types. For example:

```rust
#[verus_infer]
fn charge(balance: u64, amount: u64, fee: u64) -> Result<u64, u8> {
    let total = match amount.checked_add(fee) {
        None => return Err(1),
        Some(total) => total,
    };
    if total > balance { return Err(2); }
    Ok(balance - total)
}
```

This function safely accepts every input and characterizes both errors and the
successful result. Returns stop evaluation: operations after an exit contribute
no safety conditions, while operations evaluated before it still do. This also
applies to left-to-right operand/array-element evaluation and short-circuit
branches. Guarded return values and the continuing path are synthesized as pure
conditional formulas; the original executable and its return checks stay intact.
All three independent proof obligations are still mandatory. Eligibility checks
still inspect the whole source, so an early return cannot hide unsupported
syntax or proof assumptions. Early returns and richer patterns currently require exact mode.
`let-else` supports the same flat patterns, with a returning else branch:

```rust
#[verus_infer]
fn bump(input: Option<u8>) -> Option<u8> {
    let Some(value) = input else { return None; };
    Some(value + 1)
}
```

The initializer is evaluated once. Failed matching executes only the else
branch; successful matching introduces bindings for the continuing block. The
example requires `value < 255` only for `Some(value)`. Safety operations in the
else branch also remain enforced. Unsupported code is rejected even on a path
that always returns or a pattern that never matches.

Exact mode also supports `?` on the supported flat `Option` and `Result` types:

```rust
#[verus_infer]
fn charge_checked(balance: u64, amount: u64, fee: u64) -> Option<u64> {
    let total = amount.checked_add(fee)?;
    Some(balance.checked_sub(total)?)
}

#[verus_infer]
fn add_fee(input: Result<u8, u16>, fee: u8) -> Result<u8, u16> {
    Ok(input? + fee)
}
```

`charge_checked` safely accepts every input, returning `None` on overflow or
insufficient funds. `add_fee` propagates the original `Err` payload, and requires
that addition fit in `u8` only when the input is `Ok`. The operand is evaluated
once, before deciding whether to exit; operations evaluated before `?` retain
their safety conditions. Nested propagation, branches, early returns, and
acyclic inferred calls compose normally, including through caches and crates.
Result errors may retain their type or use the existing lossless primitive
integer `From` conversions. For example, `Result<u8, u8>` can propagate into
`Result<u8, u16>` without restricting error values. The closed set is:

- Unsigned widening `u8/u16/u32/u64` to larger fixed unsigned types.
- Signed widening `i8/i16/i32/i64` to larger fixed signed types.
- Unsigned to strictly wider fixed signed types.
- `u8/u16` to `usize`, `u8` to `isize`, and `i8/i16` to `isize`.

These are 35 concrete conversions backed by the existing vstd models, valid
under both pointer widths. Success payload types may change between the operand
and enclosing function. Bounded checked generic `From` error conversions are also
supported by the owned profile described above. Other user-defined `From`, narrowing conversions,
custom `Try`, try blocks, nested enums, and sound-mode enum inference remain
unsupported.

Propagation uses existing vstd `branch` and `from_residual` specifications.
Its internal `ControlFlow` and `Infallible` values are recognized only inside the
complete lowering; their presence does not permit general generic calls or proof
statements. Result's identity conversion relies on Rust's existing blanket
`From<T> for T` semantics and vstd's trusted identity axiom for unchanged error
types, or existing primitive `FromSpecImpl` specifications for approved widening. Its generic residual
model has no `no_unwind` clause; inference preserves that limitation. In
particular, a wrapper that can propagate an `Err` cannot establish `no_unwind`
from this model. No new axioms or stronger standard specifications were added for `?`.

Exact mode also supports immutable, flat fixed-size arrays `[T; N]` with
`0 <= N <= 64`, where `T` is `bool`, `usize`, `isize`, or a supported fixed-width unsigned/signed integer.
Arrays may be passed by value or through `&[T; N]`, copied or borrowed into
immutable locals, selected by branches, or constructed with element-list
literals such as `[x, x + 1]` and repetition literals such as `[x; 4]`. Owned
fixed scalar arrays may also be returned and composed across inferred calls,
caches, and crate interfaces. Array payloads inside enums remain unsupported.
Array results are described as whole values. Exactness quantifies over an
arbitrary valid array and compares all elements using Verus's existing
extensional equality, including lengths zero and 64. The existing vstd
`axiom_array_ext_equal` and its containing artifact are recorded as a trusted
model dependency; no new equality axiom is introduced.

```rust
#[verus_infer]
fn fee_at(fees: &[u64; 4], tier: usize) -> u64 {
    fees[tier]
}

#[verus_infer]
fn checked_fee(fees: &[u64; 4], tier: usize) -> Option<u64> {
    if tier < fees.len() { Some(fee_at(fees, tier)) } else { None }
}
```

The first function infers `tier < 4` (also printing the redundant lower bound
`0 <= tier`) and an exact result equal to `fees[tier]`. The guarded function
admits every `usize` input and describes both `Some(fees[tier])` and `None`
exactly. Branch and short-circuit guards restrict bounds checks to reads that
execute. Every element of an array literal is evaluated, so an overflowing
element still requires a safety precondition even when another element is read.
An unguarded read from an empty array has no safe input and receives an
unsatisfiable precondition; guarded functions can still use empty arrays safely.
Repetition evaluates its operand once, including when the length is zero. Thus
`[x + 1; 0].len()` still requires that `x + 1` not overflow. `.len()` preserves
the safety conditions of its receiver even though the result is a known constant.

`usize` inputs, locals, arithmetic, comparisons, results, and casts to/from
supported integer types are available for index calculations. Expressions such as
`values[index + 1]` retain both arithmetic overflow and indexing bounds checks.
Array indexing uses Verus's existing array model; its normal bounds
assertions independently check the inferred safe domain. Shared references use
Verus's existing immutable value representation. Length, array-to-slice coercion,
and repetition use a closed allowlist of existing core/vstd models, specialized
only to supported scalar elements and fixed lengths. The two pure vstd array
helpers additionally declare `no_unwind`, and inference requires this guarantee
on approved executable array models. There are no new array axioms.

This array subset excludes nested arrays, mutable references, reference-valued
elements, array patterns, and mutation. Shared dynamic slices are supported as
described below.
Signatures and local bindings support one shared reference to an array; compiler
reborrows are handled transparently. Array-to-slice conversion also supports
forwarding these arrays to inferred slice helpers. Sound interval mode does not yet support
arrays or shared-array references.

`examples/verus_infer_arrays.rs` composes array lookup, addition, and checked
arithmetic, proves a pricing property, and runs success, overflow, and invalid-tier
cases. It also constructs a repeated fee schedule and passes it through a borrowed
interface. It can be verified, compiled, and run with the workflow above.

Exact mode additionally supports read-only dynamic slices `&[T]`, with the same
scalar element types. Their runtime lengths stay symbolic, so the fixed-array
limit of 64 elements does not apply to slice inputs. For example:

```rust
#[verus_infer]
fn at(values: &[u8], index: usize) -> u8 { values[index] }

#[verus_infer]
fn get(values: &[u8], index: usize) -> Option<u8> {
    if index < values.len() { Some(at(values, index)) } else { None }
}
```

`at` requires `0 <= index < values.len()` and returns exactly `values[index]`.
`get` admits every index and describes `Some(values[index])` or `None` exactly.
The ordinary executable supplies the independent bounds checks. Guarded reads,
short-circuit conditions, length comparisons between slices, immutable aliases,
shared reborrows, and selection between slice parameters compose normally.
Scalar and flat `Option`/`Result` returns, early returns, checked arithmetic,
and `?` retain their existing support and restrictions. Arithmetic on read
elements adds its usual overflow/division requirements.

Inferred callers can pass shared slices or coerce supported fixed arrays into
slices. Array coercion retains the exact array term, specializing subsequent
lengths and reads. Receiver evaluation remains eager: `get(&[x + 1; 0], 0)`
still requires that `x + 1` not overflow, even though `get` returns `None`.
Ordinary callers can pass arrays larger than 64 elements to slice interfaces;
inferred bodies that construct arrays retain the fixed-array policy above.

Slice summaries use existing Verus length/index operations and existing vstd
specifications. The canonical `spec_slice_len` and `spec_slice_index`
declarations and their containing artifacts are recorded as implicit model
dependencies when used. The executable `.len()` model is recognized by the
same closed identity/signature checks as before. No slice axioms or stronger
library specifications were added. Cached candidates are retyped and reproved;
cross-crate interfaces and JSON reports preserve all three exact-mode guarantees.

Local immutable subslices support `&s[start..end]`, `&s[start..]`, `&s[..end]`,
and `&s[..]` for shared scalar slice inputs and aliases. Inferred conditions
include `start <= end <= s.len()` and any subsequent index bound relative to
the view. Empty views are valid; reading one still requires a valid index.
Nested views and passing views to inferred slice callees are supported. View
lengths reduce to `end - start`; reads reduce to the original base plus the
accumulated offset. Range operands retain Rust's evaluation order and all
preceding safety checks. Views are local synthesis data and cannot occur in
installed contracts or cached formulas. Each contract refers to original inputs.

Subslice calls recognize the complete concrete core `Index` lowering for these
four ranges. Ordinary Verus checks its existing standard preconditions. In
maximality queries, only validated canonical shared-range calls may use their
existing exact bounds requirements to select successful executions; generic
types and actual arguments are instantiated. This extends the prior inferred
callee path without bypassing ordinary checks. Their vstd artifact is
fingerprinted, including range, indexing, and sequence specifications.

This subset excludes mutable slices, returned slice references, direct slicing
of fixed arrays, inclusive ranges, stored range objects, reference-valued
elements, general generic slice methods, slice traversal loops, and sound-mode
slice inference. Conditional selection involving a newly constructed local view
is also unsupported. Conditional coercions
mixing arrays of different fixed lengths, or mixing an array term and a dynamic
slice term, report an unsupported conditional; selection between dynamic slice
inputs is supported even when their runtime lengths differ.

`examples/verus_infer_slices.rs` proves lookup and pricing properties, composes
checked arithmetic and `?`, and runs empty, invalid-index, overflow, signed-edge,
and 300-element cases. The three-crate Cargo workspace also composes a slice
lookup and charge through exported interfaces. The broader
`examples/verus_infer_coverage.rs` combines richer patterns, widened errors,
array outputs, and subslices, with a three-crate Cargo example as well.

## Counted numeric loops

Exact mode supports unsigned accumulator loops over two or three immutable
same-width inputs (`usize`, `u8`, `u16`, `u32`, `u64`, or `u128`):

```rust
#[verus_infer]
fn subtract_steps(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut remaining = n;
    while remaining > 0 {
        result -= step;
        remaining -= 1;
    }
    result
}
```

This infers mathematical `requires n * step <= x` and
`ensures result == x - n * step`. Addition infers
`requires x + n * step <= MAX` and `ensures result == x + n * step`.
The product is unbounded mathematical arithmetic. Zero steps/counts admit the
full input range; there is no small-count cap or incomplete unrolling.

The counter may instead start at zero and use `while i < n` with `i += 1`.
Either counter direction supports accumulator addition or subtraction. The step
may be an unsigned literal or immutable same-width input (including `x` or `n`);
`usize::MAX` is supported symbolically. Unit steps retain simpler `x + n` /
`x - n` contracts. All parameters must be used. Local names and parameter order
may differ, and the two declarations may be reversed. Updates may use `+=`/`-=`
or `v = v + operand` / `v = v - operand`. The accumulator update must precede the
unit counter update; the tail expression returns the accumulator.

The whole typed body is checked. Other guards/counter steps, extra statements,
dynamic step expressions, signed loops, nested/multiple loops, breaks/continues,
handwritten loop annotations, and general mutation remain unsupported. Loop
inference requires exact mode. `assume_termination` and
`exec_allows_no_decreases_clause` settings are rejected. Loop-free inferred
wrappers can compose the exact summaries.

The prefix invariants are `counter <= n` and
`result == x ± completed * step`. For ascending counters, `completed = counter`
and the measure is `n - counter`; for countdown counters, `completed = n - counter`
and the measure is `counter`. Loop isolation is disabled so ordinary soundness
can use its precondition without including that predicate in the invariants.
Verus checks invariant entry/preservation, arithmetic safety, termination, and
the result relation. Parameter products and symbolic platform constants select
Verus's existing nonlinear prover. Literal-step loops do not require it.
Neither selection adds an assumption or trusted helper specification.

Maximality removes the candidate precondition and checks the prefix invariants
and termination independently. At a successful exit, type bounds establish the
safe domain. Exactness retains the arbitrary-output query. All three required
outcomes gate dependent callers and export.

A production projection check removes exactly the generated invariants, measure,
and isolation setting and compares every remaining serialized typed-body field
with source. Annotation terms must belong to the restricted pure summary
language. Candidate identities bind source, installed annotations, actual
contract, and nonlinear-prover setting. Cache hits recheck eligibility and
regenerate proof settings; caches contain no bodies, annotations, or outcomes.
See `VERUS_INFER_LOOPS.md` for the design and fault-injection strategy.

`examples/verus_infer_loops.rs` demonstrates all four parameter-step families,
constant/zero steps, composition, and runtime boundary cases. The Cargo workspace
composes stepped addition and countdown subtraction across three crates.

## Bitwise operations and explicit arithmetic policies

Exact mode supports `&`, `|`, `^`, and integer `!` on every supported integer
type, including `usize`/`isize`. Operands retain their finite signedness and width
through arithmetic expressions, enum payloads, cache decoding, and imported calls.
Signed right shifts extend the sign bit. Left shifts discard high bits as Rust
does. Normal `<<` and `>>` require `0 <= amount < bit_width`; a zero left operand
does not remove that requirement. The amount may use any supported integer type.
No new bitwise axioms are added; ordinary Verus checks the original operations.

Both signed and unsigned integers support `wrapping_add`, `wrapping_sub`,
`wrapping_mul`, `saturating_add`, `saturating_sub`, and `saturating_mul` in exact
mode. Wrapping returns the representable modular result; saturation clamps the
mathematical result to the receiver's `MIN..=MAX`. Their total semantics admit
all inputs, while preserving safety checks of argument expressions. They do not
change the overflow policy of ordinary `+`, `-`, or `*`.

Sound scalar mode uses bounded numeric transfers: `x & 15` has range `0..=15`,
and `(x & 15) >> 1` has range `0..=7`, across signed/unsigned supported widths.
Branch guards can tighten shift operands. Wrapping casts and wrapping/saturating
methods retain useful ranges when the input bounds allow it; otherwise they widen.
Verus checks auxiliary conditional range lemmas with its existing bit-vector
prover and uses that identical theorem in the ordinary proof. Operand ranges and
finite type bounds are premises: the ordinary proof must establish them before
using the result. These hints add no axiom and change no executable expression.
Auxiliary failures/timeouts belong to the function's obligation and cannot be
hidden by another successful subquery. Bounded automation may still fail to prove
a valid candidate; it never converts missing evidence into a verified contract.
Compositional safety retains shift-amount checks and guarded arithmetic. Lost
relations or insufficient range information can still produce an incomplete result.
Optional precision failures retain honest labels and never replace soundness.

The new checked negation, unsigned `checked_sub_signed`, and signed saturating
specifications are explicit trusted vstd additions, audited in the repository's
`VERUS_INFER_ASSURANCE.md`. Other supported numeric methods reuse existing models.
Models require canonical core identities, matching receiver/argument/return types,
a vstd proxy, empty preconditions, and `no_unwind`; their declarations and entire
artifacts are fingerprinted. Rebuild vstd when using the added models.

This batch does not add rotate/count methods, wrapping/saturating mixed-sign
methods, explicit wrapping shifts, or general user-defined integer models.
`examples/verus_infer_integer_ops.rs` verifies caller properties and runs wrapping,
saturation, bitwise, shift, and checked-result boundary examples.

## Application kernels and evaluation

`examples/verus_infer_applications.rs` contains fifteen pure Rust kernels for
big-endian parsing, protocol fields, slice windows, checked purchases, price
pairs, capped fees, and signed wire codes. It includes ordinary caller proofs
and a native boundary runner. The corpus adds wrong caller properties, invalid
windows, an unsupported iterator checksum, and an intentionally incomplete
bit-vector proof. These examples measure selected functionality within the
supported subset.

From the repository root, after completing the build:

```sh
python3 source/tools/eval_verus_infer.py --case applications \
  --repetitions 3 --memory --output-dir /tmp/infer-app-eval
python3 source/tools/eval_verus_infer_cargo.py \
  --repetitions 3 --memory --output-dir /tmp/infer-cargo-eval
```

The first runner always rechecks proofs, including warm candidate hits. A
no-edit Cargo run may skip verification through Cargo's normal incremental
build cache; the Cargo report distinguishes that from fresh proof work. Its
contract edit must reject a stale caller and pass after its property is updated.
Both runners detect changed/missing compiler or model artifacts and abort rather
than mixing builds. Memory is the OS waited-child maximum RSS, measured in a
fresh worker, not a simultaneous process-tree peak. See
`source/tools/infer_eval/README.md` for the measurement definitions and limits.

## Analysis budgets and other restrictions

The synthesizer merges branch results as conditional expressions instead of
enumerating paths. It bounds both source traversal and expanded formula size at
4,096 nodes, and bounds dependency traversal depth at 128 functions. Solver checks use Verus's `--rlimit` and per-function
`#[verifier::rlimit(...)]` / `#[verus_verify(rlimit(...))]` settings. A resource
limit is reported as incomplete, separately from a counterexample or unsupported
syntax.

For scalar inference, static, nongeneric calls to other inferred functions in the same crate or to
certified inferred dependency interfaces are supported, including forward
references, shared helpers, conditional calls, and
short-circuit guards. Dependencies must form an acyclic graph. Callee summaries
are substituted without analyzing their executable bodies again.

Unsupported features include general loops (the bounded numeric and collection patterns are supported),
recursion, converting `Result` errors with `?`, calls to functions
without inferred contracts other than the approved standard models,
integer operations outside the allowlist above,
references, mutation, collections and generics beyond the templates described above,
and ghost/proof statements.
An ordinary verified caller may compose inferred functions at any supported depth.
Handwritten contracts cannot be combined with inference yet; they receive an
explicit diagnostic and are never silently overwritten.

A versioned registry tracks candidates and their transitive dependencies. Set
`VERUS_INFER_CACHE` to a directory to persist candidate formulas across runs:

```sh
VERUS_INFER_CACHE=/tmp/verus-infer-cache source/target-verus/release/verus examples/verus_infer.rs
```

Cache keys include the typed source, signature, function attributes, inference
schema/safety policy, verifier arguments, transitive callee keys, approved
standard-model declarations, and fingerprints of their containing artifacts.
Changes to supporting specifications and axioms also invalidate candidates.
Entries with unchanged keys remain
reusable; source-location changes can conservatively cause additional misses.
Malformed, oversized, stale, or incompatible entries are treated as misses.
Cache I/O is best effort and atomic. Cached preconditions and general postconditions use a restricted language;
types, parameter/result identities, and diagnostic spans are rebuilt from current
source. Requested precision is part of candidate identity, and cache validation
cannot turn a sound-mode interface into an exact-mode one.
Source eligibility is checked on every run. **Proof results are never cached:**
every reused candidate must pass the checks required by its requested mode before
dependent verification. Optional probes are rerun too; their outcomes are not
recovered from candidate cache data.
Fresh and cached formulas use one installer that preserves the executable body
and signature. Its candidate identity covers the original typed function, actual
contract formulas, configuration, and transitive dependencies. Query evidence is
bound to that identity and the obligation kind; duplicate or mismatched records
invalidate acceptance rather than overwriting an earlier outcome.
Old content-addressed entries remain on disk; the directory may be deleted to
clear the cache. Without the environment variable, storage stays in memory.

Source eligibility and executable arithmetic policies are shared between cache
reuse and fresh synthesis. Cache decoding and imported-summary substitution use
the same typed summary builders, which reject unsupported operators, inconsistent
operand types, and invalid enum projections. Summary traversal is bounded even
when constant conditions simplify a large input formula to a small result.

## Cross-crate inference and Cargo

Public inferred functions use Verus's existing `.vir` export/import mechanism.
A dependency can infer its public interface, another crate can infer a wrapper
around that interface, and an application can prove an explicit property using
the resulting summary. Private functions and executable bodies are pruned using
the existing library-export rules; importers need only the inferred contracts.
Exact contracts still describe implementation behavior, including behavior
computed by private helpers.

The three-crate example in `examples/verus_infer_workspace` demonstrates this:

```sh
# From the repository root, after building Verus and cargo-verus:
export PATH="$PWD/source/target-verus/release:$PATH"
export VERUS_Z3_PATH="$PWD/source/z3"
cargo verus build --manifest-path examples/verus_infer_workspace/Cargo.toml
cargo verus verify --manifest-path examples/verus_infer_workspace/Cargo.toml
```

`base` exports an inferred increment implemented through a private helper;
`middle` infers a function that calls that dependency twice; `app` infers one
more wrapper and proves that its result is the original input plus two.
The same workspace also exports sound-only fee ranges, composes them through
sound-mode wrappers, and proves their caller ranges for the full input types.
It also exports checked `debit` and exact fee interfaces from `base`, infers a
`Result`-returning validation function in `middle`, and proves successful-transfer
conservation in `app` using its inferred wrapper.
An array-based fee lookup additionally composes through all three crates; the
application proves the exact total under the inferred index and overflow bounds.
No dependency implementation is inlined into a consumer proof.

For direct compiler invocations, use the normal `--export library.vir` and
`--import library=library.vir` options alongside matching Rust libraries supplied
with `--extern`. Supply the transitive inferred `.vir` dependencies as well;
`cargo verus` resolves these automatically.

For a crate with inferred definitions, export is deferred until full-crate
verification, including each function's required inference checks, has succeeded. Publication
uses an atomic file replacement. A re-verification invalidates an existing
certified inference export before checking dependencies or local proofs, so a
failed verification cannot leave that old artifact available at the output path.
`--no-verify` and partial verification filters are rejected for local inferred
definitions.

Inference exports carry a versioned proof-metadata trailer recording soundness,
maximality, exactness, the safety policy, producer version/toolchain/revision,
contract fingerprints, per-function proof outcomes, and imported inferred-dependency
fingerprints. Approved standard-model dependencies additionally record fingerprints
of both their declarations and containing vstd artifacts; imports reject stale
or missing models, including changes to supporting spec functions and axioms. The current
schema supports mixed exact/sound libraries: a sound-only
function does not downgrade an unrelated exact interface. Exact-mode exports must
have all three guarantees, while sound-mode exports must have proved soundness;
missing precision guarantees are preserved rather than invented. Import
rejects missing, incompatible, incomplete, conflicting, or stale metadata with
a rebuild diagnostic. Changes to imported summaries also invalidate dependent
local candidate-cache entries. Cargo's existing `.vir` dependency tracking
triggers rebuilds of consumers. Fingerprints conservatively include diagnostic
locations and all imported inferred interfaces, so some semantically unchanged
rebuilds may also invalidate dependents.

The current inference policy uses metadata schema 22 and candidate-cache version 27.
Loop candidate identity version 3 also binds the selected nonlinear-prover setting.
Cache identities include the merged architecture of the crate and its imports;
exported contract fingerprints also bind that architecture. A library proved
for a fixed width constrains its consumers to that width; incompatible fixed
widths are rejected by Verus's existing architecture merge. A portable library
can be consumed at either width. All cached candidates still receive fresh proofs.
Rebuild inferred libraries and their dependents with the updated compiler.
The internal candidate-identity field introduced with schema 6 changed serialized
VIR; libraries built before that change, including `vstd`, also need rebuilding.
Ordinary exports still omit inference proof trailers.

Ordinary libraries keep their existing export representation. Imported proof
metadata uses Verus's normal trusted-library model: it records completed proofs;
it is not a portable proof object or a cryptographic signature. Dependency bodies
are not reverified on import. Local inferred wrappers receive fresh proofs using
the imported guarantees. The candidate cache remains untrusted and stores no
proof results.

Annotation is opt-in; functions without the attribute retain normal Verus behavior.

## Implementation and trust

Candidate synthesis operates on typed VIR after mode checking and erasure-mode
collection, before simplification and SST lowering. Keeping generated specs out
of source-based erasure analysis preserves executable parameter/local modes.
The executable body remains intact. The provisional `requires` and `ensures`
fields serve as the in-memory contract store and become exported dependency
interfaces only after successful verification.

The existing `dual_spec` macro was evaluated as a starting point. It copies
executable logic to a spec function, but still needs explicit contracts and does
not synthesize a safety domain. VIR provides explicit arithmetic policies and
typed branches, so the prototype implements a separate synthesis pass and uses
the existing verifier for checking and modular consumption.

The inference passes are untrusted: an unsound candidate must fail checking.
A deliberately imprecise candidate may fail optional precision probes while
retaining its proved sound guarantees.
Soundness uses the ordinary verification query. Precision checks operate on the
original executable's verifier-generated SST:

1. **Maximality:** remove the candidate precondition from the query assumptions,
   replace built-in safety assertions with safety assumptions, and prove the
   candidate precondition at every normal return. This checks `Safe => P` using
   the verifier's own safety conditions. At a call, the already-verified callee
   precondition selects successful executions and its exact postcondition
   describes the result. The ordinary soundness query still
   proves all safety assertions under `P`.
2. **Exactness:** introduce an arbitrary output `y` and prove
   `Q(inputs, y) <==> y == actual_return` using the original executable body under
   `P`. The arbitrary output is constrained to valid Rust values, including the
   enum's payload types. This rejects overly weak output relations even when ordinary contract
   verification accepts them.

These checks rely on the existing Verus/rustc arithmetic and array models and Z3, plus the
small precision-check transformation as part of the verifier. The supported
subset excludes general effects and source-level assumptions. Numeric and collection
loops allow only the recognized patterns and checked proof decoration described above.
Owned collection results use complete logical contents/field observations in the
arbitrary-output query, independently derived from the output type. Calls use inferred
summaries or the explicitly approved standard models, including the array helpers
and fixed-array length lowering. Those methods rely on trusted vstd specifications, just as ordinary Verus
verification does. The integer-operation batch additionally introduces 36 trusted
primitive-method declarations, explicitly audited in `VERUS_INFER_ASSURANCE.md`;
these specifications are assumptions about Rust library behavior. The driver names these dependencies in its diagnostics and
preserves their fingerprints through caches and crate exports.
No inferred executable body is externalized or assumed, and no array axioms are
added. The two existing external-body array helpers declare `no_unwind`; those
and the numeric specifications above are explicit trusted-specification additions.
The precision transformations and proof-publication machinery are also part of
the added trust boundary and have not themselves been formally verified.
The driver validates inferred functions in separate buckets before starting any
dependent inferred bucket or ordinary caller bucket, including when ordinary
callers are verified in parallel. Exact-mode caller precision therefore depends
on exact-mode callee guarantees; sound-mode callers never assume unproved
maximality from their dependencies.
The ordinary and precision bodies must match their query kinds. Only an original
ordinary query can record soundness, and only a matching precision query can
record its precision guarantee. Diagnostic/profile reruns do not supply evidence.
The acceptance component requires every required subquery to succeed; missing,
failed, skipped, and timed-out results cannot be repaired by later success.
Ordinary parallel workers cannot replace previously recorded inference outcomes.

Inference describes implementation behavior. Application intent, such as value
conservation, still belongs in explicit caller and system specifications.

The repository-root `VERUS_INFER_ASSURANCE.md` tracks current safeguards,
near-term consolidation, and longer-term formal verification targets.

## Regression tests

From `source/`, after building the compiler and builtin libraries:

```sh
cargo test --release -p vir infer
cargo test --release -p rust_verify_test --test infer
```

The tests exhaustively compare `u8` arithmetic, branching, and short-circuiting
summaries against concrete Rust execution. Backend fault-injection tests submit
incorrect candidates directly to the production precision checker and Z3:
overpermissive and restrictive domains, incorrect outputs, and sound but
imprecise output relations. Integration tests cover caller composition, numeric
boundaries, unsupported constructs, bounded formulas, and solver-limit
publication failures. Cross-crate tests cover a three-library chain, stale
dependency fingerprints, missing/incompatible/incomplete proof metadata, private
helper pruning, and failed/partial-verification export suppression. The Cargo
regression `cargo_inferred_contracts_across_three_crates` builds and verifies the
example workspace through `cargo verus`. Partial-inference tests additionally
check exhaustive interval containment for safe `u8` executions, useful range
proofs, bounded safety synthesis after exact formula-budget exhaustion,
independent precision timeouts, compositional domains, fresh cache proofs, and
per-function imported precision metadata.
Exhaustive testing supplements the solver proofs.
Structured-report regressions additionally check fresh proofs on cache hits,
honest optional timeouts and skips, failed caller export suppression, imported
candidate/model identities, and rejection of a corrupted cached precondition
with removal of the previous export.
Slice regressions compare generated domains and outputs with concrete Rust
reads/checked arithmetic over small vectors and every `u8`/`i8` element value,
including empty and 300-element inputs. They also cover caller bounds and
overflow rejection, symbolic lengths, aliases and guards, typed cache corruption,
restrictive domains failing maximality, three-crate composition, stale model
artifacts, and failed-proof export removal.
Enum regressions additionally compare checked `u8` arithmetic against concrete
Rust for every input pair, check supported widths and match-arm safety, compose
enum interfaces through cached cross-crate builds, reject stale model
fingerprints, and verify that poisoned cached variants or payloads fail checking.
Hardening tests cover truncated, oversized, and malformed artifacts, duplicate or
missing proof metadata, and a cached restrictive precondition that passes
soundness but fails maximality. That exact-mode failure blocks both inferred and
ordinary dependents and invalidates the previous export, including with parallel
verification enabled.
Array regressions exhaustively compare bounds and overflow predicates for
two-element `u8` arrays against concrete Rust, check empty and 64-element arrays,
cover conditional and short-circuit reads and eager literal evaluation, exercise
array summaries and literal formulas through cached cross-crate builds, and
ensure a cached wrong index fails independent verification.
Read-only array tests additionally cover shared-reference forwarding, cached
cross-crate composition, repetitions of length 0 and 64, zero-length operand
overflow, model lookalikes, and changes to trusted-model artifacts. The latter
must both invalidate candidate keys and block old inference certificates.

Assurance tests also cover body preservation, query separation, candidate/evidence
identity, exact/sound acceptance combinations, receipt ordering, duplicate and
mismatched records, and worker isolation. From the repository root, run:

```sh
python3 source/tools/test_infer_assurance_mutations.py
```

The source mutation runner uses temporary copies, cached Cargo dependencies, the
pinned Rust toolchain, and the production precision checker with Z3. Its
`--policy-only` option tests the dependency-free acceptance component without
Cargo. Each baseline must pass; selected weakened implementations must run tests
and fail them. Mutation coverage is testing evidence, not a formal proof.

Early-return regressions exhaustively compare five control-flow/operand-order
cases over every pair of `u8` inputs with Rust checked operations. Native tests
cover nested conditions, both short-circuit operators, enum matches, unit
payloads, guarded borrowed-array reads, unreachable operations, and retained
pre-exit overflow checks. A cached three-crate transfer chain verifies
conservation; a poisoned precondition must fail maximality at early exits and
suppress old exports.
