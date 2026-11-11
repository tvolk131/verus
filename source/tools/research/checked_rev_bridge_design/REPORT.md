# Checked Rev construction: observation binding is the first missing premise

**INFEASIBLE under unchanged semantics/trust.** The actual core constructor is
small enough for a structural checker. Its MIR does not define the existing
uninterpreted `rev_iter` observation. Current opaque datatype lowering supplies
neither the necessary field representation nor that observation's coherence.
There is no justified complete checker interface to implement at this stage.
No compiler/std/vstd/assumption was changed. Parent must separately scope any
semantic-definition extension. `goal_complete=false`, native correspondences=0.

Pretrial scope/source freeze commit: c356ffe0, on the supplied isolated checkout
at1171bdf7. This investigation did not create a worktree or read another checkout.
`FREEZE.json` binds the required input records. Final evidence is in
`RESULTS.json`, `IDENTITIES.json` and `INVENTORY.json`; raw sources/MIR/logs and
binaries stay private at the exact paths recorded there. `audit.py` performs
read-only, exception-gated acceptance, identically under Python and Python -O.

## The small sequents, before considering any compiler extension

Write S = slice::Iter<'a, Item>, E = Enumerate<S>, R = Rev<E>,
L(x) = x.obeys_prophetic_iter_laws(), D(x) = x.decrease(),
M(x) = x.remaining(), N(x) = x.will_return_none(), G(r) = rev_iter(r).
For the actual expression chain, existing contracts give:

```
s = borrows.iter(); e = s.enumerate(); r = e.rev()
L(s), native_count_supported(s), D(s)=Some(d), d<=usize::MAX
inner_enum(e)=s, index_enum(e)=0, L(e), rev_post(e,r)
M(r)=reverse(M(e)), N(r)=N(e), isSome(D(r))=isSome(D(e))
L(r) := L(G(r))
```

The first desired sequent is `Gamma |- L(G(r))`; the stronger sufficient one
is `Gamma |- G(r)=e`. Neither follows merely by transporting typed sequence
observations and decrease existence. Even opening *all* Rev definitions adds
only `M(G(r))=M(e)`, `N(G(r))=N(e)` and metric existence agreement (using reverse
involution). This is not equality of iterator states or Enumerate lawfulness.
Slice lawfulness is true, but Enumerate lawfulness additionally needs count
support, a finite hidden metric, and `index+metric<=usize::MAX`.

`observations.smt2` checks a deliberately qualified finite typed observational
fragment. Set W=2^64-1, let s have empty remaining, None-exhaustion=true and
hidden decrease Some(1), count support=true. Let e and j both observe s as their
inner slice, but have indices0 and W. Both indexed remaining sequences are empty.
Then L(e)=true, L(j)=false because W+1>W. Interpret G(r)=j. All three Rev post
observations hold, including the **stronger** equality D(r)=D(e)=Some(1).
Slice metric/sequence bounds and the enumerate constructor instance also hold.
Empty borrows are expressly in the source domain; no current slice constructor
contract identifies the hidden metric with length. No native empty iterator
with count W is claimed. This is a model of these observations, not of runtime
construction.

The authenticated Z3 answers SAT for that fragment plus !L(r), and UNSAT after
adding the missing G(r)=e. This covers the relevant ground constructor/post and
observation definitions, including typed indexed sequences and slice bounds.
It is **not** a model of every vstd axiom, every generic instance, state transition,
reference/prophecy encoding, count/fold contract or full crate. No universal
unprovability conclusion is drawn. Production-wide symbol search finds Rev's
current observation/post definitions and uses only in std_specs/iter.rs; no
compiler field binding for this uninterpreted function was found. That source
inspection, rather than a failed SMT search, identifies the missing semantics.

## Actual compiler evidence (diagnostics, not correspondence certificates)

Exact allowed parent receipt SHA256:
5e7e79cf6b6cef0509226cf6134c8fce0a8c47ed608fc4c5a7520b5023aa4c45.
All eight listed files were authenticated, copied privately and rehashed before
and after trials. No shared bundle was mutated. Executing engine is d3b2aa6e,
schema39/policy74/cache77. Own tracked source outside research/docs equals its
Git engine source. Matching inherited vstd2250/0 is parent evidence, not rerun here.
No historical VIR or different diagnostic compiler was used as positive input.

Pinned rustc1.99.1 reports Rust commit58cf8f9c35b355199727627d07ae34a4e2846d67,
x86_64-unknown-linux-gnu, LLVM23.1.1. `RESULTS.json` pins rustc, rustc_driver,
core/std/alloc/compiler_builtins archives, rustup, and the installed iterator,
rev and enumerate sources. Core stable crate ID17281971461272426270 and semantic
DefPathHashes are recorded separately from the misleading `std::...` display
aliases. Raw loaded generic and instantiated MIR, full signatures, ParamEnvs,
source locations, field privacy/types/layout and source wrapper bodies are hashed.
Private diagnostic build uses rustc_private, no Cargo dependencies or network.
It stops after analysis; a separate ordinary Rust compile of the witness succeeds.

With I=Enumerate<slice::Iter<'a,u64>>, loaded Runtime(Optimized) bodies are:

```
Iterator::rev(I)->Rev<I>: bb0: _0 = Rev::<I> { iter: copy _1 }; return
Rev::new(I)->Rev<I>:     bb0: _0 = Rev::<I> { iter: copy _1 }; return
Rev::into_inner(Rev<I>)->I: bb0: _0 = move (_1.0: I); return
```

Each has two locals, one argument, one non-cleanup block, one assignment and
Return; no call, closure, branch, panic/assert, allocation, mutation through a
reference, Drop or unwind edge. `rev` has already inlined `new` in loaded core;
new is therefore resolved independently through the actual Rev inherent owner,
not invented as an unobserved dependency. The full generic bodies are also dumped.
Rev has one private field0 `iter:I`, no destructor; this witness I and Rev<I>
have needs_drop=false, size24/align8. Enumerate has private iter:S and count:usize.
`into_inner` is unstable and used only as a diagnostic; it is not added to the
application or assumed modeled. Safe Rust ABI/by-value signatures are recorded.

This u64 witness preserves arbitrary borrowed Vec and literal
`borrows.iter().enumerate().rev()`. It is not the full Miri crate or its Item
DefId. Replaying a concrete Item instance at the real call site remains a
mandatory future identity check, not a conclusion of these probes. Generic
core shape inspection is evidence of a bounded candidate route, not a theorem
for all I, all lifetimes, target ABIs or future compiler bodies.

## Why the accepted helper machinery cannot close it today

`checked_nonzero_ord.rs` resolves concrete semantic owners/instances, validates
MIR, binds inherited get/primitive models, constructs an ordinary Function, and
independently reads that *actual* Function against MIR. `register` checks before
insertion as BodyBackedForeign; `query.rs` rechecks after per-function WF,
autospec resolution and mode checking. `finish_item` retains the full body;
`context.rs` binds original caller/HIR/instance; `fn_call_to_vir.rs` redirects
only that call and uses normally translated source arguments. Existing helpers
are local-only and gate complete checking/export. These are useful mechanics,
not an observation-definition mechanism.

Rev's external type proxy has external_body. `rust_to_vir_adts.rs` produces
DatatypeTransparency::Never with no fields. `datatype_to_air.rs` does not emit
a transparent AIR datatype for it. Removing external_body is insufficient:
transparent external specifications reject private fields. Constructor item
registration explicitly skips proxy/opaque datatypes. Native field0 is therefore
not a current VIR selector, and `rev_iter` is a separate uninterpreted spec
symbol without a body. No naming convention binds the two.

Even if a native-field getter H were checked, the next sequent would be:

```
checked_construct(e,r), checked_getter(r,h), h=e
    |- rev_iter(r)=h                   [MISSING observation coherence]
```

The native getter body alone proves no equation about an unrelated spec symbol.
Generating an `assert` or a helper ensure with that equation simply recreates the
unproved obligation. Adding it to a trusted helper, calling it body-backed,
identifying an unrelated transparent wrapper with Rev, or trusting L(r) would
cross the boundary.

## Smallest separately scoped semantic obligation and refusal gates

A possible frontend extension would need an internal nominal representation
for the **actual** core Rev<I>, preserving public opacity, with constructor C and
selector F, plus a compiler-owned *definition* of this exact existing observation
as F. Its obligations are distinct:

```
alpha_Rev(native Rev { iter:x }) = C(alpha_I(x))    [representation/lowering]
F(C(v)) = v                                      [datatype computation]
rev_iter(z) is definitionally F(z)                [NEW semantic binding]
```

The third line is absent today. Merely adding its equality as a fact is an
assumption strengthening. A genuine definition requires a separately reviewed
compiler TCB extension: exact spec-symbol/ADT/field identity, coherent equality,
boxing and polymorphic type arguments, privacy, all reachable Rev values and
state updates, and compatibility with inherited next/remaining/decrease models
and prophecy semantics. It must not attach a field interpretation only to a
convenient constructor result while other uses retain an incompatible opaque
interpretation. This is the smallest additional semantic-definition obligation
identified here; no broad Rust/Iris foundation project is needed to state it.
It has not been implemented, proved or accepted. There is consequently no
unconditional bridge-checker API offered here.

If parent scopes that extension, the following are rejection conditions for a
future local structural checker, not an authorization or implementation plan:

* Bind the original caller/HIR receiver, adjusted source argument, semantic core
  trait method/default implementation, Rev ADT/private field/new, and concrete
  I=Enumerate<Iter<'a,actual Item>>; no spelling-only dispatch or shadow models.
  Re-resolve all generic/trait/associated Item/ExactSize/DoubleEnded paths, host
  effects, binder kinds, lifetimes, safe Rust ABI and target. Reject unresolved
  type/const parameters, indirect calls, custom rev overrides or coercions.
* Keep the original receiver evaluation and move exactly once. Do not recompute
  borrows.iter(), clone e, erase overloaded deref or evaluate a callback.
  Rust ownership/lifetime checking must retain the shared borrow from Vec and
  its source argument through the return. Region-erased MIR may be compared
  only after source checking; it does not prove lifetime/provenance equivalence.
  MIR `copy` here is not authority to assert a source Copy trait or duplicate
  ownership. Reject unsupported drop/cleanup/storage/retag/effect changes.
* Validate every generic and instantiated body statement, operand/projection,
  field identity/order/type, locals, return and source-instance metadata. Resolve
  loaded already-inlined rev and actual new separately; only aggregate/return is
  in this bounded shape. No native safety premise, dereference, panic path or
  unwind assumption may hide in a summary. This constructor's absence of unwind
  says nothing about iterator execution, drop, or the whole fallback's safety.
* Independently parse the full proposed Function before insertion and after
  normal WF/autospec/modes. Validate signature and all types/decorations, impl
  paths, parameter/return modes, every body node/ID/read/move, exact binding and
  contracts, attributes, requires/ensures/decreases/unwind/mask/extra dependencies,
  visibility and source routing. Reject extra assumes/proofs/lawfulness premises
  and changed semantic definitions. Never validate only a synthesized twin/hash.
  Retain normal Rust lifetime/trait/erasure/whole-crate checks and every required
  ordinary proof. Bind compiler/tools/core/target/model/dependency identities;
  source-preserving transformation and before/after checking are both required.
  Local-only/no-cache/no-import publication should be the initial scope.

A standard strengthening `rev_post(e,r) ==> rev_iter(r)=e` or L(r) would bypass
this work; it is **OUTSIDE this task**, not an accepted alternative.

## Ordinary results and limits

Both plain and infer-goals runs of the same sources, --no-cheating --compile,
full crate, no filters, disabled checks or timeouts:

| Probe | Off | On |
|---|---:|---:|
| current lawfulness of e and Rev remaining/none/decrease observations |1/0|1/0|
| L(r) |0/1|0/1|
| returned inner identity |0/1|0/1|
| literal reverse-enumerate empty-body loop |1/1|1/1|

The failed loop reports generated invariant/decreases failures. These are support
failures, not strict semantic controls. All runs request/generate zero inference
helpers; success certifies one executable witness body and zero proof bodies.
Each failed whole crate certifies zero exec/proof bodies; selected source bodies
remain inventoried separately. No causal assistance gain. Failed runs do not
reach successful erased compilation. Separate Rust witness compilation passes.
No new manual Verus theorem/proof body, axioms or assumptions; ordinary assertions
and the SMT encoding are authored research probes, not native certificates.

ATTEMPTS.json preserves setup/diagnostic API errors and the keyword parse error;
none counts as semantic failure. The initial sandbox launcher failed before
execution; authorized local commands used automatic escalation. No rejection,
permission question, network, subagent, shared mutable cache, push or publication.
The last epoch alone is the final immutable evidence. Validation authenticates
its complete input and product manifest. Runtime OS/dynamic libraries and the
existing compiler/vstd model machinery remain inherited trust, not newly proved.

Exact extracted Miri fallback remains UNPROVED, cache-hit prelude excluded;
actual slice::swap remains unsupported in the full cache helper. Dependency31/0,
strict negatives30/1, parent wrapper28/0 and 879/Python148/2250/0 remain inherited
bounded results. No loop replacement, ordering export/import or source/native
progress is claimed. Full HMAC1/1 and44+3/101 remain OPEN; publication=false,
goal_complete=false throughout. Parent owns integration and further scoping.
