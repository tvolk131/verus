# Validation v4: semantic and trust inventory

The theorem covers results of the checked native validation call in the stated
positive/initial domains. It does not give every constructible Signed value an
authenticity invariant: Decode and other construction paths exist.

| Boundary | Declared behavior and evidence |
| --- | --- |
| Index | `lawful_index` defaults to false. When true, `index()` returns `observed_index()`. The actual FullUnit delegation and unrelated Packet implementation discharge this obligation. |
| Keychain | `lawful_verification` defaults to false. When true, `node_count()` returns `members()` and `verify(bytes, signature, index)` returns `index < members && accepted(bytes, signature, index)`. Outer validation requires this explicit interface. No production cryptographic backend is automatically opted in. |
| Reference provider | The checked proof-only ValidationExampleKeychain accepts signature 37 at in-range indices, independently of bytes. Accepted/rejected signature goals, universal outer error-order goal, and a natively constructed successful unit demonstrate a nonvacuous interface and path. This provider is not a cryptographic algorithm. |
| Signable/Hasher/AsRef | Typed callable interfaces, without hash, encoding, determinism, collision-resistance or byte-content equations. The signature contract retains an existential witness of the slice used in verification, without recomputing a hash or assuming repeatability. |
| Codec | Existing opaque `using_encoded` requires callback safety for arbitrary bytes and supplies no result law. New size_hint, encode_to, encode, Decode::decode, Input, Output, WrapperTypeEncode and Error signatures supply no encoding equations. The first-party Signed/UncheckedSigned/SignatureError derive bodies are checked against these interfaces. |
| Locking/cache | Existing opaque read/write/guard declarations plus typed-only RwLock::new and try_read. No stored-value, lock consistency or hash-cache correctness relation. FullUnit's required observation excludes its cache. |
| Clone | Actual FullUnit clone and PreUnit/ControlHash derived clones are checked. Error policies use their semantic clone relations, preserving coordinate/session, collection clone relations and optional-data occupancy/payload clone relation. No identity law for arbitrary Clone or hash/data payloads is added. |
| Conversion | The original SignatureError-to-ValidationError From body is checked. An explicit FromSpecImpl binds the standard conversion interface to WrongSignature with the same unchecked payload; it is not an assumed implementation. |
| Arithmetic domain | Outer validation requires `members <= usize::MAX / 2` for positive rounds, retaining the prior threshold-overflow domain. Signature rejection supplies the creator-bound fact before positive-round metadata validation. Initial rounds retain the existing opaque hash-comparison policy. |
| Standard library/logic | Existing Verus/vstd models, Rust compiler, solver and proof pipeline remain the underlying trust boundary. Vstd was reverified: 2,215 results, zero errors. |

The source inventory lists six new `assume_specification` declarations: codec
Error::chain, generic wrapper size_hint/encode/encode_to, RwLock::new and
RwLock::try_read. Every one has no requires/ensures equation. External trait and
type declarations are separately visible in the accepted syntax inventory.
No assumed contract is added for a first-party wrapper, accessor, clone,
signature check or validator implementation.

Authored input consists of semantic observations, required identity/error
policies, conditional interface laws, checked trait/conversion bindings and
example goals. The new native bodies contain no authored proof blocks, local
assertions, loop invariants or trigger annotations. The two engine extensions
are general profiles with unrelated cross-crate regressions. Error callback
assistance proposes an ordinary contract and preserves the closure body; it does
not confer an exact-inference certificate on the ordinary validator.

All 179 comparable bodies in the eight baseline source files are token-identical
to accepted v3. FullUnit::new was moved to its own selected impl block. Two Clone
exclusions were removed; none was added. All 35 prior inferred helpers and all 84
previously checked items remain in the compiled closure. This is a finite native
slice, not verification of Aleph's complete networking, protocol safety/liveness,
serializers, crypto backends or unannotated callers.
