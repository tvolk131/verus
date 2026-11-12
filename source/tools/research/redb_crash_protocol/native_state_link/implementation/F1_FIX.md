# F1 skipped-body correction

Independent review61c39752 found that `checked_item` admitted the actual
`internal_const_body` and `internal_reveal_fn` attributes, although ordinary
`rust_to_vir::check_item` omits those bodies. This is a P1 admission defect;
the later inactive refusal prevented any native proof or authority.

The root correction rejects both flags on each item and every local ancestor,
using the existing attribute parser and ancestor traversal. Default-off lowering
is unchanged. The focused regression adds16 precise early-refusal invocations
(two markers, native body/false contract/module ancestor/nested function, and
ordinary/Infer) plus two unmarked false-contract admission controls. Existing
116 admission and32 inherited invocations remain required.

Before validation, the required gates are: a freshly built corrected compiler;
the unchanged frozen seven supporting runtime artifacts; the full focused suite;
direct reproduction of the reviewed witnesses; source/tool guards; and a NEW
independent recheck. F1 acceptance remains pending. No descriptor, native-body
authority, final-state proof or erasure capability is enabled. Full REDB source
coupling remains false; HMAC/native remains DEFERRED AND UNACHIEVED.

Root validation uses a new private output/cache and preserves all previous
snapshots and logs. Actual results and source/runtime/receipt identities will be
recorded after execution; this prospective note claims no test pass.

## Parent validation

Correction9cc0f65c passes a fresh offline compiler build, then all10 focused tests
covering166 invocations. The original116 admission and32 inherited invocations
remain, alongside the18 added regressions. All source/tool guards pass. Direct
guarded before/after matrices each pass12 expected outcomes: both markers with
ordinary/Infer and feature enabled/disabled, plus unmarked false contracts.
The six default-off classifications are unchanged. Marked feature-on inputs
now fail with the skip-specific reason before the later inactive refusal;
unmarked false contracts still pass admission and reach that refusal.

The compiler build took160.616 seconds; the complete focused Cargo command
took99.378 seconds, including test-harness compilation. Its two test groups
report33.46 and6.54 seconds. These nested times are not added again. No vstd
rebuild was needed: the seven supporting artifacts are unchanged and retain
the prior NORMAL2257/0 result. Only the corrected compiler is new. Two setup
failures are retained separately, with no compiler invocation or proof credit:
copying over a read-only solver, then supplying a non-repository controller path
to a repository-relative extra-input helper. Both were corrected before the
successful new-output trial.

Exact sources, tools, commands, receipts, costs and exclusions are recorded in
F1_FIX_RESULTS.json. The immutable corrected runtime is
/tmp/owned-cell-F1-skip-fix-9cc0f65c-parent-snapshot; its receipt SHA256 is
9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f.
This is an INACTIVE snapshot. F1 independent recheck is pending; native body,
state and full REDB source proofs remain absent. These results do not accept the
complete owned-cell stage.
