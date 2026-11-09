# Frozen redb byte-ID-to-slot-selection target

Frozen November 3, 2026, before implementation, against accepted cycle80
cache79 / policy73 / schema38. This is one half of the active two-project goal,
not a proof result. The parent selects feasibility proposal R1, with its
conditional native-block/caller correspondence boundary explicitly accepted.
The whole native decoder candidate is not feasible as a bounded milestone and
is NOT claimed by this selection.

## Exact native inputs

Native pin: `c82ab865bb362bc4af067d332adc44978ff64813`.
The sealed research `source-ledger.json` binds exact full files, items, bodies,
native spans and contiguous blocks. Retain the original declarations, field
types, complete error variants/payloads and executable text in the selected
scope. Verification annotations and labeled wrappers are permitted.

Selected whole native methods:

- `header::get_u64`, `header.rs:96`, item SHA-256
  `c3c291cbc43818822fafaad0589935043433d7ed52f05f13a8ec5c76a545b516`.
- `TransactionId::new`, `transaction_tracker.rs:19`, item SHA-256
  `7291f8fd284d67792ebd29ac7a7b09e5c0f46d5e12b1dbfa4ba1906cb2a5e3fc`.
- `DatabaseHeader::{primary_slot,secondary_slot,swap_primary_slot}` and
  `UnrepairedDatabaseHeader::select_primary_slot`. Selector item SHA-256
  `b2f5b9a5748a65c98ab36bd9ba6ff36b8fc61b48c8e47d66caa686eca40e32e7`.
- Native const dependency bodies `PageNumber::serialized_size` and
  `BtreeHeader::serialized_size`, and checked native
  `From<StorageError> for DatabaseError` wrapping.

Selected contiguous native blocks, NOT whole native methods:

| Block in `header.rs` | Start | Exact block SHA-256 |
| --- | --- | --- |
| Transaction-ID decode through complete transaction record return | 582 | `b02ab1898dbd990ab168dd223e1c1f19aaa30796c252538164ba9b6d482a1bf6` |
| Physical corruption orientation through complete outer record return | 191 | `a04379789a463ed1f328c26045d7ffca6cd57949eb80dbee75db53845132d564` |
| Final slot-selection and `(inner, clean)` return in `finalize` | 279 | `f99bb9a3427d6e74dc214b40c8459c1a016c66a0915f0aad138a9fbc9fee2e72` |

Wrappers supply the in-scope native locals and retain `Self` and the exact
blocks inside their original owner impl. Wrapper names/signatures and intent
contracts are authored. The composed client is explicitly an authored
extraction/composition adapter, not the complete native caller. Pin and audit
this correspondence independently. Preserve all seven StorageError and six
DatabaseError variants, with their payloads; previously accepted opaque
datatype declarations remain explicit trust. Debug formatting, payload internals
and implicit destruction are not proved.

## Frozen original safe domains

- `get_u64`: every byte slice of length at least 8, not exactly 8; all byte
  values and all IDs including 0/MAX/high-bit values. Shorter slices must still
  fail a genuine safety obligation.
- Transaction-ID/record tail: length at least 112 and, only when corrupted,
  length at least 128. Noncorrupted lengths 112 through 127 must remain valid.
  All longer slices are included. Offset 104 and last field 112 follow the
  original native offset/size const DAG, not hand-inserted executable literals.
- Constructor: every `u64`. Swap: every `usize`, with no index precondition.
- Primary accessor: primary index < 2. Secondary: `(index ^ 1) < 2`.
- Selector: `two_phase_commit || primary_corrupted || primary_slot < 2`.
  Preserve safe early branches for invalid indices; do not impose the stronger
  valid-index invariant on all selector inputs.
- Physical orientation block: all original fields and `usize` indices,
  preserving `index == 0` versus `else`. Do not claim invalid indices are
  reachable from the full decoder. The composed adapter inherits only the
  exact tail-length and selector-safety requirements above.

The context roots, versions, corruption booleans and layout-stale boolean are
arbitrary explicit inputs. No relation to omitted checksum/layout computation
is assumed. A physical 0/1 slot interpretation is conditional on index validity;
the selector's larger branch-sensitive safe domain is separately retained.

## Frozen practical theorem

Compose both physical byte streams through exact native ID decoding, native
record/orientation blocks, original selector, and the native final return block.

1. Each transaction ID equals the exact 64-bit little-endian word at native
   offset 104 in its corresponding input. Roots, versions and all unrelated
   header fields are preserved.
2. Corrupt bytes equal the first 128 original bytes precisely when the supplied
   corruption flag is true; otherwise the optional corrupt bytes are absent.
3. Two-phase commit retains the primary unless its corruption observation is
   true, in which case the exact native corruption error is returned. Outside
   two-phase commit, a corrupted primary selects a noncorrupted secondary or
   returns the exact both-corrupt error. Otherwise a noncorrupted secondary is
   selected only when its decoded transaction ID is strictly newer.
4. Successful selection preserves physical slot contents and complete geometry,
   changing only the selected index as the native policy specifies.
5. The final result has `clean == kept_primary && !layout_stale` and the exact
   native error wrapping. Equal transaction IDs do not force a swap.

The author supplies those intent/frame/representation contracts, not local
byte, cast, xor/range, overflow, or algebraic proof scripts. Implement reusable
Infer/proof assistance where ordinary composition currently lacks it.

## Baseline and reusable feature work

The sealed feasibility package is `/private/tmp/verus-header-scope.eoB3y6`.
Its candidate composition is explicitly UNCOMPILED/UNPROVED, not baseline
success. Native `get_u64` and isolated slice-to-array inference both decline at
generic calls. The original helper verifies ordinary safety with an authored
length condition, without establishing an inferred output contract. Existing
u64 LE inference and native full-error selection are already-supported controls,
not new capability. Corrected god-byte/read-only-gate probes have their actual
failures classified separately and cannot count as R1 successes.

Required reusable work: canonical concrete slice-to-fixed-array TryInto/TryFrom
recognition using full identity/proxy/type/const checks; Result/unwrap/subrange
composition; original named constants/`size_of`/const-helper DAG handling; and
checked summary/frame/ID-order/xor composition as needed. No generic-call
whitelist, application-name gate, replacement helper body, or unchecked summary
is allowed. Do not manually fold or replace the frozen native source constants.
Any compiler constant-evaluation mechanism must be generic, source-bound and
explicitly justified within existing Rust/compiler trust, not an invented
application fact. An unavoidable trust change requires parent reassessment.

## Validation and boundaries

Use fresh debug-assertions on/off replays, default erasure,
`--no-cheating --infer-goals --compile`, exact whole-selected-scope receipts and
nonzero fresh inferred soundness/maximality/exactness work. The proposal's
machine-enumerated completion/rejection controls are frozen requirements.

Include typed semantic negatives for wrong endian/orientation, ignored
two-phase/corruption observations, swapping equal IDs, always-primary, wrong
clean flag, changed errors and corrupt-byte laundering. Separate safety controls
cover helper lengths 0–7, tail 111, corrupted tails 112–127 and genuinely unsafe
indexing; positive boundaries include helper 8/longer, clear tails 112–127,
corrupted tails 128/longer, and safe invalid-index early selector branches.
Frontend or resource failures do not substitute for semantic rejection.

Composition is in-crate for this target. Freeze import cardinality at zero;
reject both joined/separated and cross-wired import/extern flags. Do not count
ordinary `.vir` files as inferred certificates. Publication failures must remove
seeded inferred exports where Infer-aware publication scope applies. Independent
review must check native source/block fidelity, domains, constants/models,
opaque payloads, receipts/artifacts and isolated acceptance-test guards.

Whole `TransactionHeader::from_bytes`, whole outer `from_bytes`, whole
`finalize`, bytes-derived primary-index validity, checksum/xxh3 computation,
root/PageNumber decoding, geometry validation, layout reconciliation, storage
callers, backend/refinement, recovery, crash/torn-write/fsync/durability remain
UNPROVED. No checksum oracle or assumed omitted-callee contract is authorized.
Both whole decoders have real shorter safe error paths; their original domains
must not later be silently replaced by unconditional full-length assumptions.

## Seal references

- Report SHA-256: `3b99d773ac9450e6e1c0f2d6db3a1a07964d485bb7edede372fcf894ee713fb5`.
- Proposal SHA-256: `5095837239c0891008952727259edb59d22673870b3698dc1a8714d9e3c28137`.
- Source ledger SHA-256: `d8bdea6b024b55b3aa17048b9bc60d7a4ee2b3b859befd2b9892f9c0422a3ffd`.
- SEAL.json SHA-256: `b2561ba032b71f3034dc7a7b2a36066e0554f706bb1e25c53eed973df7462d69`.
- SHA256SUMS SHA-256: `a968c762d847cf05c7b8cf59d339a47a02a8caaf36d20a1e0daa8cc5a96538d5`.

The parent read the report, exact block/client proposal and isolated baseline
diagnostics, and verified the complete package seal before freezing this scope.
