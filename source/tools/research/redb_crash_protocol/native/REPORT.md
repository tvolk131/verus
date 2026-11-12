# Native finite-failure checkpoint

Both no-fault smoke profiles and all 24 prospectively registered matrix cases passed. This is **finite execution evidence against unchanged REDB**, not a source theorem. `source_coupling_proved=false`; acknowledgement honesty and coherent recovery for every lawful input/crash remain unproved. HMAC/native remains **DEFERRED AND UNACHIEVED**.

The matrix comprises 20 interrupted commits, two successful acknowledgement controls, and two fabricated detected-corrupt-slot recovery regressions. Each uses the same existing database fixture with two 5120-byte records. Both previous requested commit modes occur; the new public transaction requests two-phase Immediate. Actual ExclusiveWriter mode is explicit. Every target commit kept the file length fixed, and every requested fault fired. Unsupported creation/resize/multi-process/concurrent/crash-during-repair configurations are not included in this denominator.

All interrupted commits returned an error through the actual public API and reopened a complete registered old or new pair. In particular:

- Header-only persistence before failed barrier1 caused the real prior-mode-false repair path to fall back to the old pair; it did not serve the incomplete newer candidate. This exercises the source behavior needed to address the model counterexample, for this fixture only.
- A failed barrier1 with all writes persistent recovers new under prior=false and old under prior=true. A failed barrier2 that persists everything recovers new under both modes. Thus public error is observably not rollback.
- A publication write persisting the first ten bytes, including the complete control byte, recovers new after barrier1. A publication write persisting no bytes follows the previous-mode selection behavior.
- The two partial-secondary writes really had invalid checksums under the exact native XXH3 diagnostic and recovered old data.
- Each separately labelled corrupt-slot regression made a newer secondary checksum explicitly unequal to its computed value, forced the ordinary repair path, observed an actual header rewrite and repair callback, aborted before a fresh commit, and retained the exact invalid 128 bytes. These fabricated images are not claimed reachable from the preceding injection schedules.

The environment contract in FAULT_SCHEDULES.md is implemented by the harness, not installed as a Verus assumption. Successful sync copies all volatile bytes to stable bytes. Failure can persist nothing, only the header, or everything as selected; subsequent backend writes fail. Process loss freezes stable bytes and skips graceful Database drop. Recovery uses a fresh backend with precisely those bytes. A successful native reopen is checked by reading both complete records, not merely by accepting an open or checking a header checksum.

The pinned REDB Rust sources/build scripts and published libc source were compiled unchanged. The private manifest projection retains the active std/Linux graph and excludes inactive optional/dev/workspace dependencies; it is not a feature-complete upstream build. Prepared-source hashes, the exact release dependency products, compiler and included original XXH3 source are bound by FREEZE_2.json, the successful preparation receipt, and FAULT_FREEZE.json. The duplicate checksum compilation is solely a diagnostic oracle; no checksum correctness/collision-freedom theorem follows.

The final matrix receipt is `/tmp/redb-crash-native-20261112-faults2/receipt.json`, SHA256 `1cf2e3ee03cf42e4e4921421274a1e446e2c0005ff299169fecc2f9ea5028e67`. Final artifact guards passed. Its 24 native processes took 0.469 seconds in total; harness compilation took 3.205 seconds. This is experiment cost, not an Infer improvement. Two earlier native preparation problems were harness/setup failures, retained in PREPARATION_NOTES.md and RETRY.md; FAULT_PREPARATION_NOTE.md records a missing-freeze refusal before any fault trial. No failed setup is counted as a semantic control.

Replay from a checkout with the authenticated private preparation inputs present:

```sh
python3 -B -O source/tools/research/redb_crash_protocol/native/fault_replay.py \
  --output /tmp/UNUSED-redb-native-failure-replay
```

Use a fresh output path. The successful preparation can be independently reproduced through prepare.py with its registered freeze; a different path needs a new prospective runtime binding because these freezes intentionally name exact private artifacts. Raw receipts/stdout/stderr for the accepted matrix and smoke are preserved under evidence/; large private native binaries and original build products are not committed.

Missing universal work includes candidate-page and old-root interpretation, actual buffered-write coverage, source call/error/epilogue simulation, all relevant recovery validation and fallback paths, serialization/checksum coherence, and the environmental-device compliance boundary. Native tests discharge none of those universal obligations. Fresh independent review is required even for this finite checkpoint.
