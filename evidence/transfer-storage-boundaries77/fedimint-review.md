# Independent review: Fedimint atomic/replay conditional model pilot

Date: November 2, 2026, America/Chicago.

## Verdict

**Bounded no-findings verdict.** No actionable severity finding was identified
in the supplied 77-addition pilot as a **CHECKED CONDITIONAL MODEL-ONLY** result.
The authored invariant/replay claims, observed verifier results, control failures,
receipt binding, and stated native-refinement limitations are consistent.

This is NOT approval of native Fedimint transaction/recovery correctness,
RocksDB, engine/F1-F3 repairs, redb selection, or any new Infer capability.
No native correspondence, consensus/availability/fund-safety result, forever
exactly-once result, or inference-aware failed-export cleanup is established.
Those excluded obligations are not being silently reclassified as findings fixed.

## Confinement and reconstruction

- Used only the tool-provided isolated checkout at
  `/Users/tommyvolk/.t3/worktrees/verus/t3-6f7dfdb2`.
- Initial clean HEAD and final HEAD:
  `3b05c1912b9d7314ad00c832c4065f4362562e02`.
- Applied the full dirty77 patch exactly once with `apply_patch`, then the
  additions-only pilot patch exactly once. No other reviewed source edits.
- Baseline patch SHA256:
  `545e330ee87d539aa727154a6121f251f557a18867be62517a9867592a372e12`.
- Pilot patch SHA256:
  `a694f5a3b672595ef4df34f011954a5eef7d323cf6a89c00e3efd405eb537f65`.
- Validated the 94-path baseline inventory against clean Git objects and the
  reconstructed 1,976-file baseline. All 77 pilot paths are new, within the
  two named tools/evidence scope, with matching byte counts and SHA256s.
  The final exact file set contains 2,053 files; none of the baseline files changed.
- Validated delivery inventory links to `reconstruction.json`, `final-audit.json`
  and `final-audit-attempts.json`, plus the declared reconstruction counts.
- Read applicable checkout/native AGENTS. The native AGENTS references available
  codebase, development and Rust-style skills, which were read. Its referenced
  multipart-review and linked-specs skills are absent at this pinned checkout.
- No subdelegation, shell checkout creation, Cargo command, compiler rebuild,
  fetch, commit, ref/branch mutation, push, PR action, or native execution.
  The requested replays compiled only private authored model libraries with the
  existing sealed compiler; independent extra probes only verified private files.
- Review harnesses and all extra sources/outputs live only in this unique report
  directory. No repairs to the reviewed implementation were made.

## Fresh validation, not historical credit

1. Normal focused unittest discovery: **27 passed**.
2. Optimized (`python -O`) focused discovery: **27 passed**.
3. Fresh normal replay: **11 expected outcomes**.
4. Fresh optimized replay: **11 expected outcomes**.

Each replay has two positive default-erasure compiles, seven semantic controls,
one arithmetic-overflow control, and one seeded ordinary-export failure control.
Every positive has exactly **18 verified, 0 errors**, one checked executable
body, sixteen checked proof bodies and fifteen specification definitions.
The positive selected scope has exactly **32 named identities**. The additional
query is the recursive-specification termination check, not another native body.

For every fresh and historical semantic/safety control I also independently
checked that the failed function is exactly the added control, with eighteen
verified queries and one error; no unrelated original model failure is counted.
Frontend/VIR, panic, timeout and resource failures are not used as semantic credit.

Evidence: `focused-complete.json`, `tests-{normal,optimized}.stderr`,
`replay-{normal,optimized}/receipt.json`, and their complete per-case raw outputs,
commands, generated sources and artifacts. All command return codes are retained.

## Independent verifier probes

Ran nine additional private, whole-crate, no-cheating verifier cases using the
same sealed runtime. The positive extension adds four proof bodies and reports
**22 verified, 0 errors**; that number is separate from the unchanged pilot count.

The extension constructively proves admitted traces through initial, start,
effects-staged, marker-staged, successful-but-unobserved commit, recovery and
matching replay; it also witnesses no-write commit failure and precommit abort.
Thus phase-3, committed, recovered and replay preconditions are not vacuous.

It further proves two boundary witnesses:

- Clearing a durable marker while retaining old effects, then processing the
  request again, yields two bundles and breaks pairing. This weakened lifecycle
  is NOT admitted by the pilot and demonstrates the importance of its boundary.
- The initial state can stutter via abort. Safety induction does not imply
  progress, consensus liveness or availability.

Eight independent mutations to private copies of the ORIGINAL definitions were
all rejected for proof postconditions, with actual checked work and no VIR error:

- Erase the committed image on crash: 13 verified / 5 errors.
- Commit only the marker: 16 / 2.
- Commit only the effects: 16 / 2.
- Ignore peer equality: 16 / 2.
- Bypass the dedup branch: 15 / 3.
- Remove the commit phase guard: 17 / 1.
- Change the unrelated durable frame at commit: 17 / 1.
- Admit marker cleanup into `step`: 17 / 1.

These are expected independent counterchecks, not failures of the delivered
model and not repairs. Sources, exact commands and raw diagnostics are retained
under `independent-verifier/`; driver: `run_probes.py`, extension: `review_probes.rs`.

## Independent receipt and check-removal probes

In EACH Python mode, ran **362 fail-closed checks**, all rejected as expected:
required positive field deletion, every field of all 32 identity rows, required
negative-field deletion, negative numeric type corruption, source-prefix aliases,
and same-length duplicate/missing identity replacements.

Also removed thirteen distinct positive acceptance guards in memory, one at a
time, without editing any source file. Re-ran the authored 27-test suite against
each crippled reader. **All thirteen mutants were detected in both modes**.
The intentional test failures are retained as mutation evidence, not presented
as additional passing suites. Removed checks cover return code, success flags,
typed counts, no-inference status/counters/imports/trust, ordinary-export status,
whole selected scope, checked-body counts, exact identities, row flags,
source identity and positive verifier work.

Evidence: `receipt-probes-{normal,optimized}/receipt.json` and thirteen complete
mutation-test transcripts per mode; driver: `receipt_probes.py`.

## Model and source-ledger analysis

Reviewed `model.rs`, `scope.json`, checker/tests, REPORT/README and the complete
source-refinement ledger. Key reviewed model locations:

- `model.rs:36`: exact marker/effect pairing, not just a one-way implication.
- `model.rs:75`: all-or-nothing atomic durable image is AUTHOR-OWNED semantics.
- `model.rs:87`: independently authored crash persistence preserves the disk.
- `model.rs:91`: only admitted serialized transitions enter trace induction.
- `model.rs:149` and `model.rs:172`: step preservation and decreasing finite-trace
  induction derive the invariant from initialization, rather than assuming it
  at the trace theorem boundary.
- `model.rs:187`: recovered pairing and at-most-one bundle over admitted traces.
- `model.rs:211`, `model.rs:218`, `model.rs:225`: matching replay, item/peer
  conflict rejection, and lost-committed-acknowledgement recovery/replay.
- `model.rs:250` and `model.rs:256`: checked non-atomic and missing-dedup witnesses.

`valid_state` includes the desired invariant as an induction predicate, but
`reachable_invariant` requires reachability, NOT the invariant. Its initialization
and transition obligations are checked. No assume/external_body/admit,
application summary import, hidden theorem axiom, unsafe body or external hook
is present. Ordinary library/SMT trust remains; transition definitions themselves
are explicit assumptions of this model, not newly established DB guarantees.

Audited the ledger against the read-only Fedimint checkout at pin
`b8bd3d8ef61e1410bd4ad01d2e3b6425f7973236`, and inspected the four hashed installed
Rust/C++ transaction-binding files. The ledger correctly distinguishes:

- `engine.rs:974`: one attempted transaction; matching item AND peer skips work;
  conflicting item OR peer rejects; rejected processing returns before commit.
- `engine.rs:924`: only SnapshotTooOld retries; arbitrary ambiguous-error replay
  is not claimed for this native loop.
- `engine.rs:1016` and `engine.rs:1069`: staged marker, audits and eventual commit;
  transaction borrowing/prefixing is inspection, not a confinement theorem.
- `db/mod.rs:1477`/`:1769`: drop logging/diagnostic flags are NOT rollback.
  Rust ownership, FFI destruction and backend discard need a separate proof.
- `db/mod.rs:1438`/`:1846`: backend consumption, post-success notifications and
  hooks are boundaries, not durable/exactly-once external-effect guarantees.
- RocksDB snapshots, sync writes, commit/conflict/error mapping and WAL recovery
  are observed implementation facts, NOT verified atomicity or recovery axioms.
- `consensus/db.rs:21`: AcceptedItemKey is an INDEX.
- `engine.rs:775`: complete_session actually deletes AcceptedItem markers.
  The model does not invent a native session/index composite key; cleanup,
  reuse, old effects and historical replay are outside its current marker lifetime.

Module-write confinement, single-attempt serialization, reliable marker reads,
exact abstract item/peer equality, durable recovery and the frame/lifetime
contracts are explicit author-owned restrictions. Equality-class labels are
not claimed as native encodings/hashes or a proved collision-free mapping.
No missing native hypothesis was found that is silently presented as discharged.
**Proved native correspondences: ZERO. Unchanged native pure slices: ZERO.
Inferred functions: ZERO.**

## Historical evidence and ordinary exports

Audited, but did not credit as fresh execution, both historical 11-case receipts,
their validation transcripts and all original failed attempts. Revalidated
acceptance from both historical and both fresh receipts. Checked **228 artifact
hashes**, including live historical binary artifacts and fresh artifacts; published
historical stdout/stderr copies match their live originals byte-for-byte.

The three disclosed failed attempts remain failures: reserved-name parser,
tuple-choice proof, and incorrect failed-reader status expectation. Early model
snapshots remain labelled reconstructed, not contemporaneous. The original failed
reader snapshot matches its recorded hash/partial receipt. Early --infer-goals
experiments receive no inference or publication credit.

Observed in both fresh modes: ordinary `interface_written=false` coexists with
`.vir` files even after failed proof, and the seeded failed case replaces the
old artifact. This is the disclosed legacy behavior, NOT inference-aware safe
publication, cleanup, safe importing or a cache-equivalence result. No `.vir`
produced here is imported, and artifact existence alone grants no proof credit.

The **21 disclosed raw stderr EOF-whitespace exceptions** remain byte-for-byte
unchanged. They were not normalized. Their scope and upstream audit explanation
are retained in `evidence-audit.json`.

## Immutable integrity and trust

Before reconstruction, after reconstruction and after all fresh work, checked:

- 3,395 absolute guarded inputs and 61 toolchain entries.
- All nine sealed tools77 runtime files; cache77/policy71/schema38 identity.
- All 1,199 Fedimint and 132 redb source-ledger entries; both original native
  checkouts remain clean at their declared pins. Redb correctness is not reviewed.
- Every baseline/source/build input: 1,976 / 1,566 / 1,085, after reconstruction
  and finally; all 2,053 reconstructed files finally match their exact inventory.
- Handoff/delivery/runtime package hashes and linked reconstruction/audit files.
- The four installed inspected dependency files, also checked before/after each
  fresh replay and again in the final evidence audit.

Final shared snapshots exactly equal the pre-reconstruction snapshot. No private
Cargo home or target was created. Runtime/vstd were reused unchanged; neither
full vstd nor any native crate was freshly verified. Parent 304-test suites,
VIR/division, other applications, hosted CI, engine/F1-F3 and redb selection are
not freshly executed or approved by this review.

The first informational Git HEAD/status reconnaissance preceded the explicit
sanitized wrapper. All reconstruction, input audits, replays and probe commands
used a fresh environment without inherited GIT_* and with GIT_OPTIONAL_LOCKS=0.
No Git write/ref command was issued. Two private review-harness setup mistakes
occurred before successful reconstruction (older Python lacked file_digest;
zsh `path` variable shadowed PATH while reading skills). They caused no package
application, reviewed-source change or verifier result; details are retained
in `review-attempts.json` rather than hidden or counted as validation.

Trust is limited to the existing Verus/compiler translation, SMT/Z3,
specification libraries/reused vstd artifacts, OS and file-hash/audit mechanisms.
This review adds no proof of that trusted computing base.

Integrity receipts: `integrity-before.json`, `baseline-reconstruction.json`,
`candidate-reconstruction.json`, `integrity-after-reconstruction.json`,
`candidate-final.json`, `integrity-final.json`, `evidence-audit.json`.

## Reproduction

Run from the already reconstructed isolated checkout; DO NOT apply either patch
again there. Use a NEW scratch directory for every replay (existing outputs
intentionally reject). Exact original commands and hashes are in
`focused-complete.json` and per-case `command.json` files.

```
REPRO=$(mktemp -d /private/tmp/verus-fedimint-review-repro.XXXXXXXX)
env -i PATH="$PATH" HOME="$HOME" TMPDIR="$REPRO" GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 CARGO_HOME="$REPRO/cargo-home" CARGO_TARGET_DIR="$REPRO/cargo-target" python3 -B -m unittest discover -s source/tools -p test_fedimint_atomic_replay.py -v
env -i PATH="$PATH" HOME="$HOME" TMPDIR="$REPRO" GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 CARGO_HOME="$REPRO/cargo-home" CARGO_TARGET_DIR="$REPRO/cargo-target" python3 -B -O -m unittest discover -s source/tools -p test_fedimint_atomic_replay.py -v
env -i PATH="$PATH" HOME="$HOME" TMPDIR="$REPRO" GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 python3 -B source/tools/check_fedimint_atomic_replay.py --output "$REPRO/replay-normal"
env -i PATH="$PATH" HOME="$HOME" TMPDIR="$REPRO" GIT_OPTIONAL_LOCKS=0 PYTHONDONTWRITEBYTECODE=1 python3 -B -O source/tools/check_fedimint_atomic_replay.py --output "$REPRO/replay-optimized"
```

For independent probes, copy `run_probes.py`, `review_probes.rs` and
`receipt_probes.py` from this report directory to that new REPRO directory.
Run the first script once, then receipt_probes.py in normal and optimized modes
with the same sanitized wrapper. They use their own script directory for outputs
and the fresh normal replay for receipt fixtures, and refuse to overwrite evidence.
