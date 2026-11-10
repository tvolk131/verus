# Functional native frame: a smaller proof boundary, not an inherited safety certificate

**Decision:** full Rust borrow/provenance-history correspondence into the Iris
*typed* context is not intrinsically necessary for a functional byte/frame
property. A checked, sufficiently precise functional projection could avoid
that particular construction. **Reject the proposed shortcut as presently
justified:** the inspected inherited trust does not supply the operational
safety/erasure and source-observation theorem needed to apply that projection
to every original input. Assuming defined execution of the unsafe standard
library chain, ignoring retags/unwind, or importing a normal-execution-only
result would add an unproved premise. No native obligation is discharged here.

This distinguishes a possible smaller **proof** from an available inherited
**assumption**. It does not assert that the original Rust is unsafe, or that
faithfully representing the complete dynamic borrow tree is the only way to
prove the goal. The old source-entry “irreducible sequent” is necessary for its
chosen typed-model route; it is not a minimality theorem about every functional
adequacy proof. Its actual-source, full-domain, phase and C/F requirements still
apply to an alternative route. [Entry specification](../../source/tools/research/native_source_entry/MISSING_LAYER.md#L55),
[its erasure qualification](../../source/tools/research/native_source_entry/MISSING_LAYER.md#L76).

## Scope and unchanged status

Committed baseline: `759644e2c14430d2715889484aeb7b9cf77f32e6`. Early exact
scope/sequent: `a619a8c8`. Only this new evidence directory is owned. The active
[TARGET](../../source/tools/research/mutable_buffer_goal/TARGET.md#L1),
[NATIVE_CERTIFICATE](../../source/tools/research/mutable_buffer_goal/NATIVE_CERTIFICATE.md#L1),
[FACT_PROVENANCE](../../source/tools/research/mutable_buffer_goal/FACT_PROVENANCE.md#L1)
and [transfer domain freeze](../../source/tools/research/mutable_buffer_goal/transfer-freeze.json#L1)
remain authoritative. Canonical all-u64/arbitrary-initialized-exclusive-[10]
remains 6/0; frozen bitcoin_hashes0.14 HMAC pad-XOR, arbitrary initialized [128]
and all valid key lengths, **prefix AND suffix**, remains 1/1. Native=0,
44+3 minimal/phase and 101 writer effects open, publication=false. Same checked
automation, normal checking, inherited trust, no new assumptions and fresh
independent acceptance remain required. This audit is not acceptance or a status
change and does not rerun those results.

No core/rule/application edits, proof builds, Coq runs, Rust regressions,
dependency imports, publication, push/PR, subdelegation, sibling/parent checkout
reads, live proof-output reads or filesystem experiments occurred. The pending
root SC complete writer is treated as **UNPROVED/pending**; only its
[INTERFACE](../../source/tools/research/model_full_writer/INTERFACE.md#L1)
was read. No claim about its body or live proof outcome enters this audit.

## What trust is actually demonstrated

| Premise or component | Source and exact scope | Classification for this audit |
| --- | --- | --- |
| Rust/Verus compiler, types, traits, modes, lifetime checking and existing lowering | [verifier.rs:2918](../../source/rust_verify/src/verifier.rs#L2918) rejects inference with disabled checks; [3532](../../source/rust_verify/src/verifier.rs#L3532), [3556](../../source/rust_verify/src/verifier.rs#L3556) call borrowck on local Verus-aware non-external bodies | Inherited checking/implementation trust. This is not an operational proof of all dependency unsafe bodies. The driver commentary is historical; current call sites above determine the current scope. |
| General Rust safe/unsafe philosophy | [memory-safety.md:9](../../source/docs/guide/src/memory-safety.md#L9), especially 17–28, puts unsafe encapsulation on library implementors; 60–77 explains why Verus type checking alone is insufficient | Inherited semantic reliance on valid references and correctly encapsulated safe APIs is acknowledged. The documentation conditions the guarantee on successful encapsulation; it supplies no instance-specific machine-checked MIR safety/retag theorem. |
| Trusted external contracts | [tcb.md:7](../../source/docs/guide/src/tcb.md#L7), [reference-attributes.md:99](../../source/docs/guide/src/reference-attributes.md#L99) | Existing contracts are assumptions that the named functions meet their specifications. External/opaque types alone add no arbitrary behavioral theorem (tcb.md:14–17). Not blanket permission to assume any newly needed body property. |
| Array unsizing | [array.rs:177](../../source/vstd/array.rs#L177) | Inherited external-body C/F view equality and explicit no_unwind. No physical byte/provenance projection is exported. |
| IterMut construction | [std_specs/slice.rs:491](../../source/vstd/std_specs/slice.rs#L491), [515](../../source/vstd/std_specs/slice.rs#L515) | Inherited opaque type and assumed initial/final item-to-slice equations, length and iterator laws. No retained-tail physical frame, root pointer map, raw-operation validity or no_unwind clause. |
| next and Zip | [std_specs/iter.rs:46](../../source/vstd/std_specs/iter.rs#L46), [71](../../source/vstd/std_specs/iter.rs#L71), [892](../../source/vstd/std_specs/iter.rs#L892), [942](../../source/vstd/std_specs/iter.rs#L942) | Inherited abstract next/Zip equations, not physical raw-field correspondence or unissued-tail resolution. Native model documentation explicitly says precompiled core is not verified: [infer.md:1221](../../source/docs/guide/src/infer.md#L1221). |
| Unwinding | [reference-unwind-sig.md:3](../../source/docs/guide/src/reference-unwind-sig.md#L3), [30](../../source/docs/guide/src/reference-unwind-sig.md#L30) | Default may unwind; ensures is a normal-return statement. Safe API safety does not imply no panic, no divergence or a normal-return witness. |
| Destructors | [call-from-unverified-code.md:24](../../source/docs/guide/src/call-from-unverified-code.md#L24) | Inherited Drop convention is opens_invariants none/no_unwind; it is not “no writes”, referent release or resolution of every opaque observation. Empty queried glue is separate source evidence. |
| Existing C/F and resolution machinery | [prelude.rs:247](../../source/vir/src/prelude.rs#L247), [ast_to_sst.rs:4220](../../source/vir/src/ast_to_sst.rs#L4220), [resolve_axioms.rs:142](../../source/vir/src/resolve_axioms.rs#L142) | Inherited value encoding and reference-resolution rules. It contains no runtime address/tag/heap interpretation. |
| Raw finite-map initialization | [native_heap_initialization/REPORT.md:14](../../source/tools/research/native_heap_initialization/REPORT.md#L14), [model_input_context/REPORT.md:26](../../source/tools/research/model_input_context/REPORT.md#L26) | Recorded checked MODEL construction from the actual supplied raw map, fresh internal Iris resources and same-heap frames. No client permission premise. This genuinely removes a model resource-construction gap; it does not prove that a Rust execution projects to this map/program. |
| Caller-preserving MODEL writer `2c4795b5` | [interface.v:30](../../source/tools/research/model_caller_writer/candidates/interface.v#L30), [caller.v:28](../../source/tools/research/model_caller_writer/candidates/caller.v#L28), [trust inventory](../../source/tools/research/model_caller_writer/TRUST_INVENTORY.json#L1) | Recorded checked theorem under existing typeG/E/L/I and typed parent/shared-key resources. It retains arbitrary Frame and the parent's **original future**, creates/ends only a child, and does not assume parent future=current. It is not fresh-TypeG native entry, a final original-source heap theorem, or an imported Rust safety theorem. |
| Scalar XOR and ordinary memory MODEL laws | [native_byte_xor/REPORT.md:13](../../source/tools/research/native_byte_xor/REPORT.md#L13), [23](../../source/tools/research/native_byte_xor/REPORT.md#L23), [ordinary-access review:72](../ordinary-access-model-review/REPORT.md#L72) | Recorded checked full-byte scalar law and conditional same-cell framed memory laws; ordinary accesses have two model phases. FE/Eq_rect_eq are inherited logical dependencies as inventoried. No source safety or whole-assignment purity follows. |
| Native pointer/retag/raw-reference safety and source-phase erasure for this chain | [fragment.v:41](../../source/tools/research/native_mir_lrust_refinement/candidates/fragment.v#L41), [coverage](../../source/tools/research/native_mir_lrust_refinement/source-coverage.json#L1) | **Missing source proof**, not an established new allowance in the TCB. Eight descriptor operations have model simulations; 34 minimal operations unsupported and 2 CTFE-unreachable. Retag/reservation/activation/unwind/drop are explicitly unsupported, not proved identities. |
| Whole-path definedness, source-to-projection simulation, same-reference C/F import | [native_effect_soundness/source-results.json:40](../../source/tools/research/native_effect_soundness/source-results.json#L40), [native_reference_adequacy/REPORT.md](../../source/tools/research/native_reference_adequacy/REPORT.md#L1) | **Unproved**. Promoting these to an “unsafe std is correct” premise is precisely the unlicensed step. The trust inventories do not discharge them. |

This audit does not require re-proving rustc or the standard library generally.
An existing trusted compiler preservation rule can be reused at its documented
scope; the missing work is justifying this new projection and observation
import at the selected boundaries. A successful borrowck query or matching
source/MIR identity is not that rule.

The positive trust entries matter: it would be inaccurate to say that Rust,
standard contracts or C/F are untrusted or must all be re-proved. Conversely,
“existing dependency trust” in TARGET does not mean every operational property
of precompiled core is an existing axiom. The inspected declarations specify the
particular boundary. Existing abstract safety reliance does not automatically
provide a theorem usable by a new raw-MIR semantics. Even granting a general
safe-API safety premise would leave footprint, phase, observation and exception
obligations; safety alone never implies the desired frame.

## Concrete source path and the smaller functional projection

[COMPLETED_MIR.json](COMPLETED_MIR.json) records authenticated, explicitly pinned
completed extraction inputs and compact operation excerpts. It binds the full
32-instance/408-operation graph from native_byte_xor/SOURCE.json and the separate
6-instance/44-operation constructor/drop graph from the MIR refinement pins.
Their receipt/raw JSON hashes matched before inspection. Pinned Rust1.99.1
standard source excerpts also matched the completed receipt's source hashes.
These are identity and structured source observations, **not proof facts**.
Historical checkout source-span paths were labels only and were never opened.

The full writer root has `_1: &mut [u8;128]`, `_2: &[u8]` and the actual chain
`_7 = &mut (*_1)` -> `_6 = move _7 as &mut [u8]` -> `iter_mut(move _6)` into
`_5`. This is **not** the minimal constructor/drop fixture's `_4 -> _3 -> _2`
numbering. The latter's static loan0/region3 is the local reborrow, not the
incoming dynamic source loan. [Entry operand inventory](../../source/tools/research/native_source_entry/MISSING_LAYER.md#L9).

The full path resolves:

- slice IterMut::new and slice IntoIterator/Iter construction;
- Iterator::zip -> specialized Zip::new -> exact size hints, min, pointer distance;
- specialized Zip::next -> mutable/shared `__iterator_get_unchecked`;
- root bb7: key load through `_18` into `_19`, then s6
  `(*_17) = BitXor(copy (*_17), move _19)`;
- return, actual cleanup/drop edges, and separately queried glue.

The pinned `core/src/iter/adapters/zip.rs:293–311` creates
`index=0,len=min(a.size(),b.size())`, tests index<len, increments index, then
calls both indexed getters. None preserves that logical index and does not
consume another pad byte. `slice/iter/macros.rs:413–424` forms the reference at
`self.ptr+idx`; it does not advance the inner pointer. Its safety comment
requires in-bounds indexing, no repeated index and no conflicting methods.
These requirements are not Coq theorems. Source excerpts and MIR instance IDs
are retained in COMPLETED_MIR.json. The generic Zip comment expressly considers
panic in getters; no-unwind must be specific to these concrete getters.

A sufficient *functional* projection can use actual allocation identities and
byte offsets, initialized byte contents, slice lengths, locals/control position,
iterator fields and issued slot/index bookkeeping. It need not store a copy of
the full provenance tree in every model cell. Relate each source allocation to
a fresh injective model block; preserve same-block offsets and all relevant
alias equalities. Do not flatten addresses into independent fresh pad/key blocks:
disjoint ranges in one allocation must remain representable. The projection
covers initialized observable bytes; uninitialized unrelated memory/padding must
not become a requirement that the entire Rust heap is initialized. Callee-local
iterator storage is separate from caller-visible outside memory and may change.

For this fixed source path, the byte invariant can be as small as:

```
m = min(128,n); 0 <= i <= m
Zip.index = i; Zip.len = m
inner pad base = p; inner key base = q
pad[j] = A[j] XOR K[j] for j < i
pad[j] = A[j]           for i <= j < 128
key = K; caller-visible memory outside pad is unchanged by this invocation
```

At a yield, track exactly the element pointer `p+i`, key pointer `q+i` and
absence of another live issued pad access at that index; after its store,
advance the loop invariant. At None, preserve all remaining slots. For n=0 no
getter or key dereference is taken. For n>128 only the first 128 key bytes are
used, but no artificial n<=128 precondition is introduced. No source algorithm
is replaced by this invariant or by an indexed model loop: checked simulation
must relate the original iterator program to it.

A ghost-selected model block is a representation, not new physical authority.
The raw initializer may allocate MODEL ghost resources for the projected map
internally. To use its final heap result for Rust, one still proves that every
covered original execution has the represented transitions and observations.
This route can bypass constructing `ty_own` for the *whole original caller
context*, provided the source safety/observation obligations below are supplied.
It cannot use “fresh resources exist” as proof that erased Rust history is safe.

## Source input domain independent of model ownership

The intended source predicate is the actual lawful function-entry judgment,
not a predicate defined by existence of a successful model proof:

```
V_entry(S,p,q,A,K) :=
  S is a lawful Rust entry configuration for THIS frozen xor_key instance;
  eval_S(_1) is the incoming exclusive reference to [u8;128] at p;
  eval_S(_2) is the incoming shared [u8] slice (q,n), n = |K|;
  source initialized byte contents at p[0..128) are exactly A;
  source initialized byte contents at q[0..n) are exactly K.
```

“Lawful entry” here is source typing/reference validity **at entry**, including
its existing alias/liveness obligations, not “this function's future execution
is defined”. It is a semantic specification of the left side, **not a newly
mechanized source judgment**: the current extractor/frontend/model combination
has no independent formal runtime/history validity definition proving all its
consequences. That absence is part of the result, not a supplied assumption.
The structural consequences needed by a bounded implementation are concrete:

- p is nonnull, properly aligned (u8 alignment1), a live 128-byte subobject of
  one containing allocation; any surrounding prefix/suffix is allowed. All A
  entries are initialized bytes, unconstrained in 0..255. No allocation-base
  equality, ownership of/freeability of the entire allocation, or zero padding.
- A nonempty key has valid extent, metadata, alignment, initialized byte values
  and shared-read access. Its accessed bytes do not overlap the live exclusive
  pad bytes, as required by lawful Rust references. Pad/key may share a
  containing allocation; shared key aliases may exist.
- Empty keys have valid zero-length reference representation but need not have
  any backing readable cell or live allocation. In particular a dangling
  nonnull aligned empty slice and q=p are not excluded. No arithmetic/deref
  lemma may demand a readable key cell when n=0.
- Existing ancestor references/loans may remain suspended in a caller; no fresh
  root loan, empty history, absence of protectors, or future=current assumption
  is imposed. The validity of these incoming histories must be respected by
  the source safety/erasure argument, even if not copied into the byte model.
- Native finite-width, in-allocation, isize and one-past obligations follow
  from source validity plus the particular operations; they are not new user
  requirements. The key length ranges over every valid Rust slice length.

There is no ty_own, PointsTo, guard, model state interpretation, Iris proposition,
postcondition, checked-proofs flag, normal-return promise, or future-definedness
clause in V_entry. If an implementation replaces “lawful entry” with only
numeric bounds and byte lookups, it has dropped source validity; if it replaces
it with model ownership, it has assumed the missing theorem. The current
source-validity documentation itself notes that complete alias/liveness
semantics are not supplied by its pointer-validity references:
[native-drop-inventory.json](../../source/tools/research/mutable_buffer_goal/native-drop-inventory.json#L100).
No fully mechanized exact input predicate is claimed by this static audit.

## Required obligations and the exact missing sequent

Let Pi be the byte/local/control projection above, R the simulation relation
anchored to the actual root operands, and f the unchanged source/MIR body.
The missing **smaller** source boundary can be stated without a source-to-ty_own
entry theorem:

```
ExistingTrust |- forall S,p,q,A,K,
  V_entry(S,p,q,A,K) ->
    R(entry(f,S), Pi(S), p,q,A,K)
    /\ SafeErase_f(R)
    /\ ExitObservation_f(R, original_argument_1)
```

Here the names expand to obligations, not opaque facts to assume:

1. **Entry and identity:** Pi is obtained by evaluating the actual `_1`/`_2`
   operands; it gives the byte lookups and extent information above. Carry that
   identity through reborrow, activation, unsize, constructor, Zip fields and
   getters. Prove source initial view C(original)=A and key view=K through the
   inherited frontend equations. Root local identity is a useful anchor, but
   static operand equality alone is not a heap/observation interpretation.
2. **Safety and erasure:** for every reachable source configuration related by
   R, each enabled concrete step preserves source validity and has the claimed
   projected effect, or takes an explicitly accounted exceptional exit. Cover
   reservation/activation, entry/reference retags and protectors, raw formation,
   loads/stores, provenance-preserving offsets, reference creation, metadata,
   NonNull valid ranges and finite arithmetic. Prove retags erase to byte
   stuttering **and remain valid for every incoming history**. Alternatively
   invoke an existing theorem with exactly this scope; none was found here.
   Source progress/UB exclusion must not be replaced with forward simulation
   only for already-defined steps. The invariant can quantify the erased history
   existentially; it need not reproduce it as Iris loan ownership.
3. **Functional operations and footprint:** every memory load reads the stated
   byte; the XOR store changes exactly p+i. All helpers and real drop edges
   preserve the caller-visible bytes they are claimed to frame. Register and
   iterator-field writes do not count as pad writes. The pointer-distance
   intrinsic computes the actual same-allocation extent; exclude every relevant
   unsafe-precondition failure. Source selection/SAFETY comments/flags are not
   the semantic derivation. Compose the existing full-byte and ordinary-model
   operation theorems only after proving their operand/memory correspondence.
4. **Control, phase, exceptions and return:** simulate the actual specialized
   Zip branch/index/None behavior and the original loop. Preserve the unchanged
   executable source. Justify phase changes between pre-erasure borrowck MIR,
   selected monomorphized MIR and frontend C/F lowering, including FakeRead,
   StorageLive/Dead, retag annotations, cleanup, resume and empty glue queries.
   Either prove the concrete chain cannot unwind/panic on V_entry, or provide
   exceptional-state simulation respecting the existing normal-return contract.
   Never delete these paths because the model expression returns normally.
   Normal-return adequacy must map every actual normal return and not hide UB
   or an omitted cleanup path. Ordinary ensures does not demand total
   termination; this audit adds no new termination requirement. Any claimed
   guaranteed normal return would need its own proof, not safe-API trust.
5. **Exit and original C/F:** at normal return, with local references not escaped
   or outstanding, prove bytes at the original subobject equal
   F(original_argument_1). Preserve caller/ancestor continuations; final pad
   current is not generally the original caller parent's future. This is where
   a purely numeric memory frame becomes the requested Verus theorem. No law
   resolving an arbitrary observer from its resolved owner is allowed.

Equivalently, the missing source-facing safety/functionality sequent must
quantify over executions **including fault outcomes**:

```
V_entry(S,...) -> forall rho in RunsRustIncludingFaults(f,S),
  no_UB(rho) /\
  (Normal(rho,S') -> SourceFutureMatchesProjectedHeap(S') /\ FullPost(S'));
  permitted unwind/divergence is handled under the existing contract.
```

There is no `Defined(rho)` filter on that quantifier. This is an obligation
specification, not a newly checked theorem.

With a checked projected writer theorem the conclusion would be:

```
for each actual normal return S' of f from V_entry(S,p,q,A,K):
  F(original_argument_1)[j] =
      if j < min(128,|K|) then A[j] XOR K[j] else A[j], 0 <= j < 128;
  immutable key and the specified caller-visible frame are preserved.
```

Safety/exception treatment above is universal over V_entry; it is **not** an
extra antecedent `DefinedRun(S)` or `NoUnwind(S)` appended to the theorem. A
lawful concurrent context must not be silently replaced by a quiescent caller
for a claimed whole-heap frame: the source reference/race invariant must justify
noninterference on accessed ranges; an arbitrary external thread's unrelated
writes cannot be attributed to or forbidden by this function's frame. The
active theorem does not add a new whole-program concurrency claim.

This is a reviewable bounded proof interface for this body and call graph,
not a proposal to import a general Rust verification framework. It leaves no
client resource parameter. It also makes clear why whole-context ty_own
construction is an optional route, whereas source safety/erasure and exit
observations are indispensable unless exactly covered by inherited trust.

## Why the current inputs cannot discharge it

The model fragment starts with its own `source_step` on descriptors, not a Rust
operational semantics. Its [definition](../../source/tools/research/native_mir_lrust_refinement/candidates/fragment.v#L66)
only gives the eight chosen operations constructors, and
[unsupported_rejected](../../source/tools/research/native_mir_lrust_refinement/candidates/fragment.v#L134)
excludes the rest. Its checked frame is meaningful, but cannot prove the raw
reference step connecting the actual slice to the raw descriptor. The
[independent review](../native-mir-lrust-review/REPORT.md#L72) already identifies
that producer-to-MIR interpretation gap. Functional erasure is exactly another
source simulation to prove, not something this descriptor theorem supplies.

The full inventory includes `ptr_offset_from_unsigned` and two unavailable panic
bodies. In the actual u8 size-hint path there are UbChecks, an address-order
precondition check and constant pointee-size checks. A proof can exclude their
failure branches from valid input bounds; absent panic bodies need not force
verification of arbitrary panicking internals if checked exclusion succeeds.
At present that exclusion/primitive correspondence is not checked. Neither
branch flags nor the counts of extracted bodies prove it. Zip::new and ::next
have explicit cleanup/resume edges. The minimal drop fixture is useful for a
first bounded sequent but is not the full graph or an exception proof.

The typed caller theorem is stronger than a fresh standalone model run in the
specific way requested: its post preserves the original parent future and an
arbitrary ancestor Frame. However its **antecedent** still contains the existing
semantic parent/key resources. Its source origin is not a theorem about the
actual arguments. The raw initializer solves a different issue: constructing
fresh consistent model resources from a chosen raw map without assuming
semantic ownership. Neither theorem turns a numeric projection into Rust
validity, supplies the MIR-to-C/F relation, or makes retags identities.

The actual compiler still rejects raw borrows/dereferences through the ordinary
value frontend: [rust_to_vir_expr.rs:2631](../../source/rust_verify/src/rust_to_vir_expr.rs#L2631),
[4650](../../source/rust_verify/src/rust_to_vir_expr.rs#L4650). Unsize is redirected
to the existing summary at [1900](../../source/rust_verify/src/rust_to_vir_expr.rs#L1900).
The research certificate checker deliberately ends in
[rejection](../../source/tools/research/native_iterator_certificate/check.py#L416).
None of these is an installed checked native projection rule.

## Two small logical witnesses, not Rust counterexamples

**History erasure does not prove safety.** Consider two abstract source states
with identical bytes, addresses and next instruction but different access tags.
In one, a raw access is authorized; in the other, it is rejected/UB. Pi forgets
the tags, so it gives the same numeric read in both. A proof of that projected
read and its frame does not prove the second source state can step safely. This
is a countermodel to the inference “same projected bytes implies safe access”,
not a claim that both states are lawful entries of this function or that Rust's
safe iterator actually creates the bad state. Excluding the bad reachable state
from **all lawful entries** is exactly SafeErase_f's preservation/progress
obligation. Constraining V_entry to histories where the entire run happens to
be defined would beg the question.

**C/F permits an unproved future change.** In the constructor/drop diagnostic,
let original B0 current bytes be all0 and future bytes all1. Let reborrow A have
C(A)=all0, F(A)=all1; update_current(B0,F(A)) has C=F=all1 and is resolved.
The unsized reference and 128 pending item observations can have the same
respective current/future values. A resolved opaque iterator adds no law
resolving those item observations. This satisfies the inherited C/F/constructor
premises but violates F(B0)=C(B0). It is a logical counterinterpretation of those
summaries, not an execution of native no-write code. For a shorter key the same
freedom can remain on the unissued suffix. The [existing sequent](../../source/tools/research/native_source_entry/MISSING_LAYER.md#L163)
and [resolution distinction](../../source/vir/src/resolution_inference.rs#L80)
explain why physical memory observation and an exit bridge are still required.

Finally, [MutRef](../../source/vstd/mut_ref.rs#L9) is a ghost pack/unpack
representation. Its current/future axioms say nothing identifying its `ptr`
with the runtime address. The checked null-representation probe described in
[native_reference_adequacy](../../source/tools/research/native_reference_adequacy/REPORT.md#L39)
precludes using that field as the missing anchor. Use actual MIR operand
evaluation in R; then prove its relation to C/F. Do not assume that relation
from a local ID, a hash, or a selector's successful shape check.

## Concrete next task for the integration lead

Keep publication/status unchanged. The smallest useful next source task is a
**bounded constructor/drop source erasure certificate**, on the already pinned
6-instance/44-operation graph, with this frozen statement:

```
V_entry_drop(S,p,A) ->
  actual operand projection starts at (p,A),
  root reborrow/activation/unsize + IterMut::new + actual scope exit
  preserve projected original-array bytes and legal source access,
  with explicit retag/FakeRead/Storage/cleanup treatment,
  and at normal exit F(original) = those same final bytes.
```

Prove it for arbitrary initialized A, interior p and arbitrary lawful incoming
ancestor history; no owner/PointsTo/guard from the caller. Reuse the eight
checked descriptor operations and raw-map initializer, but add the **source**
lemmas for the currently unsupported reborrow/raw-reference/phase/exit steps.
A bounded source invariant may erase history existentially; it need only prove
that this chain preserves legal access and observations, not encode the entire
Rust type system into Iris. Each skipped source step needs a justified
stuttering/no-byte-write and validity lemma. Prove normal-path branch exclusion
or simulate cleanup, rather than assuming no_unwind. A null-ghost-ptr witness,
wrong root operand, invalidated tag and unaccounted unwind path should remain
explicit countermodel/rejection obligations at this boundary.

That milestone does **not** prove the HMAC theorem. Next extend the same source
relation to the full indexed Zip path, pointer-distance, getters, ordinary XOR
store, None and real cleanup edges. A complete SC model writer cannot substitute
for either milestone. If a source semantics/safety lemma for this bounded chain
cannot be supplied, report this exact missing sequent; do not turn it into
`UnsafeStdSafe`, `DefinedRun`, an assumed frame/resolve rule, or application
requires. Integration and fresh independent acceptance remain the root's work.

## Audit validation

This is a read-only static audit plus two explicit mathematical countermodels;
there are no new proof claims or executable tests. Current committed source,
the named reports/trust inventories and explicitly pinned completed MIR artifacts
were inspected. [SOURCE_INVENTORY.json](SOURCE_INVENTORY.json) records local
committed source identities/line ranges and the deliberately interface-only
SC writer read. COMPLETED_MIR.json records exact completed artifact pins and
selected source observations. No proof fact is obtained from those identities.
No private proof products or toolchain were rebuilt or modified. Final checking
is limited to evidence links/JSON, diff whitespace and owned-path scope.
