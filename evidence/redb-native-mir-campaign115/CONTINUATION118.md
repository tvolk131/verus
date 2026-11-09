# Continuation118 — cancellation-safe diagnostic collection

The original eighteen-cell goal remains active. This bounded dependency work
supports safe offline diagnostic collection; it is not a new database theorem
or a substitute for native WritablePage/cache/backend correspondence.

## Historical inputs retained

Original107 candidate: /private/tmp/provenance-receipt107.Puwj4s.
Original blocking review: /private/tmp/provenance-receipt107-review.NzBvWb.
Parent repair118: /private/tmp/redb-receipt-cancellation118.

Original107's manifest authenticates138 payloads and the old review's manifest
authenticates37 payloads. This parent recheck occurred after118's tests, not
before first inspection/staging. No pre-entry custody is retroactively claimed.
118's four selected source inputs and resolved interpreter match their stored
second-run hashes after the42-author/13-reviewer passing runs. Both the original
38-author/13-reviewer run and expanded42-author/13-reviewer run remain in logs.

## Independent source review and unresolved findings

<t3-thread://v2/ada676bc-6e81-41ae-be29-6dc09bd928c8/thread:delegated-task:command%3Amcp%3A8e0b49bf-52cf-472f-bc8a-1703e3df6abe%3Adelegate-task%3Af8a9afbb-2101-49f3-9942-7a472de5584b>

This source-only review authenticated all four selected files and interpreter
at entry/exit, did not write scratch or execute tests, and did not closeR1.

- HighF1: SIGINT insidePopen after OS creation but before handle assignment can
  strand a child. The original before-spawn mock never created that child.
- MediumF2: cancellation can signal a historical numeric process-group ID after
  Python's wait has already reaped the direct child.118 introduced this path.
- MediumF3: receipt-writing failure can replace the saved KeyboardInterrupt or
  SystemExit. The visible original exception/code is not reliably preserved.
- Wording: a process group is signaled; only the direct child is waited/reaped.

The interrupted-wait false-success defect itself is repaired by inspection:
success requires started/completed/reaped, zero exit, no timeout/cancellation
and no errors. That narrower result does not close the remaining lifecycle gaps.

## Parent correction under validation

Fresh scratch: /private/tmp/redb-receipt-cancellation118-revision2.
Only three small candidate source/doc files and the adapted old reviewer script
were copied. No target, dependency cache, toolchain or old result was copied.

The revised launch guard defers standard main-threadSIGINT until the direct
process handle and started status are published, then raises the interruption
for cleanup. Unsupported handlers/thread setup fail before spawning. Cleanup
signals a group only while its direct child is still unreaped; it never signals
an already-reaped historical identifier. Persistence failure preserves the
original interruption and chains the persistence error. These source changes
are not yet independently accepted and do not claim arbitrary exception or
detached-descendant containment.

New tests cover actual OS launch plus SIGINT before wrapper return, the
already-reaped case without real group signaling, unsupported handler/setup/
thread, and syntheticENOSPC while preservingKeyboardInterrupt/SystemExit. A
further realSIGINT test covers OS-reap/return-code publication; short guarded
wait slices defer delivery through that critical ownership update. Final
validation reports48 author tests and13 historical reviewer tests, both exit0.
Exact source/interpreter inputs, argv/environment, epoch-ns boundaries, raw
streams and exit statuses are retained separately for every new run. Failures
must remain visible; passing tests alone do not close the new source findings.

## Next original-native edge

Task116 was resource-cancelled when shared-disk availability fell below the
standing floor. Its partial source/build/diagnostic results must be reconciled
from a terminal handoff before continuation. No original-native capture,
proof-success, exhausted-allowance reset or replacement build follows merely
from the storage recovery. The worker was subsequently confirmed terminal
interrupted, with its last message saying validation passed and sealing was in
progress. Its directory contains an actual build binding, native-source MIR
captures,20 managed-command records and validation output, but no final worker
seal. The parent is preserving those partial results without retroactively
claiming the interrupted worker completed its handoff. The intended next
database step still derives actual
anonymous header temporary/caller/lifecycle correspondence, not another supplied
lookup fixture or standalone receipt theorem.

No integration, new primitive law, trusted producer, network or destructive
operation is authorized by this document. Ordinary Infer/full-scope/import/
export gates remain unchanged. All18 cells remainOPEN.
