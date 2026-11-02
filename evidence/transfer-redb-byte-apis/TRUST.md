# Native u64 little-endian trust inventory

This extension assumes Rust primitive behavior at exactly two new boundaries in
`source/vstd/bytes.rs`. Rust core/std implementation bodies are **not formally
verified** here. Both ordinary Verus and Infer verification are conditional on
these contracts, the existing Verus/vstd models and the existing backend.

| New boundary | Assumed behavior and justification |
| --- | --- |
| `u64::to_le_bytes(u64) -> [u8; 8]` | The array view equals the existing `spec_u64_to_le_bytes(x)`: length eight, and byte `i` is `((x >> (8*i)) & 255) as u8` for every `0 <= i < 8`. This is Rust's documented portable little-endian conversion, including on big-endian hosts; it is not native-endian encoding. Every u64 input is admitted. |
| `u64::from_le_bytes([u8; 8]) -> u64` | The result equals the existing `spec_u64_from_le_bytes(bytes@)`, the OR of all eight widened bytes at shifts `0,8,...,56`. Rust's fixed-array argument has exactly eight bytes and admits every bit pattern. No validation or application-domain restriction is assumed. |

Both declarations also assume `no_unwind`. These safe, infallible primitive
conversions have no indexing/length failure or allocation; their inputs and
outputs have fixed primitive layouts. The specification concerns numeric
conversion only, with no unsafe memory, alignment, aliasing or I/O contract.
See Rust's [`to_le_bytes`](https://doc.rust-lang.org/std/primitive.u64.html#method.to_le_bytes)
and [`from_le_bytes`](https://doc.rust-lang.org/std/primitive.u64.html#method.from_le_bytes)
API contracts. The replay records the actual Rust toolchain and core/std artifact
hashes, and tests native boundary execution on the recorded aarch64 macOS host.
Native testing supports this connection; it is not a universal proof of Rust.

No roundtrip axiom is added. The existing
`lemma_auto_spec_u64_to_from_le_bytes` supplies both inverse directions and is
reverified with vstd. Both existing u64 mathematical definitions become open,
with unchanged formulas, so ordinary verification can check Infer's generated
array/index/cast/finite-bitwise relations. The old `spec_u64_to_le_bytes_open`
entry point becomes a compatibility alias of that same definition; its existing
equivalence proof is retained. The slice/Vec executable wrappers remain separate
entry points with their preexisting external bodies.

Infer's recognition is closed over a core numeric inherent identity, its matching
vstd byte proxy, a static executable item, no generics, the concrete value
parameter/result types above, no precondition, `no_unwind`, no executable model
body and no unsafe/async/infer marker. Concrete source call types are checked
before mathematical summaries are restored to primitive types. No law is
assigned by an application's function name. Other widths, byte orders, slice
helpers and lookalikes receive no model through this extension.

Candidate generation adds no trusted executable. Every candidate retains ordinary
soundness, safe-domain maximality and full-output exactness. All array bytes are
observable through the existing array equality model. The existing array
extensionality axiom, views/indexing and bitwise backend remain dependencies;
this extension adds no array axiom, unchecked proof decoration or publication
exception. Cached output permutations fail fresh soundness, and a restrictive
cached precondition fails fresh maximality before callers/publication.

The approved-model registration uses the existing dependency machinery. The
model declaration and **every byte of the imported vstd artifact**, including
its mathematical definitions and proved lemmas, participate in model fingerprints.
The driver includes those fingerprints in cache configuration; model and callee
keys also enter candidate provenance. Provider interfaces retain model fingerprints
and imported inference dependency fingerprints; clients validate both before
using contracts and receive fresh local required checks. This is Verus's existing
trusted-library metadata mechanism, not a signed or portable proof certificate.

Policy `candidate-bound-evidence-v55` and cache
`verus-infer-v61:u64-le-bytes:4096` identify the expanded approved-model boundary.
Metadata schema 38 and cache formula types are unchanged: the extension emits
only the already-supported array/index/cast/bitwise vocabulary. No representation
change warrants a schema bump. A policy bump rejects previously certified
interfaces across the changed model boundary; a cache bump prevents old candidate
recipes from being confused with the expanded synthesis policy. Full artifact
fingerprinting remains necessary even when version/commit banners agree.

The exact source snapshot, dirty patch and built artifact hashes in each replay
identify this extension. The original base SHA by itself does not identify it.
No redb body or publication rule is weakened to make the application pass.
