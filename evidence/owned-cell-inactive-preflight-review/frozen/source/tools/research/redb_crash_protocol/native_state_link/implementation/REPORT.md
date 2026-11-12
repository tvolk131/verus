# Incomplete owned-cell implementation checkpoint

The bounded design is **not implemented or accepted**. The explicit
`--experimental-owned-unsafe-cell` flag defaults off and every request fails
closed. `check_request` has no successful return. This checkout creates no cell
representation, field authority, logical helper, assumed contract, native body
query, diagnostic state goal or exportable fact. Do not identify this preflight
as a native bridge, even though its structural and refusal tests pass.

The work stops at an incomplete implementation checkpoint. This is engineering
shortfall, not a demonstrated impossibility of the frozen owned/exclusive design.
The first remaining necessary implementation is F1's complete typed local-unit
admission with a separate canonical header/spec path. F2's client diagnostic
capability and F3's original-name erasure support remain separate blockers. None
is bypassed by the flag. Activating even a constructor rule before completing
these and the acceptance matrix would violate the frozen scope.

Ordered scoped commits are `a53caa51` (prospective interface before trials),
`9972a19c` (inactive request/native preflight and challenge handling), and
`9ad1cec1` (baseline authentication/private controller/focused refusal tests).
The following final checkpoint commit records private-cache/profile hardening
and these results. No delegation, sibling/live parent checkout inspection,
push, PR, global version change or new trusted law occurred. The only reviewer
material read was immutable git object
`4515419e:evidence/native-owned-cell-design-challenge/FINDINGS.md` and its
INTERFACE, as authorized. The implementation does not resolve that review's
acceptance-blocking F1–F3.

Production and manual inventory:

| File | Actual change |
| --- | --- |
| `source/rust_verify/src/config.rs` | Five added lines: flag declaration, default=false, parser/help. User explicitly confirmed this minimal file-list correction. |
| `source/rust_verify/src/lib.rs` | One private module declaration. |
| `source/rust_verify/src/verifier.rs` | Fourteen added lines: explicit request refusal after early selected-output invalidation, before imports/pruning/caches; applies in ordinary and Infer. |
| `source/rust_verify/src/checked_owned_cell.rs` | 573 authored lines including the real-TyCtxt structural test. Closed, private native metadata/signature/MIR preflight; no VIR translation. |
| `source/rust_verify_test/tests/local_checked_owned_cell.rs` | 167 authored lines; five inactive-feature integration tests. |
| `implementation/check.py` | 89 lines using existing `LoggedCommands`, `replay_receipt`, `require`, `ArtifactGuard`, `digest`; isolated offline build and matching vstd/test invocation. |

All research is confined here. Existing Rev factories/descriptors, VIR semantics,
WF/backend, erasure, checked_foreign registration, vstd, native/core/std sources,
REDB source, and all three frozen owned bodies/contracts remain unchanged.
There are zero added native proof annotations/lemmas/axioms. New potential
compiler TCB code is the native inspector, but presently it only rejects requests;
it must receive independent production review before any future authority use.
The normal vstd trust inventory is inherited; no new library fact was installed.
Global schema39/policy78/cache81 and trusted-registry versions are unchanged.

The preflight reads `tcx.lang_items().unsafe_cell_type()`, checks actual native
core/ADT/private-field identities and visibility, generic declaration/?Sized,
representation metadata, no destructor, and a non-ambiguous negative Sync trait
result. It resolves the actual inherent methods, pins their native DefPathHashes,
checks const/safe Rust ABI/arity/types and get_mut's same bound input/output
region before erasure. It freshly instantiates typed MIR and checks every
executable statement/terminator/local/block against aggregate, consuming field
read, or exclusive native field borrow. Diagnostic metadata is not an effect.
These are shape checks, not checked contracts, emitted-Function correspondence
or a source-to-logic simulation theorem. Layout and hashes supply no authority.

Actual loaded native identities (local hash within core stable ID
17281971461272426270) are ADT11949802078424953320,
field16821345337525786618, new10297035969456002870,
into_inner14652106901632559644 and get_mut13506035689665434230.
The observed MIR retains generic-origin `is_polymorphic=true` after substitution;
new copies its u64 argument, into_inner moves the native field with WithRetag::Yes,
and get_mut exclusively borrows the actual deref/field place. These exact flags,
places and local types are checked rather than silently dropping them.

F1 was measured on the unchanged owned_goals.rs, using pinned rustc typed-HIR
printing and privately built macros/vstd. All three scalar ensures clauses
actually produce a header closure and [bool;2], containing constrain_type with
a type-only self-call and the real scalar/tuple spec_eq predicate. In particular,
exclusive_borrow's header mentions cell in that type-only call. A raw closure,
shared-reference, function-value or recursive-call ban would reject the positive
unit. Blanket subtree omission would miss executable/cell-bearing specs.
Existing extract_ensures/BodyErasure source was inspected; no replacement or
macro-origin exemption was installed. The diagnostic HIR print is not a proof.

F2 remains unimplemented: native-method field access cannot authorize a local
client's private final-state postcondition. There is no SAME-BODY client query,
no arbitrary-entry final=next proof, no false final=old query, and no client
store/writeback-deletion versus scalar-result control. Native MIR corruption
checks below do not substitute for those requirements. F3 remains unimplemented:
there are no original-name checked native Function entries or identity-bound
erasure-mode mappings. No assume-external/erasure-off workaround was used.

Actual verification/test inventory:

| Check | Result and interpretation |
| --- | --- |
| Frozen baseline | All 11 inputs, receipt and eight accepted78 tool hashes authenticated before trials, rechecked unchanged at freeze. Historical ordinary/Infer frontend failures retained; neither baseline verifier run repeated. |
| Private compiler | Offline locked build of compiler, wrapper, builtin, both macro libraries and vstd_build. Distinct identities in RESULTS.json; never copied over accepted78. |
| Matching private vstd | Normal existing library build, full checks, 2257 verified/0 errors, on both interim and final experimental compiler. Uses the existing vstd build's trusted-library flags, not a native client --no-cheating theorem. |
| Native structural unit | One passing test: three actual loaded method shapes, 51 independent in-memory complete-MIR corruptions refused, four actual invalid signature/region mappings refused. No method body SMT query or ordinary lifetime-negative test is credited. |
| Inactive integration | Five tests pass, repeated after private cache/profile hardening. Each complete run makes 32 compiler invocations: four default-off scalar positive/false-return controls, four repeated frozen-native refusals, 18 missing/disabled/partial-check refusals, two zero-helper export refusals, four dual Cargo/explicit export or parse-failure controls. |
| Native proofs | 0 generated checked Functions; 0 checked native helper bodies; 0 frozen-client body proofs; 0 state diagnostics; 0 emitted-Function correspondence passes. |

Every client verification invocation uses --no-cheating except the explicit
missing-flag refusal controls. Disabled checking options are refusal inputs,
never accepted proofs. Default-off scalar good/false-return controls retain
normal --compile/erasure/lifetime/trait checks in both modes. Each successful
scalar proof uses its own fresh temporary Infer cache. Export/parse controls
refuse before import/cache access and clear both selected seeded-byte files.
The final controller verifies test-profile tools equal its private bundle and
avoids overwriting an identical read-only solver.

All eight acceptance gates remain incomplete overall. Gate1's baseline
identity check passes. Gate2 has the listed native preflight component only.
Gate3 complete typed unit/spec/capture/adjustment/promotion/callback/import
admission is absent. Gate4 actual full checked Function/place/read/writeback
correspondence is absent. Gate5 has request-option refusal and default-off
regressions, not native full checks. Gate6's positive/false-return/frame/state
queries and strict native body inventories are absent. Gate7 has explicit/Cargo
stale-byte deletion controls, not the complete imported abstraction/cache/wire
acceptance matrix. Gate8's original native --compile/erasure and final-input-state
fidelity controls are absent. Precise escape, privacy, descriptor, ordinary
lifetime and body corruption matrices beyond the native preflight fixtures
remain to be run only after admitting the positives. Refusing everything is not
acceptance of any of those matrices.

Runtime/cost/failures are retained under
`/tmp/owned-cell-implementation-20261112`; RESULTS.json records all 18 command
receipts, argv, exit codes, guards, hashes and durations. Raw logs are retained
there once, not duplicated in this repository. Sum of command wall durations is
789.620s, including overlapping commands, Cargo lock waits and builds; it is not
CPU or active investigation time. The five compiler-build commands total226.714s;
the two complete vstd commands total203.428s. No monetary-cost meter exists.
The registered conservative start is 2026-11-12 14:39:00 UTC; final freeze time is
recorded in RESULTS.json. The entire wall interval is conservatively charged,
without subtracting builds; this remains below45 active minutes.

Retained failures: initial sandbox bwrap loopback setup failed, so authorized
work used elevated local shell calls (no auto-review rejection). The first
controller call reversed LoggedCommands output/cwd arguments and Cargo could
not find Cargo.toml; its two logs were moved into the private run directory.
First real build found seven pinned rustc API mismatches. Early native controls
refused the generic-origin flag and scalar Copy/retag forms; subsequent fixtures
also exposed typed-index, IndexVec and bound-region API mismatches. Initial
integration had two failures: that native refusal and an invalid partial-selection
fixture missing --verify-root. All were corrected with failed attempts retained.
A manual profile copy could not overwrite the identical read-only z3; final
hash comparison matched, the controller now skips identical files, and the full
fresh-profile test rerun passed. None of these failures is evidence of a false
native postcondition or a completed semantic negative.

Reproduction on a fresh private --output directory: check.py setup, build, vstd,
then tests. Setup authenticates accepted78 and copies only its solver; compiler,
builtin/macros and vstd are newly built privately. It authenticates historical
immutable Cargo archives/index/git metadata against their frozen manifests and
Cargo.lock before private copying; it never writes shared caches. For the focused
real-TyCtxt unit, use the controller's env/run helpers with the exact cargo test
argv recorded in RESULTS.json (native-unit-sixth). canonical-header-hir's exact
rustc diagnostic argv is likewise recorded. These commands reproduce preflight
checks and explicit inactivity, not a completed native verifier extension.

Full Mutex construction, including the two atomic Transmute initializers,
same-object sharing/acquisition/poison/guard/raw access/drop, LRUWriteCache bytes
to actual writes, and the complete REDB crash theorem remain UNPROVED.
native_bridge_proved=false, source_coupling_proved=false, CP1=false, CP2=false.
HMAC/native remains **DEFERRED AND UNACHIEVED**.
