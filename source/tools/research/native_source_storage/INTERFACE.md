# Operational storage prefix — MODEL interface, no source/native promotion

Owned scope: only NEW `source/tools/research/native_source_storage/`.
Prepared base: `3f865efc`; constructor interface/graph/composition pinned at
`3d668d29`. No subdelegation, push, PR, global edits or shared builds.

The bounded implementation will execute the actual root bb0 statements 0,1,2:
StorageLive(_2), StorageLive(_3), StorageLive(_4). Control is represented
independently of locals. The next root reborrow at bb0[3], activation/unsize,
retag, raw formation and all other unimplemented instructions produce explicit
unsupported/fault outcomes. No source_entry_certificate will be manufactured.

Configuration interface: original incoming `_1` reference; MIR local map with
Dead/Live(Uninit or initialized source pointer/reference/scalar/aggregate value);
private stack allocation identities, bytes (including uninitialized bytes),
allocation/deallocation event history; independent arbitrary caller physical map
and arbitrary incoming reference/ancestor/protector history. Private blocks use a
separate address namespace and the combined physical map includes their changes.
Caller observations embed the existing allocation/signed-offset `address`;
source references retain the existing allocation/offset/tag structure. No
assumption on zero bytes, allocation-base offsets, empty history or client Iris
ownership is permitted. No StorageDead rule drops a referent or resolves loans.

Planned proved MODEL results: executable three-step root prefix with exact local
states/control, original operand/history preservation, all caller-byte lookup
preservation, and supported StorageDead effects at their exact extracted sites.
Repeated live/dead behavior must follow the pinned rustc implementation; memory
backed locals and their private deallocation must be represented. Unsupported
machine effects will be classified, not silently erased.

Interpretation boundary: pinned Rust 1.99.1 commit
58cf8f9c35b355199727627d07ae34a4e2846d67 MIR syntax/interpreter sources guide the
model, with exact private downloads and hashes. Authored Coq code is a partial
operational MODEL, not a proved interpretation of Rust. Remaining relation:
relate real interpreter local immediates/indirect stack allocations, allocation
identities, machine hook/history effects and entry/control to this model; prove
step simulation and caller observation for that relation over every lawful
entry, including arbitrary ancestor/protector histories. Source identity/shape
and closed assumptions do not prove that relation. No abstract theorem argument
will be counted as its discharge.

Existing NativeConstructorErasure.Interface physical/reference types are reused.
A concrete executable adapter will expose configuration/history/heap/operand/
control/step data for composition; any entry/reference-validity fields are
explicit MODEL structural predicates, never asserted lawful Rust entry validity.
Root C/F integration can use the original reference and caller physical map;
Verus F, pending-loan resolution and native adequacy remain unproved.

Canonical6/0 and full HMAC1/1 inherited, not rerun. Native0, 44+3 minimal/phase
and101 writer effects open, publication=false, goal incomplete. Normal coqc,
complete declaration/generated-name Print Assumptions inventory and recursive
coqchk, exact immutable foundation/tool/GMP pins, pretrial freezes, private raw
logs and final input/product guards are mandatory for claimed formal results.
