# Post-publication frame distinctions110

This parent-authored source observation supplements the actual candidate review;
it is not another independent review or proof. Exact selected bodies and source
identities are in POST-PUBLICATION-SOURCE-OBSERVATIONS.json. The lexer is the
already retained round109 utility, not a Rust type/effect checker. All current
838 selected inputs were reauthenticated before producing these observations.

## Concrete distinction

In recovered page_manager.rs:1897, non_durable_commit checks dirty-page/I/O
frontiers, extends unpersisted-page tracking, invokes storage.write_barrier,
updates the in-memory secondary header with the supplied data/system roots and
sets read_from_secondary=true. The selected body contains no direct write_header,
flush or sync_data call. This does NOT prove all transitive callees or concurrent
effects are backend-inert.

In cached_file.rs:612, write_barrier conditionally sets committed_pages_buffered
when pending bytes exist. It is not the flush_write_buffer-plus-sync_data sequence
performed by flush at :596. A product must not count it as another durability
barrier or identify the newer visible in-memory root with the acknowledged
durable root. This concrete distinction was sent to the composition implementer
while its original task/budget remained unchanged.

The enclosing transaction epilogue also extracts/frees pages, creates system-tree
updates, finalizes checksums, records allocation tracking and invokes tracker
callbacks. Source transactions.rs:2147-2174 has callbacks after manager publication;
:2180-2259 contains the optional fallible epilogue. Explicit free_helper in
page_manager.rs:2060 mutates allocator state and invokes cache invalidation/pending
write cancellation. Their resource, address-range, callback/abnormal and effect
bindings are OPEN. No destructor, WritablePage admission or excluded pipeline is
implemented or traced. No full durable-image frame follows from these observations.

## Useful proof cuts, without adopting them

1. A checked barrier-time image/history snapshot can preserve the fact that a
   modeled successful flush occurred even when later control returns Err.
   This is weaker than present-time native durable-state preservation.
2. Full-image equality through every continuation is not the only prospective
   frame. The final native theorem needs the acknowledged reachable-root/history
   property preserved while permitting unrelated allocation/cache/system changes.
   Actual resource authority and footprint separation must establish that frame;
   no such predicate is silently accepted as a native fact here.
3. Updating the in-memory secondary/read selector in non_durable_commit must be
   related to that semantic frame, not incorrectly modeled as a physical flush.
   System-tree admission, eviction/backend effects, ownership and all native
   continuation outcomes remain separate missing bindings.

The unchanged eighteen-cell target therefore still needs native cache/backend/
callback/history correspondence. Source tokens, snapshots and conditional model
composition are not an accepted native adapter R or final-cell closure.
