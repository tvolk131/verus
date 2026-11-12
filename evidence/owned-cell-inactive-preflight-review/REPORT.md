# Independent inactive owned-cell preflight review

**Decision: accept only inactive preflight/refusal infrastructure, with the limits in [FINDINGS.md](FINDINGS.md).** All 40 independent replay cases passed. No blocking production finding was found for this reject-only checkpoint. There is one source-only compiler-invariant hardening note, R1. No implementation fix was made.

Scope was committed first in `5660aace`. Parent is `4c863add2b4cbfb660a55eed9a5824a655916421`; reviewed components are from `6e5fb54d5904328b34d089688ed17f99bf99fc72`. Read the frozen implementation REPORT/RESULTS/INTERFACE/controller, all five design documents, and design challenge `4515419e`. All implementation/supporting source reads used immutable Git objects or their own prepared copies. No live implementation source/output or `/tmp/owned-cell-implementation-20261112/bundle` was read. Historical command/receipt paths in frozen RESULTS were not used as fresh runtime inputs.

## Authentication and exact identity

[AUTHENTICATION.json](AUTHENTICATION.json) records authentication before private copying. `/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot/receipt.json` has kind `IMMUTABLE_INACTIVE_PREFLIGHT_SNAPSHOT_NOT_ACCEPTED_RUNTIME`, source commit `6e5fb54d` resolving to the full commit above, both proof flags false, and identity exactly equal to frozen `RESULTS.experimental_runtime`. Receipt SHA256 is recorded there and in [RESULTS.json](RESULTS.json). All eight tool bytes matched before copying; private and snapshot copies were rechecked after trials. This is the supplied frozen provenance binding, not an independently reproduced build. The binary's historical build-info string is `0.2026.11.12.a53caa5.dirty`; it was not substituted for hash authentication or claimed to identify the full source checkpoint.

| Frozen component | SHA256 |
| --- | --- |
| config.rs | `224fc7a59360601caf4134c1c64d607b321de86f2c2902023eac8716884b39a1` |
| lib.rs | `1bdfc3735f0f3bfb588497f02c4103c10d4dfa01e983bd2ecce1794eca5cc656` |
| verifier.rs | `94373b62b62f011ec14202f452c6416739a44d801168125497710f9f8b199e16` |
| checked_owned_cell.rs | `8246a5420648b945b2076a12fd2d2005f678b3dfa9c9cbd5841c24e87c6ea8f8` |
| local_checked_owned_cell.rs | `b313ab34d6e627b72c4cca6be47b901968c5e7966ca4fa61aa020083431bcaa0` |

| Tool | SHA256 |
| --- | --- |
| rust_verify | `33cbc5e138d3ce97494d55dbc9d8ee3d1a98afb8ce23f878f56132ad038f3afc` |
| verus | `980e705f81ffd234726ee0f4a829c5a345df6d1cd4517f7d0b8a7406db1c5442` |
| libverus_builtin.rlib | `60f5ff618d302075783eb2d48bc65f403433b8b0120ac41cc1dc27bd5b58c0db` |
| libverus_builtin_macros.so | `265d3e82bf9ce06e8e8caaa43b0da575d8588e3e5ae3c58dbe9922e545279c39` |
| libverus_state_machines_macros.so | `f8b500b05f72c03c6b4e851ead0e3671a7a63321731d30c87f6f852706329443` |
| vstd.vir | `e0364f9982ac0c119b019f7ea13013d49ce893a5b4c764535c0ea8f3acf5104d` |
| libvstd.rlib | `7906b4a490da4dc8fb8f113bce723ece4aba23d4a46f22a561af0f133cb2cf13` |
| z3 | `e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765` |

[TOOLCHAIN.json](TOOLCHAIN.json) records installed `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, full toolchain commit, rustc executable hash and installed native core cell.rs hash. No toolchain was changed. The inherited vstd 2257/0 result is historical receipt identity only; vstd was not rebuilt/reverified here.

## Production source audit

The exact production/test diff is five files: 573 new inspector lines, five config lines, one private module line, 14 verifier integration lines, 167 focused test lines. No VIR/descriptor, call lowering, WF/backend, native source, trusted law, global schema/policy/cache or Rev factory change is part of this checkpoint.

`ArgsX::new` sets the new flag false; parsing sets it only when explicitly present. The old path remains behind an unchanged false condition when absent. `check_request` checks export first, then full checking/no-cheating, then native metadata/signatures/MIR, then returns an unconditional error (checked_owned_cell.rs:311–351). All `?` exits return errors to the same caller. Its only public(crate) operation returns `Result<(), VirErr>` and creates no descriptor, helper, authority, checked Function, native body query, client state goal or serializable fact. No cell semantics are installed on passing structural checks.

The caller at verifier.rs:3524 emits an error, sets `encountered_vir_error`, and stops. driver.rs:326 checks that flag before exec compilation or `publish_verified_interface`; main.rs:540 reports failure and exits nonzero. No current `Ok` path exists through the new request function. The call is outside any Infer-only guard and precedes transitive VIR import resolution/deserialization, crate translation, proof pruning and Infer cache reuse. Existing Cargo extern-path discovery in `config` can run earlier; it does not install owned-cell authority.

`can_certify_inference` requires entire-crate selection and normal lifetime/trait checks. The request additionally requires normal erasure checks and `--no-cheating`. `no_verify`, root/function/module/only-module selection, disabled lifetime/traits/erasure and missing no-cheating are refused. This is an option refusal gate, not proof that all future native bodies have been ordinarily verified.

Early `config` invalidates Cargo and direct selected outputs before parsing. The after-expansion path again invalidates authoritative explicit/Cargo destinations before the request. Existing `invalidate_export` attempts every distinct path even if another deletion fails; missing files are harmless and other I/O failures abort. `args.export` and Cargo export are checked independently, so explicit precedence cannot spare a stale Cargo sibling. The refusal runs even for scalar-only/no-helper units. No descriptor needs to survive pruning to trigger it.

## Loaded native shape audit

These checks read actual loaded compiler objects. The successful final-inactive diagnostic in four independent frozen-body invocations, plus two scalar poisoned-import/cache invocations, means all preceding native checks returned successfully on the authenticated snapshot. This is indirect evidence through the reviewed inspector; no independent raw DefPathHash dump or native corruption unit rerun was performed.

| Native identity | Pinned value |
| --- | --- |
| core StableCrateId | `17281971461272426270` |
| UnsafeCell local DefPathHash | `11949802078424953320` |
| private value field local DefPathHash | `16821345337525786618` |
| new local DefPathHash | `10297035969456002870` |
| into_inner local DefPathHash | `14652106901632559644` |
| get_mut local DefPathHash | `13506035689665434230` |

`native_cell` obtains the real unsafe_cell lang item, compares its crate to loaded Sized's core crate and stable crate identity, then pins ADT/field hashes. It requires the UnsafeCell marker, public struct, transparent-only repr flags, no align/pack/int/scalable override, one variant/one field/no tuple constructor/no destructor. Generics have one nondefault type parameter and no parent. Field parent/name/private restricted module/type Param(0) are checked against actual metadata. The parameter must not satisfy Sized in its own declaration environment. On actual C<u64>, the PostAnalysis Sync query must definitely not apply; ambiguity is rejected. The installed native source separately retains `T: ?Sized`, private `value: T`, and `impl<T: ?Sized> !Sync`. A concrete non-applicability query is not a universal proof about arbitrary T or every auto trait.

`native_method` searches only actual inherent impls of that ADT and rejects ambiguous selection. The selected item must be in the same core crate with the expected hash, public AssocFn, const, MIR-available, one total inherited type parameter and no own parameters. `signature` substitutes exactly u64 and validates before MIR region erasure. Safe Rust ABI, nonvariadic single argument, exact C<u64>/u64 input/output types are required. New/into_inner have no bound variables. get_mut has exactly one binder and mutable references to C/u64 whose region values are equal and are the innermost bound region variable zero. Static, erased, detached or shared-reference mappings cannot pass this signature check. Instance resolution must produce the original Item DefId, not a shim/function pointer. The four region/signature negative functions are inspected source controls only.

`inspect` receives a fresh clone of instantiated/normalized original method MIR. It requires optimized Runtime phase, expected generic-origin polymorphic flag, no coroutine/spread/injection/error-taint/coverage data, present empty required constants and mentioned items, exactly one argument, two locals and one block. Every local is checked for no drop, and the two local types must match the operation. The sole block is not cleanup, has Return terminator and exactly one Assign to the unprojected return place. Extra unreachable blocks/statements, calls, drops, unwind/control flow, casts, intrinsics, storage/retag statements and other rvalues cannot match. Diagnostic/source metadata is not treated as execution. The missing-terminator internal-object caveat is R1.

| Operation | Complete accepted statement shape |
| --- | --- |
| new | Actual native ADT, variant zero, exactly u64 type args, no user type/active field, exactly one Copy/Move operand of unprojected argument local1; input u64/output C. One-field metadata binds operand position to actual value field. |
| into_inner | Use(Copy/Move of local1.field0:u64, WithRetag::Yes); input C/output u64. Consuming C is bound by the original function signature; Copy of the u64 field is permitted intentionally. |
| get_mut | Ref(ReErased, Mut(Default), local1.Deref.field0:u64); input &mut C/output &mut u64 with erased MIR regions. Shared/fake/two-phase borrow, wrong place/field/referent or manufactured MIR region is rejected. Same input/output region was already checked before erasure. |

The unit test supplies 15 independent cloned-body changes per method (45), plus wrong cell and wrong operation per method (6): 51 MIR refusals. It changes counts, local types, cleanup/terminator/target/block, injection/polymorphic metadata, source places, operand/retag/region, aggregate/field/type/borrow identity. Four separate Rust functions test detached lifetime, static output, shared input and wrong inner type. **All 55 were source-inspected, none independently executed here.** They are shape-validation controls, not source-to-logic proofs or native body postcondition failures. Complete arbitrary malformed compiler-object fuzz coverage is not claimed.

## Independent execution and outcomes

[run_audit.py](run_audit.py) uses the existing frozen `infer_experiment.py`/`infer_eval_support.py`, `LoggedCommands`, `replay_receipt`, `ArtifactGuard`, `require` and `validate_run`. All sources, outputs and each invocation's fresh cache/tmp path are private under this directory. No implementation/controller fix or new evidence/security subsystem was added. Private runtime and caches remain locally retained but Git-ignored. Raw logs, exact argv/environment, statuses, source snapshots, receipt/output hashes and validation failures are retained in [receipts/](receipts/) and [RESULTS.json](RESULTS.json).

| Cases, ordinary plus Infer | Count | Outcome |
| --- | --- | --- |
| Default scalar positive / false postcondition | 4 | Two full-crate 1 verified/0 errors successes, native rlib compiled; two full-crate 0 verified/1 error ordinary postcondition failures, no frontend failure. All use no-cheating/compile and normal lifetime/traits/erasure. |
| Repeated unchanged frozen cell clients | 4 | Final inactive refusal, exit1; all loaded native preflight checks precede this diagnostic. |
| Missing no-cheating / disabled checking / partial selection | 18 | Specific full-check refusal, exit1. Missing-no-cheating is an intentional refusal input. |
| Scalar zero-helper explicit export | 2 | Local-only refusal, seeded file absent. |
| Dual explicit/Cargo outputs, valid and malformed input | 4 | Local-only refusal or parse error; both seeded files absent. |
| Cargo-only export, valid and malformed input | 4 | Local-only refusal or parse error; seeded Cargo file absent. |
| Scalar request with invalid VIR import and cache bytes | 2 | Final inactive refusal precedes import/cache handling; no helper/fact accepted. |
| Non-file explicit destination plus stale Cargo sibling | 2 | Invalidation error, exit1; directory retained, Cargo stale file still removed. |
| **Total** | **40** | **All expected outcomes passed; no panic/resource failure.** |

For all 30 request failures that produced parseable verifier JSON, additional post-run checks found empty function details, zero generated functions/checked exec/checked proof bodies, no native inventory collection or assistance preparation, no imported inference dependencies/trusted models, false crate success and false interface_written. Remaining six request cases are four early parse failures and two invalidation failures, with nonzero exits and checked file states. The two successful scalar runs use inherited baseline facts and do not establish a cell theorem.

The five focused Rust integration tests were fully read and their 32 compiler scenarios independently replayed via Python; eight additional Cargo-only, poisoned-import/cache and deletion-error controls were run. The Rust test executable itself and native corruption test were not compiled/run here. No broad build/matrix is credited.

## Failures, cost and clean scope

Initial sandbox wrapper setup failed at bwrap loopback initialization; authorized local commands used elevated shell execution. The initial scope commit lacked Git identity; a per-command Codex identity completed it without changing global configuration. No automatic approval review rejected an action.

Attempt1 made one failed setup invocation because the replay PATH omitted installed rustup. Attempt2 passed 14 cases, then ArtifactGuard stopped before the next command because I rewrote an existing frozen cargo_verus.rs copy while preparing supporting inputs; bytes were unchanged, metadata changed. Both attempts, reasons and raw evidence are retained. I completed input preparation, moved the existing guard to bracket the complete final replay, and started all 40 cases with fresh private outputs/caches. Final guard and post-run source/tool/snapshot hash checks passed. Neither harness failure was treated as production or semantic evidence.

Final compiler/process command wall-time sum is **36.322157 seconds**; all 55 invocations across retained attempts total **47.729463 seconds**, including attempt2's per-command guard work. These are command durations, not CPU or active research time. There were **zero compiler/vstd builds**, zero independently executed native corruption controls, no broad regression matrix, no delegation and no remote writes. No monetary-cost meter is available. Conservative wall budget starts 2026-11-12 15:26:00 UTC, before the first shell inspection, and final recorded time/cost is in RESULTS.json; all wall time is charged with no exclusions and is below 45 minutes.

All authored changes and prepared evidence are confined to `evidence/owned-cell-inactive-preflight-review/`. Scope and final evidence are local commits. Final diff/status checks and exact final accounting are recorded in RESULTS.json; no implementation file was edited.

No stage, complete typed admission, native checked body, emitted-Function correspondence, private native-client final-state or erasure/fidelity gate is accepted. F1/F2/F3 remain absent at this checkpoint. CP1/CP2/source coupling and native_bridge_proved remain false; HMAC/native remains **DEFERRED AND UNACHIEVED**. Native Mutex/guard/atomic construction and REDB state-to-write/crash theorems remain unproved.
