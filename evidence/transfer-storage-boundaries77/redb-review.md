# Independent candidate77 and bounded redb review

Date: November 2, 2026. Read-only review after the two explicitly authorized
reconstruction patches; no candidate fixes or parent/native-source edits.

## Decision and severity

**Approve the reviewed F1–F3 repairs and the narrowly labelled reduced-error
slot-selection milestone. No new actionable P0–P3 finding.** This does not approve
a full-native selector theorem: the full `StorageError` translation still fails,
and **reduced-to-full error refinement remains UNPROVED**. No recovery,
tree-validity, storage durability, or complete database-safety result follows.

## Repair findings closed

- **F1/P2 closed** — `source/vir/src/infer/records.rs:1813` admits the generated
  `EuclideanDiv(Allow)` term without removing the canonical unsigned-std model,
  nonzero-divisor, soundness/maximality/exactness, or import-certificate gates.
  Fresh independently authored nested scalar and single-field owned providers
  import successfully in both debug-assertion modes. Arbitrary nonzero-divisor
  quotient goals pass; zero calls and false quotient goals fail. Tampered policy,
  proof flags (including integer-for-bool), summary, model fingerprint, and
  truncated imports fail and remove their seeded exports. The regression at
  `source/rust_verify_test/tests/infer_saturating_div.rs:8` exercises the repaired
  cross-crate shape; its archived seven-test run is audited, not freshly rerun.
- **F2/P2 closed** — `source/tools/check_redb_bitmap_state.py:153` now requires
  the exact checked executable and authored-proof identities, counts, selection/
  inference/contract flags, completed native scope, zero errors, strict positive
  booleans, expected mask identity/contract, and positive integer solver work.
  Freshly rendered bitmap providers pass in both compiler modes. Independent
  receipt mutations reject missing/unchecked scope, missing lemma, nonzero error
  count, integer success/publication, string proof acceptance, boolean work,
  unrelated helper, and narrowed domain. These are receipt checks on fresh actual
  compiler reports, not synthetic application proofs.
- **F3/P3 closed** — `source/tools/check_fedimint_fee_encoding.py:213` binds both
  inferred row and checked native row to
  `fee_payment_fee::PaymentFee::absolute_fee`. Fresh private providers pass in
  both modes. Wrong/consistently renamed identities, missing/duplicate native rows,
  and wrong inference/ordinary-contract flags reject. This review of the existing
  fee-helper repair does not include the separate in-progress Fedimint task.

## Bounded selector assessment

Independent substring/hash/line and field/type checks match six pinned native
source files, eleven extracted items, five record declarations, and all four
unchanged selector/accessor/swap bodies. See `source-fidelity.json`. Native redb
remains clean at `c82ab865bb362bc4af067d332adc44978ff64813`.

The authored domain is **only `primary_slot < 2`**, not a restriction on any
`u64` transaction ID, corruption observation, or other record field. It is not
proved established by native decoding/callers. The logical-field frame preserves
both physical transaction records and their payloads/geometry/flags; only the
index may toggle. Admissibility uses entry-state observations. The kept boolean
means kept-original-primary, not a validity certificate. Independent imported
goals confirm retained observation flags after fallback; the stronger false
post-swap reclassification claim fails. Calls without the representation
precondition fail rather than assuming native validity.

Selection/helper contracts, seven clients, and the checked bit-vector toggle
lemma are explicitly authored. Only newtype comparison plumbing is inferred.
Each fresh provider checks 23 obligations, 15 executable bodies, one proof body,
and the exact 40-row native scope, including checked generated `clone`, `cmp`,
`partial_cmp`, and `eq`. Only `eq` is the inferred provider contract; the importer
infers unrestricted strict ordering and checks its raw-field interpretation.
Importer totals are five obligations and three executable bodies. Non-body
scaffolds and outside-scope derives receive no checked-body credit.

The reduced scaffold retains only `StorageError::Corrupted(String)`. Fresh full
variant-declaration probes fail on native `std::io::Error` and `Location`, before
proof; those unsupported runs receive no proof credit. Visibility/module-layout
adaptation and omitted unused derives/attributes remain explicit. The finite
ordinary-Rust harness restores the full enum but cannot close formal refinement.

## Fresh execution versus historical audit

- **113 focused Python tests per mode**, normal and `-O`, all pass. This includes
  all 37 new slot tests, plus bitmap, fee-encoding, identity, and receipt tests.
- **29 slot replay cases per Python mode**, both pass their specified outcomes:
  six positives, ten semantic failures, two representation-safety failures,
  eight import rejections, two legacy failures, and one full-enum blocker.
  Twenty inference-aware seeded failures remove their exports per replay.
  Repeated provider phases observe cache misses, not claimed warm hits.
- **42 independent compiler probes**: twelve positives and thirty expected
  negatives. Twenty-eight inference-aware seeded failures remove exports.
  The two ordinary native-zero failures instead leave uncertified legacy `.vir`
  files without `VRINFER1`; neither receives cleanup/publication credit.
- **Four additional fresh bitmap/private-fee providers**, plus **43 independent
  receipt checks**: five unchanged actual receipts accepted, 38 corruptions
  rejected. Receipt mutation acceptance/rejection is not another SMT proof.
- Each fresh slot replay runs 1,600 finite native cases in both debug-assertion
  modes. These are sampled correspondence tests, not full-domain native proof.
- Historical only: 304 Python tests per mode, seven division tests, 144 VIR tests,
  and 54 bitmap / 20 PaymentFee / 19 encoding cases per mode. I checked their
  command/receipt hashes, raw report equality, preserved phase exports, relevant
  positive readers, and retained artifact ledgers. The final historical audit
  performs **22,034 hash checks with zero differences**, including both original
  126-file slot artifact maps. It is not a fresh rerun of these large suites.

Fresh raw commands, sources, diagnostics, exports, libraries, import corruptions,
native harnesses and receipts are under `slot-fresh-{normal,optimized}/`,
`independent-probes-complete/`, and `independent-receipt-probes/`. Machine summary:
`EVIDENCE.json`; historical audit: `audited-historical-evidence-final.json`.

## Integrity and process

Started clean at `3b05c1912b9d7314ad00c832c4065f4362562e02`. Verified and applied
the COMPLETE 94-file candidate patch exactly once, then the 30-file addition-only
redb delta exactly once, using `apply_patch`. Respective SHA-256 values:

```
545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12
4477566c5f11d2d5ef41bd0c334fb614e74b1a3583aa6663c2ef80ad3ed53459
```

All 1,976 baseline / 2,006 combined files match. Before/after checks match all
1,566 source, 1,085 build-input, nine runtime, 61 toolchain, 3,395 guarded-input,
94 candidate-inventory, eleven repair, and thirty addition entries. Both native
ledgers match (1,199 Fedimint and 132 redb files), with clean pinned checkouts.
**25,273 original files retain identical bytes, sizes, modes and mtimes.** See
`integrity-{clean,candidate,combined,after}.json` and `immutable-result.json`.
`git diff --check` passes. Replay-local HEAD/branch/ref observations are stable.

Git commands remove inherited `GIT_*` and use `GIT_OPTIONAL_LOCKS=0`. Execution
homes, caches, temp outputs, Cargo home and target paths are private; existing
Rustup/toolchain files are read-only inputs. No Cargo, compiler/vstd/native-engine
rebuild, dependency/network operation, checkout/ref-writing command, commit,
push, PR action, subdelegate, or source fix was performed. The requested runner
does compile erased scratch proof crates and finite native harnesses.

Retained failed setup/audit attempts are not counted: the first private-HOME
replay lacked explicit `RUSTUP_HOME`; the first import probe collided on crate
names; the next exploratory batch expected the wrong prefix for a correctly
unsupported signed call. Final independent probes rerun the complete matrix on
fresh paths. Initial historical audit reads used reused live export paths;
preserved phase archives/snapshots resolve every apparent mismatch. None is a
candidate defect or a passing final-run substitute.

## Reproduction and remaining trust limits

From this already reconstructed checkout, do not reapply either patch:

```zsh
unset ${(k)parameters[(I)GIT_*]}
export GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true
export RUSTUP_HOME=/Users/tommyvolk/.rustup RUSTUP_AUTO_INSTALL=0
scratch=$(mktemp -d /private/tmp/verus77-recheck.XXXXXX)
export CARGO_HOME="$scratch/cargo-home" CARGO_TARGET_DIR="$scratch/cargo-target"
python3 -B -m unittest discover -s source/tools -p test_redb_slot_selection.py
python3 -B -O -m unittest discover -s source/tools -p test_redb_slot_selection.py
python3 -B source/tools/check_redb_slot_selection.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --output "$scratch/normal"
python3 -B -O source/tools/check_redb_slot_selection.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --output "$scratch/optimized"
python3 -B /private/tmp/verus-candidate77-independent-review.yhn1ot/probes.py "$scratch/probes"
python3 -B /private/tmp/verus-candidate77-independent-review.yhn1ot/receipt_probes.py "$scratch/receipts"
```

The pilot intentionally audits frozen **dirty77**, not an arbitrary later merged
tree. Absolute temporary paths, original guarded-source roots, the sealed runtime,
toolchain, and native clones must remain available; this is not a portable,
self-contained clean-machine rebuild recipe. A later integration needs a new
freeze/ledger and replay, not silently disabled gates. Summary fingerprints are
path-sensitive and are bound to actual fresh successful imports, not a golden
fingerprint across scratch paths.

Vstd artifacts are reused unchanged, **not freshly reverified at77 or here**.
Source/build/runtime ledgers attest provenance, not an independent compiler
rebuild. Trust remains in Rust/Verus, macros/backend, SMT solvers, imported
standard/vstd specifications, and unsigned `saturating_div` correspondence
(including conditional no-unwind/nonzero safety). The 377 exported standard-model
fingerprints are a registry inventory, not 377 new application axioms or claims
that every model was used. Allocation/formatting/string/reference/array models
remain trusted. No native error-sum refinement, caller/repair protocol, checksum
to tree-validity connection, fsync/crash durability, transaction-history theorem,
consensus/funds safety, full native Cargo suite or hosted-CI claim is made.
