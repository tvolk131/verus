# Independent bounded recheck: redb cumulative repair2

Date: November 2, 2026.

**Verdict: accept the two bounded corrections. No remaining finding within this recheck's scope.**
F1 (joined-option receipt/provenance bypass) and F2 (non-isolating inventory
regression) from `/private/tmp/verus-redb-p2-recheck.qaNvXj/REPORT.md` are
addressed. This is not a repeated broad semantic review, a soundness verdict,
or approval of whole-native/storage behavior. No candidate repairs were made.

## Independent reconstruction and isolation

The existing tool-created isolated worktree was confirmed by
`t3_worktree_status` at
`/Users/tommyvolk/.t3/worktrees/verus/t3-bf61a4c6`, branch
`refs/heads/t3/recheck-storage-bindings`. It was clean at precisely
`3b05c1912b9d7314ad00c832c4065f4362562e02` before reconstruction.
Checked ancestor/checkout instructions contain no applicable AGENTS.md;
the three patch envelopes introduce none. The read-only redb AGENTS.md was
read; its report/analysis exemption applies, so no native build was run.

Fresh private scratch, home, cache, config, data, cargo, rustup, temporary,
evidence, and output directories are rooted at
`/private/tmp/verus-redb-repair2-independent.KGzyRXEI`. Git subprocesses strip
inherited `GIT_*` and set only `GIT_OPTIONAL_LOCKS=0`. Commands use private
HOME/TMPDIR; Cargo is configured offline and was never invoked. No subagents,
network, commits, explicit ref writes, PR actions, runtime/dependency/target
copies, engine/Cargo/vstd/native rebuilds, or parent/original edits occurred.
Only the authorized three reconstructions modified the isolated checkout.

Applied via `apply_patch` exactly ONCE EACH, in this order:

1. Full dirty77 baseline, SHA256
   `e76b758498a692e30e5683486a4d59d453202529cd13c2c0169401542b31690e`.
   All **2,092** baseline files match immediately afterward, with no extras.
2. Original six-file redb delta, SHA256
   `cbd298f436debf9dac299b776b238e595b3cb2aba83312e69358217e512463a5`.
   All **2,098** original final files match.
3. CUMULATIVE repair2, SHA256
   `6d9c428af40d18fc46695ffdad99cdf6ca6622afc1eed224fe6ee273f3d425c4`.
   Both before and both after hashes in its two-record inventory match.
   All **2,098** final files match repair2/file-hashes.json, with no extras.

Prior repair1 was NOT applied. No Fedimint delta was applied.

## F1: joined options now reject at the intended gate

`source/tools/check_redb_slot_selection_opaque.py:249` explicitly rejects any
argv token starting with `--import=` or `--extern=` before the separated-token
validation. The existing separated-token VIR/rlib binding and alias handling
remain intact. The fresh canonical runner uses separate tokens.

For EACH Python mode, independently mutated that mode's new actual 47-case
receipt, keeping `commands[position]["argv"]` identical to the mutated case
command and leaving the actual compiler reports, sources, logs, outputs,
hashes, certificates, and import inventory unchanged:

- `provider_debug_cold`: unrecorded joined `--import` using the real provider VIR.
- `provider_debug_cold`: unrecorded joined `--extern` using the real provider rlib.
- `provider_debug_cold`: both joined options cross-wired, VIR as extern and rlib as import.
- `consumer_debug`: joined duplicate `--import` alongside correct separate options.
- `consumer_debug`: joined duplicate `--extern` alongside correct separate options.

All five reject at exactly **`noncanonical import command flags`** in normal
Python AND `-O`. Each identical bad matrix is accepted after independently
removing ONLY the joined-option require in memory. These are five direct
same-input rejection/removal/acceptance pairs per mode, not incidental gates
and not new compiler invocations or application proofs. The original report's
five constructions were reproduced, including use of provider_debug_repeat.

Both genuine fresh matrices accept. Two separated-kind controls per mode
still reject at exactly `command import binding changed: --import` (both
arguments cross-wired) and `command import binding changed: --extern` (extern
alone points to VIR), with duplicate command records kept consistent.
Two positive binding-only alias checks per mode accept independently named
import/extern aliases; these do not claim new alias compiler executions.

## F2: total-cardinality coverage is now independent

`source/tools/test_redb_slot_selection_opaque.py:289` now adds both a hash-valid
extra `.vir` and a hash-valid extra `.txt`. The `.txt` case has exactly one VIR
and one rlib, but total cardinality three, so per-kind counts cannot mask the
total-cardinality conjunct at
`source/tools/check_redb_slot_selection_opaque.py:255`.

Independently added a new hash-valid `.txt` to the actual consumer import
inventory in each mode's fresh full matrix. The unmodified reader rejects at
exactly `certified import requires exactly one VIR and one rlib`. Removing
ONLY `len(row["imports"]) == 2 and ` in memory accepts that SAME full matrix.
The remaining per-kind counts, command binding, certificate, fingerprint,
and all later matrix checks remain active.

## Fresh validation versus audited evidence

Fresh, unmodified requested executions:

- Normal Python: **63 focused tests pass; 47 portable compiler cases pass their intended dispositions.**
- Python `-O`: **63 focused tests pass; 47 portable compiler cases pass their intended dispositions.**
- Totals: **126 unmodified unit-test executions and 94 new compiler case invocations.**
- Each 47-case matrix has 10 positives and 37 expected negatives: 10 semantic,
  2 safety, 16 import, 7 frontend, and 2 lifetime cases.

Additional fresh reader checks, PER mode, separate from those totals:

- 1 genuine full-matrix acceptance.
- 5 exact joined-gate full-matrix rejection/removal/acceptance pairs.
- 2 exact separated-kind full-matrix rejections.
- 2 positive syntactic alias binding checks, no extra compiler credit.
- 1 isolated `.txt` full-matrix cardinality rejection/removal/acceptance pair.
- 2 full 63-test mutant suites: joined require ONLY removed gives exactly
  2 intended assertion failures; total-cardinality conjunct ONLY removed gives
  exactly 1 intended assertion failure. BOTH have zero errors.

The joined failures are exactly
`test_joined_import_duplicates_reject` and
`test_unrecorded_joined_import_flags_reject`; the cardinality failure is exactly
`test_import_inventory_requires_one_interface_and_library`. Every failure is
`AssertionError: ValueError not raised`, not an incidental exception. Across
both modes, the 252 mutant-suite test executions are **not** additional passing
tests or compiler proof credit. Mutants are in-memory function copies only;
the reconstructed candidate remains byte-identical to repair2 afterward.

Separately AUDITED, not counted fresh:

- `/private/tmp/verus-storage-cycle78.G9sUIt/redb-repair2-probes.json`.
- `/private/tmp/verus-storage-cycle78.G9sUIt/redb-repair2-probes-O.json`.
- Per file: 5 reported joined full-matrix rejections, 2 targeted guard-removal
  runs with consistent embedded assertion-failure logs, and one genuine-matrix
  acceptance summary. Normal/optimized labels agree. The joined records have
  labels/flags/booleans rather than full mutated argv/receipts; this audit
  checks those summaries and logs, not an independent execution. Our fresh
  matrices, exact diagnostics, and same-input removal pairs supply independent
  evidence beyond those parent assertions.

## Integrity before and after

Zero mismatches in all frozen ledgers: 1,570 source files, 1,085 Rust build
inputs, 9 runtime files, 61 installed-toolchain inputs, 3,511 absolute guarded
inputs, 132 native redb files, 1,199 native Fedimint files (ledger-only), and
701 historical candidate artifacts. An additional 481 package/prior-review/
parent-evidence files frozen by this reviewer remain byte-identical.
Source/build ledgers match before tests and after all probes. Immutable
runtime/toolchain/native/absolute ledgers match before reconstruction and
after all probes. Final candidate inventory remains exactly 2,098 files.

Owned HEAD, branch, and index hash are unchanged; its 216 reconstructed changed
paths remain exactly the authorized reconstruction. `git diff --check` passes.
The untouched original checkout remains clean at
`bad5b8896274d6df1be70e547eb81b004ab62ba1`, branch `refs/heads/verus-infer`,
with identical HEAD/branch/index/status. That original HEAD differs from the
requested isolated base and was never reset or patched.

Original native redb remains clean at
`c82ab865bb362bc4af067d332adc44978ff64813`, branch `refs/heads/master`, with
unchanged index and metadata. Native Fedimint metadata is also unchanged;
no Fedimint companion work or theorem claim is made.

Observed two shared-ref additions through the final integrity snapshot in BOTH
own/original views of their common Git store, disclosed without attribution;
no removals were observed:

```text
refs/t3/orchestration-v2/checkpoints/MTc2ODBlNGY3MGMyYzM5MzBjYjdmYTRiOWI4OThjZDI/ordinal/69
210d997d2d828f73f4199a220e06833b7f0597a5
refs/t3/orchestration-v2/checkpoints/MTc2ODBlNGY3MGMyYzM5MzBjYjdmYTRiOWI4OThjZDI/ordinal/70
d78913394afe9f9da36bb54f5eb80ae2a06fa599
```

## Retained boundaries

Identity remains **cache77/policy71/schema38**. The cumulative correction
changes only the reader and tests: no production, engine, vstd, standard-model,
or new Infer capability. This bounded recheck does not reopen the completed
broader semantic review. Existing acceptance remains limited to extracted
selector/accessor/swap bodies, authored conditional selection/frame contracts,
the native StorageError declaration, and the `primary_slot < 2` precondition.

Actual io::Error and generic panic::Location are explicit authored trusted
OPAQUE DATATYPE abstractions, not inferred standard behavior or function-level
assumptions. No approval covers Error/Location operations, explicit or implicit
destructors, drop/release/effects/panic behavior, native caller establishment
of preconditions, whole-native-type or whole-crate refinement, recovery/tree
validity, storage/fsync/crash durability, observational equivalence, or whole
database safety. An implicit-consumption empty/unit contract is not a
destructor proof. Rust/Verus/SMT and existing trusted standard/opaque models
remain trust boundaries. Existing vstd/runtime were REUSED, not freshly
verified or rebuilt. Registry/fingerprint counts are not counts of newly
proved application behavior.

## Evidence and reproducibility

All relative evidence paths below are under
`/private/tmp/verus-redb-repair2-independent.KGzyRXEI`:

- `REVIEW-RECEIPT.json`, `FINAL-INVENTORY.json`, and this `REPORT.md`.
- `outputs/replay-normal/receipt.json`, `outputs/replay-optimized/receipt.json`.
- `evidence/units-results.json`, `evidence/replay-results.json`, `evidence/fresh-dispositions.json`.
- `evidence/probes-normal.json`, `evidence/probes-optimized.json`.
- `outputs/probes-{normal,optimized}/*.json`: all exact mutated matrices.
- `outputs/probes-{normal,optimized}/mutant-{joined,cardinality}.log`: full failure logs.
- `evidence/parent-evidence-audit.json`: explicitly audited-only counts and limitations.
- `evidence/{before,after}-integrity.json`, `evidence/reconstruction.json`.
- `evidence/repair-{before,after}.json`, `evidence/source-build-before-tests.json`.
- `evidence/apply-{1,2,3}.{stdout,stderr}`: each patch's single invocation.
- `review.py`, `probes.py`, `run_probes.py`, `audit.py`: reviewer-owned harness sources.

Exact executed argv, modes, output paths, exit codes, and counts are retained
in the corresponding results JSON. The focused suite command, from the
reconstructed checkout, was (with and without `-O`):

```sh
python3 -B -m unittest discover -s source/tools -p 'test_redb_slot_selection*.py'
```

The portable command, also with and without `-O`, used the supplied immutable
runtime and read-only sysroot, and a DISTINCT fresh destination per mode:

```sh
python3 -B source/tools/check_redb_slot_selection_opaque.py \
  --application /private/tmp/verus-transfer-redb-miuzm2w3/redb \
  --verus /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/verus \
  --solver /private/tmp/verus-cycle77-review-fixes.XCkOUU/tools77/z3 \
  --sysroot /Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin \
  --output /private/tmp/verus-redb-repair2-independent.KGzyRXEI/outputs/replay-normal
```

Do not reuse either existing output destination or reapply reconstruction
patches. This report and its receipt are bounded review artifacts, not
candidate modifications or permission to expand the theorem/trust scope.
