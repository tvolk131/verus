# Accepted cycle79: native byte-prefix inference

Accepted on November 2, 2026 after separate independent production and
compatibility reviews. Identity: **cache78 / policy72 / schema38**.

This acceptance covers the frozen cycle78 baseline, the 17-file production
candidate, and the 10-file parent compatibility delta. It does not establish
whole-application, backend, transaction-confinement, or durability correctness.

## New inference and checked application facts

All three selected native Fedimint helpers infer exact contracts without
authored provider preconditions or postconditions:

- `PrefixDatabase::get_full_key`: result contents equal prefix followed by key.
- `PrefixDatabaseTransaction::get_full_key`: the same complete contents equation.
- `PrefixDatabaseTransaction::get_full_range`: each result endpoint equals prefix
  followed by its corresponding original endpoint.

Every generated precondition is `true` under the existing Verus/vstd model.
The complete original method items, private visibility, generic domains, record
fields/types/bounds and token `u32` remain unchanged. Only Infer attributes are
inserted outside the native methods. The application checkout remains clean at
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`.

Two explicitly auxiliary public inferred wrappers instantiate `Inner = u8` to
enable certified VIR-plus-rlib import; they are not extracted native methods.
The native generic methods are still checked over their full original domains.
An imported inferred caller composes two appends in order. The provider proves
23 obligations with zero errors, checking eight executable and five proof
bodies; the consumer proves six obligations with zero errors and four
executable bodies. These counts include authored goals, not just native bodies.

Cancellation and conditional namespace-separation lemmas remain explicit
authored proofs. A caller still needs prefix incomparability to obtain namespace
separation. There is no new theorem about the actual variable-length BigSize
encoding being prefix-incomparable, nor an invented fixed-width encoding.

## Bounded implementation and trust

The new profile recognizes canonical `Vec<u8, Global>::extend_from_slice` with a
mutable local receiver and an inert immutable slice-local argument. It tracks
the complete contents after each append, preserving call order. The existing
transparent Range profile additionally accepts `Range<&[u8]>` and
`Range<Vec<u8, Global>>`. Unsupported nonlocal receivers, effects, custom
allocators and arbitrary `T: Clone` are not silently ignored.

Existing vstd clone/extend contracts, primitive-byte clone semantics, resolved
slice View and `Seq::add` remain the authority. No new behavioral library spec,
application axiom, assumed conclusion or unchecked helper summary is added.
The standard-library/frontend correspondence, verifier and solver remain trust
boundaries. Contents do not establish object identity, allocation behavior or
capacity. Independent soundness/safety, safe-domain maximality and exactness
checks remain mandatory.

Owned candidates regenerate on cold and warm runs and perform fresh required
proof work. Their recorded cache outcome is a miss, not a cache-hit claim.
Poisoned-cache controls establish noninjection and fresh exact checking, not
hostile-cache-decoder coverage. Native semantic mutants remain exactly inferred
before failing the authored goal; frontend and safety failures are classified
separately. Failed inference-aware exports are seeded from genuine successful
exports and must disappear.

## Compatibility and historical evidence

The shared identity reader matches the live engine. New current-engine redb
certificate and opaque-scope fixtures are separate from unchanged historical
fixtures. They pin the exact qualified producer and all 380 standard-model
fingerprints. Three existing standard identities are newly registered; all 377
common fingerprints also change with the freshly bound vstd artifacts. The
compatibility update pins the complete fresh inventory, not three substituted
fingerprints. The original 43-row redb scope and contracts remain unchanged.

The candidate handoff and production review correctly describe their frozen
pre-integration reader/loader failures. They remain unchanged historical
records. The final parent replay and separate compatibility review establish
their resolution independently. Sanitizing inherited launch overrides lets the
shared suite use the supported rustup launcher; a fresh independent provider
run proves 23/0 without DYLD overrides. No production loader change is made.

## Validation and independent review

Fresh parent validation, using the final compiler10 runtime:

- 455 Python tests pass normally and 455 under `-O`.
- Native prefix replay: 30 expected-outcome cases in each Python mode.
- Full-error redb selector replay: 47 expected-outcome cases in each mode.
- All six existing common transfer scenarios pass.

Implementation-run results, hash-audited by the parent and reviewers rather
than redundantly rerun by them: 148 VIR tests, 93 focused Rust integration tests,
and 2,221 vstd verifications with zero errors. The final compiler, six recorded
Cargo-produced products, vstd outputs and 1,088 Rust build inputs are matched.
Earlier compiler9 artifacts receive no final proof credit.

The independent production review freshly runs 30 cases and 18 acceptance
tests per mode, plus 35 classified compiler probes. The independent compatibility
review freshly runs 65 slot tests, 18 prefix tests and 47 replay cases per mode,
plus 845 checks per mode including 830 rejection checks and 12 isolated
guard-removal controls. Neither review reports an actionable finding in scope.
The compatibility reviewer revalidates the parent's six common receipts; this
is an independent audit, not a second fresh six-scenario execution.

`production-review.md`, `production-review-receipt.json`,
`compatibility-review.md` and `compatibility-review-integrity.json` preserve the
original review bytes. Their index files bind the original paths and seals.
`validation.json` records exact parent receipts, logs and provenance.

Earlier failed setup, test and input-guard attempts are retained and excluded
from successful proof credit. In particular, the first parent common replay
correctly rejects documentation changes during its guard; only the later
stable-source replay is accepted. Historical evidence is never relabeled fresh.

## Remaining boundaries and next work

This is a detached source-connected selection, not a native module/application
build theorem. No transaction-wide confinement, backend range semantics,
async/drop behavior, crash/fsync durability, consensus, external exactly-once or
funds-safety claim follows. The redb replay remains conditional slot selection,
not recovery or tree validity. Prior conditional transaction models remain
model-only, without native backend refinement.

Some correct Vec-to-slice coercions and wider/effectful append compositions
still decline. A bounded mechanical follow-up can address inert borrowed-byte
coercions without inferring semantic namespace or storage policies. Connecting
the pure prefix equations to transaction-wide confinement separately requires
author-owned encoding, namespace, operation and backend contracts.

The exact integrated snapshot is sealed against clean checkpoint
`3b05c1912b9d7314ad00c832c4065f4362562e02`; its full patch is not a patch against
the already dirty baseline. Own HEAD and branch stay unchanged. No native
application edits, commits, pushes, manual ref writes or PR actions are made.
Shared orchestration-ref observations are disclosed without attributing them.
