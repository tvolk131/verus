# Owned-cache diagnosis99 — source-bound engineering gate

The strict persisted warm-candidate prerequisite remains **UNMET**. This
separately delegated, source-only investigation explains the observed behavior;
it does not implement persistence or waive the gate.

## Established findings

- The inherited hex helper takes the owned-summary branch, which deliberately
  regenerates its candidate and bypasses scalar-cache disk read/write. Thus the
  empty cache is explained by source control flow, not a diagnosed I/O error.
- The scalar cache wire language is deliberately restricted. Extending it to
  this helper's complete nested result/error values requires a new restricted
  codec and typed correspondence, not merely connecting existing cache calls.
- Export destinations enter the key's configuration. This is a confirmed
varying input across runs 05/06, not an isolated explanation of every observed
  candidate-ID difference. Lookup keys and installed proof identities differ.
- Fifteen operational source files match the frozen manifests. Two changed
  parent source files are recorded as drift and excluded from operational
  conclusions. Dirty parent HEAD is not substituted for engine provenance.

The report specifies a minimum future schema/key task, cold/warm separate-process
reuse, semantic invalidation, malformed-entry/failure controls and fresh checks
of all three certificates after every candidate load. Candidate data remains
untrusted: no proof-success bit, accepted flag or solver result may be cached as
authority. A different runtime needs its own qualification and review.

## Retention and disposition

[REPORT.md](REPORT.md), SHA256
`8bfdf78dba5fe484b489a09af18ff095af9b0a71d20a36f1071525dcb8d0a421`,
is unchanged. `SOURCE-DIAGNOSIS.tar.gz` contains all four source-research handoff
files: report, source/hash evidence and input/output manifests. Every member was
compared with its source bytes. The parent reauthenticated all 63 input records.
`RESULTS.json` binds the archive and individual hashes, including observed drift.

No implementation, runtime execution, integration, new trust or cache acceptance
is adopted. Another regenerated-state proof replay is not planned as a cache
test. Cache implementation remains a separate design/qualification lane; the
main proof lane returns to an existing redb validation gap rather than treating
this feature-sized dependency as a reason to stop all progress. All 18 final redb
cells remain OPEN.
