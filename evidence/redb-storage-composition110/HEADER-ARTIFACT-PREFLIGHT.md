# Header artifact preflight: matched vstd must not be mixed

The existing checked original header encoder and roundtrip provider pairs remain
available and match their successful retained applications. All 315 historical
manifest payloads authenticate, as do all 838 frozen round110 inputs. This is
content/binding preflight, not a proof replay, native instance trace, provenance
repair, accepted dependency adoption or final-cell evidence.

**Concrete constraint:** the historical recovery021 and current midpoint
`rust_verify` binaries are byte-identical, but their `libvstd.rlib` and `vstd.vir`
are not. Equal compiler binaries do not establish a complete matching dependency
set. Do not directly mix the historical `.rlib`/`.vir` provider pairs with the
round110 companion library and assume that the result is a valid import.

A distinct future checked composition can use the existing historical matched
runtime/vstd in place, with unchanged source-module copies of the storage/product
libraries reverified against that exact set. No runtime/vstd build or copy is
needed merely to attempt that check. Such a candidate must preserve the explicit
inherited `EXPLICIT_INHERITED_STD_BOOL_TCB_NOT_RUST_CORE_PROOF` qualification,
actual complete artifact pairs and all source/compiler/native-path objections.
This preflight neither performs the attempt nor promises the source libraries
will verify or the provider interfaces will suffice.

## Exact interfaces and native staging constraints

The same historical `recovery_reencoding021::DatabaseHeader` provider includes
the checked original `write_secondary_slot`, `swap_primary_slot`, `to_bytes`,
slot accessors and parser/finalizer interfaces. A future consumer can therefore
call those interfaces on one nominal header type; it need not substitute a
separately declared lookalike header or transplant a method onto another type.

The native `page_manager.rs` commit sequence at 1830-1859 prepares the secondary,
serializes the first header and flushes, then swaps the primary and assigns
`two_phase_commit = two_phase` before serializing/flushing the final header.
The first serialization still carries the **previous** mode bit. It is not valid
to assume both headers carry the new two-phase bit, especially for mixed-mode
histories. Local single-process publication sets `primary_unflushed = false`;
the shared-mode third-write path remains outside the current local product.

The acknowledgement model's `new_slot` and `old_secondary` are arbitrary integer
payload labels, not proven physical selector indices. Its retained witness uses
`new_slot = 3`, while a native selector index must be zero or one. Any new bridge
must state and check the payload/transaction relation explicitly instead of
equating these labels with `DatabaseHeader.primary_slot`. This is an interface
warning, not a blocking defect in the existing conditional ordering theorem.

The potential theorem connects successful complete final-header flush bytes to
the original header serializer/parser/finalizer's selected transaction payload.
It must retain the encoded-origin premise; parsing arbitrary medium bytes does
not establish canonical reencoding equality. In particular this is not permission
to revisit excluded noncanonical-padding work or mutation030, weaken corrupt-slot
domains, assume checksum collision freedom, or silently accept native storage.

Native buffer admission, backend/crash laws, callbacks/history, resources,
allocator/lifecycle and compiler/source correspondence remain independent OPEN
obligations. All eighteen final cells remain OPEN. The current round110 review
and its application/deadline caps are not extended by this preflight.
