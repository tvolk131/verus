# Round110: one fresh combined independent review

## Disposition

**The bounded authored-model results reproduce. No new model-level correctness
defect requiring candidate changes was identified. Native linkage/integration is
NOT ready, and this review grants no acceptance, adoption, integration or final-
cell credit. All eighteen original cells remain OPEN.**

This is the ONE combined review of both actual sealed candidates, performed by
one reviewer without subdelegation. It is not a replacement implementation or an
additional review chain. The six verifier-application allowance is exhausted;
there are no requested follow-up verifier runs in this handoff.

## Time, resource and scope boundaries

First observed UTC clock: **2026-11-08 04:57:56**. Initial df available space was
50,091,828 KiB, strictly above 16 GiB. Own hard stop is **05:12:56 UTC**, earlier
than the supplied absolute 05:14:00 sealed-handoff cap. Optional twelve-minute
stop is 05:09:56; the parent round deadline 05:33:26 and parent optional cutoff
05:21:26 do not extend this review. ACCOUNTING/HANDOFF contain final observations.

Only fresh `/private/tmp/redb-review110-LwF1feEi` was written. Authored files use
apply_patch. Candidate, parent, frozen source, reviewed libraries and Git were
not edited. There were no builds, network requests, native application runs,
new assumptions/axioms/assume/admit/external_body/primitive contracts, policy
weakening, producer/admission adoption, excluded task work or subdelegation.
An initial read-only git status reports preexisting changes; it performs no Git
write. Incidental adjacent source displayed while reading selected CheckedBackend
control is not investigation or implementation of an excluded lifecycle pipeline.

## Authentication and retained historical attempts

Frozen PLAN SHA256:
`5ac77193513096bade9dc5cae35c8a4a7237ddbee6b1f527ec8c99c92edac377`.
Frozen INPUTS SHA256:
`8efaf2981eeaee800cdab449fdfa394747675f171ceebbd90d96cafe439fa9c9`.
AUTHORIZATION and the subsequent CONTINUATION-AUTHORIZATION were read; the latter
does not reset any elapsed time, allowance or trust boundary. All **838** current
frozen input records match at entry, before and after every fresh verifier
application, before/after finite replay, and at final exit. Historical drifts are
not replaced by round109 hashes. Applicable scratch/native AGENTS were inspected.

Both candidate manifests authenticate **104 payloads each**, with their supplied
manifest and SEAL hashes. Both parent-retained candidate archives authenticate
against the supplied archive SHA256 and have **106 regular members each**, all
byte-matching the corresponding candidate files. The parent-control archive
authenticates its supplied hash and has 34 regular members. These are sampled
integrity observations, not producer trust or atomic source snapshots.

All eight original A and all ten original B invocation/result/stdout/stderr
streams were read and retained under `original-raw`; the combined parsed audit
and original accounting are retained separately. No original failure is erased:
A starts with parse/type failures, then a 57/4 proof failure; B begins with actual
native frontend blockers and then an initial concrete-witness failure. Successful
later corrections do not retrospectively certify these unsuccessful attempts.

Direct framework Python is used, not `/usr/bin/python3` or a rustup shim. Fresh
checks use the candidates' exact pinned verifier, rust_verify, z3, runtime vstd
and sysroot choices and flags, changing only fresh scratch destinations. Exact
argv, complete configured child environment, source/library hashes, raw streams,
result and full parsed scope are retained per check. Frozen and supplemental
direct/discovered transitive identities are sampled at entry/preexecution/exit.
Read/edit tools, source_guards and its transitive authenticate.py helper are
recorded too. Mach-O dependency observations are retained where accessible.
Bootstrap shell reads/imports precede complete helper enumeration; this is not
exhaustive first-use/dyld/shared-cache provenance. Those qualifications are not
silently upgraded to compiler/source correspondence.

## Six fresh verifier controls

Every application is conservatively charged before setup, runs alone with
`--num-threads 1`, has a 90-second maximum, requests an initially absent exact
fresh export, and records that path's actual presence and SHA256 after exit.
There were no setup/launch failures, timeouts or uncharged verifier retries.

| Charge | Source/control | Exit | Raw verified/errors | Exact export |
| --- | --- | --- | --- | --- |
| 1 | A final positive | 0 | 61/0, entire crate | present, hashed |
| 2 | A second-flush bypass | 1 | 59/2, entire crate | absent |
| 3 | A byte-identical restoration | 0 | 61/0, entire crate | present, hashed |
| 4 | B extended positive (07) | 0 | 15/0, entire crate | present, hashed |
| 5 | B extended best-effort latch mutant (08) | 1 | 12/3, entire crate | absent |
| 6 | B extended byte-identical restoration (10) | 0 | 15/0, entire crate | present, hashed |

A source restoration equals
`00289ba9a9189b9b24491ef68ae84fa74e75a04d1c19277db4e6bafa26456f5e`.
B source restoration equals
`149da8096fcbcc95a329c3cd01bca9007def9461145a5f31f0ed2e590feeae07`.
Distinct exports can differ because source paths differ; identical source bytes,
not identical exported metadata bytes, are the restoration criterion.

Both negatives reach the intended semantic obligations, not just an unsuccessful
status. A fails `step_invariant` and `error_keeps_actual_complete_flush` when a
failed second complete flush is represented as successful. B fails the best-
effort nonlatching clause of `step_contract`, its nonvacuous witness, and the
normal-versus-best-effort distinction in `source_negative_counterparts`. Raw
diagnostics and absent exact exports are independently retained. No negative
certificate is accepted or a failed run counted as a certified full crate.

## Lane A: connected model theorem and its cuts

The unchanged reviewed storage library SHA256 is
`29485224c99ba2f994012ff53b2ddc98c9cab79daaf7b3fc2a855b952d2fdc4f`;
the unchanged acknowledgement library SHA256 is
`447a7fcfa23fcdec6114be0ec4eb27274ed4133efbc7d81fdac896dffd9306a2`.
Their copies are rechecked unchanged. Counts reconcile to **26 + 15 + 11 = 52
proof bodies**, **61 verifier-reported obligations**, zero native executable
bodies. Repeated checks do not create additional proof scope.

`checked_run` really equates Product.control with the original acknowledgement
`run(config, signals(...))`, by induction on arbitrary finite Action sequences.
It is not label equality between two independent interpreters. At active PCs
24 and 28, the generated signal is exactly the `.ok` of original
`storage::model_flush`; supplied action.signal cannot bypass that result.
Original model_flush drains all stripes and only syncs after successful drain.
The product installs its exact result image and pending state and stores the
before-image, pending inputs, decisions, persisted set, sync choice and full Run.
Actual modeled failure-prefix effects are retained rather than rolled back.

Full symbolic visible/durable/permitted images, arbitrary valid finite pending
stripes and independent 320-byte headers are represented. Successful authored
PC23/27 admission appends a last header stripe; `complete_header` uses the
unchanged last-cover theorem, not a chosen final byte-image premise. Native
serialization/root meaning, cache coalescing, admission and ownership are NOT
established by this authored append operation. Header bytes are not yet related
to the original serializer/recovery contracts or a native root value.

The safety theorem is conditional on original `local_mode(config)`:
**immediate && !multi_process && !shrink_enabled && effective_two_phase**.
It also needs a valid product invariant/initial control, and the public-Ok
corollary assumes the actual resulting status is Ok. The arbitrary-sequence
quantification is not all native modes or a native scheduling theorem. Parent
summaries should state this guard explicitly rather than abbreviating it away.
This is a scope clarification, not a newly discovered failed theorem.

Nonvacuity is substantive at model scope: successful decisions prove generated
signals are all zero rather than assuming a desired trace; the checked witness
reaches real B public Ok for arbitrary valid initial images/pending stripes and
independent headers. Old secondary 1 differs from new slot 3. A separate PC35 Err
after publication retains prior barrier certificates/history with no ack. The
full theorem records ordered complete first/final barrier snapshots and final
drained pending state, not merely two observed sync labels.

The key remaining native cut is the nonflush frame gate. Arbitrary effect
functions are allowed, but an image/pending mismatch synthesizes signal 5,
sets status9/blocked and retains the rejected effect. Stored image on that
branch is a pre-gate snapshot, not the post-callback physical world. Therefore
the final current-image conclusion applies only to executions whose gate passes;
the success witness uses identity effects. This is nonvacuous conditional model
reasoning, NOT proof any native successful commit satisfies the frame. No
backend-inert callback interpretation is accepted.

## Lane B: source-connected extracted history, not native execution

The actual retained native probe fails on io::Error and panic::Location with
no fresh export and zero checked original native bodies. It was NOT rerun.
Exact source fragments/probe adaptations were inspected. Native fragment hashes
are hashes of marker-to-closing-brace substrings, excluding the final newline
and any indentation before the initial marker; they are not whole-line hashes.
The exploratory whole-line inventory is retained, with its false comparisons
explained and corrected by the exact-fragment audit. No mismatch is misreported
as a native source change.

The final extracted crate has **13 proof bodies**, raw **15 verified/0 errors**,
zero native executable bodies. It checks arbitrary finite histories for the four
selected modeled operations. `exact_precedence` includes all four Boolean flag
states: closed=true/io_failed=false alone returns Ok. The closed=>io_failed
invariant is only preserved for complete sequential operations from a reachable
state; it is not assumed as an atomic snapshot law.

Normal write/sync Err permanently latches; isolated best-effort Err does not.
Explicit close has no preflight/reclose guard, stores closed then io_failed in
its source/projection before the backend callback, and leaves both flags true
even on Err. Later selected guarded calls refuse, but another Close still calls.
The model's separate close_stores sequence and endpoint theorem are a sequential
projection, not native temporal/call-resolution or concurrency proof. Other
native methods, unlock's different guard and lock-error paths are not swept
into the four-operation theorem.

`Some(effect)` is unrestricted and independent of Result, including on Err;
it records an opaque token, not a physical state transition or I/O law. The
generic callback continuation retains successful-prefix AND failing-call tokens,
returns the first projected error, and omits subsequent callbacks. Its publication
marker is a supplied history marker, not an acknowledged-root durability frame.
Error transport is exact only within ProjectedError: byte-sequence corruption
messages, nat sizes and integer I/O/location tokens have no checked native
String/usize/io::Error/Location representation relation. Unsupported native lock
conversions and abnormal/panic effects are not proved by this enum's existence.

## Parent finite controls and additional source observations

Both historical parent runs and their reports/results/semantic/history records
were read. A fresh Python-only replay of the retained controls passes all **13
tests**, reproducing **17,476 histories, 51,264 steps, 24,168 backend callbacks,
27,096 preflight refusals**, six semantic mutant/corrected pairs and six selected
exact-source lexical mutant/corrected pairs. Source guards' transitive helper
was authenticated too. This is an exhaustive finite domain of histories of length
zero through three, not arbitrary-history/native/concurrent proof. Lexical
MISMATCH is distinct from parser failure/refusal and is not verifier rejection.

POST-PUBLICATION-CUTS and its source observations were inspected: non_durable_commit
updates in-memory secondary/root/tracking and invokes write_barrier; write_barrier
only conditionally sets committed_pages_buffered. It is NOT physical flush/sync.
Those selected statements do not prove transitive effects backend-inert. A final
native theorem needs an acknowledged-root/history-footprint frame, potentially
weaker than equality of every byte and pending entry. Such a frame is still OPEN.

The later NATIVE-BINDING-FRONTIER documents WERE actually inspected, including
JSON SHA256 `001e55a60ed438c89112cc3e2e87b8029ba937a46e22170c4538032394b8397f`
and the matching frozen companion atomic.rs preamble and load/store declarations.
Their source supplies ordering preconditions and no_unwind but no stored-value
postconditions. This is an independently inspected source observation, not a
new executed AtomicBool diagnostic or actual frozen-engine call-resolution claim.
Clearing opaque-type frontend blockers alone cannot establish the model's native
atomic latching/history relation. No new primitive specification or wrapper is
adopted. The proposed future serializer/recovery composition is a direction only:
those contracts were not newly verified or integrated by this review.

## Remaining native blockers and final status

Native adapter R, original native/compiler correspondence, atomic observation and
state correspondence, exact payload representation, dynamic backend bytes/crash
effects, callback/history/root-footprint/resource relations and header admission
are all unbound. Source-connected lexical checks and conditional symbolic
results do not discharge them. No lifecycle/admission pipeline or excluded task
was rerouted into this review. Native path remains **FAILED**, Git continuity
**UNAVAILABLE**, compiler/source correspondence **UNESTABLISHED**, all18 **OPEN**.

Parent can retain these reproduced bounded results and the explicit scope note;
no candidate patch or additional delegate is requested. Nothing here authorizes
acceptance, native producer adoption, changed S0/Infer/import/export/full-scope
gates, compiler/runtime/vstd/Cargo/application builds or a seventh application.
