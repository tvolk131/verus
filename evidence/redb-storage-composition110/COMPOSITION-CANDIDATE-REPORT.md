# Lane A: checked stateful storage/acknowledgement product110

## Result and scope

Implemented a genuinely connected checked model-to-model product, not another
interpreter that merely agrees on event labels. `composition110.rs` imports
source-module copies of the exact reviewed A and B libraries. The copies are
byte-identical to the sealed originals; no visibility workaround or original
trust expansion was needed. `checked_run` proves that the product control state
is the **actual B.run** on signals generated from the preceding product state.
The only modeled physical flush calls are at PC24 and PC28.

The full final crate verifies **61 obligations, zero errors**, with an explicit
fresh export inspected at its exact requested path. The semantic mutation that
bypasses the failed second flush is rejected by the intended backend
postconditions. Its corrected counterpart and a separate byte-identical full
restoration both pass with fresh explicit exports. Full raw outputs, sources,
argv, complete child environments and actual process statuses are retained.

This is not native adapter R, native executable-body verification, integration,
producer/trust adoption or final-cell acceptance. All eighteen cells remain OPEN.
Compiler/source correspondence is UNESTABLISHED; original native path FAILED;
Git continuity UNAVAILABLE. Existing S0/Infer/full-scope/import/export refusals
are unchanged. No new assume/admit/axiom/external_body or primitive law was added.

## Concrete product and checked connection

`Product` contains the actual B.State, actual A.Image, arbitrary finite pending
stripes, independent full first/second symbolic 320-byte headers, fixed image
geometry, complete FlushRecord history, barrier indices, and explicit OPEN/frame
gate state. The byte images include A's visible/durable/permitted components.
Header contents and prior pending stripe contents are not concretized or bounded
by a finite enumeration. The theorem quantifies over arbitrary finite Action
sequences, arbitrary write decisions, partial write/persist sets, sync outcomes
and nonflush effect functions. No six-run substitute or desired trace is used.

For each active PC24/28, `generated_signal` reads the `.ok` of the complete
unchanged A.model_flush result. Neither an action's supplied signal nor a chosen
success trace can bypass drain or sync. `step` installs the exact result image
and pending stripes and appends a record including before-image, pending input,
decision function, sync outcome, persisted set and full exact A.Run. Thus failed
writes and failed syncs keep A's actual successful-prefix effects and drained or
retained pending state; no rollback equivalence is inserted. The explicit
`error_keeps_actual_complete_flush` theorem also preserves the prior history as
the prefix of `history.push(record)`.

`flush_checked` **calls A.flush_refinement** and A.check_stripes. It establishes
valid output, unchanged geometry, valid pending stripes, successful drain and
sync evidence, precise durable-byte equation, and no sync after drain failure.
Each stored record is connected to the complete A.model_flush, not a duplicated
flush algorithm. No independently authored storage interpreter is introduced.

PC23/27 successful admissions are deliberately authored model operations: append
a stripe with the complete corresponding header at offset zero. Arbitrary prior
pending stripes remain present. This last stripe is a checked model scheduling
choice, not a claim about native cache/lifecycle admission. `complete_header`
uses A.stripes_reflect_last_cover to prove every full-header byte after the
successful barrier. Native header serialization, source data/root meaning,
coalescing/admission/resource and lifecycle bindings remain OPEN. No excluded
WritablePage/Drop/lifecycle pipeline was investigated or implemented.

`step_invariant` preserves B's actual CFG invariant plus product validity and
history/certificate links. `checked_run` is induction over arbitrary finite
schedules and proves exact equality with B.run on `signals`, also preserving
both header buffers and geometry. `public_ok_composes` **calls
B.public_ok_has_two_successful_barriers** on those generated signals, not on an
assumed trace. Public Ok entails both actual complete modeled flushes succeeded,
their ordered history indices, all six ordered B milestones, complete byte
evidence at both barrier times, and drained final pending stripes.

## Callback and epilogue boundary

Every nonflush Action supplies an arbitrary function proposing its image/pending
effects. The explicit `framed` gate compares the proposed physical model state
to the current image and pending stripes. Mismatch stops with B status9, sets
`blocked`, stores the rejected Effect, and preserves prior barrier history.
The retained image on this OPEN branch is a **pre-gate snapshot**, not a claim
that actual post-callback bytes stayed unchanged. `callback_mismatch_stops_open`
checks this behavior. Terminal outcomes absorb later actions.

The final current-image durability conclusion is therefore conditional on the
explicit gate accepting every intervening effect. The checker proves the gate's
behavior and the conditional model theorem; it does **not** prove native
callbacks satisfy that frame, are inert, terminate, avoid resource failure or
preserve durable history. Complete barrier snapshots remain explicit. The
whole-image/pending frame is stronger than a minimum durable-root history frame;
that conservative cut is intentional and is not concealed as native proof.

Parent's additional source observation is respected without new native-source
inspection: page_manager.rs:1897 non_durable_commit can install a newer
**in-memory** secondary/root and call cached_file.rs:612 write_barrier. That is
not another physical header admission or flush/sync. There is no product flush
outside PC24/28, and no equation identifying that newer in-memory root with the
final durable physical header. The live native in-memory header/root is not a
modeled Product field. A frame on represented physical image/pending state is
not a transitive proof about that epilogue or other system-tree/cache callbacks.

## Nonvacuity and preserved error history

`success_signals` proves successful complete flush decisions generate the actual
all-zero B control projection for arbitrary finite lengths, rather than taking
the required trace as a premise. `nonvacuous_symbolic_success` then reaches public
Ok using B's checked witness configuration for **arbitrary** valid initial images,
valid pending stripes and independent 320-byte headers. The old secondary is 1,
the new slot is 3; no stale-secondary equality or equality of the two byte buffers
is required. This witness is separate from the arbitrary-schedule safety proof.

`nonvacuous_post_publication_error` reaches actual B PC35 after 34 successful
steps, then takes a fallible Err. Publication and both checked barrier histories
remain; acknowledgement remains absent, actual modeled image is unchanged by
that framed control error, and the final barrier certificate survives. This does
not identify every native Result Err with rollback or make unframed callbacks
preserve bytes. Interrupt/resource/crash/native-abnormal refinements remain OPEN.

## Full-crate checks and semantic controls

The exact application index, sources, results and scope are machine-readable in
`ACCOUNTING.json`. Applications, including failed frontend corrections, are:

1. candidate-01: parse failure (reserved `invariant` identifier in authored
   contract position), zero obligations; no export. Retained, no success claim.
2. candidate-02: type failure for authored nat constant, zero obligations; no
   export. Retained, no success claim.
3. candidate-03: 57 verified, four backend postcondition failures; no export.
   Missing authored index-order/header-conservation facts and a missing local-mode
   precondition were corrected without weakening the final safety claim.
4. candidate-04: 61 verified, zero errors, fresh exact export. First complete
   passing product; final source later adds an explicit error-history equality.
5. full-positive: final source, 61 verified, zero errors, fresh exact export.
6. negative-bypass-second: **one-line semantic change** lets PC28 emit success
   despite failed complete A.flush. Actual status1, 59 verified, two backend
   postcondition failures: `step_invariant` and the required Err outcome in
   `error_keeps_actual_complete_flush`. No parse, unsupported-feature, crash or
   timeout substitute. Exact requested export is absent.
7. corrected-second-flush: removes that single bypass, 61 verified, zero errors,
   fresh exact export; source byte-identical to application5.
8. restored-full-positive: a separately retained byte-identical final source,
   61 verified, zero errors, separate fresh exact export.

Final/corrected/restored source SHA256:
`00289ba9a9189b9b24491ef68ae84fa74e75a04d1c19277db4e6bafa26456f5e`.
Export hashes may differ across fresh checks due to distinct source paths; the
required restoration equality is source-byte equality, not export-byte identity.
Each attempt records fresh_export_absent before invocation and inspects the
**exact requested export file**, including size/hash, after actual process exit.

## Original versus authored proof scope

Original A: 26 proof bodies, reviewed 31 obligations; exact source SHA256
`29485224c99ba2f994012ff53b2ddc98c9cab79daaf7b3fc2a855b952d2fdc4f`.
Original B: 15 proof bodies, reviewed 16 obligations; exact source SHA256
`447a7fcfa23fcdec6114be0ec4eb27274ed4133efbc7d81fdac896dffd9306a2`.
New product: **11 authored proof bodies**, plus its selected definitions/constant
and termination obligations. Final full-crate total: **52 checked proof bodies,
61 obligations**, zero checked executable bodies. Thus 41 original-library
proof bodies are reused/rechecked, not claimed as new; the additional obligation
count relative to original reviewed totals is 14, not 14 new proof bodies.
`ACCOUNTING.json` lists exact checked-body names by original A/original B/new
product per attempt. Repeated positives are repeated checks, not new scope.
No original native executable body is checked or executed.

## Authentication, resources and publication

First clock 04:38:49 UTC; own cutoff 05:03:49 UTC, earlier than 05:05 absolute
handoff. `resources.jsonl` retains sampled scratch/free-disk observations, and
the controller refuses launches near the deadline or at resource limits.
Eight charged applications out of sixteen; one verifier process at a time in
this lane, one verifier worker per invocation, ninety-second hard subprocess
timeout. Every actual attempt finishes in far less than ninety seconds. No
compiler/runtime/vstd/Cargo/Rust/application build, native application run,
network, Git write, parent edit, toolchain/cache copy or subdelegation occurs.

All 838 frozen selected records match at entry, every preexecution and exit;
the three scope/authorization hashes also match. Current GOAL/STATUS inputs are
not edited. The five historical drifts and two explicit documentation
reconciliations remain disclosed by INPUTS, not silently repaired. Round110
README is ongoing parent context, not declared immutable by this lane.

`authentication-*.json`, `BOOTSTRAP.md` and `PROVENANCE.json` record direct tools,
actually loaded Python/controller modules, discovered editing/hash shims and
sampled matching identities. Direct installed framework Python is used, not
/usr/bin/python3 or rustup. Bootstrap/loader discovery limits are explicit; no
atomic, exhaustive or first-use provenance is claimed. Hashes do not establish
source/compiler correspondence. All raw failed corrections are retained.

`publication/laneA` contains the final source and exact original library copies.
`composition110-additive.patch` adds only these three files at the proposed
laneA destination; it is supplied but **not applied to the parent**. The complete
scratch manifest/seal covers raw attempts, authentication, source, patch, scope,
cut ledger and publication files. No integration or review/trust adoption is
authorized here. One fresh independent combined review remains the parent's
responsibility. See `HANDOFF.json` for exact paths, final accounting and seal.
