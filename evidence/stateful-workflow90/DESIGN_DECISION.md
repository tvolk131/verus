# Cycle-one design decision

The parent accepts the technical design recommendation in
`/private/tmp/workflow90-design.upzH6C/REPORT.md`, SHA256
`849ca4c45e3317b1b4ee900b5c1470f089405a927ceb2bd0b58bb89aeb9576f4`,
as sufficiently concrete for ONE bounded implementation attempt. This is not
feature or theorem acceptance. The two risks remain snapshot-equality backend
performance and acceptance of the actual lowered/post-poly shapes.

Preserve ordered let wrappers, guards and triggers; capture free root values,
not arbitrary subexpressions hoisted from guards. Validate exact current types,
definite initialization, supported logical values, fresh identities and location
destinations. Reuse typed init_var. Check reconstructed frozen invariants and each
guarded point instance before using the exact same expression. Preserve original
loop metadata, executable safety, authored goals and exact AssertId pairing.

For cond=None, capture before the ENTIRE lowered body, including any condition or
iterator setup there. For cond=Some, support only empty condition setup and pure
supported expressions; decline nonempty setup explicitly. Do not put captures in
pre_stms or reinterpret body-entry values after an effectful condition as header
values. Mask inherited premises across every nested loop and unsupported proof/
effect scope. All traversal/output/capture budgets are shared and transactional.

Target B gets a separate ordinary authored-loop-goal assistance predicate, not
relaxed discovery eligibility, an inferred constructor summary or certification.
Feature OFF remains unchanged. Expose bounded preparation/decline observations
through the existing report only; preparation is not proof success.

Stage the same matched private bundle in BOTH actual test-harness runtime paths
and preflight hashes before suite launches. No test assertion/negative oracle may
be weakened or skipped to hide setup/compiler errors. No new trusted model,
library correspondence, axiom or native-body/domain change is authorized.

Core expected changes are entry_goals, narrow eligibility/goals/report plumbing,
focused tests, a reusable runner and documentation. If new SST variants, loop
encoding, mutable-reference semantics or library specifications are required,
stop for reassessment rather than silently widening this attempt. The old loop89
patch remains quarantined and must not be reapplied wholesale.

Cycle-one limits: three compiler build attempts, one successful matched-vstd
verification, 96 direct application attempts, separately itemized regression
suites. Preserve failed launches/results. At most one further implementation
cycle remains after this one; it requires a recorded reassessment, not automatic
continuation by the implementer. Parent owns the new independent review.
