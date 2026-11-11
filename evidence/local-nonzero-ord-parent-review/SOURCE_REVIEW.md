# Independent source review at 7fde950c

Reviewed the complete 1277-line module and 290-line structural unit, both required
reports, all eight integration hooks, and the ordinary read, call, query, trait,
mode, pruning, erasure and publication paths referenced below. No implementation
edits. Evidence hashes establish identity, not operational correspondence.

## Operational correspondence

`checked_nonzero_ord.rs:42–169` resolves the diagnostic core Ord and NonZero
identities through the core lang-item crate, resolves Item instances, checks
concrete u64 substitutions, associated trait identity, inherent getter ownership,
normalized safe Rust ABI signatures, and obtains Ordering from Ord::cmp's actual
signature. It does not accidentally use the atomic Ordering diagnostic item.
The enum must be the core three-constructor no-field type with the expected
names. This is a semantic identity route, independent of application spelling.

`properties`, `operations`, `inspect_bodies` (`:192–402`) accept a closed complete
alphabet and whole bodies: nine exact carrier local types/two blocks; two getter
locals/one block; five primitive locals/one block. Every block must have a
terminator and be non-cleanup. Exact ordered statement vectors account for all
14 entry statements, four exit storage deaths, return, projections, Copy/Move,
borrow kind and region, literal zero-sized FnDef target, two arguments, normal
successor and UnwindAction::Continue. Counts reject extra unreachable blocks or
effects; no selected-block traversal can hide a statement or edge. Local types
must be exact, concrete, Copy and no-drop in the instantiated environment.
Mentioned/required targets/constants and supported body metadata are checked.
Source spans/debug/scopes and generic-origin flags do not supply validity.

`check_get_expansion` (`:404`) separately maps getter formal1/result0 to carrier
(5,4)/(8,7) at statements4/11, bounds-checks maps, checks injectivity and exact
types, and matches the one uninterpreted Transmute operation under substitution.
It never asserts transmute(n)=n@. The complete accepted trace establishes live,
initialized locals, distinct fresh storage and the two original dereferenced
Copy reads. No store-through-reference, address observation, drop, hidden call,
branch or cleanup is accepted. Storage markers omitted in VIR concern only fresh
Copy/no-drop temporaries whose addresses do not escape. Primitive shared borrows
stay live through the original primitive call. Inputs may share an address.

This remains bounded compiler-TCB reasoning over the accepted exact trace; it is
not a general CFG equivalence checker, kernel theorem, or native correspondence.
The primitive Cmp MIR is inspected as an inherited model dependency. No primitive
Cmp encoding or enum discriminant arithmetic proof was added.

## Actual full Function and ordinary read semantics

`check_ordering_function` (`:815–1051`) re-inspects original MIR and expansions,
then parses the supplied actual Function. It does not rebuild a Function and
compare it to itself or trust a descriptor hash. All FunctionX fields in
`vir/src/ast.rs:1705` are covered: identity/module, visibility/body visibility,
opaqueness, proxy/kind/mode, type bounds/parameters, both returns, empty requires,
one complete ensures and no inherited ensures, returns/decreases/fndef axioms,
masks/atomic/unwind, item kind, every attribute, opaque equalities, ordered extra
dependencies, hidden and body. Diagnostic span fields are intentionally excluded.

F2: `types_equal` deliberately ignores Datatype ImplPaths (`ast_util.rs:153`),
so types are not field-complete dependency comparisons. A separate private
cfg(test)-only actual-loaded Function clone with the first parameter type
impl paths cleared is accepted by this validator. This confirms the gap without
a semantic false-success claim; see REPORT.

Parameters and all six declarations map injectively to all nine MIR locals;
return and parameter/local binders must be distinct. The six initializer nodes,
read place/name/kind, reference/output type structure (F2 exception), unique body read IDs, exact
call target/type and impl arguments, post/body absence, operand order, result,
pattern/mode/mutability/copy fields and tail type are checked. Whole block counts
reject extra statements. No mutation tests are credited as source proofs.
The exact numeric contract separately checks result binding, both spec_get
arguments, typed unsigned comparisons and exhaustive Less/Greater/Equal cases.
A true-only property cannot stand in for this helper contract.

I independently examined the initially suspicious reference types: normal
`rust_to_vir_expr.rs:4635` makes immutable dereference implicit, returning the
same reference-decorated place; `consume` and `immut_bor` (`:195–238`) govern
ordinary read kinds. New helper Copy reads consume the same genuine NonZero,
not a representation-forged value. The ordinary get/get/u64cmp mirror confirms
identical three typed model targets after whitespace tokenization, including
trait declaration, resolved implementation, Self/type arguments, impl dependency
paths and call attributes. It proves only its ordinary source property (1/0),
not correspondence for the original std caller. Its declarations/implicit
reborrows and the helper's exact original-MIR Move reads differ in ordinary,
accounted ways; byte equality of whole bodies is neither required nor claimed.

Registration validates the full Function before BodyBackedForeign insertion.
`query.rs:484` repeats validation on the actual Function after trait demotion,
ordinary WF, autospec and mode processing; later ordinary resolution/simplifying
passes remain in the inherited pipeline. Existing synthetic unit controls clone
actual compiler-loaded MIR and actual translated Function, reject all tested
mutations, and accept concrete generic-origin=true MIR. No core/toolchain patch.

## Semantic models and source binding

`bind_models` (`:457`) follows actual get/primitive DefId-to-VIR paths and vstd
proxy metadata, get's autospec to spec_get and View, and core Ord's external
trait-extension metadata to the actual u64 OrdSpec implementation. It requires
obeys_cmp_spec's body to be true; it does not invent a NonZeroOrdSpecImpl.
Numeric cmp_spec semantics are inherited from unchanged `vstd/std_specs/num.rs`
(the unsigned macro instantiated for u64), and checked by the helper's ordinary
numeric postcondition. Existing get/spec_get/View, primitive trait contracts,
impl dependencies and relevant model definitions survive into the proof closure.
Extra dependencies record View/obeys/cmp; ordinary calls, spec bodies, traits and
impl paths provide normal reachability. These are existing imported trusted
models, not new local laws or independent standard-body proofs. Matching whole
vstd2250 and compiler/core hashes are inventoried separately.

`actual_call`, `source_arguments`, `register` and `direct_helper` (`:1065–1246`)
bind HIR caller, exact expression identity/pointer and typed source arguments,
permit only builtin shared adjustments and re-resolve the concrete Instance at
redirect. The hook in `fn_call_to_vir.rs:378` changes only target selection after
ordinary argument translation; it preserves original argument expressions and
effects. FnDef values, coerced calls, generic/lookalike/overloaded deref calls do
not gain this helper. Multiple/generic callers bind the same closed helper only
after per-caller typed revalidation; no caller type variable is in its signature.
The independent mixed-context and argument-effect controls exercise this.

## Pipeline and local-only boundary

BodyBackedForeign goes through the existing Query cache/full Function checks,
not external_body. `verifier.rs:1864` records only actual successful Normal body
queries with valid outcome and no invalid/timeout/profile rerun. `:3717` requires
every local helper identity in that set. Existence of an AIR log is not the gate.
Ordinary helper, application, lifetime, trait, WF/mode, termination, auxiliary,
inference, deferred error and erased Rust gates remain. The Ord auxiliary-only
failure without selected outputs independently exercises global failure while
normal helper proof is healthy. New Ord export refusal happens before proofs and
cannot itself supply auxiliary semantic evidence.

The new barrier `verifier.rs:2913` is before private pruning/certificate creation.
Single-route selected outputs are removed for healthy/false/mixed/private crates,
valid partial filters and disabled check flags. Existing derives ignored by the
profile are marked external in `external.rs:818`, do not enter the scanner, and
do not spuriously enable the path in the full proofless Item compatibility run.
Macro/profile eligibility and genuine wrapper Ord/PartialOrd stay unproved.

One blocking exception was found: `import_export.rs:291,361` invalidates only
`args.export.or_else(via_cargo)`. When both destinations exist, Cargo's old valid
VIR survives the new local-only refusal. This inherited utility violates the
explicit all-selected-output requirement when reused by the new barrier. See
REPORT and the dual-path receipts. No new ordering interface was accepted.

Prepared root epochs are schema39/policy73/cache76. Worker reports retain their
historical schema39/policy72/cache75 description. Production integration relative
to a3b1e280 changes only policy/cache epoch constants, with documentation updates;
checked_foreign.rs and std/vstd/model source remain unchanged. Existing Bool kind
and schema work on the same fresh runtime. Old-policy metadata refuses. Candidate
cache identity does not substitute for actual required body proofs.
