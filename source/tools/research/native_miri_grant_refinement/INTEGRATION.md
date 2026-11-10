# Independent component acceptance

Independent review `a4d43ec9` accepts the exact `9593b4c7` component and historical
comparison at the scope described in REPORT.md. It reproduces22/0 with goals
enabled/disabled, the six isolated semantic controls, all seven individually
closed Coq declarations and full recursive checking. No new blocking findings.
The authored/manual proof inventory remains explicit; assistance is redundant.

The parent authenticated all42 reviewed source files, the review manifest and
both private product maps. The report's original review-pending wording is
historical. Mathematical source, selected executable bodies and original receipts
are unchanged. The authoritative record is
[parent-miri-grant-component.json](../mutable_buffer_goal/parent-miri-grant-component.json).

The required lookup precondition has not been derived from the real caller:
`find_granting(Write, ...) = Ok(Some(g))` must establish bounds and Unique/SRW
in the resulting actual stack, including wildcard/unknown/cache behavior.
Packed-tag validity and cache consistency/source initialization are separate
open obligations. No source/Rust memory-layout, actual input permission, original
C/F, native iterator or full HMAC theorem follows from this component acceptance.
Native=0,44+3/101 open, HMAC1/1 unproved, publication=false, goal_complete=false.
