# Authorized feasibility experiment: stopped at the first additional foundation

**No native lock bridge was proved.** The one user-authorized experiment is finished, having reached the first additional foundational blocker: native UnsafeCell access/permission correspondence. No new trusted law, production fact, compiler/vstd change or replacement runtime type was added. Full source coupling and CP1/CP2 remain unproved. HMAC/native remains **DEFERRED AND UNACHIEVED**.

## Frozen target and actual observations

Commit35f8e965 froze the native writer, full-u64 domain, proof relation, direct cell dependency, existing permission control and false-value control before trials. It binds installed Rust1.99.1 compiler/driver/std/core artifacts, exact std Mutex/guard/platform/poison and core cell sources, the accepted78 complete runtime, relevant existing frontend facilities and actual REDB lock/cache source. The original native writer creates std::sync::Mutex<u64>, acquires its real guard, writes an arbitrary new u64, releases it, reacquires and reads. It has no added client token or value premise.

The ordinary native Rust compilation succeeds with normal type/borrow checks. It retains unwind edges and actual guard drops, using --emit=mir,obj, explicit overflow checking and compiler MIR/layout diagnostics. No native test executable ran. Exact compiler-generated MIR shows the actual guard pointer path through its native UnsafeCell field and raw store/load, with drop/reacquire between them. Mutex<u64> and its guard are each16 bytes/alignment8 on this pinned target; UnsafeCell<u64> is8 bytes. These are concrete metadata/code observations, not a native-correctness or representation theorem.

The accepted-runtime matrix has eight invocations, all with --no-cheating and full ordinary lifetime/trait/WF/mode/body/erasure checks, private complete bundles and caches:

| Case | Ordinary | Infer | Scope |
| --- | --- | --- | --- |
| Existing PCell constructor/permission read control |1/0 |1/0 | One checked diagnostic body, zero helpers; creates a different permission-owning object. Not a native Mutex result. |
| Same PCell control, false returned-value relation |0/1 |0/1 | Exactly the intended postcondition rejection. |
| Original native Mutex handoff with desired final-value contract | Frontend rejection | Frontend rejection | Native mutex/guard/constructor/acquisition facilities unsupported; no body theorem. |
| Native UnsafeCell::get dependency | Frontend rejection | Frontend rejection | Actual cell type/getter unsupported; no raw access/ownership interpretation. |

All frontend failures are classified separately from semantic controls. Every helper inventory is empty. The two positive results do not count as a native bridge or a new application proof. No assumed Mutex::lock/UnsafeCell::get specification was installed despite compiler suggestions. The existing nominal factory remains inactive/closed to Rev; no generalized factory or new state encoding was activated.

## Exact stop and value checkpoint

MISSING_SEQUENT.md gives the first missing native-state relation. The current facilities cannot establish ownership/value rights for the actual native Mutex's UnsafeCell merely from the ordinary nominal fields. Existing PCell permissions belong to the object its own constructor creates; they cannot be silently applied to this native field. Completing that handoff requires an additional primitive/state/permission interpretation, so investigation stopped before implementing it, as authorized.

Registered clock start13:38:52 UTC and stop13:46:14 UTC on2026-11-12 give a7m22s investigation interval. No active-work stopwatch separated early scope reads; the complete bounded task stayed comfortably below45 active minutes. Ten logged commands (compiler identity, ordinary native compilation and eight Verus invocations) took10.369 seconds total. No compiler/Cargo/dependency build or foundational prerequisite ran.

The final receipt `/tmp/redb-native-lock-bridge-20261112-v1/receipt.json` succeeds as a **feasibility experiment receipt**, SHA256 `e46e388089be5f6e759caf098e6e48fa1715a46635da02f43091b6cf6d6b47a8`. It explicitly retains native_bridge_proved=false, production_facts_added=0 and source_coupling_proved=false. Final guards passed. RESULTS.json authenticates receipts, native MIR/object products, diagnostics and native source observations; binaries remain private. There were no failed setup epochs or resource failures in this experiment. One historical Stage B report path was absent during preliminary reading; no result was inferred from that absence, and the actual checked-in Stage A facilities were inspected directly.

Reproduce with a fresh output while the exact pinned inputs remain available:

```sh
python3 -B -O source/tools/research/redb_crash_protocol/native_lock_bridge/probe.py \
  --output /tmp/UNUSED-native-lock-bridge
```

No production fact needs independent acceptance because none was added. The complete crash theorem remains unachieved; the accepted source-header/backend components and earlier finite native crash results are unchanged. The real application-value gate—actual cached buffers submitted with correct bytes before a successful barrier—was not reached. No further bridge or foundation is automatically authorized by this completed experiment.
