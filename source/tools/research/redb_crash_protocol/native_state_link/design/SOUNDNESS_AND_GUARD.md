# Framing audit and the unresolved native guard relation

## Counterexample to an ordinary shared field model

This is legal single-threaded Rust. The trait has no preservation contract. Its caller does not contain a raw access:

```rust
use std::cell::UnsafeCell;
trait NativeHook {
    fn run(&self, cell: &UnsafeCell<u64>, next: u64);
}
struct Write;
impl NativeHook for Write {
    fn run(&self, cell: &UnsafeCell<u64>, next: u64) {
        // No references to the inner u64 are live; no competing access.
        unsafe { *cell.get() = next; }
    }
}
fn abstract_client(hook: &impl NativeHook, initial: u64, next: u64) -> u64 {
    let cell = UnsafeCell::new(initial);
    hook.run(&cell, next);
    cell.into_inner()
}
// abstract_client(&Write, 3, 7) returns 7, not 3.
```

Suppose a nominal logical datatype stores immutable `value: u64`, new establishes value=initial, and a call through `&UnsafeCell` preserves its logical argument as ordinary shared references do. The abstract method's empty contract then leaves value=initial framed. into_inner returns initial, proving a false result when initial != next. Private fields do not help: trusted/compiler-created constructor and accessor contracts already give the solver enough equations. Checking the native hook's method body at one implementation does not repair a modular abstract trait call to another implementation.

The error is a temporal/type interpretation error, not bad Rust aliasing. Several shared UnsafeCell references may coexist while its contents change. The cell's address/identity is not its immutable content snapshot. Normal datatype extensionality can also equate different scalar-content cells with no physical identity; it must not be used to derive pointer/guard identity.

A restricted owned encoding avoids this counterexample only by **rejecting the shared borrow and call before logic generation**, also in goal-free mode and before pruning. It does not prove that the hook preserves the value. Merely invalidating an explicit user `value()` fact is insufficient if ordinary datatype/reference equality, generated accessor contracts or framed outer values can re-derive it. Havoc of the scalar alone is also insufficient while a stable immutable shared representation still identifies it with an old field. A shared phase needs a different resource/invariant interpretation, not an exception to the rejection rule.

## Escape and framing cases

| Case | Why a shallow check fails | Required version-one result |
| --- | --- | --- |
| `type C = UnsafeCell<u64>` and std reexport | Spellings differ but type is the same. | Normalize; permit only the same native ADT in allowed owned/exclusive forms. |
| `&Wrap<UnsafeCell<u64>>`, `&Option<C>`, nested tuple/array, Arc/Box/Vec | Shared outer equality can frame an interior field; a callback can reach it. Niche/layout transformations are not identity proofs. | Unknown/custom ADT/container forbidden, even by-value; shared outer reference forbidden. No recursive positive admission based on absence of C in the immediate argument list. |
| `&T`, `T::Assoc`, dyn trait, monomorphized `Fn` | The generic/associated/captured value can hide C; a callback can retain aliases or mutate it. | Type generics, unresolved/opaque projections, dyn/callback dispatch and function values rejected. Fully concrete generic calls are still rejected unless one of the three validated native operations. |
| Return/store of `&C` or a shared enclosing object | An alias survives the local constructor's body. A later owned-looking read might inherit a stale summary. | No shared reference at a return, assignment, capture, static, const promotion or call adjustment; no affected external/abstract API admitted. |
| Destructor or unwind callback | Drop can access a previously escaped reference or mutate other live cells; T's drop is real for UnsafeCell. Cleanup can release a guard while an invented token remains. | u64 only, all admitted types need no drop, no destructor-bearing enclosure; inspect all blocks/effects, reject cleanup/unwind/coroutine paths in the slice. No change to panic strategy as a workaround. |
| Raw pointer, `get`, `raw_get`, `from_mut`, transmute, address exposure | Converting a reference to a raw pointer does not establish permission/provenance; effects can be hidden after a pointer escape. `from_mut` itself uses raw casts and is outside this target. | Reject all such operations/typed locals/coercions, including a generic or native wrapper. No pointer equality theorem or layout coercion. |
| Imported or assumed wrapper operation | An empty contract can hide a shared mutation; an imported generic summary can instantiate to C without naming it. | Only the checked local operation grammar; no affected imported/assumed/external contracts. Reject export of the descriptor or dependent facts and stale output reuse. Unrelated accepted vstd imports do not authorize cell operations. |
| Specs, Ghost, quantifiers, triggers | A ghost snapshot can reintroduce a stale native identity-to-content equation, independently of an explicit live-value getter. | User spec surface is scalar/tuple only in v1. No C-containing ghost values or reference spec functions. Internal native-method field equations stay behind checked authority. |
| Plain `&mut C` | Unlike shared C, Rust still guarantees exclusivity of C and its contents. Historical raw aliases cannot lawfully access it during this unique borrow. | Admit direct exclusive reborrow and scalar writes with ordinary lifetime/writeback checks. No added caller value/token premise; arbitrary valid initialized u64 input. |

For reliability, IMPLEMENTATION_TARGET.md chooses a small positive grammar for the **complete local unit**, not a general whole-program escape analysis. Every local body/item is validated, including ones excluded by proof pruning. A hand-coded scan only over new/get_mut call sites is not enough. Before optional later widening to containers or generic instances, re-open each row with concrete typed programs; do not interpret this rejection list as support for those cases.

## The exact phase boundary

Write `Own(p, v)` below as metanotation for the native exclusive state in the translation argument. It is not a new Verus axiom, exported predicate, or token required from clients. `p` denotes a native place; an owned move may change its address and must transport the state, not assert addresses equal.

Within the restricted domain the proposed constructor/exclusive rule justifies:

```
new(v):       fresh owned native C place p -> Own(p.value, v)
get_mut(p):   exclusive native &mut C at p -> exclusive reborrow of p.value
write(next):  that exclusive reborrow -> Own(p.value, next) at writeback
into_inner:   consume C at p -> returned u64 equals current p.value
```

It does not justify sharing a cell or a Mutex. The required additional native-lock sequent is:

```
actual successful native Mutex construction at a live native place m,
  with its actual data subobject c = m.data and exclusive initialized state v
  [including checked platform/poison constructor bodies]
  |
  | checked one-time publication / move transport; consume exclusive state
  v
actual shared mutex invariant at m, owning c's current physical contents,
  tied to the very same native platform lock and poison state
  |
  | original native lock acquisition, including contended/poison/error paths
  v
actual guard g with g.lock = &m, unique acquisition right for c
  |
  | original UnsafeCell::get + guard Deref/DerefMut
  v
valid native pointer to c.value, justified provenance/liveness/alignment,
  appropriately shared/exclusive T access for the returned lifetime
  |
  | writes through that returned reference; end reborrow
  v
updated state for that same c; actual guard Drop completes poison handling
  and native unlock restores the invariant (including relevant unwind paths)
```

No step in this chain follows just from `&mut MutexGuard`: the guard contains a **shared** `&Mutex`, not an `&mut Mutex` or `&mut UnsafeCell`. Other threads can retain shared references and wait on the same lock. Native pointer/store correspondence and mutual exclusion justify access; Rust's ordinary exclusive field-borrow rule alone cannot cross this boundary. Guard `Deref` also needs a valid shared inner borrow; it is not automatically covered when only DerefMut is considered.

The invariant must be about the same physical object as the guard's actual lock field. A second PCell, `PointsTo` from a different constructor, equal bytes, a stable crate hash, or an equal Rust layout does not discharge that fact. Once shared, there must be no persistent equation tying the native cell identity to its initial v. Later acquisitions observe the released current state, not a forever-framed constructor snapshot.

## Existing principles: reuse only after this sequent

`vstd::cell::pcell::PCell::new` is an existing trusted constructor for its own ManuallyDrop<UnsafeCell<T>> plus permission. Its PointsTo::id/value and borrow_mut operations belong to that object. PCell differs in drop and Send/Sync behavior. It cannot be allocated alongside the original Mutex and declared equal to mutex.data. The deprecated top-level cell facilities have the same object-connection issue.

`vstd::raw_ptr::ptr_mut_ref` already states the needed shape of raw-access precondition: a matching initialized PointsTo at that pointer, with final writeback. It does not manufacture the matching PointsTo for UnsafeCell::get. Even using it proof-only requires a checked conversion from exclusive same-native-object state and a checked restoration discipline. Replacing the original runtime raw operation with a PCell/raw_ptr library call is outside scope.

Existing LocalInvariant/AtomicInvariant/atomic ghost principles may host state **after** a checked connection to the actual native cell and native futex/atomic operations. Merely initializing a ghost lock with v or using a separate vstd atomic does not relate it to the original std Mutex. The pinned futex fast path alone cannot cover its contended path, release/wake, poison and unwind. None is granted a new assume_specification here.

## Decision

There is no obstruction to the narrowly closed owned/exclusive translation on these actual three ordinary operations. The concrete soundness obstruction is to promoting that immutable datatype interpretation through shared access. The native hook above is a counterexample to that promotion, not a proof that the owned route is infeasible. Implementing/reviewing that route can remove a necessary frontend primitive blocker; it does not settle publication/acquisition/raw access/release or the crash theorem.
