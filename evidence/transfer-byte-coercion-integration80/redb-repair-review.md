# Independent bounded recheck: two redb P2 receipt repairs

Local review date: November 2, 2026, America/Chicago (CST).

**Verdict: bounded acceptance. No remaining findings in the two requested repaired
receipt-validation gates.** This is a read-only review: the three authorized input
patches were restored, but no candidate fix, Verus contract/body edit, new theorem,
inference claim, native/runtime/toolchain edit, rebuild, network operation,
shared Cargo-home write, commit/push/ref/PR action or subdelegation was performed.
All experimental receipt/artifact mutations and in-memory checker mutations are
reviewer-owned scratch only.

## Exact restoration and scope

- Started from clean prepared HEAD `3b05c1912b9d7314ad00c832c4065f4362562e02`.
- Applied the accepted baseline ONCE with `apply_patch`, after checking SHA256
  `d3e2b596b63e2c1dc8a4596a373c3abcfb338a3762ed6f1cebae281e7c4a5ceb`;
  verified all 2,145 accepted file hashes.
- Applied the observation delta ONCE, checking SHA256
  `49e48bf8fd0c93dd0f013aa13d3b1316fda062f32e307550ae826d8551023960`;
  verified all 2,152 delta hashes.
- Applied the repair ONCE, checking SHA256
  `a9b4e6d7762a64ce67ad79b7485b0881bd977cd0e202236b2b9952ce8b6a36bf`;
  verified all 2,152 final hashes. The discovery fix is included; no separate
  `parent-discovery.apply_patch` was applied.
- The repair changes exactly the checker and its Python tests, not any Verus
  contract/body, native source or shared engine. See `reviewed-repair.diff` and
  `restoration.json`. `patch_sha256` and `ledger_sha256` are recorded separately.
- Read the original `REPORT.md`, both repaired Python files, supplied handoff,
  scope qualifications and native harness fixture. Original report and reviewed
  source snapshots are included in `input-snapshots/`.
- Used only sealed runtime78 at
  `/private/tmp/verus-prefix-infer79-impl.Ks6IeN/delivery/runtime78`, identity
  `[78,72,38]`, and the sealed
  `/private/tmp/verus-prefix-infer79.1eVEuO/final-package` baseline snapshot.
  Newer parent 79/73 engine/compatibility work is OUT OF SCOPE and unused.
- Native redb remains clean at `c82ab865bb362bc4af067d332adc44978ff64813` under
  `/private/tmp/verus-transfer-redb-miuzm2w3/redb`.

## F1: real positive compilation artifact

At `source/tools/check_redb_repair_observation80i.py:220`, a positive row now
requires a lowercase 64-hex string digest, a real file, and nonzero file size.
The existing `library_binding` check subsequently verifies the actual bytes.

Freshly reproduced the original forgery: change only `positive_no` to a new
nonexistent output, set its digest to null, and synchronize its row/recorded
command. The original immutable checker accepts; the repaired full matrix
rejects exactly at `positive_library`. This comparison was executed for both
fresh matrices with both normal and `-O` readers, not inferred from old logs.

Additional independent probes reject empty artifacts with their correct empty
hash, directory artifacts, null/boolean/numeric/short/long/uppercase hashes,
and a validly typed wrong digest. A genuine nonempty copied artifact with
synchronized output argv accepts. All four fresh positive artifacts are real,
nonempty, hash-matching rlibs; exact byte sizes/digests are in `final-audit.json`.

## F2: complete native receipt boundary

At `source/tools/check_redb_repair_observation80i.py:258`, the repaired checker
requires strictly typed `tests=3` and `gate_inputs=64`, source hash AND exact
`native_harness(application)` contents, a real nonempty hash-matching binary,
exact rustc/test argv and both cwd values, strictly typed integer-zero exits,
exactly four native log keys, each real log's byte hash, and the complete
successful Rust test-summary counts including zero ignored/measured/filtered.
At line 314 the full matrix actually invokes this native checker. Replay also
uses the same count checker and records all four log hashes.

Freshly reproduced the original forgery: both final commands become
`/usr/bin/false` with exit 1, and native metadata becomes a nonexistent source,
zero hashes and zero tests/inputs. The immutable original checker accepts;
the repaired full matrix rejects exactly at `native_metadata`, for all four
matrix/reader combinations. Additional probes isolate later gates instead of
relying only on that first metadata failure.

Independent near misses cover a weaker rehashed source harness, empty rehashed
and changed/missing binary files, wrong compile/test cwd and argv, test filtering
flags, float/null/negative exit statuses, near numeric metadata, every missing
log field/file and stale log hash, and rehashed stdout summaries with failed,
ignored, filtered, measured, incomplete or wrong test counts. Genuine evidence
and a synchronized native-fixture copy accept. An independent wrapped-call
observation confirms exactly one native validation call on a genuine matrix.

## Fresh execution evidence

- `normal/receipt.json` and `optimized/receipt.json`: two NEW 28-case replays.
  Each has two positive cases with 15 verified / 0 errors, 11 checked exec
  bodies, 3 checked proof bodies and exactly 23 scope rows. The 26 expected
  negatives retain their precise classifications, not arbitrary-error credit.
- Each replay genuinely compiles and executes the native Rust harness: three
  tests, 64 gate flag/index combinations and all seven StorageError variants.
  Independent final audit checks the exact three successful test names and
  one actual summary line: 3 passed, 0 failed, 0 ignored, 0 measured, 0 filtered.
  Source, binary, four raw logs and all exact argv/cwd/exits are retained.
- The 43 focused methods pass under normal and `-O` readers against BOTH fresh
  matrices: four successful runs, 172 method executions. Absent-environment
  discovery separately skips explicitly in both reader modes, with no error.
- `near-misses-{matrix}-{reader}/`: 62 independent receipt probes per
  combination (3 accepted controls, 59 intended rejections), 248 fresh probes
  total, zero incidental errors. Both original forgery shapes are included.
- `guard-removals-{matrix}-{reader}/`: ten named guard removals per combination,
  each producing its intended regression failure and no incidental errors.
  Missing native integration is an EXACT single-call AST removal from
  `validate_matrix`, not merely a mock of all native checking. Raw logs retain
  the `ValueError not raised` failures (27 subtest failures per combination).
- `predicate-removals-{matrix}-{reader}/`: eight additional single-policy
  weakenings per combination, each caught by an independent near-miss regression
  with exactly one intended failure and zero errors. These isolate positive
  nonemptiness, exact native-harness equality, binary nonemptiness, compile cwd,
  typed zero exit, typed test count, compile-stderr hash coverage and ignored
  summary rejection. Candidate source is never patched by these probes.
- The supplied parent's ten-example driver is copied unchanged and freshly
  executed against both fresh matrices in both reader modes; results are
  `parent-examples-fresh-*.stdout`. Its historical ten-removal records,
  genuine receipts and four 43-method logs are separately audited in
  `historical-audit.json`; NONE receives fresh replay credit.
- Original checker comparison uses the immutable handoff's checker bytes,
  rebasing only `__file__` for accepted evidence lookup in this prepared
  checkout. Source hash and that rebasing are explicit in
  `original-comparison-*.json`. No live parent checker/engine is imported.

## Commands and exact repros

All fresh replay/test/probe argv, cwd, sanitized environment overrides, exit
codes and durations are in `commands.jsonl`; their stdout/stderr files are named
by command label. The two raw receipts retain all 30 compiler/native commands
each. The entry commands actually executed were:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/audit.py restore
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/fresh_runs.py
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/probe_runs.py
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/extra_runs.py
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/finalize.py
```

Do not re-run the restoration or reuse sealed output destinations. To reproduce
the original/repaired comparison and guard-removal regression against preserved
fresh evidence, use NEW owned output (repeat with `python3 -O`):

```sh
scratch="$(mktemp -d /private/tmp/verus-redb-p2-reproduce.XXXXXX)"
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/original_comparison.py --probes /private/tmp/verus-redb-p2-recheck.EmdqR2/near-misses-normal-normal --output "$scratch/comparison.json"
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/guard_removals.py --receipt /private/tmp/verus-redb-p2-recheck.EmdqR2/normal/receipt.json --output "$scratch/guard-removals"
PYTHONDONTWRITEBYTECODE=1 python3 /private/tmp/verus-redb-p2-recheck.EmdqR2/independent_probes.py --receipt /private/tmp/verus-redb-p2-recheck.EmdqR2/normal/receipt.json --output "$scratch/near-misses"
```

Git context is sanitized by removing inherited `GIT_*` variables and supplying
read-only Git configuration. Replays construct private HOME, Cargo, Rustup,
temp and XDG directories, with Cargo offline and explicit sealed executable,
solver and installed toolchain paths. Only the bounded requested compiler
matrix/native harness and small receipt-focused checks run; no compiler,
Cargo or vstd rebuild, new broad compiler suite, or network access occurs.

## Trust and exclusions

This accepts the two repaired consistency gates, not a cryptographic execution
attestation or arbitrary evidence-fabrication defense. Hashes bind supplied
bytes, and exact command records bind claimed execution provenance; the reused
Rust/Verus/vstd/SMT binaries/models and opaque datatype boundaries remain trusted.

No new theorem or inference claim is made. Only one successful executable body
is unchanged original native application code; other checked helper/proof bodies
and semantic contracts are authored. The match-arm sublemma is not a proof of
native checksum/storage production or the original wrapper as a whole. Finite
native tests do not prove the full application, native async behavior,
concurrency/cancellation, internal opaque behavior, implicit destruction/Drop,
or a storage protocol.

Full decoder/endian/parser/checksum/tree/caller refinement, I/O/locking/cache/
allocation correspondence, repair/recovery and crash/fsync/durability obligations
remain excluded. Invalid-index acceptance and false fallback goals establish no
reachable crash or native defect. Historical ordinary exports may remain after
failure despite `interface_written=false`; they are NOT certified imports or a
new failed-inference-export-cleanup claim. This recheck generates no exports
or imports and does not repeat the previous review's additional 14 semantic
probes or grant them fresh credit.

## Integrity and preservation

`guarded-before.json` / `guarded-after.json` cover 1,292 immutable inputs:
original supplied packages/review, parent repair package/driver/receipts,
complete 132-file pinned native tree, all 9 runtime inputs and 61 toolchain
inputs. Fourteen additional parent logs/removal records have separate before/
after manifests. Every byte hash is unchanged. The three original package
SHA256SUMS ledgers also verify in full.

All 2,152 reviewed worktree hashes match again; candidate-before/after are
identical. Each fresh replay independently preserves 2,354 guarded inputs.
Native state/pin and own HEAD/ref `refs/heads/t3/redb-repair-recheck` are
unchanged. At the final audit there are no shared-ref changes; closure records
any later concurrent shared-ref delta without claiming exclusive ownership.
See `final-audit.json`, `closure-audit.json`, the Git manifests and
`shared-ref-delta.json`.

`SHA256SUMS` binds all review package files except itself. The entire owned
review directory is sealed read-only after closure verification; file/manifest
hashes and readonly modes are checked again after sealing. This is a local
integrity seal, not a signed attestation. Original inputs are never chmodded.
