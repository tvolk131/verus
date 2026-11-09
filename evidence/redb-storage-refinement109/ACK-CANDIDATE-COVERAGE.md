# Lane2 native anchors and fallible-call coverage

All anchors are recovered CONTENT, not an original native checkout. `T` is
`recovered-native/redb/src/transactions.rs`; `P` is
`recovered-native/redb/src/tree_store/page_store/page_manager.rs` under
`/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007`.
`native-anchors.json` gives absolute names, full-file SHA256s, and exact retained
line excerpts. The checked program is authored `ack_cfg.rs`, NOT these Rust bodies.

## Interpreter conventions

`signal=0` completes the selected abstract action; `1` reports a Result error
only at an enabled fallible call (otherwise protocol-OPEN); `2/3/4` stop with
panic/abort/crash, separately, at ANY running node. Stopped states absorb all
later signals. Normal successful milestones alone append to `events`.
`call_spine` records successful enabled nodes, including non-milestone actions;
an error's retained `pc` identifies the failing abstract call. Neither sequence
asserts a backend write occurred. Input signals are outcomes, not finished traces.
All callee-internal Result implementations are ABSTRACTED, even when the outer
call/error propagation is COVERED. Loops may diverge in Rust; finite summary
completion does not establish native termination or interleavings.

| PC | Native anchor / action / conditional | Result/control coverage; native semantics |
|---|---|---|
| 0 | T1752-1762 public commit -> commit_with(None), completed=true | Successful control node; original ownership/Drop OPEN |
| 1,2 | T1763-1766 is_poisoned; abort_inner? then TransactionPoisoned | Both poison guard edges; abort success and failure both public Err. T2025-2059 rollback internals ABSTRACTED, allocator/resource binding OPEN |
| 3 | T1770-1774 arm AllocatorStateLatch, call helper | Control node; arm/unwind/Drop semantic binding OPEN, no destructor implementation |
| 4 | T1785-1794 quick_repair OR multi-process forces two_phase | Actual effective guard expression computed, not assumed from requested mode |
| 5 | T1798-1800 restored_transaction Some -> discard deferred records | Guard represented by restored boolean and enabled action; record/history effects OPEN |
| 6 | T1802-1803 flush_and_close? | Error propagation COVERED; roots, dirty pages and allocator producers OPEN |
| 7 | T1808-1811 Immediate -> take/adopt post-commit allocations | Conditional enabled action; allocator/resource semantics OPEN |
| 8 | T1816-1824 Immediate store_data_freed_pages? | Immediate Result edge COVERED. None route stops at explicit status8 OPEN rather than receiving durable credit |
| 9 | T2069-2071 loop pending data-freed records, each store...? | All loop Result exits ABSTRACTED into one summary result, zero/many iterations and record effects OPEN |
| 10 | T2074 header_hold(None)? | Separate Result node; P1021-1029 lent/owned guard implementation ABSTRACTED; no ownership proof |
| 11 | T2075-2081 oldest_active_transaction...? | Separate Result node; local Ok(local) / multi-process scan internally ABSTRACTED (P1036 onward) |
| 12 | T2082 process_freed_pages? | Separate Result summary; all internal frees, horizon and repeated table failures OPEN |
| 13 | T2086 flush_data_allocated_pages? | Separate Result summary; savepoint horizon and allocations OPEN |
| 14 | T2091 flush_table_root_updates? | Separate Result summary; table roots OPEN |
| 15 | T2092-2094 delete allocator-state table, mapped error? | Separate Result summary, mapping ABSTRACTED |
| 16 | T2098-2132 quick_repair && !needs_repair snapshot/retry closure? | Guard computed by enabled_action. Entire create/reserve/store/try-save/last/remove retry CFG ABSTRACTED as a fallible summary; native retry termination and allocator correspondence OPEN |
| 17 | T2135 finalize_dirty_checksums? | Separate Result summary; checksum byte and root correctness OPEN |
| 18 | T2139 -> P1797, P1809-1810 dirty assertion; check_io_errors? | Distinct manager entry Result edge. Dirty-page assertion and mutex failures can stop via panic; their truth is NOT established |
| 19 | P1814-1818 optional try_shrink? | ShrinkPolicy::Never local branch proceeds. Any enabled shrink stops at explicit geometry-OPEN status6; even shrink-enabled/no-change and errors are NOT claimed covered |
| 20 | P1820-1828 clone/release; assert new transaction newer | Guard failure produces PANIC, not Result Err. Header clone/state/resource binding OPEN |
| 21 | P1830-1831 save old ID; prepare secondary | Preparation control milestone. Actual root fields, slot bytes, valid old history and accessors OPEN |
| 22 | P1836 first header_hold? | Separate fallible node |
| 23 | P1837 first write_header? -> P1617-1628 | Separate fallible node; successful marker 23. Local storage.write? / mem_mut / copy and implicit page cleanup bindings OPEN; multi-process write_direct branch OPEN |
| 24 | P1841-1843 if two_phase storage.flush? | Separate enabled Result node; success emits first-barrier marker 24. One-phase skips it. Drain/sync/byte effects NOT established here |
| 25 | P1847-1854 swap_primary, set phase, set primary_unflushed | Ordered control node; actual atomic selector write and encoded fields OPEN |
| 26 | P1858 second header_hold? | Separate fallible node |
| 27 | P1859 second write_header? | Separate fallible node; successful marker 27, not a backend-write claim |
| 28 | P1861 final storage.flush? | Separate Result node; success emits final-barrier marker 28, not durable-byte proof |
| 29 | P1862-1873 shared third hold?/write? and optional multi-writer record | Single-process skips. Multi-process stops at explicit status7 OPEN; third-call failures/visibility protocol NOT inherited |
| 30 | P1875-1877 if shrunk storage.resize? | Local fixed geometry skips. Resize/error behavior outside local domain remains OPEN (already stopped at 19) |
| 31 | P1880-1891 unpersisted.clear, assert old secondary ID, install header, return | Successful publication marker 31. Mutex/assert/bookkeeping may panic; generic panic signal explicitly retained, concrete assertion predicate and state binding OPEN |
| 32 | T2147-2161 take allocated tracker, clear pending, free system pages, drop lock | Non-Result continuation summary; loops, resource/free/Drop semantics OPEN, panic/abort/crash stops retained |
| 33 | T2162 apply_savepoint_state_on_commit | Separate continuation control node; savepoint effects/locks OPEN |
| 34 | T2167-2174 post_commit_free Enabled && !multi_process | Both actual guard edges; false skips to helper assertions, true enters post-publication epilogue |
| 35 | T2180-2217 process_data_freed_pages_after_commit: horizon clamp, extract_freed_pages? | Post-publication Result error COVERED as summary; horizon/read/savepoint races/free effects OPEN |
| 36 | T2218-2220 if !freed_any return Ok | Both actual guard edges; false skips remaining epilogue work |
| 37 | T2222 flush_table_root_updates? | Post-publication Result node |
| 38 | T2223-2228 store_system_freed_pages? | Post-publication Result summary |
| 39 | T2230 finalize_dirty_checksums? | Post-publication Result node |
| 40 | T2236-2244 take/record epilogue allocations, mem.non_durable_commit? | Post-publication Result summary, may install a later NON-DURABLE root. No claim that current visible root equals the durable main root |
| 41 | T2245-2252 reserve ID, register_non_durable_commit? | Late post-publication Result summary |
| 42 | T2257-2260 mark freed IDs processed; epilogue Ok, durable_commit T2177 Ok | Non-Result continuation summary; tracker semantics OPEN |
| 43 | T1842-1859 helper system/data freed-page emptiness asserts | Both predicates explicitly checked; failure is PANIC, not Err |
| 44 | T1861-1867 optional logging, helper returns Ok | Success control node; logging/panic semantics OPEN |
| 45 | T1775-1777 successful result disarms latch | Success control node; actual latch/resource binding OPEN |
| 46 | T1778 -> T1767 -> T1753 public normal return | Successful public acknowledgement marker 46, strictly later than manager publication. Original return-time destruction/abort/panic correspondence OPEN |

## Scope boundaries

Fixed geometry is an explicit local fragment, NOT a weakened full-cell gate.
One-phase is represented with its first flush disabled and is explicitly outside
`local_mode`. None and multi-process have named OPEN frontiers, not an inherited
two-phase theorem. Quick-repair may force two-phase even if the request was false.
The config predicates are abstract branch observations, not proved producers
from native state. Initial roots, native histories, allocation/resource authority,
cache admission and byte semantics remain OPEN. No requires clause asserts any
of those native bindings or licenses a native final-cell claim.

`secondary`/`secondary_valid` are a small AUTHOR-DEFINED slot projection, used only
for the pre-first-write stale witness (PC22). Preparation changes `prepared`, not
this entry snapshot. Its later update on first successful barrier is a reference
choice only; it is NOT a cache/backend durability law or a claimed native validity
proof. No theorem claims this projection describes post-write native bytes.

The broad symbolic theorem is over arbitrary finite outcome schedules. It does
not establish that panic/abort/crash cannot occur. Generic non-Result stops are
an over-approximation, not mechanically bound native interruption points. A
Result proof cannot discharge Rust panics, allocation failures, poisoned mutexes,
logging, process aborts, crashes, or any WritablePage/Drop/latch destructor pipeline.
