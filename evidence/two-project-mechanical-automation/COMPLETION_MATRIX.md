# Completion audit — requirements, not proof evidence

November 3, 2026. Both scopes were frozen before cycle 1 implementation.
Every row below remains **pending** until the integrated candidate supplies the
listed evidence and receives independent acceptance. A helper-only success,
frontend decline, finite native test, historical log audit, or receipt-consistency
check cannot discharge a composed application theorem.

The authoritative scopes remain `FEDIMINT_SCOPE.md` and `REDB_SCOPE.md`.
This checklist maps those requirements; it does not expand, replace or weaken
them. Feasibility packages and parent baseline probes are before evidence only.

## Fedimint

| ID | Required result | Completion evidence |
| --- | --- | --- |
| F-SOURCE | Unchanged native Amount/PaymentFee layouts, checked-add and limit bodies, and existing fee bodies | Re-extraction from the pinned clean native tree; item/body hashes; omission/annotation ledger; independent source audit |
| F-DERIVE | Actual generated named-field Amount comparisons are checked, not assumed | Generated-body checks, dependency identities and incorrect-body/missing-dependency controls |
| F-ADD | Exact Some/None characterization on all original u64 component inputs | Unconditional local and certified imported theorems; base/ppm/both overflow and exact-M controls |
| F-LIMIT | Exact componentwise limit check with equality and aliased arguments | Local/imported equivalence; excessive base and excessive ppm negatives; alias/equality positives |
| F-CALLER | Overflow, send-limit, receive-limit and accepted results in native decision order | Full-domain source-connected pure adapter theorem; separate pinned correspondence to both native validation blocks |
| F-FEE | Accepted actual send/receive fees are bounded by the corresponding actual limit fees for all u64 amounts | Composition using the existing checked monotonicity theorem, preserving saturation-before-division and private visibility |
| F-AUTO | Eliminate both authored checked-add contracts and new low-level Option/record/field plumbing | Frozen before failure/manual 6/0 control versus final unchanged-body inferred contracts and composed proofs; explicit remaining author work |
| F-IMPORT | Genuine consumer uses the fresh provider's exact certified summaries | Nonempty hash-bound VIR/rlib artifacts, certificate/dependency identities, cold/warm fresh checks, stale/tampered/missing/crosswired import controls |
| F-CONTROLS | Execute the complete frozen S1–S8, N1–N12 and I1–I4 matrix | Per-requirement map to raw fresh outcomes; semantic, safety, unsupported and resource results classified separately |

The authored pure adapter's correspondence seam is accepted, not hidden.
Whole async/configuration callers, formatting, configuration provenance,
database writes, backend behavior and funds safety remain outside this target.

## redb

| ID | Required result | Completion evidence |
| --- | --- | --- |
| R-SOURCE | Exact whole selected items and all three contiguous native blocks | Re-extraction from the pinned clean native tree; hashes, wrapper/annotation ledger and independent source audit; blocks never labeled whole methods |
| R-CONST | Original size_of/named constants/const-helper DAG determines native offsets | Source-bound constant evaluation justification, no hand-folded replacements or application-name gate; changed-helper-value semantic control |
| R-BYTES | Exact little-endian ID inference through canonical slice conversion | Fresh mandatory soundness/maximality/exactness checks, original length-at-least-eight domain, high-bit and longer-slice positives, wrong-endian negative |
| R-TAIL | Exact native transaction tail preserves roots/version and corrupt bytes | Universal theorem over length >=112 with corruption implying length >=128; clear 112–127 positives and corrupted 112–127 safety rejections |
| R-ASSEMBLY | Exact physical payload/frame preservation and corruption orientation | Original assembly block over the full usize input domain; valid-slot interpretation conditional on a bound, not an assumed decoder invariant |
| R-POLICY | Original slot policy, exact errors and branch-sensitive safety | Universal source-faithful selector proof, arbitrary IDs/flags, safe invalid-index early branches, equal-ID no-swap and unsafe-index negatives |
| R-COMPOSE | Both decoded IDs flow into selection and exact final clean/error result | Executable composed client through all selected blocks; preserved complete fields; clean iff kept_primary and not layout_stale; exact error wrapping |
| R-AUTO | No handwritten byte/cast/xor/frame/algebraic proof plumbing | Baseline decline versus final unchanged-body composition; checked reusable support and explicit author intent/representation contracts |
| R-CONTROLS | Execute every frozen completion/rejection control | Fresh debug-on/off outcomes, complete scopes/counts, semantic wrong orientation/policy/clean/error/corrupt-byte controls, short-buffer and indexing safety controls |
| R-IMPORT | In-crate target has zero claimed imports | Reject all separated/joined/crosswired import/extern flags; ordinary exports are not claimed as certified inference imports |

Arbitrary supplied root/corruption/layout context values are not checksum
oracles or assumed omitted-callee facts. Whole decoders/finalize, bytes-derived
primary validity, checksums, root parsing, geometry/layout reconciliation,
storage, recovery and durability remain outside this target.

## Integrated acceptance

| ID | Required result | Completion evidence |
| --- | --- | --- |
| G-DOMAINS | Neither original safe domain is narrowed | Generated contracts/maximality and universal caller signatures audited against the frozen domains; no overflow-excluding preconditions |
| G-TRUST | No unauthorized application/standard-library axiom family | Actual production/model delta and trust inventory; any unavoidable new trust explicitly reassessed before acceptance |
| G-RUNTIME | All final evidence uses matching source/build/runtime identities | Integrated rebuild, matched vstd verification, exact producer/model pins and source/build/runtime inventories; historical receipts remain historical |
| G-REGRESS | New and existing behavior remains checked | Parent generic tests, focused Rust/VIR/Python suites, both complete native target replays, existing six-scenario replays and broad configured regressions |
| G-GATES | Acceptance tests independently protect the actual decision gates | Fresh malformed-receipt and isolated guard-removal tests with intended failures, not incidental errors |
| G-REVIEW | Production, target proofs and parent integration accepted independently | New review delegates; findings fixed and freshly rechecked; fresh execution versus historical audit clearly separated |
| G-PRESERVE | Accepted dirty baseline/native inputs preserved except intended delta | Baseline-relative patch inventory/reconstruction, unchanged native pins/cleanliness and guarded originals; no commits, pushes or manual refs/PR actions |
| G-HANDOFF | Review-ready integrated change with an honest effort account | Requirement-by-requirement final audit, remaining author responsibilities, explicit nonclaims and reproducible commands/artifact hashes |

Parent generic integration coverage includes
`infer_fee_validation_composition.rs` (three tests, including u8/u64 and genuine
imports) and `infer_byte_decoder_composition.rs` (four tests, including a changed
const-helper-value rejection). A third crate,
`infer_mixed_mechanics_composition.rs`, adds three regressions requiring the
new byte/const mechanics to compose with record-valued checked addition and
two borrowed bounds, both locally and through an actual certified import.
It also rejects treating checked overflow as wrapping. These are ten generic
test methods in total, not ten application theorems. Their existence or
successful Rust compilation alone is not candidate-verifier success and does
not replace either real frozen target.

Reassess after three implementation/review cycles if any required row remains
unmet. An honest blocked or partial outcome is not goal completion, and an
easier replacement target requires explicit reassessment rather than relabeling.
