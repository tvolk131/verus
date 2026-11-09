# Lifecycle compiler-validation114 — reviewed engineering evidence

## Result and actual scope

The isolated revision2 capture/refusal candidate passes a full `rust_verify`
package check and five narrowly filtered metadata tests. One independent
source/receipt review found no blocking P1/P2 defect in the revised repairs.
This is bounded engineering acceptance, not source/runtime integration, a
working verifier-engine build, native lifecycle validation, or final-cell proof.
All eighteen redb property/history cells remain OPEN. The unchanged goal service
was observed BLOCKED again during the parent audit; no agent resume is claimed.

The candidate records typed temporary-origin observations, distinguishes missing,
incomplete, completed-empty, no-local-match and matching captures, and retains
an explicit refusal before inherited normalization for unsupported native drop
sites. It avoids new eager glue-resolution queries and labels unqueried glue
identity honestly. No ordinary Infer production behavior or accepted native
resource producer is added. Multiple observations do not establish one unique
MIR occurrence or an ownership/lifecycle correspondence.

## Compiler commands and review

Exactly two compiler-producing commands were used, both offline/locked with
the installed pinned 1.99.1 toolchain, one Cargo job, and separate fresh targets:

1. Full `cargo check --package rust_verify`: exit 0, 53.203 seconds. Both library
   and binary targets were checked; no working compiler executable was linked.
2. Filtered `cargo test --package rust_verify --lib` for
   `checked_normal_drop::origin_capture_status_tests::`: exit 0, 69.533 seconds.
   Exactly five pure capture-status tests passed; seven others were filtered
   out, not passed. Actual Builder capture, MIR/glue and native effects were
   not exercised. No source correction or compiler retry was needed.

The isolated source differs from the frozen historical snapshot at exactly four
paths: the three revision2 candidate files and the metadata build script.
The build script emits `unversioned-source-snapshot` and source digest
`c9cb297436ab24053c254ed95bd4e520ef6fca5b86832d217b37302cb5ad7b23`.
That digest identifies inventoried bytes, not a Git commit, bootstrap attestation
or compiler/source correspondence. Nine inherited function bodies remain
byte-identical under the independent review's targeted comparisons.

The review resolves the three earlier source findings: eager glue querying,
incomplete glue identity/successor labels, and missing-versus-empty origins.
The original revision1 verdict remains historically CHANGES REQUIRED. The fresh
review applies to revision2 and the retained command receipts only.

One nonblocking P3 documentation observation is carried forward explicitly:
both Cargo stderr streams report failed last-use-data persistence because the
cache database is read-only. Both commands nevertheless exit 0. These nonfatal
bookkeeping warnings, and the two retained Rust dead-code warnings, were not
suppressed; this was not a warning-free run. No retry or cache mutation followed.

## Retention and authentication

`PARENT-AUDIT.json` authenticates all 4,064 worker payloads, all 21 revision2
inputs, the seven review content payloads and the separately hashed review seal.
All 1,478 parent baseline records match before the disclosed final document
updates. `DOCUMENT-RECONCILIATION.json` records the exact old/new GOAL and STATUS
hashes; their pre-update bytes are retained alongside this summary.

- `WORKER-SOURCE-AND-RECEIPTS.tar.gz` retains all 1,230 non-target worker payloads
  plus its sealed manifest: complete source snapshot, overlays, metadata delta,
  raw logs, receipts, inventories, identities and failures. All 1,231 members
  were read back and compared byte-for-byte.
- `INDEPENDENT-REVIEW.tar.gz` retains seven review content payloads plus its
  content manifest and seal. All nine members were read back and compared.
- The 2,834 compiled target files are deliberately not copied into this evidence
  directory. Their hashes remain in `WORKER-SEALED-MANIFEST.json`; their bytes
  remain at the original worker root below. This is not a self-contained copy
  of all build outputs or a preservation claim for original filesystem metadata.
- Raw logs, exact source/metadata deltas, command ledger and both reports are
  also retained separately for inspection. Earlier sealed rounds remain intact.
- `PARENT-RETENTION-NOTES.md` discloses the first parent audit script's manifest
  key error. Its corrected retention run exits 0; no compiler/test rerun occurs.

Original worker root:
`/private/tmp/redb-compiler-validation114-worker-20261108T151020Z`.
Original review root:
`/private/tmp/redb-compiler-validation114-review-20261108T152149Z-independent`.
`ROUND-MANIFEST.json` hashes this round's final retained payloads, excluding
itself to avoid recursion. It is a content seal, not an acceptance certificate.

## Native gaps and next boundary

The next useful engineering target is a real-MIR capture/refusal integration
fixture, rather than another pure metadata test or another unchanged unsupported
native probe. It must expose real temporary observations, capture completion,
local/phase binding and refusal behavior without silently admitting resource
effects. No such fixture has been executed here; both authorized compiler slots
are exhausted, and no automatic extra build/review chain is started.

Even that fixture would not establish original Mutex/BuildHasherDefault
admission, Arc uniqueness, same-allocation byte authority, cache reservation and
return, poisoning, destructor/guard/field glue, header-to-flush correspondence,
backend/crash/history semantics or any final cell. Native-path preservation is
still FAILED, Git continuity UNAVAILABLE, compiler/source correspondence
UNESTABLISHED, and the historical TCB remains conditional and unadopted.
No new law, assumption, trusted producer, admission/export relaxation, source
integration or native proof credit is granted by this round.
