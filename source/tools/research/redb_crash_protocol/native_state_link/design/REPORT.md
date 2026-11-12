# Design checkpoint: checked owned/exclusive native cell route

**Recommendation:** implement and independently review the closed owned/exclusive translation in IMPLEMENTATION_TARGET.md as one bounded prerequisite experiment. It can check the actual native UnsafeCell<u64> constructor, consuming field read and exclusive field reborrow without a new assumed library law or replacement runtime object. It must reject shared escapes and the full native Mutex path. No implementation/prototype is installed by this design checkpoint.

The actual retained exclusive_borrow native field write is mandatory, with an additional checked final-input-state obligation. Proving only the two scalar-elided functions, or only exclusive_borrow's returned scalar, is insufficient. The next implementation must use normal body/lifetime/trait/WF/mode/erasure/safety checks and --no-cheating in ordinary and Infer runs, on all u64 values and the unchanged runtime call bodies.

## Deliverables and ordered local commits

1. `10c2861a` — early interface, restrictions, risks and exact shared-guard debt in INTERFACE.md.
2. `e04e2ca2` — constructive implementation/files/gates in IMPLEMENTATION_TARGET.md; adversarial framing analysis and missing sequent in SOUNDNESS_AND_GUARD.md; typed acceptance and semantic/admission negatives in ACCEPTANCE.md.
3. This report checkpoint — evidence, costs, limitations and integration recommendation. All changes are confined to native_state_link/design/ in the prepared fe536b27b5dd259aa03320db0b9b4fa51714e89a checkout. No push/PR or delegation.

## Findings tied to actual facilities

* The two actual internal_repr.rs files implement an inactive, closed core::iter::Rev descriptor with private native-field validation, nonserializable authority and ordinary backend support. Existing IR tests include field reborrow/writeback and omission failure. They provide useful mechanics but neither install UnsafeCell nor establish any shared cell law. A simple additional one-field descriptor without escape restrictions would be unsound.
* checked_foreign.rs accepts a narrowly inspected actual bool-to-u64 MIR body and constructs a full ordinary checked body; it is currently registered only with infer_goals. Its call-target plumbing and independent emitted-body comparison are reusable patterns, not a ready cell engine or a permission rule. The proposed cell path must also work goals-off and must retain actual native call identity at erasure.
* Installed core cell.rs:2311-2313 declares the actual private `value` field. new (2334), into_inner (2353), and get_mut (2458) are aggregate/field-read/exclusive-field-borrow source operations. Shared get (2434) performs raw casts and is outside the proposed interpretation. Validate loaded identities/types/regions and actual bodies rather than relying on these source line numbers or layout equivalence.
* The original owned_native MIR genuinely retains `((*_1).0: u64) = copy _2` and its field read in exclusive_borrow. owned_roundtrip and exclusive_update optimize to scalar moves. These compiler observations classify the target; they are not proofs of its native-state simulation.
* The original native_handoff MIR first constructs its actual UnsafeCell and moves it into actual Mutex field2. It then shares the Mutex, calls original lock/unwrap, forms the guard's raw data pointer, stores through it, drops the real guard, reacquires, loads and drops again, with unwind edges. This is the real same-object target; no PCell control establishes it.
* The full Mutex constructor also contains **two native Transmute operations**, initializing the futex Atomic<u32> and poison Atomic<bool>. Installed core atomic constructors at atomic.rs:555 and 2665 actually use transmute. These effects are outside the three-method target, not automatically proved by its constructor rule. No representation-equality shortcut is recommended.
* Actual REDB cached_file.rs:347-349 builds Arc<Mutex<LRUWriteCache>> stripes; flush_write_buffer:490-518 accesses their guarded entries and submits bytes. Owned u64 support can remove a primitive prerequisite at the real data initializer, but does not establish those container/drop/guard effects or reduce their status to proved.

## Adversarial result and exact boundary

An abstract NativeHook taking `&UnsafeCell<u64>` can legally change its contents, then into_inner returns the changed value. Ordinary logical-field framing across that abstract call would falsely prove the constructor value survives. SOUNDNESS_AND_GUARD.md gives a complete native implementation with distinct initial/next values, and audits nested containers, aliases, generics/projections, callback captures, raw pointers, drop/unwind, imports and ghost snapshots.

The reliable small answer is a positive typed grammar for the complete local verification unit, with no shared executable reference or unsupported container/callback/foreign operation, not a shallow scan for UnsafeCell method names. Unsupported forms are rejected before logic/pruning/cache reuse. No shared-state selector is exposed in specs. Native &mut C still grants ordinary exclusive access to its field; this is the constructive part that can be checked with existing reborrow/writeback machinery.

The unresolved sequent is one-time transfer of the **same actual mutex.data subobject's** owned state into an invariant tied to that mutex's actual platform lock, then acquisition of its current physical permission by the original guard, justified get/raw access for the returned lifetime, and restoration by actual poison/drop/unlock/unwind behavior. Moves/publication must not duplicate a constructor snapshot. `&mut MutexGuard` does not supply `&mut Mutex`; its lock field is shared. Existing PCell/raw_ptr/invariant principles apply only after a checked same-native-object connection. No assumed operation summary, client-supplied token/equality, structural twin or layout match supplies it.

## Evidence and validation performed here

Read GOAL/CURRENT/WORKER_BRIEFS, native_state_link interface/frozen sources/results, native_lock_bridge report/missing sequent and original native source/results; inspected actual checked-in rust_verify/vir internal representations, checked_foreign/plumbing, relevant WF/export/type construction/config hooks and tests, vstd cell/raw_ptr principles, pinned installed core/std bodies, and the frozen REDB cached_file sites. No live parent or sibling source was read. The failed guesses rust_to_vir_datatype.rs and native_writer.mir were corrected to rust_to_vir_adts.rs and the recorded native/lock_bridge_probe.mir path; no conclusion relies on absent files.

A read-only authentication command used existing infer_eval_support.digest/ArtifactGuard:

* All **11** BASELINE_FREEZE input hashes matched, mapping `/home/dev/projects/verus/` repository-relative paths into this prepared checkout. It did not follow those paths into live parent source.
* Accepted78 receipt and **all eight** tool hashes matched their frozen values; tools were only read, not run or modified. No new runtime claims, private compiler bundle or cache was needed.
* Owned native MIR matched SHA256 `969757fd6e8060f87270fe985f67aa4c383960d6fb3439eabc99c689792d3485`.
* Native lock MIR matched SHA256 `27076c576f5dda5472c92182b9198b9d78842a84633b09ce540bf67777dac101`.
* ArtifactGuard passed; git diff whitespace and final scope checks passed.

Historical BASELINE_RESULTS remain: ordinary and Infer both reject the native owned cell facilities at the frontend, with no new native body theorem. The earlier PCell positive/false-property controls establish only their existing separate-object scope. No verifier/compiler invocation, native execution, broad build, new positive/negative proof result or independent review occurred in this design turn. ACCEPTANCE.md programs and corruption tests are requirements for a future implementation, not reported test successes. No new evidence framework, production fact or safety-erasing switch was added.

## Cost and stop

Clock start 2026-11-12 14:19:57 UTC; value checkpoint 2026-11-12 14:35:48 UTC. Budget conservatively charged as the entire 15m51s wall interval, including reading/setup/writing/commits; no build exclusions and no separate active stopwatch. Final commit/status verification may add less than one minute and remains below 45 active minutes. The one static authentication/check command took 0.722s. No monetary-cost meter is available; compiler/build costs are zero for this turn. This was design/analysis, not a measured automation improvement.

Initial sandbox setup failed at bwrap loopback initialization; in-scope local work used approved elevated commands. The first commit attempt lacked git author configuration; retry used the existing repository commit identity via per-command flags, without changing global configuration. Neither was a verifier or semantic failure.

## Integration recommendation and status

Proceed only with the pre-registered closed owned/exclusive implementation and its independent review; do not broaden it into a general effect/foreign-MIR/raw-memory program. Its acceptance must include final native input state, source/private/import refusal gates and the adversarial abstract callback. Keep full original Mutex initialization, shared publication/acquisition/guard/drop relation and actual LRUWriteCache bytes-to-write-trace coupling explicitly open. If constructor atomic validity or the shared seal is needed, checkpoint that exact next sequent before a separate bounded effort; do not assume it follows from this design.

`native_bridge_proved=false`, full `CP1=false`, `CP2=false`, `source_coupling_proved=false`. HMAC/native is **DEFERRED AND UNACHIEVED**. This design does not complete a REDB crash-proof property or authorize hiding any original unsupported application cases.
