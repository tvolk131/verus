# Native real-MIR integration campaign115

## Immediate target

Replace status-only test evidence with an actual scratch compiler build and
source-connected fixtures exercising Builder temporary capture, elaborated MIR
drop-site collection and explicit refusal. A passing diagnostic is not native
resource correspondence, ownership proof, a trusted producer or a final cell.

Inputs are the frozen reviewed revision2 snapshot under
`/private/tmp/redb-compiler-validation114-worker-20261108T151020Z/snapshot`,
bound by source fingerprint
`c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23`,
and round114's immutable source/receipt and independent-review manifests.
The changed live parent is not a substitute compiler baseline. Parent source
and all sealed prior evidence stay unchanged; only new scratch and campaign
evidence documents may be authored, through apply_patch.

## Parallel work

1. Implementation lane: build a provenance-bearing actual scratch engine,
   minimal companion libraries as required, and real-MIR temporary/refusal
   fixtures with matched ordinary controls. Capture full commands, inputs,
   outputs and true child status. Repair small in-scope failures and keep every
   attempt. Do not rerun the unchanged native Mutex/Hasher probe or use pure
   metadata tests as a replacement for executing actual MIR hooks.
2. Sensitive independent lane: inspect the capture/lifecycle boundary and
   implement an independent bounded diagnostic/receipt oracle with parser and
   mismatch controls. Distinguish refusal from panic/tool failure, matching
   observations from unique occurrence, and raw capture from resource proof.
   This lane may run its own small offline Python tests, but no compiler or
   native/verifier execution and no candidate modification.
3. Parent: reconcile the lanes, repair small findings, independently review the
   actual executable evidence, and advance the next defensible native dependency
   without another ordinary checkpoint approval. No automatic acceptance or
   trust adoption follows from any local success.

## Engineering guardrails

Only one compiler-heavy worker at a time, Cargo jobs1. Each compiler command
has a 15-minute timeout and diagnostics a 90-second timeout, with owned process
groups terminated/reaped on cancellation. Implementation lane uses at most
45 minutes and 6GiB fresh retained scratch before handing back exact partial
results; the independent lane uses at most20 minutes and64MiB. These bound
individual work packages, not the campaign or theorem domains. Parent may
continue with a distinct useful task or a justified correction under standing
authority; old accounting is never erased.

Maintain global free disk strictly above16GiB, use installed pinned toolchains
and existing dependency sources in place, never copy targets/toolchains/caches,
download or auto-clean. Preserve honest unversioned source-digest metadata and
full build/probe/test receipts. A source correction changes the source digest;
do not retain a stale digest or retrospectively relabel an earlier failed run.
No Git writes, parent/native source integration, original native application
execution or hidden admission bypass is allowed.
