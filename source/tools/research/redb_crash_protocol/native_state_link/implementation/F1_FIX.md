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
