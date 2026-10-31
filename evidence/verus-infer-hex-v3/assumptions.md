# Trust and bounded capability inventory

The application has no assumed specification, external body, source proof block,
source assertion, manual trigger, source loop invariant, or added implementation
exclusion. All 218 upstream signatures and executable bodies are compared by a
Rust/Verus parser. Its required ASCII numeric digit/pair predicates are independent
of the actual 256-byte table; that table's initializer and lookup body are checked.

## New primitive trust

`vstd::byte_vec` adds four explicit external-body boundaries for the global
allocator and **u8 only**. Their implementations execute the named Rust primitive;
the native lowering accepts resolved core/alloc function items, never a name-only
match, closure, indirect function pointer, user shim, arbitrary element type or
allocator. Carrier stubs require false and grant no fallback permission.

* `allocate`: successful `Vec::with_capacity(request)` returns length zero and
  capacity at least request; zero request has zero capacity. Allocation failure
  or capacity overflow may panic/abort; the theorem is partial correctness and
  does not assert allocation always succeeds. Capacity is not assumed equal to
  request. [Matching Rust Vec documentation](https://doc.rust-lang.org/std/vec/struct.Vec.html).
* `acquire`: `as_mut_ptr` lends the exact live empty Vec allocation exclusively.
  A noncopy tracked token carries its identity, base/provenance, capacity, raw
  permission, and fresh ghost acquisition epoch. Nonzero addresses and capacity
  bounded by isize::MAX follow Rust Vec allocation requirements. Empty Vec uses
  a nonnull aligned dangling base and an empty permission range. A PhantomData
  exclusive borrow ties the lease to the actual Vec lifetime; normal Rust/Verus
  ownership checking forbids drop, reallocation, mutation or duplicate use while
  the write lease is live. The epoch is trusted *ghost protocol state*, not a
  fictitious native Vec field. Reacquisition invalidates earlier sealing receipts.
* `advance`: actual `*mut u8::add`, restricted to the same allocation/provenance,
  mathematical nonwrapping addresses and the closed range 0..=capacity. No
  one-past dereference permission is granted. [Rust pointer requirements](https://doc.rust-lang.org/std/primitive.pointer.html#method.add).
* `finish`: actual `Vec::set_len`, consuming an allocation/epoch-matching sealed
  receipt. Length must be no larger than its initialized prefix (which is no
  larger than capacity). The Vec view becomes exactly that prefix. A receipt
  carries no pointer write permission, and no lease survives the transfer.

`write`, its raw permission split, conversion to a typed u8 cell, initialized
value map, prefix extension, `seal`, `abandon`, and the two-byte validity lemma
are **ordinary checked vstd bodies**. The write delegates only to the existing
`raw_ptr::ptr_mut_write` boundary with its tracked PointsTo requirement. Existing
Verus raw-pointer/layout axioms and normal allocator/drop semantics remain trust
inherited from vstd. No newly trusted drain/iterator/decoder equation is added.

The acquisition and transfer boundaries bridge native Vec storage to the existing
permission model; this is not a proof of Rust's allocator or Vec implementation.
Receipt matching is deliberately lost after unrelated mutable Vec operations.
Tracked permissions are affine: on early Err they can be discarded. Native Vec
length is still zero, so drop frees the allocation without exposing uninitialized
bytes or running destructors on the abandoned initialized u8 prefix. The current
profile never unwinds between write and pointer advancement, reads raw cells,
escapes a pointer/lease, aliases/reallocates, or handles T/destructors.

## Runtime iterator length

A safe ExactSizeIterator trait name is insufficient for unsafe memory bounds.
The new opt-in exact-hint law is checked against actual selected native `next`
and `size_hint` bodies. The native Rust impl inventory rejects ignored len/hint
bodies, custom Drop, and unsupported custom adapter overrides. Selected len
bodies must verify the inherited equation. The **provided core len implementation**
uses size_hint's lower bound after asserting matching upper bound; this exact
standard-library behavior is the minimal trusted default-method rule. An authored
`exact_len` observation cannot dictate the actual runtime result.
[ExactSizeIterator requirements](https://doc.rust-lang.org/std/iter/trait.ExactSizeIterator.html),
[core default len source](https://doc.rust-lang.org/src/core/iter/traits/exact_size.rs.html).
The online Rust source matched compiler 1.99.1 / 58cf8f9c3; rust-src is not installed.
Existing first-party carrier impls receive a concrete checked false capability
when absent. No extension-trait default body is used.

## General checked assistance and erasure

`verus_verify(byte_buffer)` recognizes a finite native allocation/pointer/forward
loop/set_len shape. It proposes tracked calls and loop invariants, preserving
normal permission, lifetime, safety and body checks. Resolved primitive identity
and byte/Global instantiation are validated by the compiler. Unsupported alias,
reallocation, raw read, pointer escape or extra loop effects are rejected as
outside this profile, separately from failed safety obligations.

`byte_pair_semantics(result_sequence, digit_query)` inserts a call to a generic
checked two-byte prefix lemma. Both paths name ordinary authored policy functions;
the caller must prove its universal precondition. It assumes no ASCII, table,
first-party function result or generated summary. General sequence-index trigger
assistance supplies alternate instantiation patterns for an unchanged pointwise
proposition. It contributes no axiom or weakened requirement.

Both annotation branches pass the untouched original function to ordinary erasure
rewriting. Proof instrumentation and ghost requirements erase. Compiler --compile
and linked native smoke tests supplement the full upstream token-body comparison.
The v1 pair inference retains ordinary soundness, maximality and exactness; native
iterator/drain/public requirements are body-checked, not counted as additional
inferred exact helpers. All publication remains full-scope, proof-success gated,
artifact-bound and no-cheating checked. The byte resource profile rejects disabled
lifetime checking, including cross-crate consumers.

## Application domains

The application replay target is aarch64-apple-darwin (64-bit usize). Engine
regressions also exercise 32/64-bit models; these are not additional native
application target acceptances.

Public decode_to_vec has no input precondition: arbitrary Rust str, UTF-8 **bytes**,
empty, odd, mixed-case, invalid and non-ASCII inputs are included. Odd byte length
wins before character decoding. Internal forward drain requires cursor + remaining
pairs <= usize::MAX/2; a separate checked theorem covers every reachable native
forward string state. This excludes synthetic overwritten cursors from the drain,
not public strings. Prior generic saturation goals remain checked separately.
Fixed-array `2*N` overflow and mixed backward iteration remain outside this phase.

A failed development attempt at extension-trait defaults was exposed by a wrong
len mutation and removed. Final policy 54/cache 60 invalidates prototype exports;
only fresh replays of the corrected concrete-method design count for acceptance.
The failed attempts are tooling defects under development, not upstream hex bugs.
