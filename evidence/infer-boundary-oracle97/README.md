# Boundary oracle97 — parent correction of a review finding

The independent follow-up96 review identified a nonblocking runner weakness:
mentioning each expected test name anywhere in a log plus a passing aggregate
summary was enough to satisfy `checked_test_log`. That does not establish that
each named test actually completed successfully.

The runner now requires each exact expected libtest name to finish once with
`ok`, followed by one matching successful aggregate summary. Unknown names,
duplicates, incomplete outcomes, failed/ignored tests, premature or repeated
summaries and concatenated runs are rejected. The parser handles the actual
single-threaded `--nocapture` format, where driver diagnostics separate a test's
start from its final standalone `ok`. It is not an authenticator for malicious
or forged libtest output and does not establish complete filter conformance.

## Validation and retained evidence

- Added regression tests first: the old parser produced eight expected failure
  records, including subtests, across an eight-test suite. After the fix all
  eight Python tests pass. No Rust build, Cargo test or verifier was rerun.
- Re-evaluated ten unchanged archived logs: all seven historically successful
  logs pass the stronger oracle and all three failed logs are rejected.
  These are log checks, not new current-input Cargo acceptance receipts.
- The initial log-check harness incorrectly expected hardening95's failed
  replay1 to pass. Its sealed receipt/log confirms failure; the parser correctly
  rejected it. The corrected classification and this harness error are retained.
- `SOURCE.patch`, `ORACLE-DELTA.tar.gz` and `RESULTS.json` retain exact old/new
  runner/test bytes, raw pre/post unit logs, the recheck script and log hashes.
  Every archive member was compared with its source bytes. The results bind the
  archive and manifest hashes; no runtime binary or build cache is copied.

This is a small parent correction of the review finding, not another independent
review or an amendment to the frozen95/96 source snapshots. It changes only the
Python runner's log predicate and unit tests. Compiler semantics, native resource
admission, trust and all eighteen OPEN final redb cells remain unchanged. Other
follow-up96 findings, including marker-only pairing, stronger driver assertions
and runtime/provenance limitations, remain open.
