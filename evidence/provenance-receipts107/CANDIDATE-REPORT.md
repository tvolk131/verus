# Provenance receipt hardening107 — implementation candidate

## Status and integration boundary

Implemented, hermetically tested, and returned as an additions-only patch.
**Independent review is required before parent integration.** No independent
review was performed by this implementation sub-agent; no subdelegation occurred.
Parent source, evidence, and Git were never written. No Git commands, network,
Cargo, verifier, solver, compiler, vstd, native target, or native runtime runs
occurred. Only Python/shell/bookkeeping utilities and harmless Python fixture
subprocesses ran. No runtime, toolchain, or cache was copied.

The existing receipt correctly and explicitly limits its selected-input scope.
That limitation is not characterized as an existing defect. This candidate is
an opt-in reusable helper, not a replacement, producer, adoption, or provenance
claim for the frozen engine. No trust, admission, export, certification, or S0
changes are included. All18 final redb cells remain OPEN; native proof credit0.
No excluded growth049, zero-growth050, mutation030, refused-padding95, or
drop-pipeline work was resumed or rerouted.

## Actual candidate

The baseline-relative `candidate.patch` adds exactly three previously absent files:

- `source/tools/capture_provenance_receipt.py`: standard-library capture helper.
- `source/tools/test_capture_provenance_receipt.py`: 38 hermetic unit tests.
- `source/tools/PROVENANCE_RECEIPTS.md`: strict schema, use, and limitations.

Patch SHA256: `e319638a6ff0c5552f1136e041b652b4131995133df5b365caa1db83dad3707d`.
Patch size: 38831 bytes. `CANDIDATE-IDENTITIES.json` records each resulting file.
`BASELINE-INPUTS.json` records 107 read-input hashes and the three absent-file
baselines. `BASELINE-EXIT.json` rechecks them with no endpoint drift. These hash
snapshots were taken after discovery, not before first reads, and are not atomic
or complete parent-checkout authentication. No Git continuity is asserted.

## What the implementation checks

- Strict allowlisted plan schema; rejects malformed/unknown fields, unsafe paths,
  invalid digest formats, duplicate selectors/outputs, and nonfinite, Boolean,
  nonpositive, or unrepresentably oversized timeouts before evidence creation.
- Expected source/input hashes must match the complete selected file/tree
  inventory. Source file modifications, selected additions/removals, symlinks,
  unreadable tree scans, and missing inputs fail closed. Unselected files are
  explicitly outside scope, with a positive test showing that boundary.
- The exact launched argv/cwd are retained in the plan; a fresh receipt directory
  is mandatory. Executable identity and optional declared compiler-file identity
  are separately hashed before/after the command. Compiler role is caller-declared,
  not authenticated merely because a file has that name.
- Required outputs must all be absent before launch; seeded bytes/symlinks are
  refused without deletion. Missing, nonregular, symlinked, or old-timestamp
  outputs fail. Output identities are retained even on command failure.
- Spawn errors, nonzero exits, timeouts, source/compiler drift, missing/tampered
  logs, and failed rescans prevent success. POSIX timeout kills the process group;
  a hermetic child-marker control exercises this behavior.
- The child receives only LANG=C, LC_ALL=C, TZ=UTC. No inherited environment is
  copied/dumped; a fake-secret canary confirms exclusion. Arbitrary command
  arguments/output must themselves be kept free of secrets by the caller.
- Argv-only classification separates `vstd-only-command-shape` from the narrowly
  recognized `engine-build-command-shape`; vstd flags take precedence. Neither
  label nor a zero exit establishes a compiler build or compiler/source linkage.

## Raw validation and retained fixtures

Final candidate: `logs/unit-004-regression-green.log`, 38 tests, OK.
Exact patch dry-run and reconstruction: `logs/patch-dry-run.log` and
`logs/patch-apply.log`, both exit0. Reconstructed files are byte-identical to the
candidate; `logs/unit-005-reconstructed-final.log` independently runs the same
38 tests from those reconstructed files, OK. This is reconstruction validation,
not independent human/agent review.

All earlier logs remain: unit-001 (30 OK), unit-002 (37 OK), unit-003-regression-red
(38 run, one failure and one error). That red run exposed an oversized-timeout
OverflowError and a scan-error expectation failure in the developing candidate.
The subsequent changes reject the oversized number with ValueError and use an
error-propagating directory walker. No failed result was discarded or counted as
a passing run. Earlier incremental logs are not runs against the final source.

`SCENARIOS-FINAL.json` indexes nine retained final-code receipts: one success plus
missing output, stale output, seeded output refusal, source modification, source
addition, nonzero command, timeout, and compiler drift. All eight negative
controls refuse success. Every command is a harmless Python invocation with
fake source/compiler/artifact bytes. No compiler binary is executed. Earlier
development fixtures also remain under `scenarios/`; final fixtures are under
`scenarios-final/`. Exact fixture commands and selected input/output identities
are in their plans and receipts; the retained driver reconstructs original fake
source bytes for the intentional drift cases.

## What none of this proves

Source/input/output digests and arbitrary command receipts do not establish
binary/source correspondence, authentic tool roles, an engine build, complete
dependency/cfg/toolchain/runtime inventory, historical custody, reproducibility,
or native semantic correctness. The receipt explicitly records these as not
established. An existing compiler compiling/verifying vstd is not a compiler
build. The frozen compiler's source/build relationship remains unestablished.

These are endpoint observations, not atomic snapshots or execution attestation.
Transient changes/restorations, executable swaps between hash and launch,
interpreter/shared-library provenance, artifact copying/retimestamping, and
detached or surviving descendants are not excluded. Commands must be trusted;
this helper is not a sandbox or receipt-authenticity validator. Coarse file
timestamps/clock rollback can conservatively reject a genuinely fresh output.

## Handoff inventory

`COMMANDS.json` records the substantive execution ledger and exit statuses.
`ACCOUNTING.json` records elapsed time, scratch logical/allocated bytes, disk
samples, and zero native/Git/network/delegation counts. `MANIFEST.json` lists
all retained payload files by path, byte size, and SHA256; it excludes itself and
the final seal to avoid circular identities. `SEAL.json` authenticates the
manifest/accounting/report/patch contents only, not their truth or provenance.

No parent integration is authorized by metadata or test success. The parent
must arrange independent review and decide separately whether to integrate.
