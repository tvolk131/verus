# Cycle-two pre-edit logical-value justification

The cycle-two decision is read in full. Targets, domains, source hashes and
authored proof allowances remain unchanged. Cycle one is frozen at
`/private/tmp/verus-workflow90-cycle1.o06Kvk`; cycle two uses fresh output
`/private/tmp/verus-workflow90-cycle2.DJ31Jt` and separate counters.

## Existing generic semantics, not a new correspondence

`source/vir/src/ast.rs` defines Datatype as concrete **or abstract**, Projection
and TypParam as inherently SMT-boxed, and Opaque as an existing type constructor.
`sst_to_air::typ_to_air` already maps transparent datatypes to their existing
sort, abstract monomorphic datatypes to their existing opaque sort, and Boxed,
TypParam, Projection and Opaque to Poly. No visible fields are necessary to bind
another variable of one of these same first-order logical sorts.

`ast_to_sst::init_var` introduces a typed VarLoc destination of exactly the RHS
type. `sst_to_air` lowers this initialization to `assume_var`, the ordinary
equality between the new variable and the existing RHS value. Thus a snapshot
of a live free root is an ordinary typed logical value binding, not a Rust copy,
a heap clone, a new axiom about an opaque type, or a claim that later reachable
storage remains unchanged. Reconstructed invariants and point instances still
undergo ordinary backend checking before identical expressions are assumed.
Any relationship across mutation must follow from those checked expressions
and existing pure logical functions; it is not inferred from field visibility.

The bounded generic classifier may accept existing abstract datatype definitions
with no visible variants, type parameters, associated projections and opaque
constructors, checking their explicit type arguments. Known visible fields
remain recursively checked. No application, standard type name, path selector
or new library/model correspondence is needed. This directly covers the actual
lowered iterator wrapper's opaque argument and associated-item history value.
Explicit raw pointer, reference decoration, MutRef token, function/effect and
internal-only sorts remain rejected. A MutRefCurrent expression is supported
only at an exactly typed free root, as already authorized, never future/final
state. An abstract logical value does not authorize projecting a mutable token
or hoisting any call, index, dereference or guarded expression to the header.

## Definite assignment and scope

`sst_to_air::loc_to_field_update_data` removes Loc wrappers before recognizing
VarLoc. Its non-init assignment case performs a whole-root assignment when the
field-update list is empty. Therefore typed Loc(VarLoc) with is_init=false is a
dominating assignment, not an uninitialized read. Only exact whole-variable
locations with matching declaration/RHS types qualify; field/index/deref writes
do not initialize a root. SST locals have globally unique identities and Blocks
contain no binder declarations. Straight-line block assignments therefore remain
live after block exit. If joins intersect branch live sets; dead-end/proof/effect
scopes and loop bodies do not export new live roots. This changes no location,
loop, reference or backend semantics.

## Focused tests planned before implementation

- Actual typed Loc(VarLoc), is_init=false ghost-assignment form, nested straight
  blocks, both-branch versus one-branch assignment, and type mismatches.
- No initialization from field/index/deref writes, proof dead ends, unsupported
  effects or a possibly unexecuted loop body.
- Non-application-named abstract datatype, associated projection, generic and
  opaque first-order sorts; exactly typed snapshot initialization.
- Raw pointers, explicit references, future state and location expressions
  decline. Existing guards/triggers/let ordering, provenance, checked pair
  identity, lexical masking, condition policy and transactional bounds remain.

This is a typing argument for an implementation attempt, not theorem acceptance
or an inferred constructor/export claim. If existing IR cannot express it safely,
stop rather than add a model, correspondence or semantic encoding change.
