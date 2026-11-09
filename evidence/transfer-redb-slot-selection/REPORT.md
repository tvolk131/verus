# Bounded redb slot-selection cycle

Date: November 2, 2026. Baseline: dirty77 (cache77/policy71/schema38), reconstructed
once from clean `3b05c1912b9d7314ad00c832c4065f4362562e02` using the authorized
patch `545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12`.
The baseline's review repairs remain **not independently accepted**. This work
also awaits a new independent combined review.

## Milestone and native blocker

The unchanged executable bodies of `select_primary_slot`, `primary_slot`,
`secondary_slot`, and `swap_primary_slot` check against explicit authored
contracts in a **reduced-error scaffold**. The original record fields, array
lengths, `TransactionId(u64)`, original comparison derives, strings and branch
bodies are retained. This is a source-linked conditional model, **not a theorem
about the complete original native method or native recovery**.

The separate full-declaration probe retains every original `StorageError`
variant. It rejects before proof because `core::io::error::Error` and
`core::panic::location::Location` are unsupported. The frozen probe and raw
diagnostics are in `blocker/`; fresh replay paths and hashes are in
`replay-summary.json`. Neither an external type specification nor an assumed
application summary was added to conceal this blocker. Formal refinement of
the reduced error sum to the full native enum remains unproved.

The actual pinned application is redb
`c82ab865bb362bc4af067d332adc44978ff64813`, rooted at
`/private/tmp/verus-transfer-redb-miuzm2w3/redb`. The target is
`src/tree_store/page_store/header.rs:324`, accessors at 448/452, and swap at 480.
`source-ledger.json` binds complete source hashes and each selected declaration,
line and executable-body hash. `native-items/` contains the exact extractions.

## Authored domain and properties

- Representation validity requires only `primary_slot < 2`. Transaction IDs
  range over all `u64`; every corruption flag and other field is arbitrary.
  Native callers/decoding are not proved to establish this precondition.
- The validity relation `slot_admissible` is defined over **entry-state input
  observations**. A non-corrupt primary is admissible; the non-corrupt secondary
  is additionally admissible only outside two-phase mode. It does not mean
  that checksum equality establishes page-tree validity or durability.
- Two-phase mode never swaps, ignores even a newer apparently valid secondary,
  returns `Ok(true)` when the primary observation is non-corrupt, and otherwise
  returns the original corruption message without changing state.
- Outside two-phase mode, both-corrupt observations fail unchanged. A corrupt
  primary falls back to a non-corrupt secondary regardless of ID ordering. A
  non-corrupt primary yields to a strictly newer non-corrupt secondary; equal
  or stale IDs, or a corrupt secondary, keep the primary.
- The returned boolean is exactly the kept-original-primary flag, not an
  independent validity certificate. On success the selected physical index is
  admissible in the entry-state relation. Corruption flags themselves remain
  unchanged even after a swap, as in the original body.
- The complete logical-field frame preserves both physical transaction records
  (including roots, checksums, version and retained corrupt bytes), all geometry,
  recovery/two-phase/unflushed flags, and both observation flags. Only the
  primary index may toggle. This is not a raw-memory-padding or disk-byte frame.

The three helper bodies are checked, not assumed. The authored broadcast lemma
`slot_bits::two_slot_toggle` proves the XOR bound and distinctness using
`bit_vector`; it adds no executable statements to any original method. Seven
authored caller goals expose the branch rules, selected observation, kept flag
and frame. All contracts and the lemma remain explicit in the evidence.

## Scope, adaptation and trust ledger

The proof scaffold flattens native module paths and exposes declarations and
fields. It keeps all fields of `DatabaseHeader`, `UnrepairedDatabaseHeader`,
`TransactionHeader`, `BtreeHeader`, and `PageNumber`; `Checksum = u128`,
`TRANSACTION_SIZE = 128`, and the native `Result` alias are checked against the
pinned sources. The sole datatype reduction is `StorageError` to
`Corrupted(String)`: `ValueTooLarge`, `Io`, `Unsupported`, `PreviousIo`,
`DatabaseClosed`, and `LockPoisoned` are omitted. Its native `Debug` and
`non_exhaustive` attributes are omitted in that scaffold. The full probe restores
the full variant declaration and adds an exhaustive specification-only wildcard;
that proof probe also omits the unused `Debug`/`non_exhaustive` attributes. The
ordinary-Rust harness restores those attributes. No exact full-native-type
refinement is inferred from these declaration adaptations.

Unused native record derives are omitted in the proof
scaffold. The native `TransactionId` derive list is retained, with
`infer_derive(PartialEq, PartialOrd, Ord)` requesting checks rather than trusting
the proposed comparison correspondence. Actual generated `cmp`, `partial_cmp`,
`eq`, and `clone` bodies appear in the checked scope. Only `eq` is an inferred
provider contract, with unrestricted domain `true`; **selection is an ordinary
authored-contract proof, not an inferred selection/recovery invariant**. The
import client additionally infers unrestricted strict ordering and proves its
raw-field interpretation. No transaction allocation, ID increment, wraparound
protocol or globally monotone transaction history is proved.

Each successful provider run has exactly 23 verified obligations, zero errors,
15 checked executable bodies (four original methods, four generated newtype
methods, seven authored clients), and one checked proof body. All 40 native
scope rows are matched, including 19 specification definitions, three explicitly
outside-scope derives (`Hash`, `Debug`, and the `Eq` marker), and the two
selected non-body scaffolds `VERUS_UNERASED_PROXY__TRANSACTION_SIZE` and
`_::__VERUS_REVEAL_INTERNAL__`. Those last two are not counted as proved
executable bodies. The importer has exactly five verified obligations, three
checked executable bodies and no proof bodies. Typed booleans, exact identities,
whole selected scope, integer counts and positive fresh solver work are checked.
Source/proof body hashes and actual generated-source hashes accompany receipts.
Generated derive bodies are bound by their checked identities, original derive
text and frozen compiler/macro hashes, not a fabricated textual body digest.

Trust remains the existing Rust/Verus frontend and backend, SMT solvers, and
imported vstd/standard models for arrays, references, strings/`ToString`,
formatting, standard comparisons and allocation. The interface binds 377
existing standard-model registry fingerprints (Core/Alloc/Vstd only); this is
neither 377 new application axioms nor a claim all 377 models were used by the
selector. Vstd is the reused sealed verified artifact described by baseline77,
not freshly rebuilt or fully reverified here. The exported summary fingerprint
is path-sensitive; the exact artifact and metadata fingerprint are bound by a
fresh successful import, not an invalid cross-scratch golden fingerprint.

No proof connects the two error representations, native module/privacy layout,
the actual caller/repair protocol, input observations to XXH3, reachable pages,
transaction publication, allocator state, storage IO, fsync, crash durability,
or complete database safety. Those are explicit omissions, not inferred facts.

## Fresh validation and publication behavior

`replay-summary.json` records final normal and optimized runs, commands, hashes,
and exact retained paths. Each final replay performs the same 29 Verus cases:
six fresh positives; ten semantic negatives (kept flag, two-phase preference,
strict-vs-equal ID comparison, corrupt secondary rejection and state frame in
both build modes); two missing-representation safety negatives; eight imported
policy/proof/summary/standard-model-fingerprint negatives; two legacy negatives;
and the full-native-declaration blocker. Each provider is reproved with fresh
outputs twice per build mode. These repeated-output runs observe candidate
cache **misses**, not a claimed cache hit. Required-check solver work is fresh.

All positive and negative source-linked invocations use `--no-cheating` and
default erasure; positives also compile the erased Rust. Each of the 20
inference-aware semantic/safety/import negatives is explicitly seeded with the
checked prior provider export and rejects/removes that seed. The full-native
probe is unseeded and receives no seeded-cleanup credit. The separate legacy
non-Infer negatives actually replace the seed with a new legacy `.vir` despite
proof failure and `interface_written=false`: no inference-aware cleanup or
publication guarantee is claimed for them.

Fresh Python tests run normally and under `-O`. They exercise the strict receipt
readers, scope identities/counts/flags, solver-work typing, certificate identities
and standard-model inventory, source/field/body fidelity, semantic controls,
private Git/Cargo context, and the legacy distinction. The synthetic unit
fixtures are never presented as verified application artifacts.

A separate ordinary-Rust harness restores the **full native error enum** and
uses the four unchanged bodies and original record fields. In each replay it
passes one test containing 1,600 cases in each debug-assertion build mode,
including `0`, `1`, `2`, `u64::MAX-1`, `u64::MAX`, all flag combinations, both
physical indices and payload/geometry frame checks. These finite native tests
support the correspondence investigation but do not prove the missing formal
refinement. They compile only private scratch harnesses with the frozen Rust
compiler; no engine/compiler/Cargo build or dependency fetch occurs.

Every final replay audits the 1,976-file baseline, 1,566 source files, 1,085 Rust
build inputs, nine runtime files, 61 toolchain files, all 3,395 guarded inputs,
94 inventory entries, 11 repair entries, both native ledgers (1,199 Fedimint and
132 redb files), native clean pins, and local HEAD/branch/refs before and after.
The broader 304-test baseline suite, division/VIR tests, other transfer suites,
full native Cargo suite and hosted CI were **not freshly rerun**; their snapshots
were audited and their earlier results remain historical.

## Failed attempts and bounded follow-up

`attempts.json` records the retained exploratory artifacts and hashes. Early
probes first exposed private-field opacity; public visibility adaptation fixed
that without changing a body. Missing XOR facts, then a module-level broadcast
cycle, then a missing proof-module prelude were repaired with the checked lemma.
An explicit kept-index equivalence goal required its checked XOR-distinctness
fact. The first unit run found a non-unique mutation selector, subsequently
made body-specific. Two replay attempts rejected an inappropriate cross-path
summary golden and an assumed warm cache hit; neither is counted as passing.
The final checks instead bind the actual imported fingerprint and report the
observed misses. The initial exploratory wrapper printed compiler statuses but
returned zero itself; only the later strict replay's validated statuses count.
An initial read-only shell inspection also shadowed zsh's special `path`
variable; it was rerun correctly before the single baseline patch application.

The genuine remaining blocker is native declaration translation, not an unsafe
integer domain or a need for an application axiom. A bounded follow-up can
investigate retaining unused native `std::io::Error`/`Location` enum payloads as
opaque, non-observable declarations with correct lifetimes, rejecting all
unsupported payload operations, and rechecking the exact full enum and body.
It must preserve all native variants, the full current input domain, checked
comparison bodies and publication gates. This cycle adds no such tool support
or trusted storage summary and stops at this first useful bounded milestone.

## Delivery

Only `source/tools/check_redb_slot_selection.py`,
`source/tools/test_redb_slot_selection.py`, and this unique evidence directory
are added relative to dirty77. No existing production/shared/controller/identity,
native application, runtime, package, Cargo/rustup, branch or ref is edited.
There is no commit, push, PR or subdelegation.

The exact baseline-relative apply-patch delta, its SHA-256, changed-file hash
inventory, final validation and byte-map reconstruction receipts are retained at
`/private/tmp/redb-slot-selection77-aofyb_lo/handoff/`. The reconstruction checks
dirty77 plus the addition-only delta **in memory**, without creating a checkout
or reapplying the baseline patch. `RECONSTRUCT.md` contains replay instructions.
