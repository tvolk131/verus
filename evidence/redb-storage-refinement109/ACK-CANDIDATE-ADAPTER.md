# Exact later composition obligations (not implemented / not trusted)

This lane proves an authored finite-outcome CFG generates the ordered SUCCESS
markers 23 < 24 < 27 < 28 < 31 < 46 on public Ok in its local mode. Lane1's
pending-cache interpreter, whatever its eventual checked interface, must not be
identified with these markers by equal labels or matching bytes alone.

The needed checked simulation is a stateful adapter with these obligations:

1. **Entry/resource and mode relation.** Relate the actual WriteTransaction,
   TransactionalMemory, header guard, cache, backend and physical image to BOTH
   interpreters' initial states; establish Immediate, single-process, unchanged
   geometry, ownership/non-aliasing, initial dirty-barrier/history and valid
   root/header premises. Derive branch observations, not oracle them as accepted
   native facts. Preserve the strict ID/assertion obligations. This is OPEN.
2. **Write admission at markers 23 and 27.** Prove that the two actual
   `write_header` calls at P1837/P1859, via P1617-1628 and the cache `write` API,
   admit exactly `header.to_bytes(true)` at offset0, length DB_HEADER_SIZE=320,
   to the SAME cache instance modeled by lane1. Include fallible header holds,
   mutable buffer copy and termination/cleanup effects. WritablePage/Drop/latch
   semantic bindings remain OPEN: this document does NOT implement that pipeline.
3. **Full-header framing.** Prove actual preparation/swap/mode serialization
   relates the two admitted byte arrays and native roots to the selected
   existing header_order_model relations. In particular, establish the payload
   frame outside the atomic god byte, simultaneous selector/mode update and
   unchanged geometry; do not substitute a god-byte-only write for the real
   whole-header copies. Existing symbolic slot projection is not this lemma.
4. **Flush expansion at markers 24 and 28.** For each actual `storage.flush()?`
   returning Ok, bind it to the ENTIRE lane1 drain program plus backend sync,
   including all stripes, coalescing and retained failing-stripe cases. Its
   successful result must establish no pending application writes from that
   call, and the precise backend-visible/durable relation of its checked model.
   No `sync_file`, write_barrier, or queued-copy-only substitution is permitted.
   Connect that model to persistence_model::sync_success only with checked
   backend primitive bindings; those bindings are still OPEN.
5. **Error and stop simulation.** Relate each native Result Err to the selected
   summary call or expanded lane1 successful-prefix failure, WITHOUT restoring
   old storage by default. Preserve completed barriers through post-publication
   error paths 35,37..41. Relate concrete panic/abort/crash/resource outcomes
   separately; this lane's nondeterministic stop labels are not such a proof.
6. **Continuation/history frame to public return.** Prove manager bookkeeping,
   durable_commit continuations, optional post-commit reclamation/nondurable
   epilogue, helper assertions and successful latch disarm preserve the earlier
   acknowledged durable root/history property. The epilogue may install a newer
   NON-DURABLE in-memory root; do not equate it to the main durable root. Show
   public Ok really follows this frame and native return/destruction behavior.

A useful eventual lemma shape is a checked finite-prefix simulation preserving
an explicit relation `R(native, cfg_state, cache_state, StorageImage)`: each
successful CFG write expands to lane1 admission; each successful flush expands
to a successful lane1 drain+sync run; each error expands to an allowed prefix;
all other nodes preserve the appropriate durable-history frame or stop OPEN.
No such R or native producer is assumed in `ack_cfg.rs` requires clauses.

Only AFTER these bindings could the present ordered-event result be composed
with lane1's byte model and persistence/header theorems. Physical primitive
semantics, root reachability, allocator/resource authority, stale/torn slot
recovery, all histories and all18 original final acceptance cells remain OPEN.
This lane neither waits for nor certifies the parallel byte-level candidate.
