# Bounded checked native byte-vector filling

Baseline: Verus 5b8a980e9, native hex ae20db0. Hex v1/v2, the
phase3 blocker archive, and Aleph v3/v4 sources/archives/pinned engines are immutable.
Proof/native Rust 1.99.1; native MSRV 1.74.0; offline locked Cargo configuration.

Acceptance is the actual public decode_to_vec and original drain_to_vec against
the existing independent UTF-8 byte/output/error requirements. The four known
unsupported calls are inputs to this work, not its completion criterion.

Runtime length: add an opt-in, checked exact-size-hint law with a conservative
false default. Selected first-party size_hint bodies must prove exact lower and
upper values. Default ExactSizeIterator::len uses the real core implementation's
size_hint lower bound; its model must rely on this checked hint law, not trust
first-party trait names or independently authored exact_len equations. Test wrong
hints and overridden lengths with unrelated iterators and the native string path.

Memory scope: Vec<u8>, global allocator, one fresh with_capacity allocation,
local nonescaping pointer(s), monotone initialized prefix, no reallocations or
arbitrary aliases/reads. Capacity is >= request, not equal to request. Empty
allocation and one-past advancement are allowed; one-past writes are forbidden.
Arbitrary T/destructors, pointer integer casts, concurrent access, reallocation,
raw reads, resource escape and general unsafe Rust remain unsupported.

Use a noncopyable tracked allocation lease and existing PointsToRaw/PointsTo<u8>
permissions. Allocation/acquisition and final transfer are explicit minimal
standard-library boundaries tied to the actual Vec allocation and view. Checked
resource code enforces allocation identity, exclusivity, pointer provenance and
bounds, each initialized cell/value, and transfer only for an initialized prefix.
Early Err drops a zero-length byte Vec; no resource or pointer escapes.

The proposed native bridge is proof-only macro instrumentation plus resolved
callee validation, not spelling-based substitution. Marker calls carry the
original primitive function identity and tracked arguments through Rust's normal
borrow checker. A compiler check must validate actual core/alloc DefIds and u8
instantiation before selecting the resource-checked effect. Unvalidated/indirect
marker calls have an impossible domain. Erasure returns the original executable
body. Original pointer writes/add/set_len remain the production operations.
No first-party algorithm/drain equation or unchecked resource duplication is
trusted. If this design cannot pass ordinary permission/body checking, retain a
reduced architectural failure rather than replacing it with unconditional specs.

General support must first pass unrelated filling and cross-crate clients.
Negative controls: OOB/one-past writes, uninitialized set_len, invalid lengths,
permission duplication, wrong allocation/provenance, stale pointers, post-transfer
use; unsupported aliases/reallocation must be identified separately from failed
safety proofs. Ordinary inferred guarantees retain required certificates,
no-cheating and fail-closed failed/no-verify/filtered publication. Add semantic
mutations to original decoder/table/error/length behavior, fresh restored proofs,
native feature/MSRV tests and source/config/trust/erasure inventories. Application
proof counts are separate from compiler/vstd tests. Commit coherent validated work
locally, without agents, external messages, pushes or settings changes.

Design refinement before acceptance: the live write lease carries an exclusive
Vec borrow lifetime. Its non-dereferenceable initialization receipt is produced
only after consuming that lease and returning all cell/raw permissions. Each
acquisition increments a ghost Vec generation; set_len requires the receipt's
allocation identity and generation to match. This prevents Vec drop/reallocation
while a write permission is live, and invalidates stale receipts after reacquiring
and rewriting a buffer. The ghost generation is protocol state, not a claim that
Rust stores another runtime field. Native erasure remains unchanged.

Trusted primitive inventory is limited to Vec allocation, exclusive acquisition
with a fresh ghost epoch, in-bounds pointer advance, and receipt-checked set_len.
The write implementation, raw-permission split, initialization map, lease sealing
and abandonment are checked vstd code; write delegates to the existing
PointsTo-checked ptr_mut_write boundary. Native marker stubs have requires false
and grant no contract unless the compiler validates the original primitive's
resolved core/alloc identity and byte/Global instantiation. They are not fallback
unconditional assume-specifications for missing calls.

The attempted external-trait boolean default was rejected during mutation
validation: a wrong len override escaped its intended obligation. That attempt
is not an accepted capability. The replacement uses a required spec method and
concrete checked false definitions inserted conservatively in carrier impls.
There is no trait default body or altered default-dispatch frontend support.
Fresh control/application/vstd replays are mandatory after this correction.

A general checked whole-two-byte-block validity lemma may be requested using
explicit binary Result and unary Option policy paths on a string endpoint. It
proves the universal policy relation before supplying indexed-byte consequences;
it assumes no ASCII/table/first-party result. This is policy vocabulary plus
routine proof assistance, with erased native body preserved and no source proof
blocks required. The standard-library proof itself is counted as tooling.

The native constructor requirement additionally names the exact canonical input
pair sequence (equivalent to its length and element requirements). The internal
string-backed drain domain is cursor + remaining_pair_count <= usize::MAX/2.
This is satisfied by every real reachable string-backed state and the public
constructor; it places no additional precondition on arbitrary public Rust str
inputs. It excludes artificially overwritten private cursors from this drain
proof; the earlier generic iterator saturation theorems remain separate.
Within that domain, error.pos/2-cursor is the unique failing pair index, so the
first-failure requirement can be stated deterministically without an existential
witness. Native saturating operations and original bodies remain intact.

The shared expected-result sequence is an independent mathematical definition
using the retained numeric ASCII digit/pair requirements and byte-position
arithmetic. Iterator observations delegate to that required policy; their real
bodies must check it. The public byte-level predicate remains unchanged. General
prefix assistance accepts the semantic result-sequence and unary digit-query
paths, proves complete-block validity, and supplies pointwise indexed-byte facts.
This avoids requiring implementation-specific source assertions or witnesses.
