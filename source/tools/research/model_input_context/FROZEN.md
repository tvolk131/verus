# Pretrial construction contract

This lane owns only model_input_context. Source/tool identities are snapshotted
by each pretrial freeze; trials retain their exact copied inputs, including failures.
Foundation and both prerequisite object libraries are read-only. No delegates.

Proposed public theorem: for arbitrary finite h, p and list zs, with Forall
(0<=z<256) zs and forall i z, zs!!i=Some z -> h!!(p+i)=Some #z,
construct adequate NotStuck (Skip;; e) (initialized_state h) phi from a WP
callback universally quantified over the internally fresh typeG, tid, fresh
nonempty lifetime kappa, prophecy indices and loan index. Callback receives
ordinary llft/proph/uniq/time/invariant contexts, empty external and singleton
local lifetime context, ty_own (&uniq{kappa} bytes_ty (length zs)) at EXACT p
with current bytes zs, the genuine alloc_uniq_body future, its ordinary lifetime
end restoration continuation, and exact h-minus-interval frame. No external
owner, PointsTo, freeability, lifetime guard, or prophecy equation premise.

Pure inverse bytes_of_list : forall zs, ~~bytes_sort(length zs), exact byte_list
roundtrip and inverse, exact bytes_physical and bytes_initialized. Zero length
permitted; arbitrary locations including strict interiors; 10 and 128 included.

Administrative setup: initial credits £(1+k), one consumed by llft_alloc, k
retained for callback. Explicit Skip prefix supplies cumulative ticket 2 via
wp_cumulative_time_receipt2. One ticket converted with receipt 0 into persistent
receipt 1; one ticket consumed by alloc_uniq_body at payload d=0,g=0. Result
reference d=1,g=0. Setup WP callback mask is top minus advN; no hidden model
rewrite, no claim this expression is actual Rust. May strengthen credit budget
if kernel development demonstrates an additional real use, documented before
new trial. AllRSt0/global-lock=false/empty-freeability convention explicit.

Operational composition: nonempty zs, scoped source_context_access, actual read
at p, restore SAME loan, existing source_loan_release resolution consumes it.
Use two callback credits (later payload and one resolution later). Preserve frame
through callback; no duplication. Resource controls for wrong pointer/current
bytes and full fragment duplication; prophecy future remains symbolic until
resolution, with no universal arbitrary assignment equation asserted.

All declarations including helpers/obligations get Print Assumptions and recursive
coqchk. Only inherited FE/Eq_rect_eq allowed; pure vector laws should be closed.
New controller authenticates the retained GMP manifest to existing baseline pin.
Original Rust source/context/provenance/ancestor loan and original future/C/F,
44 minimal simulations, 3 phases and 101 effects remain unproved. canonical 6/0,
HMAC 1/1, native 0, publication false. Parent integration and new review required.

Pretrial clarification after trial017: the polymorphic continuation expression e
must satisfy Closed [] e, because Seq is encoded by a lambda and the existing
pure_rec rule checks closure. This is an expression well-formedness premise,
not a restriction on h,p,zs, allocation offsets or initialization histories.
Skip;;e takes two pure beta steps (Skip itself and its sequencing binder).

Pretrial refinement after trial018: bind the Skip subexpression when producing
tickets so the enclosing continuation resumes at top mask. Construct the typed
reference after the two prefix steps, preserving its exact syntactic resource.
Callback mask is now top, enabling the existing top-mask scoped accessor.
No additional steps or credits; byte/raw-map domain unchanged. Trial018 failed
compilation during final callback application; it was not kernel-checked.
