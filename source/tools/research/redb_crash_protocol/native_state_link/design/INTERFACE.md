# Design worker interface — frozen first checkpoint

Prepared checkout: fe536b27b5dd259aa03320db0b9b4fa51714e89a. This worker owns only this design directory. No production installation, native source edit, delegation, shared cache, push or PR. Start clock: 2026-11-12 14:19:57 UTC (includes sandbox setup/read time); maximum 45 active minutes excluding genuine builds.

## Proposed deliverable

A reviewed-next-step design for a **closed owned/exclusive UnsafeCell<u64> frontend extension**. Validate the actual loaded core ADT and the bodies of new/into_inner/get_mut; lower their real construction/move/exclusive field borrow to existing ordinary VIR operations on that same nominal datatype. The candidate must admit the unchanged owned_goals.rs bodies with full checks, including exclusive_borrow's retained native field store. Constructor and local update optimize to scalar moves and alone cannot establish live state linkage.

No ordinary logical selector for live shared cell contents is justified. Initially reject shared cell borrows, shared enclosing-container borrows, raw pointer escape/access, callbacks, unresolved/generic dispatch, destructors and imports of the encoding. Reject unknown shapes rather than infer absence of interior mutability from a spelling or one shallow type walk. Restriction enforcement is part of the candidate's soundness obligation, not an informal user promise. A blanket Rev-style datatype installer without these gates is unacceptable.

This is a compiler translation/ownership restriction design, not a new assumed cell law or permission constructor. The existing Rev descriptor is inactive and closed to Rev; its tests are evidence about existing mechanics only. It must not simply be generalized to all single-field wrappers. A new source-specific descriptor needs independent review of its restricted simulation and all bypasses before activation.

## Early risks to resolve

1. Abstract trait/native callback taking &UnsafeCell<u64> can legally mutate behind the shared reference. An immutable logical field can falsely survive that call, even without any raw operation in the caller. Exclude the call/borrow before emitting any logical datatype facts; inspecting only direct UnsafeCell method calls is insufficient.
2. Nested structs, aliases, generic instantiations, trait-associated types, closures and opaque/imported containers can hide the same escape. Initial admission must use normalized typed structure plus a closed grammar; unknown is rejection.
3. Snapshots/equality in specification code must be historical owned/exclusive state, never an immutable shared identity-to-contents function. Source privacy alone does not repair framing.
4. UnsafeCell<T> drops T, unlike PCell. Restrict T to u64, no destructor-bearing enclosure, no cleanup/unwind/coroutine path in the admitted slice. Any later broadening needs actual semantics, not a hash alone.
5. &mut UnsafeCell grants exclusive access; MutexGuard::deref_mut starts from &Mutex and raw get. The first does not license the second. No layout equality or structural twin creates a permission for mutex.data.
6. Export/import and Infer-only installation can bypass a local restriction. The first version must reject exporting the descriptor and affected facts, and apply the same checks to ordinary and Infer verification, including code pruned from proof output.

## Required unresolved bridge

Native Mutex::new constructs its actual data subobject. To use the owned initializer beyond this phase, a checked transition must transfer that same subobject's exclusive state into an invariant held by the actual native lock, without duplicating a snapshot. Successful native acquisition must recover that state's exclusive permission; get/deref_mut must target that same live subobject with justified provenance, and actual drop/unlock/unwind must restore it. Poison/error/concurrency paths remain obligations. No client token or equality premise discharges this transition.

The follow-up design will give exact file targets, typed gates and semantic negative programs, explain actual Mutex initialization value, and name remaining application debt. Full CP1/CP2, native_bridge_proved and source_coupling_proved remain false; HMAC/native is DEFERRED AND UNACHIEVED.
