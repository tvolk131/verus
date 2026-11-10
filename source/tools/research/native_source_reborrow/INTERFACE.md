# Typed scope, frozen before proof trials

Only root instance0 bb0[3], `_4 = &mut (*_1)`, after StorageLive2/3/4.
`Storage.state (Model.history H)` extends arbitrary incoming H with existing
regions, tag nodes, byte grant stacks, fresh counter and prior retag trace.
No empty history or new caller allocation is constructed. `_1` must evaluate
to `Storage.original_operand`; static borrowck loan0 is not its dynamic tag.
Output is `Immediate (Init (RefValue child))` at local4 and pc bb0[4].

`Model.child_reference r fresh` retains allocation and signed offset and changes
only provenance. Positive generic n,10,128 MODEL spans will require structural
current range, non-null physical address, ancestry, grant and protector checks.
These are representation conditions, never added application requires and never
coverage of all lawful Rust inputs. Native0/publicationfalse remain mandatory.

Rule table (primary source paths and SHA256 in pins.json):

| Source | Selected behavior / limitation |
| --- | --- |
| pinned rustc step.rs:94,214–239 | Assign evaluates Ref, force_allocation, to_ref, mode TwoPhase, machine retag/validation, write_immediate; statement counter advances only on success (55–61). |
| pinned rustc machine.rs:546–571 | Retag policy is machine-specific; default no-op supplies no source safety/frame theorem. |
| pinned rustc syntax.rs:306–338,1350–1358 | Assignment details undecided; reference aliasing semantics left to alias model. |
| inspected Stacked Borrows bor_semantics.v:300–338 | Per-byte grant/protector guidance only; old Untagged raw rules are not used. |
| inspected Simuliris bor_semantics.v:790–835,944–955 | Fresh child plus read effect, not merely metadata insertion; reserved/first-write activation differs from borrowck two-phase. Not imported. |
| Simuliris simulation/slsls.v and retag lifting | pool_safe premise cannot prove missing source safety. |

Chosen conservative tagged byte-stack MODEL: find an enabled parent in each
byte's stack; remove only unprotected younger entries; keep parent/ancestor
suffix unchanged; insert a fresh Reserved child. Require structural lineage,
range coverage and readable mutable permissions on the retained suffix. This
is an explicit semantic choice, not a claim to implement Tree Borrows or the
pinned compiler's unspecified machine. No write/activation is implemented.
Unsize/raw/getters/protector cleanup/normal/unwind exit are Unsupported.

Next source sequent remains: for an independently lawful actual Rust entry and
its actual incoming reference/history, construct a relation to these structural
states covering ALL lawful inputs; simulate all enabled Ref/retag/validation/
assignment outcomes with unchanged physical caller bytes and source reference
validity. No such theorem or C/F binding is supplied by MODEL proofs.
