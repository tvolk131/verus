# Original backend context: checked bodies, effects still unproved

The two stock opaque nominal error/location declarations clear the original backend type boundary. Adding the ordinary application FromSpecImpl definition for StorageError::Io then checks the original context **15/0 ordinary and Infer**, with **10 executable bodies, zero proof bodies, zero inferred helpers**. This is type/safety/body checking and a checked error-conversion meaning. It is **not** a flush coverage, backend persistence, latching-state or acknowledgement theorem. source_coupling_proved=false; HMAC/native remains **DEFERRED AND UNACHIEVED**.

The exact context preserves original CheckedBackend fields, complete check_failure/sync_data, real error variants and From<io::Error> body, and the full original StorageBackend trait/default bodies. The ten checked bodies are that From conversion, two CheckedBackend methods and seven StorageBackend defaults. Five abstract StorageBackend methods (len/read/set_len/sync_data/write) remain explicitly inventoried interfaces without bodies. No callback implementation or persistence summary was supplied. Complete-selected-scope success must not be mistaken for certification of those abstract methods or the complete application graph.

The two charged pure application definitions state obeys_from_spec=true and from_spec(error)=StorageError::Io(error). The unchanged original conversion body proves that relation; replacing it by StorageError::Unsupported causes precisely its one postcondition failure in both modes,14/1. These are neither assumed foreign function laws nor a disabled/vacuous conversion obligation. The exact original backend methods receive no new behavioral contract.

Preserved stages:

- v1: accepted launcher could not find rustup because PATH omitted /home/dev/.cargo/bin; no Verus source trial ran. RETRY.md retains this setup failure.
- v2: nominal-only source reaches verification,14/1 both modes, solely the missing application From relation.
- v3: checked pure adapter,15/0 both modes, measuring exact body inventories.
- strict positive/negative: fresh complete inventories,15/0 and isolated14/1 respectively, with final guards. All use the frozen two nominal declarations and no extra identities or behavioral assumptions.

The remaining source effect is genuinely stronger than these results. Current vstd/std_specs/atomic.rs explicitly provides no reasoning about values inside native atomics; its load/store contracts constrain valid memory-order use without relating values across calls. The real PagedCachedFile additionally owns std::sync Mutex/RwLock guards, striped LRU caches, Arc buffers and counters. No native cache/lock-to-model relation, buffered-write coverage or backend trace interpretation was constructed. Merely accepting these types or calls cannot prove the required write/flush/sync relation. No new state law, private-representation bridge, replacement lock/cache implementation or foundational chain was begun.

Next required source sequent: every relevant completed candidate write resident in the real cache is submitted successfully before a successful flush sync, with explicit entry cache/page invariants, actual borrow/lock effects and all failure edges. Then prove complete lower/public commit and recovery coupling. Candidate consistency, old-root retention, native serialization/checksum correspondence and physical-device compliance remain separate. These are not discharged by native tests or the pure conversion definition.

The successful strict stages can be reproduced with fresh output paths:

```sh
python3 -B -O source/tools/research/redb_crash_protocol/storage_boundary/probe.py \
  --adapter --freeze source/tools/research/redb_crash_protocol/storage_boundary/FROZEN_4_positive.json \
  --output /tmp/UNUSED-redb-backend-positive
python3 -B -O source/tools/research/redb_crash_protocol/storage_boundary/probe.py \
  --adapter --wrong-adapter \
  --freeze source/tools/research/redb_crash_protocol/storage_boundary/FROZEN_4_negative.json \
  --output /tmp/UNUSED-redb-backend-negative
```

Normal --no-cheating/lifetime/trait/WF/mode/body/erasure checks remain active on accepted78 tools and private copies/caches. No engine/vstd/native rebuild, new trust or Infer gain. Earlier freezes bind their corresponding historical controller epochs; original failures remain intact. RESULTS.json records raw identities and logged costs. Fresh independent review remains required for this bounded continuation.
