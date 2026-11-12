# Smallest constructive next implementation

Status: design only. Nothing described here is installed or checked by a new verifier build. The recommendation is a bounded source-specific translation, not a universal UnsafeCell model. The unchanged native functions and all u64 inputs remain the first denominator. No shared/guard theorem follows.

## 1. Admission unit and types

Use one local verification unit with complete body checking in both ordinary and Infer modes. When this feature is requested by a real native cell operation, validate all local executable items before function/module pruning, including external/skipped/native items, signatures, statics, constants, nested bodies, adjustments, and resolved call operands. An unsupported local item rejects this experimental unit. Imported vstd remains the accepted baseline; no new cell-bearing imported contract is accepted. This deliberately restrictive unit can contain the three frozen functions without pretending to support a complete REDB crate.

First grammar, after alias/associated-type normalization:

```
S := () | u64 | bool | tuple(S, ...)
C := loaded core UnsafeCell<u64>                // same native DefId/type arguments
V := S | C | &mut C | &mut u64
```

No C inside tuples in version one. No user ADTs, generic function/type parameters, unresolved projections, opaque types, dyn types, function values, closures, coroutine captures, heap/container handles, shared executable references, raw pointers, unions, casts or user destructors in this unit. Region binders required for the two reference forms remain; these are not rejected as type generics. The real core declaration remains generic, with its real private `value: T` and ?Sized declaration; admission of native instantiations is limited to T=u64. Do not synthesize a new OwnedCell64 runtime type or silently turn C into u64 in VIR.

This is a positive grammar, not a best-effort `contains_UnsafeCell` search. Aliases resolving to C are accepted as C; a lookalike is rejected. Any unexpected type/effect is rejection. Calls are only direct, fully resolved original new/into_inner/get_mut instances, or acyclic local functions admitted by these same rules. Their full bodies must verify, without external/assumed contracts. No FnDef-as-value coercion, virtual call, trait callback, even monomorphized unknown callback, or foreign function gets an exception. Scalar arithmetic/control flow can initially stay within the existing ordinary frontend and its safety checks; unexpected cleanup/unwind paths reject the experimental unit. No new optimization or summary proposal is needed. There is no REDB/function-name recognizer; admission is the closed native operation/type grammar.

Source specification expressions may use scalar/tuple facts (the frozen postconditions need only these). Do not expose C values/selectors/equality to user ghost storage, quantifiers, uninterpreted spec functions or shared spec reference APIs in version one. Compiler-generated native-method contracts may mention the private field only under a validated native-method authority. A `Ghost<C>` escape is not an owned native move. Historical scalar snapshots are harmless; a function from a stable shared cell identity to its live contents is forbidden.

## 2. Validate identity, then lower actual operations

The Rust frontend must freshly check:

* `tcx.lang_items().unsafe_cell_type()` identifies the loaded native ADT; stable core identity and DefPathHash bind the pinned artifact, not a source spelling. Reexports resolve to this same item. Check struct/one variant/one private `value` field with actual Param(0), source owner/visibility, real generic declaration including ?Sized, no destructor, layout-affecting attributes and marker/auto-trait behavior. Retain native !Sync and original Rust type/trait checks. `repr(transparent)` is an observation, not the justification of this translation.
* Instantiate only u64; `needs_drop(C)` and `needs_drop(u64)` are false. Check the complete method identity, safe Rust ABI, const/signature metadata, actual argument/return types and regions. In particular validate `for<'a> fn(&'a mut C) -> &'a mut u64` **before** normalizing/erasing regions for MIR inspection. No manufactured `'static`, detached returned borrow, raw conversion, or fn-pointer instance.
* Validate loaded typed method MIR, every executable statement/terminator/local, including unreachable/cleanup blocks and borrowed places, against the closed operations below. Source hashes and the pinned compiler/core artifact bind provenance; they do not prove source/body equivalence or native correctness. Loaded typed bodies are the translation authority. Missing MIR, unexpected optimization/body phase, extra effect, cast or drop is rejection. Inspect identity/body afresh independently of inferred summaries/caches.

| Actual method | Accepted computation | Ordinary VIR computation and proof obligation |
| --- | --- | --- |
| `UnsafeCell::<u64>::new` | Aggregate of the actual C ADT with its actual value field initialized from the input | Constructor of the same nominal datatype; derive returned field = input by checking the full body. |
| `into_inner` | Read/move of that field from the consumed C parameter; Copy of its u64 field in instantiated MIR is allowed | Ordinary private field read, caller consumes C by Rust ownership; check result = entry field. |
| `get_mut` | Exclusive `&mut` borrow of `(*self).value`; never shared/raw access | `BorrowMut(Field(value, DerefMut(self)))`; check current returned referent = entry field and final cell field = final returned referent through the existing ordinary reborrow/writeback rules. |

Generate full body-backed functions, never `assume_specification`, `external_body`, an uninterpreted value axiom, or a permission from nothing. Keep their contracts as ordinary obligations. The private field authority must be compiler-owned and bound to the actual native method DefId and validated operation; source attributes, a forged owning module and imported descriptors cannot mint it. Recheck that authority at WF and backend entry. A minimal implementation must resolve this privilege explicitly; placing a root helper in core's owner by fiat is not a checked access rule.

An independently written correspondence pass must compare the **actual emitted Function/places/types/read modes**, not a duplicate proposed descriptor, with the original typed operations and maps. In particular, retain the input C place in exclusive_borrow; match its field store and subsequent load. Comparing only return expressions would accept a scalar-elided implementation and miss the task. Descriptor construction and correspondence checking are compiler TCB changes requiring independent review; SMT proof of the generated postcondition alone cannot validate a mistranslation.

The simulation is limited and direct: an owned native C place contains its u64 field; a unique native &mut C authorizes an exclusive field reborrow; field writeback updates that place; moves transport the native value and consume the former owner. There is no global cell ID, stable address, shared-state selector or new linear-token axiom. Rust's original exclusive borrowing discipline suffices only because the admission unit has no shared/escaping operation. This is why the native exclusive_borrow is a necessary acceptance target.

## 3. Exact proposed production file touch points (future work only)

| File | Narrow change |
| --- | --- |
| `source/rust_verify/src/rust_to_vir.rs`, `rust_to_vir_adts.rs` | Insert the native metadata-backed DatatypeX into the local pre-crate before unsupported-type/body processing, after the complete typed-unit admission check. The existing external-type-specification route rejects transparent private fields; do not relax it or require a source proxy. `rust_to_vir_base.rs::def_id_to_datatype` already preserves the native path/type arguments and should keep doing so. |
| `source/rust_verify/src/internal_repr.rs` | Separate closed cell metadata factory and pre-lowering typed admission check; preserve Rev factory unchanged. Alternatively put the longer scan in one new `checked_owned_cell.rs`, declared by `lib.rs`. |
| `source/vir/src/internal_repr.rs` | New closed descriptor variant for the native cell, with u64-only use validation; source-private fields and local-only, nonserializable authority. Revalidate actual DatatypeX, not cached hashes. Do not generalize the Rev one-field recognizer. |
| `source/rust_verify/src/checked_foreign.rs` | Reuse body-backed plumbing or the dedicated module for the three exact native instances and independently checked correspondence. Its current bool-to-u64 recognizer is insufficient and its Infer-only registration must not gate the new ordinary path. |
| `source/rust_verify/src/fn_call_to_vir.rs`, `context.rs` | Bind direct native call HIR IDs to freshly resolved instance + checked method authority. Leave erasure/native executable call identity intact; include --compile and original erased-call inspection in acceptance, rather than assuming a VIR helper name guarantees runtime fidelity. Do not globally remap a method spelling or function pointer. |
| `source/vir/src/well_formed.rs` | Enforce descriptor use grammar and the limited compiler method authority; reject source/private field access and ghost/shared escapes. Existing `validate_datatype` hook is a starting point, not sufficient enforcement. |
| `source/rust_verify/src/verifier.rs` | Run full eligibility before pruning and both modes; require full body/lifetime/trait/erasure checks, refuse partial verification modes, prevent experimental export including zero-helper cases. |
| `source/rust_verify/src/import_export.rs` | Confirm the descriptor cannot deserialize; reject affected exports and stale artifacts before writes/reuse. No new wire schema or publication authority for this phase. |
| `source/vir/src/infer_checks/tests/internal_representation.rs` and new `source/rust_verify_test/tests/local_checked_owned_cell.rs` | Focused IR/correspondence corruption tests plus real-source positives, semantic failures, safety failures, ordinary/Infer/import/escape refusal tests. |

Do not change ordinary datatype/global immutable reference semantics, PCell, raw_ptr, atomic primitives, native std/core source, base REDB source, or the frozen u64 bodies. If the implementation needs a general mutation-effect inference or raw/provenance foundation to admit these three functions, it has exceeded this target.

## 4. Acceptance before any shared-phase work

See ACCEPTANCE.md for the typed gates and negative programs. Mandatory positive: both modes prove the three frozen goals; the exclusive_borrow translation additionally proves its **final native input field** equals next, with unconstrained initialized entry u64, via a body-identical diagnostic query. The private postcondition may be proposed by test/compiler code but must pass the ordinary full-body query; it is not a supplied premise or a new public selector. Deleting the writeback/store must make that state query fail even if `result == next` can still hold. No native proof is claimed from the optimized scalar-only functions.

The existing core generic methods need three checked body instances, and every client body plus any additional diagnostic body/contract must be inventoried exactly. Freeze the actual inventory after implementation rather than promise an unexplained verification count. Failed frontend cases are distinct from failed postconditions. A fresh private accepted78 baseline remains the comparator; an experimental build, if later authorized/created, must have a distinct identity and may not masquerade as that accepted runtime.

## 5. Actual Mutex initialization: useful prerequisite, explicit limit

The unchanged installed `std::sync::Mutex::new(t)` constructs its **own** `data: UnsafeCell::new(t)` (mutex.rs:350-351). The prior native_handoff MIR actually contains that aggregate and places the same cell in Mutex field2, before `_6 = &_3` and the original lock call. Therefore the constructor rule above is the missing local rule at a genuine native initialization site, not a replacement PCell or twin.

Version one nevertheless rejects the enclosing Mutex and its shared publication. It does **not** prove the complete Mutex constructor or native_handoff. A scalar-only demo must not be integrated as completion. To credit later progress at initialization, a narrowly checked **same-object initializer embedding** must validate the actual Mutex aggregate/field2 move and transport this exclusive state through that move, then stop at the first shared borrow. Dataflow observations or a caller-provided equality are not sufficient to export a live state fact.

There is also measured, concrete extra constructor debt: the existing native_handoff MIR initializes both platform/poison atomics using `Transmute` (0_u32 -> Atomic<u32>, false -> Atomic<bool>). Installed Atomic constructors actually contain `unsafe { transmute(v) }`. These are outside the three-operation grammar. Do not erase them, assert native-layout equality, treat their bodies as pure assumed constructors, switch toolchain, or declare the whole constructor checked because its data initializer is ordinary. A separate bounded design could treat typed opaque values only if it validates their actual construction/validity/effects; no such rule is supplied here. Version one should reject this exact full constructor and report why.

The next application-facing acceptance remains actual LRUWriteCache inside Arc<Mutex<...>>, not extrapolation from u64. Its nested allocation/drop and shared lock handling are unproved. The original cached_file.rs:347-349 initializes those native locks; flush_write_buffer:490-518 acquires their actual guards and writes their actual entries. A future extension earns application value only by carrying that original state's bytes to those real successful writes before sync, including errors. Do not start that foundation chain automatically as part of version one.
