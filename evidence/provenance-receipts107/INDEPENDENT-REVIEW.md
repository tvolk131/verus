# Independent provenance-receipt107 review

## Verdict

**BLOCK the current patch from even narrow opt-in parent integration until R1 is fixed.**
The objection is an ordinary cancellation correctness defect with a trusted
Python fixture, not an adversarial isolation requirement. There is one blocking
source finding and one nonblocking wording/test-scope finding. No candidate or
parent files were edited; the parent should make the small fix directly, retain
the red reproduction, and rerun the supplied suite plus cancellation regression.
No additional automatic review chain is requested.

After that fix and successful regression validation, the reviewed boundary is
only the three new opt-in helper/test/documentation files. This review authorizes
no producer, trust, admission, export, S0, native acceptance, or build-pipeline
changes. It supplies no compiler provenance acceptance or whole-native claim.
All 18 redb cells remain OPEN and native proof credit remains zero. No excluded
growth049, zero-growth050, mutation030, refused-padding95, or drop-pipeline work
was resumed or rerouted.

## R1 — blocking, high priority: interruption can seal success without an exit

Source anchors in the candidate:

- `candidate/source/tools/capture_provenance_receipt.py:244`: wait can raise KeyboardInterrupt.
- `candidate/source/tools/capture_provenance_receipt.py:257`: Exception excludes KeyboardInterrupt/SystemExit.
- `candidate/source/tools/capture_provenance_receipt.py:295`: success requires started and no errors, but not exit_code == 0.

Reproduction uses the actual reconstructed helper, an actual harmless Python
child, and a real SIGINT delivered only to the capture process. The child first
writes the required fake artifact, then sleeps. SIGINT interrupts process.wait;
the helper enters finally without recording an error or stopping its separate
child process group. Endpoint checks pass, so receipt.json is written with:

```json
{"success": true, "command": {"started": true, "exit_code": null, "timed_out": false}, "errors": []}
```

At return-by-exception the child is still alive. The reviewer fixture then kills
and reaps the child group in its own finally block. The run ends with the intended
assertion failure, not a cleanup error. See `INTERRUPT-OBSERVATION.json`,
`logs/reviewer-adversarial.log`, and
`reviewer-observations/test_keyboard_interrupt_must_not_seal_success_or_leave_child_running/receipt.json`.

This is observable even with a trusted command and honest source/output bytes.
Consumers reading the durable receipt after user cancellation can incorrectly
accept an unfinished operation. It violates the documented command-outcome
meaning of success; it is not the documented detached-descendant limitation.

Required small fix: make completed zero exit, no timeout, and no errors necessary
for success; record interrupted capture as failure; terminate/reap the launched
process group before final endpoint observations on cancellation, while preserving
the caller's KeyboardInterrupt/SystemExit semantics. Retain a regression that
asserts success=false, recorded interruption, and no still-running fixture.
Merely adding exit_code == 0 prevents the false-success receipt but does not fix
the live-child cleanup aspect. This is one cancellation-lifecycle finding, not
two unrelated blocking demands.

## R2 — nonblocking: log-tampering language exceeds the tested endpoint check

`candidate/source/tools/capture_provenance_receipt.py:291` hashes the final pathname
as a regular file; it does not bind that pathname to the original stdout file
descriptor. `candidate/source/tools/test_capture_provenance_receipt.py:270` tests
unlink-and-symlink replacement, although its name speaks of tampering generally.
HANDOFF.md similarly describes missing/tampered logs as preventing success.

The reviewer-only diagnostic replaces command.log with another regular file
after emitting harmless output. The helper returns success=true and records the
replacement bytes' hash. The behavior is retained in
`reviewer-observations/test_regular_log_replacement_is_endpoint_observation_not_authentication/`.

This is **not a blocking authenticity/isolation defect** at the documented
trusted-command boundary: the documentation expressly excludes trustworthy command
behavior and requires separate receipt custody. Recommended narrow action is to
rename the test to describe missing/nonregular/symlinked-log rejection and limit
the handoff wording to that actual oracle, optionally documenting final-path-only
log identity. Do not infer general tamper resistance or require a new sandbox.

## Independent reconstruction and validation

- All four user-pinned SHA256 identities match: patch, manifest, handoff, parent archive.
- All 138 manifest payloads authenticate; all 140 regular archive members match candidate bytes exactly.
- All 107 parent baseline input hashes match BASELINE-INPUTS, BASELINE-EXIT, and BASELINE-FINAL.
- The three proposed parent paths were absent. No scoped AGENTS.md was found in the parent/candidate trees or their checked ancestor directories.
- Applied the authenticated patch in a fresh scratch tree after a successful dry run; exactly three reconstructed files match candidate bytes.
- Reconstructed final source: **38 supplied hermetic tests pass**, exit 0, `logs/unit-38.log`.
- Reviewer-only suite: **13 checks, 12 pass, one fails**, exit 1, `logs/reviewer-adversarial.log`. The sole failure is R1, not an original-suite failure.
- Additional checks cover final rescan failure, an extra expected inventory key, FIFO inputs/outputs, canonical cwd/output-parent refusal, executable drift, separate/direct declared identities, multiple inherited-environment canaries, further malformed types, vstd flag precedence, and the R2 scope diagnostic.
- All 25 supplied raw unit/scenario/packaging logs were read and retained verbatim in RAW-CANDIDATE-LOGS.json. All nine final retained scenario summaries agree with their receipts: one success, eight refusals.
- Earlier 30/37 green and 38 red development logs are historical only. The historical red reports one oversized-timeout OverflowError and one scan-error expectation failure; it is not validation of final source. Supplied final 38-green/reconstructed logs remain distinct from the new cancellation red.

## Scope assessment

No additional blocking objection was found in strict schema/type/timeout
validation, complete selected-file/tree inventory matching, endpoint additions/
removals/content drift and scan-error refusal, static unsafe paths/symlinks,
seeded/missing/stale/nonregular outputs, executable versus caller-declared
compiler identities, argv-only classification and vstd-only precedence, or the
tested spawn/nonzero/timeout paths and same-process-group timeout cleanup.
The initial runner's explicit selected-input scope is correct and is not a bug.

The child launch uses exactly LANG=C, LC_ALL=C, TZ=UTC. The reviewer independently
tested absence of synthetic HOME/PATH/RUSTFLAGS/token canaries. Neither inherited
environment values nor real secrets were dumped. The outer test harness adds only
its scratch TMPDIR to the same three variables; programs can alter their own
environment. Caller-controlled argv, paths, and command output still require
secret hygiene.

Trusted commands are not sandboxed. Endpoint hashes are not continuous custody
or atomic snapshots; executable/interpreter/library substitution is not excluded.
Command-shape labels prove no build. Source/compiler/output identities and these
receipts do not cryptographically attest source/binary correspondence, dependency
closure, reproducibility, or native correctness. R2 does not expand that boundary.

## Reproducibility, accounting, and custody

Fresh scratch: `/private/tmp/provenance-receipt107-review.NzBvWb`.
Parent and candidate are read only. No Git commands, subdelegation, network,
downloads, Cargo, Rust compiler, verifier, solver, vstd, native target/runtime runs,
or toolchain/runtime/cache copies occurred. Only Python tests, harmless Python
fixtures, and read-only bookkeeping/patch utilities were used. No prior evidence
was deleted; temporary fixtures clean up their own newly created workspaces.

Substantive commands, exact argv/cwd, sanitized harness environment, stdin hash,
exit statuses, and raw logs are in COMMANDS.json. The orchestration entrypoints,
executed from the unchanged parent cwd, were:

```text
/usr/bin/python3 -I -B /private/tmp/provenance-receipt107-review.NzBvWb/audit.py prepare
/usr/bin/python3 -I -B /private/tmp/provenance-receipt107-review.NzBvWb/audit.py adversarial
/usr/bin/python3 -I -B /private/tmp/provenance-receipt107-review.NzBvWb/audit.py finish
```

Those three orchestration processes did not dump or pass their inherited
environment to child commands. Exploratory read-only shell discovery commands
and apply_patch authoring of reviewer scratch files are not a syscall-complete
ledger. No permission escalation was required.

ENTRY.json and EXIT.json record candidate and all used parent input identities,
archive identity, proposed-path absence, and the executed Python identity.
The entry snapshot was after initial discovery/source reads and before patch
reconstruction/tests; pinned digests authenticate the supplied baseline, not
continuous history. ACCOUNTING.json records endpoint drift, elapsed time, and
phase resource samples. Samples are not continuous global monitoring. Scratch
must stay below 256 MiB and sampled global free space above 16 GiB. Final manifest,
seal, and compressed handoff add only small bounded bookkeeping after the
accounting sample. No optional work continued past the 12-minute cutoff and the
review is sealed inside the 15-minute total budget.

MANIFEST.json hashes retained review payloads; SEAL.json hashes the manifest,
report, and accounting; REVIEW-HANDOFF.tar.gz contains those payloads and seals.
The archive's independently printed hash authenticates bytes, not truth,
compiler correspondence, or historical custody.
