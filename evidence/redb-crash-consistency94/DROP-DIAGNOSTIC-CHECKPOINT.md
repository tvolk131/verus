# Destruction diagnostics: useful tooling, not a new proof capability

The compiler prototype records where Rust may destroy values. It observes an
early compiler representation, before Rust resolves conditional destruction and
generated field cleanup. That is useful for implementation, not enough to infer
that a destructor ran or that its effect reached a particular borrowed page.

Independent review:
`/private/tmp/campaign94-drop-edge-review-20261105.jPkC1OB1/REVIEW.md`.
Its90 file/link entries authenticate under its declared manifest schema.
The review found two diagnostic-output collisions and one omitted scope field;
it did not find a new proof-soundness defect or approve destructor semantics.

Parent repaired those issues in isolated staging. Twenty applications preserve
their expected proof outcomes. Log/output aliases and the actual PID-derived
proof-export temporary now disable the diagnostic with a warning. A normal report
is written after ordinary outputs finish; create-new preserves preexisting files
and symlinks. Root-only scope is explicit. The fresh build succeeds and matched
vstd verifies2225/0. Focused controls preserve false-proof and Drop-precondition
rejections. Selective verification is not mislabeled whole-crate success.

The sealed source/runtime/results package is:
`/private/tmp/verus-crash-consistency94-20261105/drop-diagnostic-repair-001/handoff`.
Its seal SHA256 is
`945f8f0e923fea0a7d391048a8ea0c9bb9438d2675a783ee3957b64c798d3c79`.
All1244 copied payloads rehash and are read-only. The repair still needs independent
closure alongside the next semantic review. No production compiler was adopted.

The next meaningful milestone is smaller than redb but real: prove that the
existing WriteBack destructor changes the caller's original borrowed value,
while rejecting an effect on an unrelated value. This needs authoritative
destruction events, real binder/loan identity, checked destructor calls and
correct borrow-retirement order. Current universal Drop restrictions stay intact.

This does not yet handle redb's incomplete builders during panic, nested page
cleanup, or generic/imported destruction. Those remain required for the native
producer route. All eighteen original database property/history cells are OPEN.

Parent continuation reaches the small borrowed-value milestone in private staging:
the unchanged immutable WriteBack checks 2/0 ON and 1/1 OFF. Wrong-borrow and
wrong-value controls reject at the backend. This requires a new post-destruction
field-release compiler rule that remains unaccepted pending semantic review; it
is not a theorem established just by those tests. The isolated checkpoint is
`/private/tmp/verus-crash-consistency94-20261105/drop-effects-parent-002/handoff/REPORT.md`.
The diagnostic output repair's independent closure is still open, and there is no
live compiler adoption or native producer credit.

The subsequent independent review supports the narrow field-release rule's
ownership argument, but withholds adoption pending real compiler-mapping
mutation tests. Its twenty fresh applications include the unchanged ON/OFF
example and both specific output-collision fixes; they do not close arbitrary
diagnostic behavior. The new straight-move control still rejects an unsupported
bool first and receives zero isolated ownership-transfer credit. The separate
parent erratum corrects the sealed checkpoint without rewriting history.
