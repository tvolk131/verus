# Hex body-preservation98 — separate checked metadata audit

The independent hex96 review found that a function-name set could collapse two
distinct `From` implementations. Its manual review confirmed both bodies were
present. This separately delegated audit closes that narrow automation gap for
the retained evidence without changing the original harness or proof runs.

## Result and scope

- Exact Rust implementation identity, source-relative filename and normalized
  four-coordinate source spans are compared as multisets. Both same-labelled
  conversions match individually in positives 03, 05 and 06: **32 original rows per
  positive, 96 individual matches total**. Counts/labels alone are insufficient.
- The audit checks the pinned source archive, unchanged other files, full
  positive source maps and raw output bindings. Original lib.rs spans are mapped
  through insertion-only source correspondence; ambiguous/unmappable spans fail.
- All **23 AUDIT/METADATA tests** pass, including missing-overload/duplicated-
  sibling controls with unchanged total count and function-label sets. Other
  controls cover implementation/span/path changes, multiplicity, scope counts,
  line-map failures and duplicate JSON keys. These are not native semantic
  negatives or additional proof obligations.
- The parent inspected the audit and reran it and the same23 tests in fresh
  scratch. All commands exit 0 and the complete matches equal the delegated
  result exactly. This is a parent replay, not a separate independent review
  or 46 distinct test cases.

This establishes source/metadata identity correspondence for these records,
not executable-body equivalence, semantics of inserted code, compiler-reporting
correctness, fresh proof replay or runtime acceptance. It remains conditional
on the pinned metadata and selected-scope classifications. No product tool or
application source is integrated. The original frozen name-set harness remains
unchanged; future reliance should use this separate layer, not claim that the
historical harness was stronger than it was.

## Retention and rerun

- [Report](REPORT.md), unchanged, SHA256
  `52782004ae3dc74d650ab9ce610765a46d6c1e35ece8ceb82c771d0ececb0c9a`.
- `audit.py` and `test_audit.py` are exact delegated source copies for inspection.
  The audit deliberately permits outputs only beside a fresh scratch copy under
  `/private/tmp`. Do not run it against the frozen handoff directory. With the
  pinned inputs still at their recorded paths, copy these two files into a new
  `/private/tmp` directory and run `python3 -B audit.py`, followed by
  `python3 -B -m unittest -v test_audit` there.
- `AUDIT-HANDOFF.tar.gz` preserves all 18 delegated manifest entries plus the
  manifest itself, and the separate parent-run code, logs, matches and receipts.
  Every member was compared with its source bytes. `RESULTS.json` binds archive
  and member hashes and keeps the original candidate result separate.
- The delegated manifest SHA256 is
  `026030e30be4d6c66371b65c7756ed624ba986f0f8b674e76bd184ab34e6d809`.
  Entry/exit/final authentication preserves all 143 original repair files.

No native executable, build, verifier or solver ran. The persisted warm-cache
gate remains **UNMET**; the [separate cache diagnosis](../hex-cache-diagnosis99/README.md)
now explains the owned-summary bypass without implementing persistence. No new
trust/admission or acceptance decision is made. All 18 final redb cells stay OPEN.
