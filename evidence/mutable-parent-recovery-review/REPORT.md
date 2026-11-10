# Exact MODEL changing-child/original-parent recovery review

Review target: committed `9b759adb4516e88ecc51ecf407d32b95d1a25dda`,
implementation `b9b0209e`/`33edad89`, accepted record `3ea096c1`.
Scope is exactly the five constants in trial008, reproduced in `STATEMENTS.md`.
The six-declaration physical-callback specialization is excluded and was not
read in a parent checkout or live trial directory.

## Decision

Accept the exact five-declaration **conditional MODEL checkpoint**. No
mathematical, resource-conservation or evidence-identity finding requires a
root fix in this scope. This does not accept a completed writer, a public
adequacy theorem for a changing callback, or native correspondence.

Fresh normal compilation, all five assumption inventories and full recursive
`coqchk` succeeded in a new private review output. All eight resulting `.v` and
`.vo` hashes (including the generated inventory) equal accepted trial008.
The two shared module objects also equal the appropriate earlier protocol
and child checkpoints; their shorter generated assumption inventories naturally
have different hashes. Exact receipts, sources, objects, logs, tool identities
and command durations are recorded in `audit-results.json`.

## Resource and identity audit

`shorter_run_resources` quantifies over general semantic type, E/L/I, thread,
invariant lifetimes, parent value, closed expression, desired current value,
and arbitrary Iris Rin/Rout/Frame. Both parent lifetime-aliveness obligations
remain explicit. Its callback must work for **every** child lifetime and value
with the checked parent's location/current, not for a hand-picked replacement.
It receives the actual child ownership obtained from `tctx_reborrow_uniq` along
with L/I and Rin. The blocked parent remains outside. Frame is absent from the
callback premise and is held once through the child execution and end rule.
Rin is transferred once; Rout is received once. Separating conjunction forbids
using the same exclusive interval as both child ownership and an extra input.
Persistent key resources may be used in Rin without forcing exclusive key
vector ownership. The law supplies no key-access or transformation proof.

The proof creates the shorter MODEL lifetime, invokes the actual checked
`ReBor` rule and retains its location/current/blocked-future observation.
`proph_obs_sat` extracts structural location/current equalities for the actual
child. It does not replace the universal callback or assert a future value.
After the callback, `Obs` and `Updated` are combined at the **same** prophecy
assignment. Child resolution fixes that child's future to its new current;
`source_loan_release`, `resolve_unblock_tctx` and `unblock_tctx` restore the
blocked parent. The resulting parent observation is exactly
`(vπ x_final π, uniq_bor_future m π)`. The generic law deliberately accepts any
returned child satisfying its ownership and future/current conditions; it does
not itself state syntactic equality of the callback's old/new loan indices.
The concrete `byte_child_transform` establishes the stronger same-index route.
Generic L and I are both restored and returned, alongside Frame and Rout.

`bytes_vector_mapsto` is a bidirectional conversion of **existing full physical
ownership**, not a pointer-to-permission conversion. It uses the byte type's
physical representation, the fixed vector length and empty cloc paths. No
allocation, new heap ownership or arbitrary value equality is introduced.
The zero-length case is covered; it gives empty interval ownership.

`byte_child_transform` fixes one `n` for x0/x1 and accepts arbitrary p, E/L,
child lifetime/value, thread, expression and Rin/Rout. It requires aliveness,
exact child location/current, an initialized x1 range proof, and a conditional
WP transformation of the actual old interval to the new interval. Old range
comes from the opened child's ghost state. The transformation premise has no
extra ownership output assumed outside its own WP proof. Callback writes and
any additional credits must be justified by that proof and supplied Rin.

The proof destructs the actual child and calls `uniq_body_acc`, which preserves
its loan index. L is split into H1/H2 with guarded restoration. The accessor
provides a delayed payload, a linear `Ticket` and `Close`. **One explicit £1**
(`CreditOpen`) eliminates the payload later. Persistent `Time` and new byte
range establish the delayed payload needed to close. The callback receives
only the genuine physical vector and Rin; Close/Ticket/H1/H2 remain outside.
The post-callback interval is consumed in `Close x1 di gi`; Ticket is returned
once; H1/H2 restore L. The reconstructed child keeps precisely
`(p,repeat [] n), ξi, di, gi, idx`, changing only x0 to x1.
`uniq_prvar_app_irrel x1 x0 ξi π` proves equality of the future despite the
inhabited-type encoding's change of current. This is not a fresh loan with a
convenient new future. No standalone physical interval is retained while the
child also owns it. I is not an input of this helper; its caller frames I.

`initialized_nested_run` consumes the existing `model_reference`, retaining
its original ξi, ζi, idx, observation and `Back`. The initial Skip supplies the
time receipt used to package the same original model parent as a typed context
element; it does not allocate a substitute parent. The resource protocol returns
that parent's current/future observation. The final Skip invokes
`type_resolve_instr` on the **returned parent** using `source_loan_release`.
Combining this resolution with the recovered observation yields the future of
the **original** parent equal to `vπ x1 π`. The proof re-existentializes exactly
the ξi/ζi/idx originally unpacked and returns the untouched original `Back`.
The public postcondition hides these names existentially; the stronger retained
identity is visible in the proof. `Back` itself is not rewritten to claim x1:
it still returns x' linked to the original ξ.

`initialized_nested_payload_run` calls this changing wrapper, consumes the
single local root lifetime token from `llctx_interp [κ ⊑ₗ []]`, ends κ, and
passes its actual death token to the retained `Back`. Its recovered payload is
initially x', not pre-assumed x1. `bytes_vars_nil` discharges the syntactic
independence premise for resolving the x' equality; prophecy saturation of
**both** the original-parent resolution and equality gives one common π.
`bytes_observed_injective` then proves x'=x1 for this finite, prophecy-free
byte-vector sort. This is not injectivity for arbitrary prophecy-dependent
sorts. Finally `bytes_vector_mapsto` returns the actual physical vector of x1
at p. Its length is the original `length zs`, so no interval resizing occurs.
It never rewrites that vector to zs or invokes the imported no-write wrapper.

The initialized run returns L while consuming the typed parent through
resolution. Its specialized empty invariant context is not promised in its
postcondition (the proof discards it after resolution). The payload wrapper
ends the parent lifetime and therefore promises neither L nor a live parent;
it returns Frame, Rout and the new interval. This is intentional disposal in
the specialized affine logic, not an unproved restoration of a native context.

## Later, masks and credit accounting

All five statements use ordinary default-mask WP. `uniq_body_acc` returns an
`↑Nllft` closing update, widened to the enclosing mask by checked
`fupd_mask_mono`; both closure and guarded-L restoration discharge the required
namespace inclusions. The child callback runs after the opening update has
completed, with linear closure capability retained by its caller. `iNext`
provides the delayed new payload before closing, rather than eliminating that
later without a credit. Its retained time receipt is persistent; the linear
accessor Ticket is never confused with the spent later credit.

The imported `end_child_context` resolves, ends and unblocks using the exact
MODEL Endlft Skip sequence and two receipt-indexed step-fupd eliminations;
H1/H2 are restored before lifetime ending and again after unblocking. The
original-parent resolution uses the baseline receipt/credit-aware
`type_resolve_instr`. Final parent death uses `wp_mask_mono`/`wp_step_fupd`,
then the `↑Nllft` Back update and actual subsequent Skip steps to obtain its
later payload. Namespace obligations are discharged by normal checked proofs.
No unsafe reduction, admitted timing step or externally asserted mask
transition is used. There is no new global credit source: the separate £1 is
explicit, callback costs remain its obligation, and instruction time receipts
come from the existing step rules and time context.

## Assumptions and helper inventory

| New constant | Inherited logical assumptions |
| --- | --- |
| `resource_protocol.shorter_run_resources` | FE, Eq_rect_eq |
| `child.bytes_vector_mapsto` | FE |
| `child.byte_child_transform` | FE |
| `initialized.initialized_nested_run` | FE, Eq_rect_eq |
| `initialized.initialized_nested_payload_run` | FE, Eq_rect_eq |

FE means `Coq.Logic.FunctionalExtensionality.functional_extensionality_dep`;
Eq_rect_eq means `Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq`.
There are exactly five new lemmas, no new definitions, constructors, instances,
axioms, parameters, Program obligations or other helper declarations. Local
`have`/`iAssert`/`set` facts are inside the checked proof terms, not additional
assumed constants. The generated `Assumptions.v` contains ten trivial marker
goals around the five Print Assumptions commands; these are inventory plumbing,
not additional application laws. Recursive kernel context lists only FE and
Eq_rect_eq, predicative Set, no rewrite rules, type-in-type, unsafe fixpoints or
assumed positivity.

New proof obligations discharged inside these five lemmas include: transferring
Rin/Rout while preserving Frame and restored L/I; full vector ownership
conversion; opening/closing the same child with changed initialized bytes and
one later credit; original-parent resolution from changed child current; and
post-death recovery of exactly x1 through a common prophecy assignment.
Inherited helpers include `child_inclusion`, `child_alive`, `end_child_context`,
`source_loan_release`, `bytes_parent_alive`, `fresh_local_alive`,
`bytes_vars_nil`, `bytes_observed_injective`, byte physical/initialized/length
laws, baseline `uniq_body_acc`/`uniq_prvar_app_irrel`, guarded-L operations and
time/prophecy rules. The existing six `byte_ty` Program obligations belong to
the pinned semantic dependency, not this checkpoint. Recursive checking includes
the dependencies actually imported by these laws; none is excluded. Imported
no-write/read laws are not used to prove changed payload recovery.

## Authentication and reproduction

`audit-results.json` records an independent read-only hash audit of both
mandatory foundation gates and the compiled dependency objects, not just their
success labels. It authenticates official baseline 115a618d, 523 copied source
files, the entire 284-object map, _CoqProject, runner/coqc/coqchk/opam and the
required GMP manifest. All five dependency receipts are authenticated against
committed pins; every retained dependency .v/.vo is checked, and every
nongenerated .v matches its counterpart in this checkout. All 887 inputs of
accepted recovery trial008 are checked, with archived `/source/` names rebased
to this checkout before reading. Earlier protocol/child record receipts and all
their compiled artifacts are authenticated separately.

The frozen committed controller is used unchanged. It authenticates frozen
bytes before proof execution, retains source and product identity guards,
requires complete marker-delimited assumptions, parses the recursive kernel
context fail-closed, and marks checked only after final input/product guards.
Its command line uses normal `coqc -q` and `coqchk -silent -o` on all three
modules plus inventory, without `-admit`, `-norec` or quick/unsafe modes.
The review reruns and reparses the real logs. Its scope check rejects tracked
changes outside this evidence directory. No source/parser mutation experiments
or filesystem-race tests were performed.

See `REPRO.md` for verification commands. Raw logs, the unchanged frozen copy,
source copies and VO files remain in the private output; only compact evidence
is committed. Failed development trials are not semantic evidence and were not
rerun. No Rust suite or actual HMAC execution was performed.

## Exact remaining boundary

These lemmas are internal MODEL WPs with explicit initialized reference and
resource/callback premises. The existing `raw_model_context_adequacy` constructs
a fresh model reference and initial credits from a finite initialized model
heap; it does not reconstruct an incoming Rust reference or ancestor loans.
There is no theorem in these five declarations composing that initializer,
the actual writer's changing callback, two-region pad/shared-key setup and
final-memory observation into public writer adequacy. Such a composition must
derive all permissions and credits internally; it may not append model loans,
exclusive key permissions or a transformation WP as Rust application premises.
Rin/Rout permit resources to flow but do not solve those obligations. If the
physical transformation only proves a conditional WP, this checkpoint remains
conditional on that proof; no supplied permission has been promoted to a native
or public result here.

Recovered physical payload is stronger than pointer metadata, but is still a
MODEL interval, not proof of execution of the original native Zip body. Actual
Rust entry/context/provenance, live ancestor preservation, operations, retags,
source phases and C/F correspondence remain open. Native correspondence is
**zero**; minimal native operations **44** and phases **3** remain open against
**101** effects; original full HMAC remains **1/1 open**. Publication is false;
the overall goal is incomplete. No production theorem is installed or original
executable body/domain changed by this review.
