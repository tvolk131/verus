# Independent bounded cycle80 parent compatibility review

**Verdict: BOUNDED ACCEPTANCE. No findings within the exact ten-path parent compatibility scope.**
All required remaining checks are complete. The earlier interrupted review remains
partial evidence and supplies no approval. This verdict is independent, bounded,
and does not expand the production or native application theorem scope.

## Candidate and review boundary

Own tool-prepared worktree: `/Users/tommyvolk/.t3/worktrees/verus/t3-6af70113`.
Own scratch/evidence: `/private/tmp/verus-cycle80-complete-review.eaqa9w7s`.
Initial worktree was clean at HEAD
`3b05c1912b9d7314ad00c832c4065f4362562e02`; no reset or ref mutation was needed.
The accepted baseline was applied once, followed by the four supplied deltas once
in the required order, using `apply_patch`. All five patch hashes matched first.
All **2,165** resulting file hashes matched the supplied final manifest, and match
again after every check. The scratch mirror also matches all 2,165 bytes.
This review claims clean-before/final-after inventory checks, not an unrecorded
intermediate 2,145-file baseline checkpoint check.

1. `/private/tmp/verus-prefix-infer79.1eVEuO/final-package/combined.apply_patch` — SHA256 `d3e2b596b63e2c1dc8a4596a373c3abcfb338a3762ed6f1cebae281e7c4a5ceb`; applied once, exit 0.
2. `/private/tmp/verus-borrowed-vec80._x31ts_v/delivery/baseline-relative.apply_patch` — SHA256 `58491583d589785f0a65a12171b740dc139aa34e69f1cc32c9e5b0c0ee5049a1`; applied once, exit 0.
3. `/private/tmp/verus-redb-observation80i-handoff.f1HEV7/delta.apply_patch` — SHA256 `49e48bf8fd0c93dd0f013aa13d3b1316fda062f32e307550ae826d8551023960`; applied once, exit 0.
4. `/private/tmp/verus-fedimint-oplink80a-tcmgj0yj/delivery/baseline-relative.apply_patch` — SHA256 `0f3a3dc55522902382f682b0d49fd7eb10ed8603b556459a3b86139fac51a33d`; applied once, exit 0.
5. `/private/tmp/verus-cycle80-parent.rwZwT3/parent-integration-review/parent.apply_patch` — SHA256 `5d88b027cfd9a63006ea0aee93603b9ae87e83a4b56d0890643f66b4cd6663b6`; applied once, exit 0.

The parent patch changes exactly these ten manifest paths:

- `evidence/transfer-byte-coercion-integration80/redb-certificate-expectations.json`
- `evidence/transfer-byte-coercion-integration80/redb-opaque-fixtures.json`
- `evidence/transfer-byte-coercion-integration80/redb-provider-certificate.json`
- `source/tools/check_fedimint_prefix_infer79.py`
- `source/tools/check_redb_repair_observation80i.py`
- `source/tools/check_redb_slot_selection.py`
- `source/tools/infer_transfer_identity.py`
- `source/tools/test_fedimint_insert_link80a.py`
- `source/tools/test_redb_repair_observation80i.py`
- `source/tools/test_redb_slot_selection_opaque.py`

The seven production files match the independently reviewed production delivery's
after hashes. Both repaired redb Python files match the exact two-file recheck's
candidate ledger. Their separate bounded reports and all their checksum entries
were audited; their full production/repair reviews were not repeated.

## Pins, scope preservation, and acceptance regressions

Shared identity is exactly **cache79 / policy73 / schema38**, with
`verus-infer-v79:borrowed-byte-append:4096` and
`candidate-bound-evidence-v73`. The prefix checker imports the actual shared
identity constants; its import AST and runtime constant binding both match.

The exact producer string and the complete ordered **381-entry model inventory**
in the new certificate match the sealed production requirements, the actual
qualified provider certificate, and the production review's independently fresh
certificate. The requirements file itself is covered by the delivery's verified
SHA256SUMS. The pinned expectation digest is computed from that sealed inventory,
not accepted from arbitrary fresh output. Both redb certificate readers reject
an isolated fingerprint change at **each of all 381 positions**, in both normal
and optimized readers. Each mode has **800 intended rejections**, two genuine
controls, and zero incidental errors; these are reader probes, not new compiler
proofs. Archived identities, float/bool/string identities and schemas, wrong
policy and wrong producer reject at their exact intended guards.

The new fixture's complete expected provider/consumer/boundary reports, opaque
declarations and boundaries equal the historical cycle79 fixture. The exact
**43 provider scope rows**—all identities, typed flags and classifications—are
unchanged, including in the actual qualification stdout. Every one of the
baseline's **288 historical evidence files** remains byte-identical. Old fixture
identity/README text stays historical and receives no relabeling as current proof.
The qualification's source, VIR, rlib and both streams match all recorded hashes;
its exact runtime argv and zero exit are retained.

Each of the two calls in `validate_outcome` was independently deleted from a
scratch in-memory copy, never from source. The unmodified intended regression
passes first. Deleting `validate_shape(report_shape(data, source), expected)`
causes exactly four `ValueError not raised` subtest failures in
`test_outcome_checks_complete_report`. Deleting `validate_metrics(data, positive)`
causes exactly one such failure in `test_outcome_checks_fresh_work`. Both isolated
deletions were repeated under `-O`, with zero errors or skips. Raw mutation traces
and exact mutated function bytes are preserved. No other acceptance call was
removed with either mutation.

## Fresh execution versus audited evidence

- **Fresh:** 99 focused tests normal and 99 under `-O`, all successful with zero
  failures/errors/skips. These include prefix, opaque redb, integrated Fedimint,
  and the repaired redb receipt guards. The redb tests consume the existing real
  repaired receipt; this is fresh reader execution, not a fresh native repair replay.
- **Fresh:** optimized opaque replay **47/47**, and optimized prefix replay
  **29/29**. Actual compiler commands, raw streams, sources, paired exports,
  certificate mutations, seeded cleanup outcomes, source fidelity and input
  guards are preserved. Complete matrices and their bindings were independently
  revalidated after completion.
- **Audited:** the prior independent normal opaque **47** and prefix **29** raw
  receipts, source/stream hashes, exact matrix categories, scopes, contracts,
  certificates, import bindings and guards revalidate. The old interrupted
  optimized attempts receive zero credit.
- **Audited:** parent normal and optimized current opaque/prefix receipts also
  revalidate, with exact sealed model inventories. These are not counted as
  independently rerun by this review.
- **Audited:** the parent's broad normal/optimized logs each contain exactly 508
  success dots, the corresponding `Ran 508 tests` summary and `OK`, with no
  failures/errors/skips. They do not record individual method names, so this
  review does not infer per-method identities or fresh execution from them.
- **Audited:** the corrected all-six shared receipt's six summaries were
  independently recomputed using the reconstructed readers and match the recorded
  summaries exactly. Its **122 embedded compiler reports** equal actual stdout;
  **396 raw command streams**, **1,363 outer source hashes**, and **1,371 outer
  artifact bindings** were checked. Every child receipt is hash-bound, every
  controller stdout selects that same child directory, and the outer native
  before/after pin/status logs bind the correct clean applications. Full-request
  dependencies and provider pairs, page-relation complete cases, ordinary proof
  negatives, metadata/frontend controls and explicit trust/limits remain distinct.

Receipt readers were loaded only from the reconstructed independent code. The
historical all-six readers bind absolute source paths and command cwd at the
recorded origin. The audit retains those exact original checks, verifies that
origin's full 2,165 files, and enumerates/compares source inventories with Git only
in this review's own worktree. Relocated support paths are mapped to the recorded
paths only after byte equality checks. No parent reader module is imported and
no command executes in the parent checkout. The explicit adaptation and each
mapped path are recorded in `parent-suite-audit.json`.

The first shared-current attempt failed offline `dependency_resolve` using
`--dependency-home .../dependencies`; it is retained as a setup failure and gets
no acceptance credit. The successful corrected attempt uses
`.../dependencies/cargo-home`. The raw offline failure and final six successful
receipts were audited. Frozen manifest `running` fields are historical state;
no completion is inferred from them. The optional native80/runtime79 seven-case
receipts were guarded as inputs and receive no optional new proof/audit credit.

## Domain and meaning of the results

The current redb full-declaration provider still has **23 verified / 0 errors**,
**15 checked executable bodies**, **one checked proof body**, **43 scope rows**,
and **one inferred function**. The separate consumer has 5/0 and three executable
bodies. Identity and implicit-drop boundary positives preserve their exact five
and four executable-body scopes; they prove no general object identity or Drop
behavior. The extracted native selector methods, full seven-variant StorageError
and record fields stay source-connected; authored opaque `io::Error` and
`panic::Location` TYPE boundaries remain explicit. The selection/frame result is
conditional on the existing representation/domain obligations, not a decoder,
tree validation, writer recovery or storage theorem. Semantic, frontend,
lifetime and import rejections retain their separate classifications.

The current prefix provider still has **23/0**, **eight checked executable
bodies**, **five checked proof bodies**, **16 scope rows**, and **five inferred
functions**. The consumer has **6/0**, four executable bodies and one auxiliary
inferred composition. The three original generic methods preserve full native
item/body bytes, field order/types/bounds, generic domains and the u32 token.
The two concrete Inner=u8 public wrappers and imported composition are explicitly
auxiliary. Exact content/range equations retain precondition true; namespace
separation still needs its explicit prefix-incomparability premise. Actual
variable-length BigSize module encoding is not proved prefix-incomparable.
Cold/warm/poisoned owned candidates regenerate; this is not a cache-hit claim.

The Fedimint insert-link tests enforce the existing conditional preparation /
operation-footprint contract, backend and namespace premises, complete report
scope and fresh work. They do not add native async proofs. Separate native finite
forwarding/cancellation/error tests remain finite evidence, and the actual async
trait extraction remains a frontend blocker. No production backend, concurrency,
cancellation, external exactly-once, funds or whole application theorem is added.

The separately accepted redb repairs enforce real positive artifacts and complete
native receipt bindings. Their authored read-only validation/error/frame gate
retains the full record and usize input domain with no precondition; its separate
match-arm normalization auxiliary is not the checksum-producing wrapper. The
gate does not prove index bounds, checksum/tree validity or fallback repair.
The 15/0, eleven-exec/three-proof, 23-row repair scope and finite three-test native
controls retain the meaning documented in that separate report. No new repair
native replay or ordinary failed-export cleanup claim is made here.

Reuse of Rust/Verus/vstd/SMT binaries, existing standard specifications and explicit
opaque datatype boundaries remains trusted. Soundness/safety, safe-domain
maximality, exactness, ordinary caller verification and ordinary lifetime/native
compilation remain distinct requirements. Hash/receipt consistency is not signed
execution attestation or a defense against arbitrary fabricated evidence.
No whole native async/backend, recovery, decoder/checksum/storage, crash/fsync/
durability, or application soundness theorem is claimed.

## Integrity, commands, and seal

Primary review-start ledger: **9113 files**, zero byte-hash drift. Supplementary
audit-start ledger: **9900 files**, zero byte-hash drift. Archived receipt origin:
**2165 files**, zero drift. Their union covers **21128 unique immutable file paths**.
Original symlink targets are unchanged. All 2,165 worktree and mirror files
match the final manifest; own HEAD/ref and native pins/clean states are unchanged,
with no staged paths. **0 shared ref entries changed** across the observed Git
inventories; exact deltas are recorded without attribution to this review.
The production delivery, production review, redb repair review and prior review
checksum ledgers all verify (3,085; 649; 1,108; 309 + 2 entries respectively).

The review performs no source fix, native input edit, delegation, Cargo/compiler/
vstd rebuild, network operation, shared cache write, commit/push/manual ref or PR
action. Fresh compilers use immutable `runtime79/verus` through supported rustup
with `RUSTUP_TOOLCHAIN=1.99.1`, explicit
`/Users/tommyvolk/.rustup/toolchains/1.99.1-aarch64-apple-darwin`, and no DYLD/LD
overrides. All fresh outputs, mutations, caches and temporary files are private
scratch. Runtime78 is hashed, not executed. Sealed implementation `replay.py`
was never run. Inherited GIT_* is removed and optional locks are disabled.
Historical all-six commands use recorded supported rustup selection; their lack
of explicit sysroot flags is recorded separately from the stricter fresh runs.

Exact independent subprocess argv, cwd, effective execution controls, exit codes,
durations and raw stream hashes are in `commands-and-log-hashes.json`. The raw
optimized replay receipts retain every compiler command. Entry commands were:

```sh
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/prepare_review.py
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/independent_runner.py initialize
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/independent_runner.py focused
python3 -B -O /private/tmp/verus-cycle80-complete-review.eaqa9w7s/independent_runner.py focused
python3 -B -O /private/tmp/verus-cycle80-complete-review.eaqa9w7s/independent_runner.py opaque
python3 -B -O /private/tmp/verus-cycle80-complete-review.eaqa9w7s/independent_runner.py prefix
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/pins_and_gates.py pins
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/pins_and_gates.py probes
python3 -B -O /private/tmp/verus-cycle80-complete-review.eaqa9w7s/pins_and_gates.py probes
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/pins_and_gates.py gates
python3 -B -O /private/tmp/verus-cycle80-complete-review.eaqa9w7s/pins_and_gates.py gates
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/receipt_audit.py matrices
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/receipt_audit.py suite
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/receipt_audit.py seals
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/additional_bindings.py
python3 -B /private/tmp/verus-cycle80-complete-review.eaqa9w7s/finalize_review.py
```

The recorded test/audit commands were wrapped with `dispatch.py LABEL ...` to
capture both streams and exact invocation metadata. Six reviewer audit-harness
setup/parser attempts failed before final audit acceptance; all are preserved
in `uncredited-audit-harness-attempts.json` with zero acceptance credit. They are
not candidate findings or incidental target-test errors. No compiler replay or
focused test was failed/omitted by these attempts.

Do not rerun the restoration, finalizer or writer entrypoints in this sealed
directory or on an already dirty tree. Reproduction requires new owned scratch
and explicit output/root rebasing. The supplied sealed writer was never replayed.
`receipt.json` is the machine verdict; `SHA256SUMS` binds all review files except
itself. Only this owned directory is sealed read-only, with symlink targets left
untouched. This is a local integrity seal, not a signature.
