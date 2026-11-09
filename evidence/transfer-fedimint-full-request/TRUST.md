# Trust inventory and proof scope

The external-type correspondence is an explicit new trust boundary. The
extracted proof declares exactly:

```rust
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExSha256Hash(sha256::Hash);
```

`sha256::Hash` is the actual type reexported by Bitcoin 0.32.8 from
bitcoin_hashes 0.14.0, compiled from the application's pinned registry inputs.
This declaration associates an uninterpreted logical type identity with that
Rust type. It supplies no hash algorithm, byte representation, constructor,
inhabitation, trait, equality-operation, layout or invariant theorem. It is not
a verified implementation of SHA-256 or a proof of the external type system.

The frontend's existing external-type declaration checking, Rust type/borrow
checking, native erasure, Verus lowering/backend, Z3, and existing vstd/native
Option correspondences remain trusted. No application helper is external-body,
and no unchecked application summary, arithmetic precondition, axiom or assumed fact is
introduced. The ordinary authored policy remains the intended security rule;
exact inferred equations do not establish intent by themselves.

## Narrow use of the correspondence

The new selector accepts only an explicit public exec opaque datatype with a
distinct local proxy recorded by the frontend, matching proxy ownership, zero
type arguments/parameters/bounds, one empty opaque variant, no destructor,
extensional-equality declaration, sized constraint or user-defined invariant.
It retains the identity as a typed leaf in the complete record shape. It does
not extend general value eligibility or the existing first-party opaque-value
profile. Unknown fields still fail closed.

This leaf is permitted only below a transparent shared-borrowed record root in
exact readonly inference. Root use, by-value receivers (including implicit drop),
approximate mode, mutable records and executable field
observations are rejected. Expression, place and projection checks prevent
access, including borrowing a hash field. Existing local alias, trait, callback
and readonly gates remain in force. A value can be carried as part of the
original immutable request passed to another checked readonly helper; no model
of copying, replacing, constructing, dropping, comparing or calling methods on
the external value is supplied.

The absence of an explicit destructor does not rule out implicit drop glue in
private foreign fields. Requiring a shared-borrowed receiver avoids modeling
that effect; a dedicated by-value array-query control locks this boundary.

The helper therefore needs no foreign layout or validity theorem. It neither
inspects nor changes that value. Native validity remains a property of the
real well-typed Rust reference, and the abstraction asserts no absence of hidden
foreign invariants. Invariant-bearing VIR shapes and invariant-bearing request
records are explicitly rejected. Any future operation on the hash, or mutable
transport, needs its own independently checked model and audited correspondence.

## Source, cache and import binding

The runner checks the clean application pin and full source/lock hashes, exact
seven-field item, full impl, two full method items and tuple constant. It checks
every selected registry package's version, source and checksum against that
lock, hashes its cached `.crate` archive, and compares every archived file with
the source Cargo compiles. Sources are guarded before and after the locked
build. It then freezes the generated proof/native inputs, registry source and
artifacts, engine source, compiler/library/solver tools and application inputs
during verification. The compiled provider rlib and exported interface are
independently guarded during imported-client and negative-control replays.

Record candidate dependencies already include the complete datatype descriptor,
including the external name, proxy, bounds, fields and invariant metadata.
Readonly record candidates regenerate and reprove on warm runs; no cache hit or
proof-result cache is claimed. Imported certificates fingerprint the complete
serialized exported crate, including its datatype correspondence, and retain
the required proof guarantees and trusted-model/dependency fingerprints. Rust
also checks the native provider's transitive dependency identities. Registry
source/artifact hashes are replay provenance; the inference trailer is not a
standalone proof certificate for external Rust implementations.

Cache 71 and policy 65 invalidate older candidate/certificate assumptions.
Schema 38 is unchanged because no wire representation was added. Negative
controls check the wrong real hash proxy, missing correspondence, field reads,
aliases, mutable access, trait observations, legal invariant-bearing shapes,
and corrupted policy/proof/summary/dependency/model/missing metadata. These
frontend/import rejections are not semantic counterexamples.

## Domain and next boundary

All seven original fields and their native types remain. Both amount fields
range independently over every u64; short_channel_id remains Option<u64>;
expiry is every u32; channel and HTLC identifiers are every u64; height is None
or every u32. The hash ranges over the actual external type without inspecting
its representation. Both helper entry requirements are true. The additional
independence theorem compares two requests with equal timing/circuit fields
without constraining either hash, either amount or short_channel_id.

The actual LNv2 caller reads the hash, awaits persistent contract lookup,
obtains height from RPC and crosses asynchronous operation-state, federation
and transaction effects. Those operations exceed unused opaque transport.
The preserved trace confirms amount wiring only. Its proposed funding policy
uses the helper's zero-circuit classification; it does not authenticate a HOLD
invoice/LDK origin or prove the amount field's backend provenance. That policy
remains blocked; no amount-authenticity, fee, crypto, concurrency, atomicity,
whole-gateway or stateful fund-safety theorem is claimed.
