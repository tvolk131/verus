# Correspondence ledger: lane 1 candidate, not native admission

Native root is the recovered redb tree stated in PLAN.md. Every cited native
file is read-only, hashed in the selected-input snapshots; source-anchors.json
records exact ordered tokens and their current line numbers. These guards are
source-token/CFG observations, not a formal extraction or execution binding.

| Native source anchor | Authored model/proof | Established scope and missing edge |
| --- | --- | --- |
| cached_file.rs:490-495: stripe iteration, lock, cache iteration, fallible write | model_entries, model_stripes; check_entries, check_stripes | Arbitrary finite symbolic nested sequences, not sorted order; generates write attempts from the definitions. Actual LRU iteration enumeration, Mutex/Arc/borrow/std binding OPEN. No trace is required as a theorem premise. |
| cached_file.rs:495: `?` precedes the transfer loop | successful_prefix_retained; check_stripes | For any prefix length, successful writes occur before the failing attempt; that whole stripe remains pending; completed earlier stripes are empty, later stripes unchanged. Err may change and persist some bytes. Actual backend Result binding OPEN. |
| cached_file.rs:500-518: take/transfer/read-cache admission and clear | Drain event, check_stripes, stripe_drains_only_after_writes | The aggregate drain is generated only after every write in the stripe returned modeled Ok. Pending empty sequences represent cleared cache stripes. Read-cache insertion/eviction/metrics and partial take-loop abnormal states are NOT proved. |
| cached_file.rs:520-524: release flag then Ok | Clear event kind 2; check_stripes | Generated only on modeled successful drain completion, after all stripes. Atomic flag and reader-visibility bindings OPEN. |
| cached_file.rs:596-600: flush_write_buffer `?` then sync_data | model_flush; flush_refinement, flush_sync_order | No sync after a write error; a sync attempt is last and all pending stripes are already drained, including on sync Err. No panic/abort exclusion is asserted. |
| cached_file.rs:242-258 and 195-204: preflight, backend call, failure latch, Result mapping | Decision.ok and sync_ok | Preflight errors can be represented by a failed decision with no changed bytes; backend write failures also allow partial changes. Formal latch state, closed state, error identity, multi-invocation retries, and Result conversion binding OPEN. |
| cached_file.rs:64-85 and 881-910: offset keys, insertion requires absent key, write takes prior value | buffer_application, buffer_repeated; same_offset_coalesces, unbounded_pending_coalescing | Last-map-write-wins is proved for the authored pending map, backend unchanged by buffer operations. This does NOT identify native insert with replacing insert: native insert asserts absence. Actual WritablePage/lifecycle write-buffer admission/glue edge OPEN. Prohibited drop pipeline neither implemented nor investigated. |
| page_manager.rs:1617-1629: single-process whole-header write; multi-process direct branch | coalesced_pending_flush | Symbolic repeated same-offset byte buffers project to a singleton pending stripe and flush to the latest whole buffer. This projection is authored model glue, not a native producer. Native admission OPEN; multi-process branch excluded, not established. No old-secondary/prepared-slot equality is assumed. |
| file_backend/optimized.rs:372-403,445ff; fallback.rs:112-120; db.rs:69-87 | backend_write, backend_sync; write_valid, sync_valid | Native tokens delegate to write_all_at/seek_write/write_all and sync_data. These are NOT proved backend laws. All storage laws below are conditional model semantics. OS/filesystem/device/crash and std I/O bindings OPEN. |
| persistence_model.rs:StorageImage, overwrite, clean_storage, persist_some, sync_success | Image, overwrite, clean, permitted sets | Vocabulary reused conceptually; NO import or equivalence theorem is claimed. The historical issue_write leaves durable unchanged; this candidate combines a write with optional independent persistence and permits partial visible change on Err. Exact relation to the old module remains OPEN. |

## Explicit model semantics

* Pending map/stripe buffers, backend-visible bytes, durable bytes, and permitted
  crash bytes are distinct. Enqueuing a modeled application buffer changes no
  backend image. A successful write overwrites its complete range; an Err can
  change an arbitrary subset of that range. Neither whole-write atomicity nor
  rollback-on-Err is assumed.
* A decision's persisted set independently chooses per byte between prior
  durable and current visible bytes; this can persist only part of a failed or
  successful write, or previously visible writes outside the current range.
  Permitted sets accumulate visible values. Sync Ok sets durable to visible and
  cleans the permitted sets; sync Err may persist a subset without rollback.
  This is a chosen, restricted persistence schedule, not a claim to exhaust all
  historical/native crash images. Delayed persistence of other historical
  permitted values, resizing, and filesystem details are not modeled.
* Success is checked symbolically for all finite geometrically valid byte images
  and arbitrarily long stripe/cache enumerations. For overlaps, the final image
  is the exact enumeration-order fold; for a byte covered by only one entry,
  arbitrary_order_nonoverlapping_byte establishes its final durable value
  regardless of its stripe/entry position. No native non-overlap admission is
  established or silently assumed for the general fold theorem.
* Boundary::OpenAbnormal explicitly distinguishes unestablished panic, poison,
  unwrap failure, arithmetic/resource failure, and abort paths from modeled
  Returned. It carries no state/rollback/durability guarantee. No proof connects
  the native execution to Returned or proves abnormal paths unreachable.

## Refusals / inherited qualifications

This is a checked authored ghost/specification program, NOT the original Rust
body, original-body verification, trusted adapter, or accepted native producer.
Original path preservation FAILED; native Git continuity UNAVAILABLE; frozen
compiler/source correspondence UNESTABLISHED. Checking the candidate does not
repair any of them. No trust adoption, runtime build, source integration, gate
change, acceptance or promotion is claimed. S0/full-scope/Infer/import-export
barriers remain unchanged. All eighteen cells remain OPEN. No prohibited growth,
mutation, padding, or drop pipeline was implemented or investigated.
