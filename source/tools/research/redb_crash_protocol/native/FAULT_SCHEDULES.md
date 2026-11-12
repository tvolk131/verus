# Prospective native failure matrix

This matrix extends the two successful smoke profiles without modifying REDB. It is supporting finite evidence, not a universal theorem or a device-compliance test. The exact public API, two 5120-byte records, ExclusiveWriter, 1MiB cache and both prior requested modes remain. Current requested mode is two-phase Immediate. Fixed layout must be observed, not assumed from a setter.

The backend starts with identical volatile/stable bytes at the target commit. Ordinary writes update volatile bytes; successful sync copies all volatile bytes to stable bytes. Failure makes the backend permanently unwritable, preventing cleanup after the failed operation from changing the captured image. The public commit return is recorded before process loss; no graceful Database drop runs. A fresh backend reopens precisely the captured stable image through actual recovery, and both records must agree with a complete old or new pair. No recovery rejection is accepted where the matrix expects a valid fallback.

For each prior mode, execute these ten interruptions plus the successful acknowledgement control:

1. First header write fails without any bytes issued/persisted: old pair.
2. First header write fails after persisting a prefix through half the secondary slot, leaving its old checksum: old pair. Record actual checksum detection using a separately compiled exact checksum implementation; do not assume every tear is detected.
3. First sync fails with no pending bytes persistent: old pair.
4. First sync fails after persisting only the first complete header write: old pair, including actual validation/fallback in prior-mode-false history.
5. First sync fails after persisting all volatile bytes: new for prior=false, old for prior=true. This is a failed commit, not rollback.
6. Second sync fails without persisting its pending control publication: new for prior=false, old for prior=true.
7. Second sync fails after all volatile bytes persist: new pair despite the public error.
8. Publication header write fails with zero prefix: new for prior=false, old for prior=true.
9. Publication header write fails after the first ten bytes (including the whole control byte) persist: new pair; barrier1 has already persisted the candidate pages and both slots.
10. First page write after the submitted secondary header fails without persisting pending bytes: old pair.
11. No injected failure: public Ok, two successful barriers, new pair after process loss.

Event ordinals are matched by kind and target-commit counters, not global setup counters. Every requested injection must fire exactly once. Prefix writes persist only the selected prefix; header-only failure persists the actual observed header bytes and no page bytes. Failed-sync-all really returns Err after persisting every byte. Successful sync is explicitly an environment behavior here, not a newly trusted Verus contract.

One additional recovery regression per prior profile fabricates a newer secondary from the valid primary, changes its transaction id, installs a checksum explicitly different from the actual computed checksum, and sets recovery-required/prior-two-phase=false. This is a labelled torn-slot diagnostic, not an execution claimed reachable from the preceding schedule. Actual repair is aborted through its public callback after the initial header rewrite and before a new commit. The invalid 128-byte secondary must remain byte-for-byte unchanged, and a rewrite plus callback must actually occur. The separately included exact native XXH3 is a diagnostic oracle, not a proven checksum law.

The freeze must bind this schedule, harness/controller, exact native dependency products, the successful preparation receipt, original checksum source and ordinary compiler before any trial. Report all deviations, errors, and out-of-configuration cases. No HMAC/native work resumes; that objective remains DEFERRED AND UNACHIEVED.
