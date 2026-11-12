# Independent F1 directive recheck

**Four-directive finding CLOSED; complete F1 admission OPEN.** Request changes
for new **F1-DIRECTIVE-RECHECK-1** below. Correction70a9c39022df4953ea6c321a43373a4cf83745ac
correctly excludes reveal_group, item_broadcast_use, size_of_global and
structural_const_wrapper on items and all local ancestors. The earlier
const_body/reveal_fn finding remains closed. Every experimental request still
refuses; no current native proof exploit or whole-stage acceptance is shown.

Frozen combined root: ce0f00897bec7f8ea143cd42409563440d9c8784. Early exact scope/
source/tool interface commit:2b586aa0a. Only this NEW evidence directory is
committed. No production edits, subdelegation, push/PR, parent validation polling,
mutable parent-cache use or multi-gigabyte build-cache copy. Exact ancestor
instructions were searched (no applicable AGENTS.md found); all four required
frozen goal/worker/acceptance documents were read and hashed. D1/D2 were not reviewed.

## F1-DIRECTIVE-RECHECK-1: crate-root opt-in is not ordinary body coverage

**P1 for complete F1 acceptance**, at
`source/rust_verify/src/checked_owned_cell_admission.rs:679`.
Minimal valid witness:

```rust
#![verifier::verify]
use vstd::prelude::*;
pub fn unit() {}
```

On the freshly built frozen combined compiler, ordinary AND Infer feature-on
runs reach `inactive: F1 admission passed; functions=1, bodies=1, headers=0; ...`.
Both default-off runs succeed, verified=0, empty func-details, zero checked
exec/proof bodies, with unit classified `outside_selected_scope`. No frontend
error or panic supplies this evidence. The authenticated old compiler gives the
same four outcomes:8 independent finding invocations, with asserted oracles.

Cause: checked_item walks through the crate owner and accumulates its positive
verify/verus_macro/sets_mode flags. Ordinary `external.rs:get_crate_items:138`
starts Verify only for the actual no-external-by-default option and visits root
contents without processing crate-owner verification attributes. Experimental
admission can therefore count a body ordinary translation classifies external.
This is a coverage mismatch, not a demonstrated false native theorem: the later
gate still refuses with zero proof/helper/export credit.

Smallest fix: retain every ancestor skip/external/trust exclusion, but do not use
crate-owner opt-in flags as positive checked status. Preserve the actual default-
verify option and ordinary item/module/nested contexts. Add both-mode root-only
negatives and item-marked/module-marked/default-verify positives. Independent
controls here already confirm the latter routes, root-plus-item/module and a
normal macro. No production fix is made. Actual admitted-owner versus translated/
checked-Function reconciliation remains a FUTURE activation requirement; this
small fix or an attribute list cannot substitute for it.

## Independent execution and directive closure

**All11 focused tests /198 verifier invocations PASS**, retaining all166 old calls
and exactly32 additions. Original bodies/oracles are unchanged. The isolated
harness removes only unused rustc_private and adjusts the frozen include path;
a minimal InferCrates shim replaces broad common code. It uses pinned
serde_json1.0.151/tempfile3.27.0 and no compiler test dependencies. Private launchers
record calls and relay authenticated tool output/status unchanged. The reversible
transformation is checked and hashed; raw cases are retained privately.

All166 JSON admission calls have zero checked bodies/helpers/export credit.
The32 new and16 earlier-marker cases give their precise early refusal and no
later-admission message. Original option, selection, import/callback, capture,
type/header/quantifier/recursion, Cargo and stale-output tests pass. The frozen
trio still gives3 functions/6 bodies/3 headers. The suite also compiles its import
fixture once with native rustc; that is not a verifier call or native proof.

**160 additional calls** give **358 total**, all matching their recorded oracles:

*128 direct calls include20 old and20 new replays of valid reveal-group,
 broadcast, checked unit, scalar/header and false-contract witnesses, both modes
 and feature states. All10 default-off old/new comparisons match on stable result,
 function/contract, body-scope and helper/export fields, excluding source paths.
 Unmarked false contracts reach later INACTIVE refusal when enabled and fail
 ordinary postconditions when disabled; valid units are checked.
*88 of those128 are additional corrected controls:72 directive negatives across
 deep module/crate/const/function ancestors, unused nested modules, generated
 macro modules/nested items, cfg_attr and verus!;8 two-marker-priority negatives;
 8 cfg-disabled unmarked positives reaching only later refusal. Marked requests
 have zero proof/helper/export credit and no emitted .rlib/.vir.
*24 classification controls check marked item/module, root-plus-item/module,
 explicit default-verify, ordinary macro and encoded_const routes in both modes
 and feature states. Default-off emits ordinary checked Functions; feature-on
 reaches only later refusal with zero credit.
*8 old/new crate-root witnesses establish the new finding above.

334 calls use the fresh corrected compiler;24 use the immutable old one.
Malformed special-const default-off fixtures are NOT counted as successful
admission/semantic evidence. Prior frontend failures/panics remain excluded and
were not rerun. Only valid reveal/broadcast and checked unit witnesses establish
that contrast. AUDIT.md distinguishes true omissions from FunctionInfo already
recorded before Ok(None), encoded ConstInfo dispatch and out-of-grammar paths.
Existing type/call/capture/header/trust guards remain; source markers and macros
confer no compiler authority.

## Authentication, costs and exclusions

All8 baseline tools authenticated BEFORE copying; only7 supporting artifacts
reused, then rechecked. Receipt SHA256:
`9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f`.
Old compiler:
`d48a95360484cda8128131b6fd33548de2a454d48c03bf1a23e66d48806c0dd3`.
Independent offline compiler-only build used a fresh private cache/output and
existing implementation/check.py, infer_experiment and infer_eval_support.
All guarded compiler-build source hashes match exact frozen-root blobs.
Fresh rust_verify SHA256:
`c3c2033a44b7b987662f96fbd587f703bf1875bdda13cd12274533e791676ada`.
Compiler/core toolchain remains Rust1.99.1. Offline archive/index/git and source
Cargo.lock checks are retained. NORMAL vstd2257/0 is inherited, not rebuilt or
reverified. D1 descriptor remains uninstalled; no new library type/serialization
or matching vstd rebuild is assumed. Parent validation supplies no pass claim.

AUTHENTICATION.json binds old/new8-tool maps; SOURCE_HASHES.json binds audited
source. RESULTS.json records precise argv, receipts, controller/raw-manifest
hashes and costs. REPRODUCE.md documents commands/transformations. Measured
compiler build397.763s; separate small harness build38.269s; focused54.753s
(including3.31s rebuild); direct calls152.526s, classification87.347s, root
witnesses43.883s. Concurrent command sums are resource costs, not elapsed wall
time; nested focused calls42.766s are NOT additive. Conservative wall/active
upper bound starts17:23UTC and includes setup/builds/waits, remaining under45
minutes. No CPU/dollar meter. Raw repeated sources/logs/final binaries remain
PRIVATE under `/tmp/owned-cell-f1-directive-review-ce0f0089/`, hashed in its
RAW_HASHES.json. Disposable compiler/dependency intermediates are not evidence;
this review's compiler build intermediates were removed after successful
validation when disk filled during report writing. Final compiler/tools/logs/
cases/receipts were retained, and truncated documentation restored.

Sandbox startup failed before execution; escalated workspace commands were
approved. Imports briefly generated bytecode outside this folder; only those
generated files were removed and bytecode disabled. One broad process listing
incidentally exposed unrelated build argv; it was not followed or used. No
sibling/live-root/actor source or cache was inspected or modified.

## Bounded acceptance

Only the four-directive defect is closed, preserving the closed two-marker
finding. Complete F1 remains unaccepted due to the new concrete mismatch and
future actual Function reconciliation gate. D1's three native Function proposals
still FAIL ordinary private-field WF: ZERO field authorities, native body queries
or state proofs. This review does not reexecute or accept D1. Full correspondence,
F2 client final-state, F3 native erasure and user installation remain absent.
No native/whole-stage/application acceptance. CP1=false, CP2=false,
source_coupling_proved=false; REDB recovery unproved. HMAC/native
**DEFERRED AND UNACHIEVED**.
