# First missing layer: source context initialization, before model access

This is an inactive obligation specification. The mathematical notation below
names work to prove; it introduces no admitted relation, new validity definition,
new application requirement, or fact consumable by Verus.

## Exact entry to interpret

The unchanged selected `drop.rs` has one argument `_1: &mut [u8;128]` and returns
unit. In the authenticated pre-erasure body:

1. `bb0[3]`: `_4 = &mut (*_1)`, two-phase reborrow.
2. `bb0[4]`: `_3 = move _4 as &mut [u8]`, unsize/activation.
3. `bb0.terminator`: actual `slice::iter_mut(move _3)` into `_2`, normal target
   bb1, unwind cleanup bb2.
4. bb1 contains `FakeRead(ForLet(None), _2)`, unit assignment (`retag: Yes`),
   storage end and return. bb2 is cleanup/unwind_resume.

The sole extracted loan `0`/region `3`, issued at fact `[3,0,7]`, is created by
step 1 and assigned to `_4`. **It is not the original incoming runtime loan.**
The extraction has no runtime pointer, memory contents, dynamic tag, protector,
ancestor-loan structure, or caller semantic-context proof for `_1`. Those values
are correctly not fixed by a static extraction. Treating loan 0 as the incoming
dynamic loan would change its identity and issuance point. Its `borrowed` place
`(*_1)` is the source connection that still needs semantic interpretation.

The six-instance/44-operation optimized extraction binds the same constructor
operand chain. It includes three queried empty drop-glue bodies, not three extra
runtime destructor invocations. Its identity and normal borrowck completion do
not turn pre-erasure execution into optimized execution by a theorem.

## Required source premises and their present status

For every *actual* valid source entry, an eventual formal source judgment must
supply the following information. These are obligations of source typing and
its soundness, not conditions to append to `xor_key`.

| Information | Why this domain needs it | Evidence available / missing |
| --- | --- | --- |
| Actual incoming argument and type | Bind the theorem to `_1` of this body | Typed local and source pin present; no checked source typing derivation into the model |
| Live containing allocation, byte address and extent | A nonempty 128-byte referent must be addressable | Source-validity consequence to prove in an explicit source semantics; not a borrowck fact |
| Correct subobject range and alignment | Arbitrary interior [128] within larger allocation; u8 alignment 1 | Layout size/alignment present; runtime containment/liveness interpretation missing |
| All 128 initialized byte values | Every vector in 0..255, no zero initialization | Model byte type checked; source bytes to model cells interpretation missing |
| Provenance, incoming loan, suspended ancestors and any entry protector | Preserve actual exclusive access without destroying caller rights | Local static loan 0 present only for `_4`; incoming dynamic context absent from extracted evidence |
| Source lifetime/context interpretation | Obtain ordinary lifetime guards and respect blocked parents | Model E/L/context rules checked; source region/loan-context translation missing |
| Same physical pointer and byte layout in the model | Reach the payload at exactly `l.1`, with the original subobject offset | Model accessor proves equality after model ownership is supplied; physical source mapping missing |
| Original current and future observations | Retain source C/F relation rather than choose an easy prophecy | Existing AIR reborrow equations present; source/model C/F correspondence missing |
| Whole-context consistency and linear partition | One original loan; no overlapping new owner or extra freeable authority | Model separation/blocked-context rules checked; initialization of that partition from source context missing |

The original Rust reference may itself be a reborrow. Its ancestors can remain
suspended in a caller; they are not separate active owners to discard. All source
histories that legally yield this input must be covered. A theorem only for a
fresh model allocation and freshly selected loan is not universal entry adequacy.

## Irreducible sequent

Let the left side be an actual source execution configuration at this entry,
with an independently defined Rust operational state, source typing derivation
and the original valid exclusive argument. An adequate source interpretation
must produce, in one coherent Iris initialization:

```
source configuration + original typing derivation + original runtime argument
        (NO supplied model ownership/heap/token premise)
   -- checked source-state AND source-context initialization -->
existing model state_interp of a faithful physical memory representation
  * existing lifetime/prophecy/uniqueness/time/invariant interpretations
  * interpretation of the whole original source typing context,
    whose _1 component is
      exists m d. time_receipt d *
        ty_own (&uniq{kappa} byte_array128_ty) m d d tid [FVal #p]
  * proved equalities for actual pointer/provenance/subobject/current bytes
  * proved source-current/future to model-current/prophecy correspondence
```

A source memory representation can erase irrelevant details, but its simulation
and context interpretation must account for the erased provenance/loan rights.
Putting `ty_own` inside a newly named validity predicate on the left side only
assumes the missing implication. Static type membership alone is not the right
left side; the valid runtime configuration and context soundness are required.
The claim here is missing derivation in the inspected inputs, not impossibility
of defining and proving such a source semantics.

Once this sequent is proved, the existing v12 context accessor is usable. The
result must then be carried through the **actual** `_4` reborrow, `_3` unsize,
and native body. Reusing a model rule requires proof of the corresponding source
step, not merely selecting a model expression with the desired type.

## What the existing initialization rules actually provide

The checked inspection records the exact instantiated constant statements.
All source paths below are under the immutable baseline `src/lambda_verus/`.

- `lang/adequacy.v`, `lrust_adequacy`: concludes adequacy starting at `(empty,
  false)`. Its caller proves a model WP from later credits/time context. Its
  proof initializes an empty non-atomic map and empty freeability authority.
  It does not quantify an incoming Rust argument or nonempty source state.
- `typing/soundness.v`, `type_soundness_closed`: requires a closed model
  `typed_val main main_type c`, starts from the empty heap, and proves nonracing
  and progress. `main_type` has no user argument. No source-MIR translation,
  incoming context, or byte-preservation postcondition appears.
- `util/non_atomic_cell_map.v`, `non_atomic_map_alloc_empty`: initializes the
  empty physical map. `non_atomic_map_insert` can extend a fresh location with
  a value and yields its fragment; it requires the location absent and updates
  the interpreted map. It is not a rule for extracting a permission from an
  already represented, potentially borrowed, live source subobject.
- The apparent `non_atomic_map_alloc_heap` at source line 1337 is inside an
  unfinished comment. `Fail Check non_atomic_map_alloc_heap` is compiled as an
  availability control. This is NOT a semantic failure or a permission proof.
  A new total initializer can plausibly be built from existing algebra; its
  absence is **not** claimed to be a fundamental obstacle or a missing package.
- `llft_alloc`, `proph_init`, `uniq_init` establish fresh logical infrastructure.
  Their results do not identify the caller's source loan, physical input bytes,
  or existing C/F observations. Ghost names may legitimately be fresh, but their
  correspondence to the actual source context still has to be proved.
- `typing/uniq_util.v`, `alloc_uniq_body`: creates a model unique borrow from
  **supplied** later type ghost contents, the actual model `l #mapsto! phys`
  payload and time ticket, plus the global contexts. It does not derive that
  payload from an original Rust reference. This rule itself needs no freeable
  capability; supplying its payload as an application premise would still miss
  the task.
- `typing/borrow.v`, `type_uniqbor_instr`: starts from `p : own_ptr n ty`; its
  proof splits payload and freeable resource, borrows the payload, and retains
  a blocked owner for release. Imposing this input on the borrowed subobject
  would add unjustified deallocation authority. Whole-program context soundness
  could legitimately interpret a containing owner, but that derivation is absent.
- `tctx_reborrow_uniq`: starts from `p : &uniq{kappa} ty` and retains the parent
  as a blocked reference. This is the promising rule for `_4` **after** `_1` is
  interpreted. Its model `ReBor` is two Skips; the Rust reservation/activation,
  entry retag/protection and phase correspondence remain to be proved.
- `tctx_elt_interp`, `typed_instr`, and the v12 context accessor explicitly take
  model semantic ownership as a resource. A `typed_instr` proof does not create
  its initial type-context interpretation. Replacing input interpretation with
  that antecedent is circular for this task.

The ordinary model `state_interp` is `heap_ctx sigma * time_interp stepcount`.
The physical `state` is a cell/lock map plus a global-lock Boolean; locations
are `(positive block, Z offset)`. Runtime reference tags, incoming loans and
Rust validity/typing derivations are not fields of this state. They would need
a faithful source relation and ghost/context invariant. More pure layout facts
or initializing a map of the right length does not establish that invariant.

## Why the proposed nonempty-heap route is insufficient by itself

A generalized adequacy proof can initialize a nonempty model heap from a chosen
finite cell map, allocate its fragments once, and place them into fresh model
borrows. This would discharge a useful *model* initialization exercise. It would
not show that the chosen map, borrow partition, context and prophecies interpret
**every actual source state** at `_1`, nor that they simulate its execution.
Likewise a model `main` allocating and borrowing 128 arbitrary bytes is a different
program until a checked source simulation relates it to this entry and caller.
No such replacement harness or model-only entry lemma is delivered here.

An initial-state theorem alone is also insufficient for native-drop: the native
execution must preserve the same physical array and the post-execution relation
must bind model observations back to the original source reference. The general
Iris adequacy machinery can support stronger initialization and observations;
exported model safety is not that source result. There is no reason to install
another toolchain merely to state the absent theorem.

## Exit and phase obligations remain separate

The inherited source sequent is:

```
C(A)=C(B0); B1=update_current(B0,F(A)); has_resolved(B1)
ens_array_unsize(A,S); ens_iter_mut(S,I); has_resolved(I)
                                  ? F(B0)=C(B0)
```

These premises imply `F(B0)=F(A)` under the inherited update/resolution rules,
not `F(A)=C(B0)`. Actual physical no-write preservation, **pending reference**
resolution and original-array C/F import must provide the latter. The owned
iterator already resolves; resolving it again is redundant. No imported native
summary, new frame equation or ghost unpacked pointer supplies the missing fact.
The checked null-ghost representation control remains decisive against inferring
physical authority from `MutRef::unpack.ptr`.

Normal return is the intended successful postcondition path, but call cleanup /
unwind_resume cannot disappear from source adequacy. Prove no-unwind for the
covered primitive/call chain or give an explicit simulation/postcondition for
that path. FakeRead erasure, StorageDead, retag markers, impossible u8-ZST branch
exclusion and queried drop glue need source-phase justifications. Empty runtime
drop is not semantic loan release. Model Offset permits unbounded Z; native
finite-width/isize/in-allocation and one-past formation need a restricted
simulation with no one-past dereference. Every one of the 44 source-operation
simulations remains open, plus the three already identified phase obligations.

## Exact requirement for the integration lead

Needed: a checked operational/typing definition for the actual Rust1.99.1 input
configuration and source loan context, plus a sound mapping of that context and
physical memory into VerusBelt's existing interpretations, with the C/F relation.
The mapping must cover incoming reborrows/ancestors and arbitrary interior array
subobjects. It cannot merely accept `ty_own` or the model heap fragment. Then
prove reservation/activation/retag/unsize and remaining native primitive/phase
simulation and exit observation. This is missing proof/source semantics, not a
missing executable, Coq package, toolchain readiness file or artifact hash.

Tree Borrows' inspected Retag rules are useful operational material, but no
checked Rust-MIR-to-Tree-Borrows-to-VerusBelt source-context theorem is supplied;
its integer Offset also needs restriction. It is not built/imported here.
RefinedRust and Corten remain inspection-only as previously recorded. No new
framework, dependency install, native import or inactive core implementation is
justified by this audit. Parent review and integration remain required.
