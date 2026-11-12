# Broader-goal native-state link continuation

User: “Good to keep ripping on the broader goal?” Resume application-directed work past the previous experiment's first unsupported primitive. No new trusted law, backend summary or original-runtime rewrite.45-active-minute value checkpoints remain. The full crash theorem is the objective, not an isolated Cell example.

## Concrete decomposition to evaluate before implementation

The installed core UnsafeCell<T> has one private T field. Its actual new, into_inner and get_mut bodies are ordinary aggregate construction, owned field move and exclusive field borrow. Its shared get instead forms a raw pointer, and native MutexGuard dereference requires that pointer's justified access rights. Investigate whether a closed, source-checked encoding can establish native owned construction/exclusive mutation directly while preserving the actual type and privacy. Do not treat that partial encoding as permission for shared raw access.

An owned/exclusive route would need to bind the very same native cell value, not create a parallel PCell or ask the client for a permission. It must not expose an immutable logical field snapshot through shared aliases that actual interior mutation could invalidate. Generic/drop/unsized cases, foreign callbacks, interior aliases, imports and unsupported operations must fail closed unless actually covered. New private compiler representation is a TCB change requiring independent review; calling it a definition does not by itself establish soundness.

The complete guard path additionally requires native cell access/permission, actual lock acquisition/poison/atomic behavior and release/unwind state restoration. Reusing existing vstd memory/atomic principles is acceptable only with a checked connection to the actual native object/operation. New assumed equations, supplied synthetic tokens, silent operator replacement or a proof of an unrelated logical lock are not acceptable.

The real application-value gate remains source PagedCachedFile::flush_write_buffer: entry buffers become actual successful writes with exact bytes before the successful barrier, retaining relevant old pages and all error edges. Owned-cell proof alone would be a prerequisite component, not completion or substantial application reduction. Both prior modes and all unsupported cases remain counted. HMAC/native remains DEFERRED AND UNACHIEVED.
