# Source-refinement ledger: inspected is NOT proved

Native Fedimint pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
Read-only root: `/private/tmp/verus-fedimint-early-returns-jyfO2M/fedimint`.
Whole-file hashes are fixed in the runner's `SOURCE_HASHES` and receipts;
all 1,199 native source ledger entries are independently audited each run.
No production file is modified, compiled as this model, or proved by this cycle.

| Native location | Inspected step | Authored analogue | Proven native correspondence / remaining obligation |
| --- | --- | --- | --- |
| `fedimint-server/src/consensus/engine.rs:924` | Only `SnapshotTooOld` is retried by the outer attempt loop; other DB errors assert | Unsuccessful atomic commit can retry with a fresh `start` | NONE. Prove precise error classification, no-write failure and fresh snapshot correspondence; the model's lost-ack path is recovery replay, not a claim that this loop retries arbitrary ambiguous errors. |
| `engine.rs:974` | Attempt begins `self.db.begin_transaction`; `ignore_uncommitted` is called | One transaction, idle to absent-marker check | NONE. Prove transaction creation/read isolation and lifetime mapping; helper `ignore_uncommitted` only suppresses diagnostics. |
| `engine.rs:987` | Lookup `AcceptedItemKey(item_index)`; matching item AND peer returns accepted early | `start` and `decision` | NONE. Prove representation/equality reflection, faithful lookup and no durable/module action on this branch, including async effects. Authored `replay_decision` is checked separately but not an unchanged native slice. |
| `engine.rs:992` | Different item/peer returns an inner rejection | `start(...).reply == 2` | NONE. Same representation/read/control-flow obligations. No item-content inference is credited. |
| `engine.rs:998` | Process item with borrowed `dbtx.to_ref_nc`; rejected item returns before commit | `stage_effects` and `abort` | NONE. Prove all module writes are confined to this same transaction, successful effects map to the bundle and rejected staged writes cannot escape. |
| `engine.rs:1016` | Enable uncommitted warning, insert `AcceptedItem {item,peer}` in same apparent transaction | `stage_marker` | NONE. Prove no marker overwrite, same-index identity and exact shared transaction aliasing. Warning flags are not commits or rollback. |
| `engine.rs:1033` | Module audits and net-assets assertion precede commit | No analogue | EXCLUDED: all audit bodies, balance reasoning, overflow, panics and side effects. No progress or financial claim. A refinement would need to handle these. |
| `engine.rs:1069` | `commit_tx_result().await?`, then metric update and accepted result | `commit(takes_effect, observed)` | NONE. Prove cancellation/linearization/error-return/recovery and acknowledgement mapping. Metrics are external to the model. |
| `engine.rs:1095` | Module processing receives a module-prefixed transaction | Opaque bundle footprint | NONE. Prefixing observed, not a proof of write confinement, global-token discipline, module purity, or lack of external effects. |
| `fedimint-core/src/db/mod.rs:1438` | `BaseDatabaseTransaction` takes raw transaction once, awaits raw commit and submits notifications on success | Atomic DB boundary | NONE. Prove consuming alias discipline, notification exclusion and backend behavior. |
| `db/mod.rs:1477` and `db/mod.rs:1769` | `CommitTracker::drop` logs; ignore/warn only change diagnostic flags | No durable action / abstract abort | NONE. No explicit rollback happens in this logger. Rollback/discard on early return depends on ownership/drop/backend; not established by logging. |
| `db/mod.rs:1846` | Marks tracker committed, awaits backend; success drains `on_commit` hooks | `commit`, with hooks excluded | NONE. Tracker flag is not a durability certificate. All hooks and panics/external effects are outside this result. |
| `fedimint-rocksdb/src/lib.rs:291` | Snapshot option and `WriteOptions::set_sync(true)` used | Single-attempt snapshot and durability premises | NONE. Observing synchronous writes is NOT proving durability, crash recovery or atomicity. Fsync alone cannot imply atomic all-or-none transaction behavior. |
| `fedimint-rocksdb/src/lib.rs:457` | Calls RocksDB transaction `commit`; Busy maps to WriteConflict, TryAgain to SnapshotTooOld | Linearizing all-or-none `commit` | NONE. Prove actual backend atomic batch, conflict checks, failure semantics, WAL/flush/recovery and hardware guarantees. |
| `fedimint-rocksdb/src/lib.rs:81` | Recovery configured `TolerateCorruptedTailRecords` | No independent implementation analogue | NONE. Tail recovery is not verified; comments are not axioms of atomicity or persistence. |
| `fedimint-server/src/consensus/db.rs:21` | `AcceptedItemKey(pub u64)`; value is AcceptedItem | Fixed `State.key` and marker payload | NONE. It is an INDEX. No native session field is invented by the model. |
| `engine.rs:775` | `complete_session` removes AlephUnits and AcceptedItem prefixes and writes signed session outcome in a transaction | OUTSIDE current marker lifetime | NONE. Need separate cleanup, session/index reuse, recovery and old-effect lifecycle contract. This is why no forever-exactly-once theorem is claimed. |

## Locally available Rust/C++ binding inspection (not verification)

The native lockfile selects rocksdb 0.24.0 and librocksdb-sys 0.17.3+10.4.2.
No dependencies were fetched or built. Four installed package source files were
read, with full hashes in `inspected-dependencies.json`, guarded before/after.

- `rocksdb-0.24.0/src/transactions/transaction.rs:141`: consuming `commit` calls
  `rocksdb_transaction_commit`; at line 191 explicit `rollback` calls the rollback
  FFI, documented to discard batched writes; at line 895 `Drop` calls destroy.
- `librocksdb-sys-0.17.3+10.4.2/rocksdb/db/c.cc:6667`: commit/rollback dispatch to
  the transaction object; destroy at line 6684 deletes it. Early-return cleanup
  thus is not a call to the Fedimint warning logger's rollback operation.
- `rocksdb/utilities/transactions/optimistic_transaction.cc:51`: default
  destructor; Commit dispatch at line 60, serial/parallel validation paths;
  explicit Rollback at line 150 calls Clear. Commit paths call DB Write or
  WriteWithCallback on a write batch and clear on success.
- `rocksdb/utilities/transactions/transaction_base.cc:83`: destructor releases
  snapshot; Clear at line 88 clears write/commit batches and tracked locks.

These inspected calls identify the rollback/commit boundary. Their implementation,
destructors, member ownership, DB Write, C++/FFI, crash behavior and WAL recovery
are NOT checked correspondences here. In particular, source inspection of a
batch call is not a proof that all recoverable images are atomic and durable.

## What actually IS proved

All 32 authored definitions/lemmas are included in the selected model scope.
The 16 checked proof bodies establish initial/step/reachability invariants,
volatile staging and abort frames, recovery pairing, atomic-branch images,
matching replay/no extra bundle, mismatching replay rejection and lost-ack replay.
Two proof bodies CHECK counterexample witnesses for non-atomic writes and
missing dedup. One authored executable decision helper is checked against its
authored spec. Every function identity and body hash is in `scope.json`.

Proved native source-to-model correspondences: **ZERO**. Unchanged native pure
slices proved: **ZERO**. Inferred native functions: **ZERO**. A faithful forward
simulation from the unchanged async program, including memory, cancellation,
transactions, modules, marker lifetime, errors and persistent backend, is a
remaining obligation, not an imported or assumed theorem.

No Infer blocker was exercised by this ordinary model-only crate. Existing
full-async/native translation blockers remain proposals, unchanged; this cycle
does not broaden the engine or claim native inference capability.
