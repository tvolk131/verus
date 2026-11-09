# Source/refinement and trust ledger

Pin: `b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.
Whole inspected source hashes and six frozen items are in `source-ledger.json`.
The prior `transfer-fedimint-atomic-replay` evidence and archived independent
`transfer-storage-boundaries77/fedimint-review.md` remain unchanged. Their ZERO
native correspondences remain accurate for that historical model-only milestone.

## Newly checked pure-body correspondence

| Native selection | Executable fidelity | Checked result | Authored/unproved boundary |
| --- | --- | --- | --- |
| `db/mod.rs:712`, `PrefixDatabase::get_full_key` | Entire original Vec/slice body, byte-identical | Vector view is prefix followed by suffix | Contract/view are AUTHOR-OWNED; std/vstd clone/extend specs are trusted |
| `db/mod.rs:767`, `PrefixDatabaseTransaction::get_full_key` | Entire original Vec/slice body, byte-identical | Same exact concatenation | Same library/spec boundary |
| `db/mod.rs:773`, `PrefixDatabaseTransaction::get_full_range` | Entire original Range body and calls, byte-identical | Start and end are each prefixed exactly | No lexicographic scan, range-membership, stream slicing or backend theorem |
| `prefix.spec.rs`, `goals.verus.rs` | Entirely AUTHOR-OWNED, checked | Membership, cancellation/injectivity, conditional disjointness and overlap witness; caller connection to native methods | Does NOT establish that real encoded module prefixes satisfy prefix-incomparability |
| `consumer.verus.rs` | Entirely AUTHOR-OWNED, checked against guarded successful provider imports | Native key/range contracts and conditional separation usable across ordinary crate interface | Not an inferred/certified candidate-interface experiment |

Record scaffolding retains ALL native fields, their order and their types:
`prefix: Vec<u8>`, `global_dbtx_access_token: Option<GlobalDBTxAccessToken>` and
generic `inner: Inner`. The native `PrefixDatabase` Debug bound is retained.
The native token tuple declaration `GlobalDBTxAccessToken(u32)` is retained.
Records/fields and selected methods are made public only to test imports;
derive macros are omitted. Native signatures gain named results and authored
contracts; the bodies are not changed and contain no inserted ghost statements.
Other impls, trait dispatch, token construction and derives are not compiled in
this isolated scaffold. Nominal types are defined in the verification crate;
no layout/ABI equivalence or verification of the complete native module is claimed.
There is no hidden record/body projection or opaque application's effect summary.

## Inspected native calls: NOT proved

| Native location | Inspected source behavior | Remaining contract, never silently discharged |
| --- | --- | --- |
| `db/mod.rs:823`, `raw_insert_bytes` | Passes `get_full_key(key)` to inner async insert | Async dispatch/aliasing, backend key/value semantics and all writes confined to this transaction |
| `db/mod.rs:837`, `raw_remove_entry` | Passes the prefixed key to inner async removal | Backend honors the supplied key and does not mutate other state |
| `db/mod.rs:872`, `raw_remove_by_prefix` | Passes prefixed prefix to inner async removal | Backend implements exact prefix deletion; no extra writes/effects |
| `db/mod.rs:860`, `raw_find_by_range` | Uses prefixed endpoints, dispatches and adapts stream | Ordering, endpoint interpretation, cancellation and safe stream prefix stripping |
| `db/mod.rs:687`, `module_instance_id_to_byte_prefix` | Starts with `MODULE_GLOBAL_PREFIX=0xff`, appends `u16::consensus_encode_to_vec()` | Correct/native encoding and pairwise prefix-incomparability of actual namespace encodings |
| `encoding/mod.rs:521`, `:543` | u16 encoding delegates to `BigSize(u64::from(*self))`, not a fixed `to_be_bytes` encoding | Variable-length encoding must not be invented as a fixed two-byte namespace; actual BigSize correctness/prefix-freeness unproved here |
| `db/mod.rs:1714`, `to_ref_with_prefix_module_id` | Borrows the underlying transaction into a prefixed wrapper and creates/returns a global-access token | Borrow/alias discipline, no independent effects/commits, token policy |
| `db/mod.rs:802`, `global_dbtx` | Matching token returns unprefixed inner transaction; otherwise delegates | Token escape intentionally exists; prefix arithmetic alone cannot imply global write confinement |
| `db/mod.rs:1813`, token/from_prefix | Token has u32 representation and a runtime-derived code | No unforgeability, collision resistance, capability security or confinement theorem |
| `db/mod.rs:1438`, `BaseDatabaseTransaction::commit_tx` | `raw.take().ok_or(TransactionConsumed)?`, awaits raw commit, submits notifications after success | No ownership/consumption slice added in this cycle; commit atomicity/errors, notifications, async cancellation/drop and rollback remain unproved |
| `db/mod.rs:1846`, `commit_tx_result` | Sets tracker flag, awaits backend, drains hooks on success | Tracker is not a durable certificate; hooks/panics/external effects excluded |
| `engine.rs:974`, `process_consensus_item_attempt` | Begins transaction; matching marker returns; processes item; stages marker; audits; commits | No refinement of the async function, equality/read reliability, serialization, backend, recovery, frame, module confinement or model bundle correspondence |

The prefix-incomparable condition is a PUBLIC explicit precondition of the
separation lemma/callers. It is checked algebraically, not assumed as an axiom;
no theorem asserts it for any module IDs. The native methods themselves have no
new preconditions. Comparable namespaces can overlap, as the positive witness
checks. Original wrapper calls are inspected, not replaced by unverified summaries.

## Exclusions and trust

The prior atomicity, durability/recovery, serialized attempt, module confinement,
item/peer representation, frame and marker-lifetime contracts remain AUTHOR-OWNED
and undischarged at the native boundary. Session cleanup/index reuse, external
hooks and exactly-once external effects remain excluded. Fsync durability alone
does NOT establish atomicity. No original async function, Rust/C++ backend,
RocksDB, consensus, whole transaction, financial invariant or fund safety is proved.

Trusted: sealed tools77 identity77/policy71/schema38 (all nine hashes), existing
Verus/Rust translation and SMT/Z3, reused vstd/specification library including
Vec clone/extend and u8 clone specifications, OS/process/hash mechanisms.
Neither vstd nor the standard-library executable implementations are freshly
verified. No production Rust, vstd, engine identity, shared guide/controller/CI
or previous reports are edited relative to the restored reviewed baseline.
