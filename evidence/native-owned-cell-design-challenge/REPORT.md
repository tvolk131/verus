# Frozen owned-cell design challenge report

**Decision:** scoped feasibility, conditional on the stated complete-unit restrictions and two concrete refinements in [FINDINGS.md](FINDINGS.md). No legal counterexample satisfying all proposed type/call/spec/effect conditions was found. This is an independent design review at `0dd6130fc221f403078797b1f878fba4ada7ea7e`, not acceptance of an implementation. No implementation worker or live parent output was consumed.

The owned/exclusive interpretation is plausible because the actual installed new/into_inner/get_mut operations are construction, consuming field read, and an exclusive field reborrow. UnsafeCell changes shared immutability, not mutable-reference uniqueness. A lawful arbitrary initialized `&mut UnsafeCell<u64>` therefore needs no constructor-history or client token premise. Ordinary current/future/writeback machinery is a plausible mechanism; the existing inactive Rev tests only demonstrate mechanics.

The two acceptance-blocking refinements are:

1. Define the canonical specification-scaffolding boundary for the full typed-unit gate. Even the frozen scalar `ensures` generates a closure, bool array and type-only recursive call under builtin `constrain_type`. A literal raw typed-node whitelist rejects the positive; skipping all spec/capture subtrees opens the intended spec gate. Recognize the actual builtins, extract and check the scalar predicate surface, and preserve rejection of real executable captures/C-bearing specs.
2. Define a narrowly scoped compiler diagnostic authority for the local client's final native input field. Native-method-only field authority does not itself authorize `exclusive_borrow`'s private diagnostic postcondition. This must be an ordinary checked body obligation with exact input-place/body binding, not a supplied equation, public selector, forged core owner or SST-only demonstration. Remove-store/writeback controls must fail even with `result=next` retained.

The source audit additionally identifies exact integration hazards: existing foreign registration is Infer-only and omits external bodies; ordinary crate translation does not validate native bodies; descriptor validation does not validate all uses; generated helper substitution records original call names for erasure and therefore needs checked original-name modes/mapping; imports, zero-helper export refusal and pre-pruning admission must remain independent of proof/cache selection. Early selected-output invalidation already exists. See the detailed findings for anchors and minimum regression cases.

## Evidence obtained

[AUTHENTICATION.json](AUTHENTICATION.json) records successful SHA256 matches for all 11 BASELINE_FREEZE inputs, accepted78 receipt and all eight tools, plus all eight BASELINE_RESULTS raw artifacts. Repository paths were rebound into this prepared checkout; external pinned tool/artifact paths were read directly. Five frozen design documents were separately hashed. An ArtifactGuard snapshot/check passed. These authenticate available inputs, not native correspondence or a new proof.

Read the exact five frozen design files, native source/goals, baseline interface/script/results, and the needed existing internal_repr, datatype/WF, call lowering, body-backed Query, macro/header extraction, erasure, verifier/config and import/export code. Read installed Rust1.99.1 core cell source, including private `value`, native !Sync, the three actual methods and uniqueness documentation. The historical native MIR has the actual input field store/load in exclusive_borrow; the other two functions are scalar-elided. No PCell stand-in was used or credited.

[NATIVE_RESULTS.json](NATIVE_RESULTS.json) records the final bounded ordinary Rust diagnostics, replayable with [run_native.py](run_native.py):

* Compile/run [native_witness.rs](native_witness.rs), directly including the unchanged frozen native functions. Direct and outer-shared callbacks turn 3 into 7; both are outside the proposed grammar. Four selected initial values and three next values exercise the exact functions, including native input readback after exclusive_borrow and a prior lawful shared mutation that ended before the unique borrow. A return-only control returns 7 while its actual input remains 3.
* Four independent native compilation failures from [native_rejections.rs](native_rejections.rs): native C is not Sync (E0277), overlapping unique borrows (E0499), escaping local reference (E0515), and private field read (E0616). These check ordinary Rust behavior, not enforcement by the proposed verifier extension.

Final diagnostic commands used pinned rustc, edition 2024, no verifier, and temporary binaries. Six compiler/process invocations took 0.231 seconds total in the recorded final run. An earlier identical diagnostic pass used edition 2021 before aligning the runner to the frozen edition; it also passed and made no proof claim. ArtifactGuard passed around the native run. A transient Python utility bytecode cache was removed and the runner disables bytecode writes; the final tracked/untracked scope is only the owned evidence directory.

## Limits and remaining obligations

No verifier/compiler rebuild, production edit, new verifier invocation, erasure compilation with the proposed extension, live implementation test, shared cache, delegation, push or PR. No new positive/negative body-proof results. The macro issue is established from checked-in expansion/lowering source, not a newly dumped typed-HIR artifact. Loaded typed MIR for the three library method instances, independent emitted-body correspondence, future authority representation and all-u64 final-input-state checking remain implementation obligations. Native finite-value checks cannot establish any of them.

No drop/unwind/native foreign wrapper was admitted or modeled. Unknown cleanup/destructor effects must be refused, not inferred absent from a scalar signature or a hash. Native !Sync must survive the future verifier's original trait/erasure checks; its ordinary Rust diagnostic here does not establish that integration. Full ordinary/Infer positives must precede credit for future admission negatives; the historical baseline still rejects cell facilities at the frontend in both modes.

Sharing remains explicitly outside this phase. The callback witnesses refute immutable shared field framing, not the closed exclusive route. Publication of the same actual mutex.data state into an invariant, lock/poison/atomic construction and acquisition, raw pointer access, original guard release/drop/unwind and LRUWriteCache bytes-to-write coupling remain unresolved. Nothing here proves the full native Mutex constructor, guard path or crash theorem.

`native_bridge_proved=false`; full `CP1=false`, `CP2=false`, `source_coupling_proved=false`. HMAC/native is **DEFERRED AND UNACHIEVED**.

## Ordered evidence-only commits and budget

1. `c6e880a1` — small scope/interface commit before deep review.
2. `6420905e` — detailed design challenge, input authentication, native witness/rejections and recorded diagnostics.
3. This final report checkpoint — erasure mode-lookup clarification, evidence/limits and scope/budget verification.

Start 2026-11-12 14:42:19 UTC; final clock checkpoint 14:54:12 UTC (11m53s). Conservatively charge the full wall interval, including setup/reading/diagnostics/commits, with no build exclusion. Final verification remains well below the 45-active-minute cap. No monetary-cost meter is available. The initial sandbox failed at bwrap loopback setup; approved elevated local commands were used thereafter. This was an environment failure, not a verifier result.
