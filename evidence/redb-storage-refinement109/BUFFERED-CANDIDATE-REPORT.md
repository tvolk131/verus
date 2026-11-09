# Lane 1: checked buffered-flush candidate, not native closure

**Result:** the final authored ghost/specification library passes a full-crate
existing-verifier check: **31 verified, 0 errors, direct child status 0**.
All eighteen native redb cells remain **OPEN**; no acceptance or integration.

## Deliverables and exact identity

* Source: `buffered_flush109.rs`, SHA256
  `29485224c99ba2f994012ff53b2ddc98c9cab79daaf7b3fc2a855b952d2fdc4f`.
* Exact additive candidate patch: `candidate.patch`; hashes in HANDOFF.json.
* Final exported candidate: `attempt-013/buffered_flush109.vir`, SHA256
  `9c7ae00ff60288e612e557b4cdc770d1e38c71a9c94b697ae8c0195eb0a67c3f`.
* Full correspondence ledger: CORRESPONDENCE.md; nine passing ordered-token
  source observations in source-anchors.json. Neither is a formal extraction.
* results.json, scope.json, restoration.json, per-attempt input/argv/environment/
  direct status/raw stdout/raw stderr/artifact hashes; MANIFEST.json,
  HANDOFF.json, and SEAL.sha256 seal the retained handoff.

## Checked symbolic fragment

The definitions generate write attempts, error returns, stripe drains, flag-clear
and sync events from arbitrary finite nested pending sequences and backend
decisions. No desired issue_write/sync trace is a theorem premise. Recursive
decreases/induction are unbounded, not finite testing or loop unrolling.

* Application pending maps are separate from backend-visible and durable images;
  arbitrary-length same-offset buffering coalesces to the last bytes without
  changing the backend. A singleton snapshot bridge checks the coalesced flush.
* Every successful-prefix length before a failing write is covered; the entire
  failing stripe is retained, earlier completed stripes are empty, later ones
  unchanged. A failed write may visibly and durably change part of its range.
* Iteration order and stripe/cache lengths are arbitrary. The final successful
  image is the exact overwrite fold; any uniquely covered byte has its buffer's
  durable value regardless of its stripe/cache position. Overlaps use last-write
  order rather than an invented native non-overlap guarantee.
* No sync is generated on a write error. All drains precede the final sync;
  pending stripes remain drained on sync Err. Successful modeled flush makes
  durable equal the appropriate backend-visible fold.
* Partial persistence can occur on both write and sync Err without rollback.
  These are chosen conditional model laws, not adopted native contracts; the
  allowed schedule does not exhaust arbitrary historical/native crash images.
* Boundary::OpenAbnormal explicitly separates unestablished panic/abort/resource
  paths; it gives no state or durability guarantee and is not an acknowledgement.

## Controls and accounting

| Attempts | Outcome |
| --- | --- |
| 001,004,006 | Retained development failures: trigger, incorrect event-index equality/instantiation/unfolding, and incomplete multi-variable trigger. Repaired, not hidden. |
| 003,005,007,008 | Full-crate positives: 13,23,29,31 verified respectively. |
| 009 | Err-as-rollback mutant rejected at the partial-write witness. |
| 010,011 | Eager-drain mutant rejected at stripe retention and arbitrary successful-prefix contracts. 011 is a repeated mutant after a failed patch, despite its original label; patch-failure.txt discloses this. |
| 012 | No-op successful sync rejected by symbolic durable-equals-visible postcondition. |
| 013 | Corrected, byte-identical restoration of 008: 31 verified, 0 errors. |

Twelve actual verifier starts plus one failed mutation-setup patch are charged
conservatively as **13/16**. Attempt number 002 is absent due to an early filename
counting bug, not a missing launch. Total direct-child application time is 9.70s;
one worker, no timeout, each below 90s. Start 2026-11-08T03:42:24Z; restored
positive 03:56:05Z. Final elapsed/free-space/scratch samples are in the manifest;
all observed free space exceeds 16 GiB and scratch remains below 768 MiB.
Observations are sampled boundaries, not continuously enforced quotas.

## Authentication and explicit remaining edges

Both supplied PLAN/INPUTS hashes match. All 369 current selected records match
at entry, before every verifier execution, postchecks and exit. The five disclosed
historical document drifts are not missing compiler provenance. Existing runtime,
matching rust_verify/z3/vstd/builtins, direct installed toolchain libraries and
additional used source/tool files are hashed; no unrelated extern/imports.
The controller's /usr/bin/python3 shim was resolved to its installed framework;
its real target/modules were additionally authenticated before attempts 012/013
and at exit, not retroactively claimed authenticated before discovery. OS shared
cache/system loader libraries are not an exhaustively authenticated dependency
closure. No atomic first-read provenance claim is made.

Actual WritablePage/lifecycle write-buffer admission/glue, LRU enumeration,
std/locks/Arc/atomics/read-cache transfer, I/O/backend laws, panic/abort/resource
reachability, and original-body execution binding remain OPEN. The prohibited
drop pipeline was not implemented or investigated. No untouched old secondary
is required to equal a newly prepared slot; no native header-format theorem is
claimed. Original path preservation **FAILED**, native Git **UNAVAILABLE**, frozen
compiler/source correspondence **UNESTABLISHED**. Existing-verifier checking does
not repair these. No new assume/admit/axiom/external_body, trusted adapter,
toolchain/cache copies, native execution, builds, network, Git writes, integration,
trust adoption, gate changes or cell promotion occurred.
