# Campaign91 shared-outcome review repair1

The independent workflow review at
`/private/tmp/verus-review91-workflow.eAgPfN/REPORT.md` found F1/P2: calling the
shared positive validator without an expected error count accepted a report
that simultaneously claimed success and nonzero proof errors. Existing pinned
integrations supplied `errors=0`, so their recorded results were not affected.

The repair makes zero proof errors an intrinsic requirement of positive
outcomes, independent of whether an expected count is omitted, explicitly None,
or explicitly contradictory. No proof expectation, scope, source identity,
runtime qualification or native code is changed. Two regression methods cover
omitted/None/default expectations and attempts to explicitly authorize positive
errors. An isolated removal of the new predicate must make its regression fail.

Fresh parent validation is retained in
`/private/tmp/verus-operational91.x12fMK/repair1`:

- Focused shared tests:13/13, exit0, wall2.368s.
- Integrated Python suite:112/112 normal, exit0, wall10.744s.
- Integrated Python suite:112/112 optimized, exit0, wall10.277s.
- Authenticated original-versus-repaired comparison: all12 original shapes
  accepted; the three valid zero-error positives still accept and the nine
  nonzero-error contradictions reject under the repair. A valid default
  negative outcome remains accepted by both implementations.

These are fresh Python checks, not compiler/vstd or native proof reruns. The
full compiler and matched-vstd results remain those of the earlier source-built
qualification; this repair changes only Python tooling and its tests.

This is the first of at most two authorized small review-repair rounds. A NEW
independent recheck remains required. Overall campaign status stays PARTIAL:
there is no completed operational reference, conditional Infer feature or
independently accepted native-counterexample result. The attempted native
review was unavailable due to a provider safety filter; it is not a pass and
will not be retried around that filter. The original goal remains unfinished.
