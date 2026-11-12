# D1 checkpoint: inactive descriptor/body scaffolding, no native proof credit

The feature remains INACTIVE/UNACCEPTED. Every experimental source request
continues to the existing early refusal or F1 later-phase inactive refusal.
This phase installs no native datatype/helper into a user crate, rewrites no
native call, grants no field authority, and runs no native helper body SMT query.
Three full Function proposals exist only behind an uncalled private compiler
proposal API, exercised by a real-TyCtxt test. Ordinary full Function WF rejects
all three with `disallowed: field expression for an opaque datatype`.
That private-field obligation is the first unproved obligation.

Prospective interface af389f366 was committed before trials; production milestone
a942fba8f contains the closed descriptor and inactive proposals. D1_RESULTS.json
records the final source/runtime/receipt hashes, command costs and failures. The
focused phase runner is 3ee281c6d; the final evidence has its own subsequent commit.

The root-routed P1 admission repair is immutable
9cc0f65c5e8ebb75f6858475277991e3bfd940d7. This phase leaves admission untouched
and does not duplicate root's repair, fresh build or independent recheck. This
checkout therefore still has the historical F1 omission for internal_const_body
and internal_reveal_fn. Root must preserve its repair during integration. The
existing F1 tests are regressions only; F1 independent acceptance is PENDING.
No proof or authority can be activated while that coverage gate is unresolved.

Production/manual/TCB inventory:

* rust_verify/checked_owned_cell_lowering.rs (new) is a private proposal module.
  It freshly validates the actual loaded native lang item, existing metadata,
  native generic sizedness, original methods/signatures/regions and complete
  typed method MIR before building the native nominal datatype and three full
  monomorphic Function proposals. It preserves the native core path, private
  value:T, generic declaration and native runtime type. The proposals use real
  local helper scope; none claims to own core's private module.
* vir/internal_repr.rs adds a separate closed OwnedUnsafeCellU64 family. It
  preserves the Rev validator, checks actual descriptor-bearing DatatypeX
  shapes and permits only u64 instantiations. Encoding does not imply source
  access. Internal descriptors remain nonserializable and undeserializable.
  The factory/validator and future correspondence/authority are compiler TCB;
  no new trusted logical law or trusted-registry entry was added.
* vir/well_formed.rs checks descriptor instantiations through the existing normal
  type-check path. vir/context.rs revalidates descriptor shapes and walks actual
  Function/datatype type uses at backend entry. Rev-only paths retain the old
  shape checks without the new owned-cell traversal.
* rust_verify/checked_owned_cell.rs only declares the inactive module and invokes
  its test from the existing loaded-native unit. Its production check_request,
  admission, option/export refusals and normal compiler pipeline are unchanged.
* implementation/d1.py uses existing check.py/ArtifactGuard/LoggedCommands and
  adds new D1 labels, immutable snapshot guards and explicit VIR/test inputs.
  Research stays entirely in implementation/. No new evidence subsystem exists.
* config.rs remains in the authorized prospective/cumulative feature inventory:
  its prior off-by-default flag is unchanged in D1. There is no new global
  schema39/policy78/cache81, default, wire or registry version change.

Original/body/proof inventory:

* The frozen owned_roundtrip, exclusive_update and exclusive_borrow source bodies
  and their native calls/types are unchanged. This phase does not lower or prove
  any of these client bodies. Their existing admission/refusal checks remain.
* Three actual native method instances are inspected: new (native aggregate
  with copied input), into_inner (native moved private field), and get_mut
  (native exclusive field reborrow from the mutable input). Their proposals are
  full ordinary FunctionX objects with bodies and four proposed postconditions:
  new result.field=input; into_inner result=input.field; get_mut current result
  equals current input.field and future input.field equals future result.
  Preconditions are empty; no external_body, assumption, axiom or proxy is used.
* The independent computation validator consumes actual emitted Function/types/
  places/read modes and original typed MIR, rather than the builder's operation
  descriptor or proposed postcondition. It checks the actual nominal constructor,
  argument/local identities, private field, copy/move, mutable dereference and
  reborrow. This is a partial correspondence milestone: it does not yet validate
  contracts/current/future/client stores/writeback or install a method authority.
* The native descriptor passes initial crate WF with no-cheating=true. All three
  full Function WF checks then reject their private field expressions. There are
  ZERO passed full native Function WF/mode/lifetime/body obligations and ZERO
  native/helper/client state SMT proofs. No skipped WF, forged owner, source
  attribute privilege, public selector or SST-only substitute was introduced.

Focused validation and bounded costs are frozen in D1_RESULTS.json. Eight
existing Rev tests preserve wire/privacy, corrupted shapes, ordinary backend
computation/reborrow/writeback and pruning behavior. The loaded-native unit
retains 54 MIR plus four signature/region negative controls (including R1's None
terminator) and exercises the new proposals. New controls cover eight corrupted
descriptors, six invalid type-argument vectors, both wire directions, source
privacy, wrong-width use at normal WF and backend hooks, and 45 independent
emitted Function/origin/read/target corruptions across the three methods.
The normal full Function checks explicitly assert the actual privacy diagnostic;
passing that negative test is not a checked helper proof. Existing F1/refusal
suites are run in ordinary/Infer and keep zero proof/helper inventories and exact
inactive-refusal expectations. The root's P1-specific tests are not rerun here.

Historical artifacts were authenticated and guarded, never modified:
/tmp/owned-cell-preflight-6e5fb54d-parent-snapshot and
/tmp/owned-cell-F1-2cb432c6-parent-snapshot. The latter receipt SHA256 is
51ae6b41380ac4bdd8dd362889a4818df7b352d86aa032d306055377a6b98bf1.
The continuing /tmp/owned-cell-implementation-20261112/bundle is mutable build
output, not either historical identity. Old logs/receipts remain unchanged;
all new commands use D1 labels. Seven noncompiler artifacts exactly retain the
F1 snapshot identities. Its private vstd2257/0 is inherited, not reverified by
this compiler. No vstd/library/macro source, serialized library datatype or
native Rust runtime type changed; the new uninstalled compiler-only descriptor
has no imported instance. Thus no unrelated full vstd rebuild was performed.
The new compiler has a distinct eight-tool bundle identity and never substitutes
for the accepted78 baseline. No shared mutable cache/output, delegation, live
parent/sibling read, push or PR occurred.

Costs/failures: start 2026-11-12T16:26:02+00:00, freeze
2026-11-12T17:02:30.995305+00:00. Conservatively charging ALL elapsed time, including
builds and waits, uses 2188.995s (36.483 minutes), below
45 active minutes without any build subtraction. Ten recorded commands total
577.025s; four compiler-build commands total
215.934s. Native test commands also include test
compilation. Final native test execution is0.11s, Rev0.34s; existing refusal and
admission test execution is33.02s and7.75s. The combined command is
107.928s including compilation. No CPU or monetary meter exists. Two early attempts
failed compilation (five pinned Rust API/name-shadowing/span-borrow errors, then
one nonexistent test new_typed API); corrections passed. Those original receipts
and logs remain at their unique D1 labels. Native full-Function privacy refusal
is retained as an expected unresolved obligation, not counted as a proof pass.

The compiler SHA256 is
`d6f766f264925bf350980f26fa087163ced9839196ce448bdb3b187b1f5f670c`.
All eight runtime hashes and every command's receipt/log hashes are in
D1_RESULTS.json; raw logs are referenced once rather than copied into the repo.
The final runtime remains in the private mutable bundle until root authenticates
and relocates it; no build remains running at this checkpoint.

Remaining activation gates: root P1 repair/recheck and admitted-versus-translated
body inventory; compiler-private authority bound to the validated original
native method, rechecked after full Function construction/WF/backend; three
ordinary verified native bodies/contracts; complete independent emitted body,
contract, current/future/input/store/writeback correspondence; normal client
mode/lifetime/trait/safety/body/erasure checks; false return/frame/state and all
privacy/corruption/lifetime/export/import/cache tests; real original --compile
native erased calls and F3 original unresolved/resolved native-name mode entries.
F2 is still wholly unimplemented: DISTINCT full SAME exclusive_borrow body goal
on the original incoming cell with arbitrary entry, final.field=next, false
final.field=old and deleted store/writeback while result=next, plus wrong input/
field/region refusals. No constructor history or supplied entry equality may
replace it. All requests stay inactive until ALL these gates pass.

Native bridge, source coupling and REDB CP1/CP2 remain false. Full Mutex
constructor including atomic Transmute, shared acquisition/guards/poison/drop,
LRUWriteCache and the complete REDB crash theorem remain UNPROVED. HMAC/native
remains DEFERRED AND UNACHIEVED.
