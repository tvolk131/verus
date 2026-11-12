# Authorized single native-lock bridge feasibility experiment

The user explicitly answered: “Allow the one bounded checked-bridge experiment.” This is the one exception described in NEXT_SCOPE.md, not permission for a subsequent foundation chain. Start clock:2026-11-12 13:38:52 UTC.45 active minutes maximum, excluding genuine build waits; stop at the first further foundational blocker even if the time limit has not elapsed.

## Frozen target and proof relation

Target the unchanged native `std::sync::Mutex<u64>` constructor, lock acquisition, actual MutexGuard deref/deref_mut and Drop/release. The minimal writer creates the original native mutex from an arbitrary initial u64, acquires the native guard, writes an arbitrary new u64, drops the guard and reacquires to read. Desired result is exactly the new value, without a client permission/token or value-equality premise. A second minimal getter targets the actual `core::cell::UnsafeCell<u64>::get` used by native guard dereference; this is a dependency diagnostic, not a safe permission-free mutable-borrow claim.

Required bridge relation: the actual native Mutex data subobject initially contains the supplied value; an acquired native guard refers to that same subobject with justified access; a write through its actual DerefMut changes that subobject; guard release restores the same state/ownership for later acquisition while preserving unrelated storage. Poison, unwind and platform-lock behavior must not be silently assumed away. No law merely asserting the result is allowed. The minimal candidate and direct dependency probe have no extra premise or tracked input.

The existing internal-nominal factory is closed to pinned core Rev with one ordinary private field, no destructor, and no production installer. Existing foreign-body checkers cover narrow checked core operations; neither fact proves a Mutex/UnsafeCell representation or raw-borrow law. The prototype must use actual loaded/source identities and ordinary checks, not invent a structural twin as a native proof.

Before trials, freeze candidate hashes, exact installed std/core source and compiler/library artifact identities, relevant existing facilities and the accepted runtime. Run bounded ordinary rustc MIR extraction/native type checking plus ordinary/Infer accepted-runtime baselines. No native execution or abstract model substitutes for correspondence. If native guard access requires an unsupported UnsafeCell/raw-permission or synchronization primitive not covered by the existing facilities, record it and stop; do not implement that foundation.

Only research files and durable status are changed. No engine/vstd/std source edits, new trusted summaries, library-model substitution, core hook, compiler rebuild, Coq/Iris/Miri dependency or other native objective. Full CP1/CP2 remain unproved. HMAC/native remains DEFERRED AND UNACHIEVED.
