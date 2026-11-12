# Typed acceptance gates and negative programs

These are **required future tests**, not results of this design turn. No prototype is installed. Accepting a test because the current frontend rejects every UnsafeCell program is not success. Both ordinary and Infer must first admit the positive frozen bodies under the new reviewed extension.

## Gate order

1. **Frozen inputs:** Authenticate accepted78's receipt and all eight tools against BASELINE_FREEZE before private copying. Use Rust1.99.1 and original core/std, the prepared commit's source and unchanged native/u64 bodies. Rebind frozen repository-relative paths to this checkout, not live parent files. Keep the native MIR reference and the existing ordinary/Infer frontend failures as historical baseline.
2. **Actual types/source:** Reject wrong native ADT/field/method identities, instantiated type, lifetime/ABI, loaded artifact or body. A correct public name or one-field layout is insufficient. Mutate the descriptor/typed-operation fixture one component at a time, including native target place, field identity, ownership/move mode and return-borrow region.
3. **Closed unit before pruning:** Enforce the positive grammar on all local typed bodies/signatures, arguments/results, implicit adjustments/coercions, nested bodies, statics/const promotions and spec surface. Direct-call registration is not the escape checker. Reject all unknown imported/native/abstract/callback uses and all skips of affected bodies; neither `verify_function` nor a cache hit can bypass this gate.
4. **Actual full body correspondence:** Compare emitted full checked functions and places to the original typed operations. Add independently constructed corruption tests, not a second serialization of the recognizer output. Check both current and future/writeback cells. Preserve the retained native exclusive field store in exclusive_borrow.
5. **Normal verification:** Ordinary full body, lifetime, trait conflict, WF, mode, safety and erasure checks; --no-cheating in every invocation. Reject --no-verify, --no-lifetime, --no-trait-conflicts, --no-erasure-check, partial verification, added assumptions/axioms/external bodies or any shortcut to this experimental feature. Keep operator safety checks and all original input values.
6. **Ordinary AND Infer:** All three frozen goals pass in both. Explicit original contracts must verify goals-off; the new primitive must not be available only to Infer. Inventoried native helpers also pass their complete body queries. An extra identical-body state query checks final(cell).value=next for exclusive_borrow with no initial-value premise. False properties below fail as postconditions, not as frontend failures.
7. **Local-only import/export/cache:** No Internal descriptor serialization/deserialization; reject export/Cargo export even if no helper survives pruning. Seed a stale export then ensure the refused run invalidates it. Imported generic/abstract wrapper cannot smuggle a cell operation. Repeat with fresh private outputs/caches and goals-off consumers. No cache is proof authority.
8. **Native fidelity/value gate:** Erasure still calls the original UnsafeCell methods, on the same original types/places. Native MIR for the frozen body still has the native &mut field write in exclusive_borrow. Store/writeback deletion must break the additional final-state obligation. Record constructor/exclusive scope separately from scalar optimized programs. Full Mutex/guard and REDB remain unproved.

Use existing infer_experiment.py / infer_eval_support.py for execution, authenticated private bundles, fresh output/cache directories and failed-attempt retention. Extend the existing scenario's expected function/body inventory only after inspecting actual outputs; record original clients, every native body and every diagnostic query. Do not add a general certificate/evidence subsystem. Any new compiler build is a separate explicit implementation cost, not part of this design's evidence.

## Semantic negatives inside the admitted language

These should reach ordinary body verification in both modes and fail the intended obligation:

| Program/property delta, keeping all runtime calls unchanged | Expected failure |
| --- | --- |
| owned_roundtrip ensures result != value | False postcondition, all-u64 domain. |
| exclusive_update ensures result.0 == initial, with no next==initial premise | False postcondition when next differs. |
| exclusive_update ensures result.1 == next instead of frame | False frame claim, independently arbitrary frame/next. |
| exclusive_borrow ensures result != next | False postcondition after actual field store and reload. |
| Diagnostic final-state query changes required final(cell).value from next to initial | False state postcondition on arbitrary initialized input with differing next. This must not be hidden by the original function's weaker returned-value contract. |
| Corruption fixture removes native field writeback but retains a scalar result=next | State proof fails; if correspondence to original MIR is checked first, reject there as well. A successful returned-value query does not count as state correspondence. |
| Corruption fixture directs get_mut to a second distinct cell/place, with equal initial values | Correspondence fails even if scalar results match. Equal values are not the same native subobject. |

The last two are internal fixture corruptions, not changes to the base native program. Keep both verifier-level state failures and earlier correspondence rejection tests; neither alone substitutes for the other.

## Soundness/escape negatives outside the admitted language

Each must be refused with a specific typed-gate reason in both modes, before any result is certified. These are **admission negatives**, not false-postcondition evidence. The native programs below are legal Rust where indicated; rejection is the experimental model's scope limitation.

### N1 — adversarial abstract shared callback (mandatory)

Use the complete NativeHook/Write/abstract_client program in SOUNDNESS_AND_GUARD.md, and ask for result==initial without next==initial. Compile its ordinary Rust implementation as a supporting witness if needed; Write returns next. The verifier must refuse the caller's shared borrow/abstract call, including when the concrete body is in a separate native crate. It must not frame the constructor's initial field value. No native hook operation needs to appear in the caller for this test to matter.

Test direct native function, trait generic dispatch, dyn dispatch and Fn callback forms separately. A private/opaque native callback with an empty contract is equally dangerous. Repeating with a concrete monomorphized callback must not silently create an unchecked exception.

### N2 — hidden in a shared outer container

```rust
use std::cell::UnsafeCell;
struct Wrap<T> { value: T }
trait Hook<T> { fn run(&self, x: &T, next: u64); }
fn hidden(h: &impl Hook<Wrap<UnsafeCell<u64>>>, initial: u64, next: u64) -> u64 {
    let w = Wrap { value: UnsafeCell::new(initial) };
    h.run(&w, next);
    w.value.into_inner()
}
// A legal Hook implementation may write through x.value.get().
```

Refuse the user ADT/generic/shared interface. Repeat with aliases, Option, tuples/arrays, Box/Arc/Vec and an associated type. For some combinations extra library methods are needed; their unsupported calls are also legitimate reasons to reject, but include a minimal struct fixture so a shallow argument-type scan cannot pass the test accidentally. Opaque/external containers are unknown, never assumed cell-free.

### N3 — escape through a return, capture or promotion

```rust
fn shared(c: &std::cell::UnsafeCell<u64>) -> &std::cell::UnsafeCell<u64> { c }
fn raw(c: &std::cell::UnsafeCell<u64>) -> *mut u64 { c.get() }
```

Refuse signatures/body adjustments; test a closure capture of a local C, a function value stored in a local, a raw address cast in a macro, and a shared constant promotion. A raw pointer need not be dereferenced in this function to violate the positive grammar. Copying a raw pointer does not give access rights.

### N4 — generics, drop and unwind

```rust
fn generic<T>(value: T) -> T { std::cell::UnsafeCell::new(value).into_inner() }
struct HasDrop;
impl Drop for HasDrop { fn drop(&mut self) {} }
```

Refuse generic<T> even if one caller uses u64; refuse C<HasDrop>, nested C<C<u64>>, MaybeUninit<C>, ManuallyDrop<C>, Pin/container wrappers and custom Drop-bearing enclosures. A callback that can panic/execute cleanup while a cell or a guard is live stays outside this phase. Do not change panic=unwind into abort to avoid modeling it. Reject original MutexGuard drop/unwind operations rather than silently dropping their state restoration obligation.

### N5 — imported abstraction, private access and ghost reification

An external crate's `fn callback(&UnsafeCell<u64>)` with no useful postcondition must not be usable while preserving a cell snapshot. Try a generic imported identity/wrapper instantiated to C, and an imported trait method with an abstract body. Refuse unsupported calls irrespective of cached or exported summary availability. Deserialization of an Internal descriptor must fail, and source cannot opt in with an attribute.

Source attempts at `cell.value`, constructing the private core field, forging a descriptor, declaring a ghost `live_value(&C)`, saving C in Ghost/Seq/quantifiers or proving C-to-pointer identity must fail. Public/native constructors remain the only creation route; internal contracts do not make the field public.

### N6 — real lock boundary

Run unchanged native_lock_bridge/checked.rs (original native_handoff with its desired final-value contract) as an admission negative. A full native bridge must remain unsupported under this first extension. Distinguish constructor Transmute, enclosing type, shared borrow, lock acquisition, raw get and guard Drop rejection; do not count one generic frontend message as evidence all were audited. Use small typed fixtures to test each gate. The actual Mutex constructor MIR is not silently rewritten to a pure cell constructor.

### N7 — safety checks remain active

Reject overlapping live mutable cell borrows, consuming a borrowed C, returning a reference to a local cell's inner value, and forging a longer lifetime in a method mapping. Ordinary Rust/Verus lifetime failures must remain failures. Validate --no-lifetime refusal independently. Normal arithmetic overflow/WF/mode/trait negatives from the existing test harness must still fail; do not introduce unrelated broad testing or modify the native bodies to force these checks.

## Stop condition for the implementation recommendation

A new owned/exclusive result is acceptable as a prerequisite component only after all these gates and a new independent review of the frozen implementation. The next unsupported sequent remains publication into a same-native-lock invariant and recovery of physical access by the original guard. If version one requires generalized foreign-MIR certification, trait effect inference, arbitrary raw pointers, drop/unwind semantics or an installed new law, stop and checkpoint the precise obstruction; do not grow the task into that work.
