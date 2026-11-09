# Round111 lane H: checked original-type header staging

## Result and exact source scope

PASS, conditional on EXPLICIT_INHERITED_STD_BOOL_TCB_NOT_RUST_CORE_PROOF;
this boundary is retained, not adopted. The final authored source is
`staging111.rs`, with exact add-file patch `FINAL.patch`. It is a new checked
executable consumer, not additional native method-body proof credit.

The consumer uses the actual imported `recovery_reencoding021::DatabaseHeader`,
`TransactionId`, and `TransactionHeader`, with original imported contracts for
`write_secondary_slot`, `to_bytes`, and `swap_primary_slot`. It executes the
secondary-slot write, obtains the FIRST `[u8; 320]`, swaps physical primary,
assigns `two_phase_commit = true` and `primary_unflushed = false`, and obtains
the FINAL `[u8; 320]`. Its old state is ghost-only; no native clone is used.

The frozen vocabulary is included directly as the unchanged source module
`contract111` from
`/private/tmp/redb-header-recovery111-20261108T051418Z/shared/contract111.rs`,
SHA256 `839e12109cc0ae0699536cbc7f401dc6d2b7d5f6f44eeccf8c9453bf450277ef`.
No library, runtime, compiler, vstd, or application was built or copied.

## Exact shared definitions

```rust
pub open spec fn prepared_header(before: DatabaseHeader, transaction_id: TransactionId,
    user_root: Option<BtreeHeader>, system_root: Option<BtreeHeader>) -> DatabaseHeader {
    let replacement = TransactionHeader {
        version: before.transaction_slots@[(before.primary_slot ^ 1) as int].version,
        user_root, system_root, transaction_id, corrupt_bytes: None,
    };
    DatabaseHeader {
        transaction_slots: if before.primary_slot == 0 {
            [before.transaction_slots@[0], replacement]
        } else {
            [replacement, before.transaction_slots@[1]]
        },
        ..before
    }
}

pub open spec fn committed_header(prepared: DatabaseHeader) -> DatabaseHeader {
    DatabaseHeader {
        primary_slot: prepared.primary_slot ^ 1,
        two_phase_commit: true,
        primary_unflushed: false,
        ..prepared
    }
}
```

## Established contract and nonvacuity

`stage_two_phase` requires exactly the frozen `stage_domain`: the original
roundtrip domain, transaction ID strictly greater than OLD physical primary's
ID, and canonical optional user/system roots. It does not require either old
mode value, primary index 0 rather than 1, clean old slots, absent roots, or a
new ID greater than the old secondary ID. Both original physical-slot choices
and every old `two_phase_commit`, `primary_unflushed`, and `recovery_required`
combination satisfying the original domain are quantified, not enumerated.

It proves:

- FIRST equals `database_bytes(prepared_header(before, id, roots), true)` exactly.
- FINAL equals `database_bytes(committed_header(prepared_header(before, id, roots)), true)` exactly.
- The resulting mutable actual header equals that same shared committed header.
- Both prepared and committed headers preserve `header_roundtrip_domain`.
- OLD primary's complete transaction slot is unchanged, including any retained
  original corruption bytes. Only physical OLD secondary is replaced.
- FIRST retains OLD `two_phase_commit` and OLD `primary_unflushed`; only FINAL
  sets the former true and the latter false. FIRST is not silently strengthened.
- FINAL's selected transaction ID and both roots equal the supplied arguments,
  and its primary index is OLD physical primary XOR 1.

`stage_nonvacuity` constructs a legal witness for either physical primary and
arbitrary prior mode/recovery flags. `canonical_root_nonvacuity` additionally
covers arbitrary canonical optional roots and any transaction ID above 1 on
that concrete geometry. `executable_witness` constructs the actual header with
old mode false and old unflushed true and calls the checked executable consumer.
It is verifier-checked executable source, not natively executed evidence.

## Charged attempts and controls

All 12 applications count, including setup/frontend failures. No allowance was
reset or extended; the cap was 16, one worker, 90 seconds per application.

| Attempt | Outcome | Interpretation |
| --- | --- | --- |
| 01 | VIR frontend failure | Field projection unsupported in isolated bit-vector query; retained. |
| 02 | PASS, 8 obligations | Primitive local selector correction. |
| 03 | PASS, 9 obligations | Complete final contract and canonical-root nonvacuity. |
| 04 | Backend resource limit | Premature mode implementation mutation; INCONCLUSIVE, not semantic rejection credit. |
| 05 | PASS, 9 obligations | Correct mode ordering with explicit FIRST-byte relation. |
| 06 | Backend resource limit | Incorrect concrete selector-byte assertion; INCONCLUSIVE. |
| 07 | Backend resource limit | Corrected concrete selector-byte assertion; NOT a positive control. |
| 08 | VIR frontend failure | Existing hash-spec hide hints nested after a statement; retained. |
| 09 | VIR frontend failure | Same hide hints nested in a proof block; retained. |
| 10 | Intended backend assertion failure | Premature mode implementation mutation, with original hash-spec hide hints placed correctly. |
| 11 | PASS, 9 obligations | One-line implementation correction of attempt 10. |
| 12 | PASS, 9 obligations | Byte-identical restoration of attempt 03. |

The valid semantic mutant is attempt 10, not attempts 04 or 06. Its extra
`header.two_phase_commit = true` occurs before FIRST serialization and fails
the assertion `header.two_phase_commit == before.two_phase_commit` at line 77.
There is no VIR error, resource-limit error, timeout, or metadata/setup failure
in that valid negative. Attempt 11 removes just that premature assignment and
passes with exactly the same contracts and verifier settings. Existing
`hash94::header_hash` and `hash94::scalar_spec` hide hints in the controls only
prevent irrelevant expansion; they introduce no producer, assumption, or law.
The final restored source contains neither those hints nor control assertions.

`CONTROL-MUTANT-CORRECTION.patch` records the exact one-line correction.
`RESTORATION.json` authenticates complete byte equality between attempt 03,
attempt 12, and the final source. All failed source snapshots, raw stdout/stderr,
full verifier JSON, original environments/argv, scope records, and exact fresh
export-path observations remain in their original attempt directories.

## Authentication, execution, and resources

Every application authenticates all 1416 frozen selected records at entry,
immediately before execution, and after exit, including expected byte lengths
and SHA256 values. The PLAN, INPUTS, and shared vocabulary hashes are separately
checked. All six complete `.vir`/`.rlib` pairs are recorded each time.

Only the existing executable midpoint `verus`, its direct `rust_verify`, and
`z3` are used; each is checked byte-identical to its historical counterpart at
all three checkpoints. Historical runtime files stay NONEXECUTABLE 0400 and
are neither copied nor chmodded. `VERUS_ROOT` remains the historical recovery021
runtime, `VERUS_Z3_PATH` is executable midpoint z3, `VARGO_TARGET_DIR` is absent,
and matching historical vstd/providers remain in place. The launcher is direct
pinned Python 3.9, not an `xcrun` or `python3` selection shim. Every invocation
retains its exact allowlisted child environment. There are no builds,
`--compile`, network calls, native application executions, Git writes,
subdelegates, parent-ledger calls, parent edits, or parent/native integration.

`FIRST-CLOCK.txt` preserves the first action at 05:24:00 UTC and the raw free
disk sample. Own hard deadline is 05:49:00 UTC, earlier than the absolute lane
deadline. `ACCOUNTING.json` retains the charged ledger, all durations and
resource samples, observed free-disk minimum and scratch maximum, and sealing
time. These are boundary samples, not continuous peak/quota measurements.
`authentication-task-exit.json` repeats the full frozen-input authentication
after source restoration and documentation. `MANIFEST.json` authenticates all
retained files except itself and `HANDOFF.json`; `HANDOFF.json` then binds that
manifest and the exact final source, patch, report, accounting, and export.

## Handoff and limitations

Lane P/parent can include the final source as a module and call
`stage_two_phase`; shared definitions are exposed as its `contract111` module.
The nominal database/header/root types are the actual imported historical
ones, not redefined model types. `HANDOFF.json` gives exact paths and hashes.

This is only checked staging/serialization composition over supplied roots and
ID under the stated domain and inherited conditional TCB. It does not prove a
native fetch, native write admission/coalescing, native cache/backend producer,
backend byte or crash effects, successful flush, acknowledged durability,
callback frames, allocator/ownership/lifecycle, or arbitrary native histories.
It does not replace lane P's recovery proof or any encoded-origin premise.
All eighteen original cells remain OPEN; native-path preservation FAILED;
native Git continuity UNAVAILABLE; compiler/source correspondence UNESTABLISHED.
No automatic followups are authorized or started.
