# Full-declaration redb selection with explicit opaque types

November 2, 2026. Implementation milestone only; a NEW independent review is
pending. Identity remains cache77 / policy71 / schema38. Production engine,
vstd, application, shared guides/controllers/CI and all older evidence are
unchanged relative to the reviewed full dirty cycle77 baseline.

## Result and exact boundary

The original extracted `select_primary_slot`, `primary_slot`, `secondary_slot`
and `swap_primary_slot` bodies now verify with the COMPLETE native `StorageError`
declaration, including every variant/payload and its native `Debug` and
`non_exhaustive` attributes. Only two **authored opaque external TYPE**
declarations are added: `std::io::Error` and `std::panic::Location<'a>`.
No standard-function specification, application axiom, assumed selector
summary, or engine extension is added. This is the original extracted native
method with explicit opaque datatype abstractions, NOT whole-crate recovery,
durability, or establishment of native caller invariants.

All five complete record field sets, both physical transaction records,
`Checksum = u128`, the 128-byte corrupt-slot payload, the original Result alias,
full `u64` transaction IDs and the original TransactionId comparison derive
list are retained. Extraction still flattens module paths, exposes visibility
and fields, and omits unused native record derives, exactly as cycle77 did.
These are explicit adaptations, not an inferred whole-native-type refinement.
The complete source is regenerated and hash-bound; all four native method
bodies are byte-identical. Six complete source hashes, eleven extracted items
and their original line positions are checked. Every one of the 132 tracked
application files is also checked against its pinned Git blob, with clean HEAD
`c82ab865bb362bc4af067d332adc44978ff64813` before and after.

## Authored theorem, unchanged domains

The sole representation precondition is `primary_slot < 2`. All prior flag,
record-field and transaction-ID domains remain unchanged. The cycle77 authored
contracts, seven clients and checked bit-vector XOR lemma are reused without
weakening. Only exhaustive specification matching changes for the complete
error sum: all other error variants are explicitly forbidden as selector
outputs, and the unchanged body proves this. No reduced-to-full error theorem
is silently assumed.

The theorem covers two-phase primary preference/failure, both-corrupt failure,
fallback from a corrupt primary, strictly-newer noncorrupt secondary selection,
equal/stale/corrupt secondary preservation, exact kept-original-primary flag,
and the logical field frame: only the physical primary index can toggle.
Admissibility uses ENTRY corruption observations, which remain unchanged after
a swap; it is not post-swap reclassification, checksum/tree validity, or a
durability certificate. Native callers/decoding do not here establish the
precondition. Full repair/publication/recovery, allocation/reuse, torn writes,
I/O/fsync, destructor behavior and crash semantics remain outside the theorem.

Each provider has 23 verified obligations, zero errors, 15 checked executable
bodies and one checked proof body. The EXACT 43-row scope is enforced:
15 exec, one proof, 21 spec definitions, four outside-scope derives and two
selected scaffolds without translated bodies. Three new payload projections
(`ValueTooLarge`, `Io`, `LockPoisoned`) are specification definitions, NOT
fabricated checked bodies; the single-variant `arrow_0` becomes
`arrow_Corrupted_0`. StorageError's native generated Debug is outside selected
scope and receives no proof credit. The exec bodies are four original methods,
four checked TransactionId derives and seven authored clients. Only comparison
plumbing is inferred; selection contracts/theorem and XOR lemma are authored.
Fresh certified consumers have five verified obligations and three exec bodies.

## Opaque identity, lifetimes and drop

The proxy names are directives, not replacement application payload types.
The existing frontend maps them to the external Rust ADT identities, checks
generic parameters/bounds and represents their internals opaquely
(`source/rust_verify/src/rust_to_vir_adts.rs`, external type processing).
The generic `'a` is retained; `LockPoisoned` remains the original
`&'static panic::Location<'static>`. Rust lifetime checking is not replaced by
an assumption that every location is static.

Fresh imported identity probes move an actual `io::Error` and transport a
generic borrowed `Location<'a>` unchanged (7 obligations, 5 exec bodies).
A borrowed-to-static lifetime escape rejects in Rust lifetime checking, with
zero verified SMT obligations. `Error::raw_os_error`, `Location::line` and
explicit `std::mem::drop` each reject as unsupported functions; no behavior
specification is supplied to make them pass. An empty function consuming an
Error DOES check/compile (6 obligations including the unchanged imported
clients, 4 exec bodies). This last probe deliberately exposes the limit:
implicit Rust destruction is not a proved Error destructor semantics. Its
empty/unit contract establishes no drop correctness, absence of effects,
resource release, panic behavior, or observational equivalence. Do not promote
either the opaque declarations or this probe into an Error-operations/drop
correctness claim.

## Fresh validation and trust

Both normal Python and `-O` replays pass the SAME 47 fresh Verus cases. Each
replay includes debug-assertions=yes/no: four provider positives (cold/repeat),
two fresh certified consumers, four boundary positives, ten semantic failures,
two missing-representation failures, sixteen certificate/import rejections,
six unsupported-operation failures, two lifetime-escape failures, and one
separate original full-enum frontend-blocker negative WITHOUT the opaque
declarations. All 36 inference-aware seeded failures remove their export;
the unseeded original blocker receives no seeded-cleanup credit. Both original
unsupported-type diagnostics are retained separately, not rewritten as success.

Providers and consumers use `--no-cheating`, whole selected scope, default
erasure and erased-Rust compilation. Each case has fresh destinations; typed
counts/flags, every scope identity/source, positive required-check solver work,
actual source/log/report hashes, VIR/rlib hashes, certified-import fingerprints
and exact semantic/certificate mutation bytes are bound. Repeated provider
phases observe candidate-cache MISSES and fresh solver work, not reused proof
acceptance. Cache files carry the exact string CACHE_VERSION, not integer 77.
ArtifactGuard checks 4,995 current proof/engine/native/runtime/toolchain inputs
before/after each complete replay, plus each case's generated/import inputs.

20 new focused Python tests pass normally and under `-O`; the combined old/new
selector suites pass 57 tests per mode. A separate reader audit accepts both
actual fresh matrices and rejects 64 receipt/certificate/binding mutations per
Python mode. Synthetic tests and receipt mutations receive NO additional
application-proof credit. There are no finite native tests substituting for
these proofs, no native Cargo/engine builds and no network activity.

Trust remains Rust/Verus/SMT, the reused vstd artifact and existing standard
models, plus the two explicitly authored opaque datatype boundaries. Both
provider and consumer certificates bind the unchanged 377 Core/Alloc/Vstd
registry fingerprints (hash
`676ba31b43183dac21a4863e549c53b2daae45d5cd041b1c889ee322105b0246`).
That inventory is not a claim that every registered model is used. There are
NO new standard behavior specifications or application-theorem assumptions.
Full vstd, historical engine/VIR/division tests, the broader old Python/common
transfer suites, whole native redb tests and hosted CI are AUDITED/HISTORICAL,
not freshly rerun here.

## Portability, attempts and delivery

Normal replay requires only repository fixtures/helpers, the pinned clean
application, a compatible installed Rust sysroot and supplied `--verus` and
`--solver`. It never calls the old hard-coded `audit_baseline`. `--snapshot`
is an optional EXTRA provenance audit, never a switch disabling proof gates.
Compiler work runs with fresh private HOME/Cargo/rustup/XDG/tmp/cache paths,
direct installed sysroot access and sanitized Git; no shared Cargo/rustup
writes or target copies occur. Runtime paths below describe this execution,
not defaults embedded in the runner.

Persistent scratch: `/private/tmp/verus-redb-opaque78.reABJTPu/`.
`validation.json` binds successful receipts, logs, scope, runtime and guard
ledgers. All failed attempts remain: initial Python 3.9 hashing API mismatch
before restoration, first runner's wrong truncated-metadata diagnostic,
second runner's incorrect integer CACHE_VERSION expectation, and the six
exploratory boundary probes. Neither failed runner is accepted retroactively;
the final complete normal/optimized replays are the acceptance runs.

All 2,092 reviewed baseline files and its source/build/runtime/native/guarded
ledgers are verified before restoration and after the milestone. The authorized
FULL baseline patch was applied to clean HEAD exactly ONCE; no parent file is
edited. The delivery directory contains the addition-only baseline-relative
patch, SHA-256, changed-file inventory, full file-hash map, scratch hash index
and exact in-memory baseline-plus-delta reconstruction. See `RECONSTRUCT.md`.
No commit, ref mutation, push, PR action or subdelegation occurs.
