# Bounded native read-only validation gate and repair-observation sublemma

Implementation handoff, November 2, 2026. Independent review is parent-owned and
has NOT occurred for this result. The accepted cycle79 baseline and immutable
runtime are reused at cache78 / policy72 / schema38. Earlier cycle78 identity
qualifications remain historical; no old report or shared reader is rewritten.

## Exact result, and why this boundary

Following the actual selector callers leads from `header.rs:324` through
`finalize` (`header.rs:244`) to writer-open/recovery. A distinct read-only path
uses `finalize_transaction_slots` (`header.rs:364`), called by
`page_manager.rs:949` and `page_manager.rs:1405`. This is a real native
validation gate, NOT an invented state machine or a replacement selector.

The complete original body of `UnrepairedDatabaseHeader::finalize_transaction_slots`
is checked unchanged. Over its full record input domain, with NO precondition:

- A corrupted recorded primary returns exactly
  `StorageError::Corrupted("Primary commit slot is corrupted".to_string())`.
- Otherwise it returns exactly the original `DatabaseHeader`, including every
  field, both physical transaction records, roots, transaction IDs and the
  optional 128-byte corrupt-slot payloads.
- It does not swap slots, inspect transaction-ID ordering, validate the index,
  or repair a bad primary using a good secondary. All flag combinations and the
  full `usize` index domain are retained, not restricted to reachable headers.

This advances the read-only validation/error/frame link, not the production of
checksum observations. In particular the native gate does NOT establish the
selector's `primary_slot < 2` representation precondition or actual tree validity.

The actual repair path produces a DIFFERENT observation: `db.rs:1400`
`primary_verifies` maps `Ok(verified)` to that same boolean, a `Corrupted` error
to `Ok(false)`, and every other error to the unchanged error. The authored
`normalize_observation` auxiliary checks exactly those native match arms over
the COMPLETE native `Result<bool>` domain. Its match input is the already-produced
observation rather than the native checksum call. The arms are mechanically
compared with the source, including the error binder/payload transport; this is
a genuinely source-connected sublemma, NOT verification of the entire native
`primary_verifies`, its `Arc` plumbing, or its checksum-producing callee.

## Native fidelity, authored and inferred classifications

Native application: read-only clean redb checkout at
`c82ab865bb362bc4af067d332adc44978ff64813`. All 132 tracked native files are
checked against their pinned Git blobs before and after each accepted replay.
The runner additionally pins the complete `header.rs` and `db.rs` hashes and
the previously accepted extraction ledger for the native records/error enum.

The generated source retains complete native field sets/order/types for
`PageNumber`, `BtreeHeader`, `TransactionHeader`, `DatabaseHeader` and
`UnrepairedDatabaseHeader`; `Checksum = u128`, `TransactionId`'s `u64`, and the
original generic Result alias remain. Extraction flattens modules and exposes
visibility/fields. Unused record and TransactionId derives are omitted; the
TransactionId field is exposed. Those are explicit scaffold adaptations, not
whole-crate/native-type identity proofs. Only the gate's signature gains a
named result/authored postcondition; its body is byte-identical.

The complete native `StorageError`, including all seven variants, all payloads,
native Debug and non_exhaustive attributes, is retained. The two accepted
authored opaque TYPE declarations for `std::io::Error` and
`std::panic::Location<'a>` are reused. No behavior axiom, assumed application
result, checksum model, storage contract or unchecked function summary is added
to the accepted proof. Opaque payload behavior and implicit destruction are
not verified. These types do not become assumed static by erasing lifetimes.

Every positive reports exactly **15 verified / 0 errors**, **11 checked exec
bodies**, **3 checked proof bodies**, and exactly **23 scope rows**:

- 1 original native executable: `finalize_transaction_slots`.
- 10 authored executables: the normalization auxiliary, six observation clients,
  and three gate clients (including the invalid-index boundary).
- 3 authored proof bodies distinguishing true/false, corruption/success and
  I/O/boolean outcomes.
- 7 spec rows: three authored predicates and four generated enum projections.
- 1 native generated Debug outside selected scope, receiving no proof credit.
- 1 selected constant scaffold without a translated body.

No function is inferred: requested/generated counts are exactly zero,
`functions` is empty and status is `no_inference`. The semantic contracts,
clients and helper proofs are authored. The auxiliary helper is not counted as
an unchanged native method. Diagnostic stubs and finite test fixtures are
model-only scaffolding and receive no application-proof credit. `scope.json`
pins every scope identity, flag, classification and checked-body count.

## Honest decoder/frontend boundary

The observations used by both the selector and the read-only gate originate in
`TransactionHeader::from_bytes` (`header.rs:543`): version/error handling at
544-556, little-endian stored checksum at 557-561, inequality against
`xxh3_checksum` at 562, root parsing at 564-582, and corrupt-byte retention at
589-593. `UnrepairedDatabaseHeader::from_bytes` decodes the primary bit at
`header.rs:128`, obtains both slot observations at 185-190, and orients them
at 191-195. **None of those decoder bodies is proved here.**

The replay keeps a complete untouched-body `TransactionHeader::from_bytes`
reproduction, with the complete native `DatabaseError` as well as StorageError.
Its native body hash is checked. Excluded helper scaffolds and a checksum stub
are present ONLY to resolve Rust names for this failed diagnostic; they are not
called, assumed correct, or used in the successful provider. The failure also
reports constant-mode `size_of`/native const-helper boundaries and deliberately
excluded native dependency calls. It is not presented as a single-error repro.

A separate minimal `endian_boundary` isolates the concrete missing library/
frontend binding for native `Checksum::from_le_bytes` (`u128::from_le_bytes`).
It rejects as unsupported before SMT proof, independently of hashing, storage,
format strings, other native helpers and the earlier constant-mode errors.
This is the smallest isolated mechanical capability identified, NOT a claim
that adding that binding alone verifies the complete decoder. Slice lengths,
slice-to-array conversion, native helper contracts/bodies, const modes and the
actual checksum contract still need work. No production compiler/vstd change
or rebuild was made to cross that boundary.

An exploratory attempt to use an uninterpreted storage-observation oracle and
an unchecked checksum-call helper was rejected by `--no-cheating`. It is
retained as failed diagnostic evidence, NOT relabeled a native proof. The final
provider contains no such function/specification assumption.

## Validation and nonvacuity controls

Accepted replay destinations are `normal/` and `optimized/` under
`/private/tmp/verus-redb-observation80i.VitFCA/`. Each executes 28 fresh Verus
cases, with debug assertions both enabled and disabled: 2 positives, 12 native/
helper semantic mutants, 4 false stronger goals, 2 opaque-operation failures,
2 Rust lifetime failures, 2 no-cheating failures, 2 complete-decoder blockers
and 2 isolated little-endian blockers. `outcomes.json` requires exact typed
negative results/status/scope counts, rather than treating any compiler error
as an intended semantic rejection. Native gate and helper mutants are distinct.

The false representation goal and positive invalid-index boundary deliberately
show that accepting the recorded primary does not validate the index. The
false fallback goal shows that a good secondary does not make the read-only
gate a repair routine. Neither is an actual redb defect or a proved reachable
native crash. Decoder-produced index bounds still require a caller refinement.
No actual redb defect was established in this work.

Three separately compiled Rust tests execute the original gate over 64 concrete
flag/index cases, including 0/1/2/usize::MAX, and exercise all seven real error
variants and both boolean observations. Actual I/O codes and panic-location
pointers are preserved in these finite tests. These tests support nonvacuity
and boundary checking; they do not replace the universal Verus proofs or verify
the original tree/storage implementation.

The 33 focused Python tests run normally and with `-O`, on both real receipt
matrices. Targeted negatives exercise typed status/count/schema guards, every
scope/body classification, source/log/report/library bindings, proof flag
cardinality, default erasure, command/library/crate/mode bindings, matrix labels/
sources/commands, native-body/match-arm/field/error fidelity, immutable ledgers
and negative-outcome classifications. No acceptance check depends on Python
`assert`. Both separated and joined import/extern spellings, including
cross-wired kinds, are rejected: accepted import cardinality is exactly ZERO.
No nonzero-import or certified cross-crate-composition result is claimed.

Accepted runs use `--no-cheating`, whole selected scope, ordinary default
erasure and erased-Rust compilation. They intentionally use NO exports or
imports. The preliminary ordinary-export probes demonstrate the limitation:
`probe1/provider.vir` remains after a failed proof even though
`interface_written=false`; `probe2` also has an ordinary export without an
inferred interface. Neither is imported or accepted as certified evidence.
There is no seeded inference-aware failed output in this experiment, hence no
new failed-inference-export-cleanup claim. Existing accepted inference-aware
cleanup behavior remains baseline/historical rather than freshly reverified.

All 2,145 baseline file hashes, nine immutable runtime hashes and 61 toolchain
hashes are checked, along with per-replay guards over baseline/new/native/runtime/
toolchain inputs. Commands have fresh private HOME/Cargo/rustup/tmp/XDG/cache
locations, use the already installed sysroot directly, and do not invoke Cargo,
the network, native redb builds, full vstd verification or any compiler rebuild.
No app/native/parent/shared reader/controller/CI file is edited. No commit,
push, branch/ref/PR change, or subdelegation is performed.

## Replay and handoff

From the restored baseline plus this addition-only delta:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 source/tools/check_redb_repair_observation80i.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78/verus \
  --solver /private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78/z3 \
  --snapshot /private/tmp/verus-prefix-infer79.1eVEuO/final-package \
  --output /private/tmp/CHOOSE-A-FRESH-REPLAY-PATH
REDB_OBSERVATION80I_RECEIPT=/private/tmp/CHOOSE-A-FRESH-REPLAY-PATH/receipt.json \
  PYTHONDONTWRITEBYTECODE=1 python3 source/tools/test_redb_repair_observation80i.py -v
```

Repeat both commands using `python3 -O` and a different fresh destination.
The snapshot's three ledger identities are pinned; it is not a switch that
weakens proof gates. The supplied runtime already exists and is reused as-is.
The immutable external handoff contains the baseline-relative apply_patch,
SHA256, complete new-file inventory, exact full-tree reconstruction check,
source/runtime/native guards, all commands and hashed run/test logs, and failed
attempts. The external validation manifest identifies the actual accepted runs;
earlier partially rejected attempts are not silently substituted for them.

## Remaining recovery/refinement obligations

- `header.rs:128,185,191`: prove the bit-decoded index bound and correct physical
  slot-to-primary/secondary observation orientation from the original decoder.
- `header.rs:543,557,562,589`: check the full version/error/endian/checksum/parser
  path and exact retained corrupt bytes over its actual safe slice domain.
  Establish a precise authored checksum contract, not collision freedom or
  universal cryptographic correctness. Header-slot checksums are not tree validity.
- `header.rs:244,324,364`: compose real decoded inputs, selection, writer layout
  reconciliation and the distinct read-only gate. Preserve error payloads and
  distinguish intentionally non-repairing readers from repairing writers.
- `page_manager.rs:868,931,1399`: connect read_direct bytes, locks/snapshots,
  page-size assumptions and header publication to the checked extracted records.
- `db.rs:806,814,1400`: prove the actual root reads and TableTree checks plus the
  complete native classification wrapper; the authored match-arm sublemma is
  not proof of those callees or of mutable-memory/Arc correspondence.
- `table_tree.rs:95` and `btree.rs:941,950`: establish reachable-tree validation,
  including page parsing, checksums, depth/cycle checks, child links and I/O errors.
- `db.rs:1412,1426,1427,1431`: prove fallback exclusivity, cache invalidation,
  revalidation and repair publication; then allocation/reuse and crash/torn-write
  reasoning. There is NO fsync, power-loss, durability or whole-recovery theorem.

Trusted foundations remain the reused Rust/Verus/vstd/SMT stack and its existing
standard models plus the explicit opaque datatype boundaries. Receipt consistency
and hashes are provenance checks, not cryptographic proof attestation.
