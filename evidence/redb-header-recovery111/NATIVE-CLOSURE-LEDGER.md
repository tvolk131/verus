# Native closure ledger for the eighteen-cell objective

This parent-authored source audit accompanies the two independent proof lanes;
it is not their verdict or a new imported/native theorem. The companion
`NATIVE-LINKAGE-SOURCE.json` captures exact frozen input identities and line
excerpts. There was no new verifier application, binding trace or native run.

## What this round can connect

Historical recovery021 imports share one nominal `DatabaseHeader` type and
checked original-method contracts. New consumers can use those contracts to
establish exact prepared/final header encodings and then call the original
parser/finalizer on supplied bytes. The unchanged complete-flush product can
establish that its model's public Ok leaves the final encoded header durable.
This would remove the product's previously uninterpreted transaction/root
meaning. Its caller-supplied byte-origin and native linkage premises remain.

The native sequence first replaces the secondary and serializes with the OLD
mode bit, then performs the optional first flush, swaps the physical selector,
sets the new mode and local unflushed flag, serializes again and flushes. This
is not equivalent to setting the new mode in both headers. Model payload labels
such as3 are not physical header indices0/1. The native commit's increasing-ID
assertion is a runtime control point, not a global history invariant established
merely by adding a precondition to an authored consumer.

## Remaining cuts to a final acknowledged-durability cell

1. **Native header-to-buffer correspondence.** The local `write_header` source
   copies the original encoder's bytes through a writable cache handle; the
   multi-process branch uses a different direct write. Existing checked model
   headers do not establish that either actual native operation produces the
   represented pending buffer. This ledger does not restart the refused
   WritablePage/Drop/lifecycle work or adopt a new producer.
2. **Native buffer-to-backend correspondence.** `flush_write_buffer` iterates
   actual stripes and buffers, uses fallible backend writes, and only later
   transfers/drains entries. An early error can leave a written prefix. The
   checked complete model represents these outcomes but is not a proof that
   the actual Rust buffers and operations refine that model.
3. **Backend storage and crash boundary.** Native `flush` first drains writes
   and then calls `sync_data`. The exact native Result/byte/persistence/history
   relation remains unbound. Native type admission, atomic stored-value laws,
   operating-system persistence and crash prefixes are distinct obligations.
   Clearing io::Error/Location frontend failures would not discharge them.
4. **Native callback and public-result frame.** Source operations remain after
   the final successful sync and before public acknowledgement. Product110
   has an explicit frame gate and local-mode restriction, not a proof that all
   actual callbacks preserve the recovered header and reachable data footprint.
   A source call-spine observation is not outcome-complete native execution.
5. **Reachable payload and allocation state.** The imported roundtrip domain
   fixes4096-byte pages and constrains geometry, slot/root encodability and
   selected-slot normality. A single staged call does not establish these for
   all reachable native histories, all files, or the committed tree's key/value
   meaning and allocated/recoverable footprint.
6. **Program/tool correspondence and reviewed trust.** Native-path preservation
   remains FAILED, native Git continuity UNAVAILABLE and frozen compiler/source
   correspondence UNESTABLISHED. The inherited standard-boolean boundary is
   conditional, not a Rust-core resource proof or newly adopted assumption.
   Matching hashes/runtime compatibility preserve evidence, not these bindings.
   `PRESERVATION-QUALIFICATION.md` distinguishes the historical disappearance of
   original paths from matching recovered method contents and from the separate
   unproved native semantic relation; none of the three is silently conflated.

## Scope across the original property/history grid

Two-phase acknowledged durability remains the first-cell target, not a claim
that it is cheapest or almost closed. It still needs all six cuts above for the
actual native program and the corresponding final independent review. Recovery
correctness additionally needs native medium/fetch and interrupted recovery
correspondence. Error correctness needs native outcome/history and post-error
closure. Atomicity/preservation need recoverable-state relations over native
crash prefixes. Logical refinement additionally needs reachable-tree and
associated-value correspondence. One-phase and mixed histories retain their
own checksum/selection/interleaving obligations; old-mode preservation is a
necessary connection, not their completed proof.

Do not keep manufacturing additional lookup or header fixtures as a substitute
for these missing cuts. Retain any checked round111 consumer as a dependency,
but choose subsequent work by the earliest unbound native edge. No compiler
build, primitive-law adoption, native integration or refused/exhausted retry is
authorized by this ledger. All eighteen final cells remain OPEN.
