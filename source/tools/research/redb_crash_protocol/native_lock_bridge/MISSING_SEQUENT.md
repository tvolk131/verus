# First additional foundation: native UnsafeCell access and permission

The selected route stops here. It does not establish global impossibility of every Mutex verification design.

The exact native std Mutex has fields `inner: sys::Mutex`, `poison: poison::Flag`, `data: UnsafeCell<T>`. Its native guard has `lock: &Mutex<T>` and `poison: poison::Guard`, and implements Drop. DerefMut's original body is `unsafe { &mut *self.lock.data.get() }`. Guard Drop calls poison completion then native unlock. Neither an ordinary private-field selector nor a nominal type identity supplies access rights to the UnsafeCell allocation.

The missing sequent, specialized only to lawful native Mutex<u64> inputs, is:

```
actual native construction/acquisition state for (mutex, guard)
  -> identify the very same live initialized mutex.data subobject
  -> justify the pointer obtained by UnsafeCell::get and its raw mutable access
  -> preserve/update that same physical value under the guard
  -> restore ownership/state at the actual guard drop/release
```

No client premise may simply supply that initialized permission or assert the final value. The alias/provenance, lifetime, poison and release/unwind obligations are part of this correspondence, not new preconditions to hide it. They are **unproved**, not silently excluded. Existing facilities provide no checked native UnsafeCell interpretation covering them.

The ordinary Rust compilation's actual MIR includes inlined guard DerefMut and UnsafeCell::get: load guard.lock, form `&raw const` of field2, cast to `*mut u64`, write through that pointer, drop the original guard, reacquire, obtain the same kind of pointer, read, and drop. This makes the required source operation concrete. Layout/source/MIR identities alone do not prove the ownership or value relation.

The current compiler's internal representation factory is closed to pinned `core::iter::Rev<I>`, one ordinary private field and no destructor. It has no native Mutex/UnsafeCell installer. The checked foreign-body facilities inspected do not lower this raw interior-mutable access with ownership. Merely extending the nominal field table would therefore not discharge the missing primitive.

The existing PCell control verifies because `PCell::new` creates a different object together with its own tracked PointsTo permission. Native Mutex::new instead creates its actual UnsafeCell<T> field. Constructing a PCell alongside it, reinterpreting layout, or supplying a token from the client would not establish that the token belongs to the native field. No such substitution or synthetic permission was attempted. The original Mutex's native representation/algorithm was not replaced.

The accepted runtime rejects both native UnsafeCell type/use and UnsafeCell::get. Installing the compiler-suggested assume_specification would add an unproved library summary; it was not done. Developing a checked native UnsafeCell/physical-permission interpretation would be another foundational task beyond the authorized experiment's first-blocker stopping rule. No implementation of it was started.
