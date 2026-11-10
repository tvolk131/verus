# Reference encoding adequacy investigation; native theorem remains open

This lane does not prove the actual native constructor/drop frame or the full
frozen HMAC writer. Native publication is false and no production fact is
installed. All edits are new files in native_reference_adequacy. No compiler,
vstd, application, sibling, parent, global setting, policy/cache/schema, push,
PR, subdelegate, unchecked API, or runtime rewrite is included.

The early minimal interface/probes are committed at `1bf4ec46`; guarded probe
sources and retained trials at `90b93f28`. The separate core obligation proposal
is `7e0601c6`, [CORE_RULE_PROPOSAL.md](CORE_RULE_PROPOSAL.md). It installs no hook.
It is an implementation boundary, not an admitted theorem or review acceptance.

## Concrete findings from the actual existing compiler

`encoding-inventory.json` records inspected identities/anchors. Identities are
not evidence for effects. The existing AIR prelude gives `&mut T` typed current
and future projections, plus update-current laws. `borrow_mut_to_sst` creates a
fresh reference A with C(A)=the source place and updates that place to F(A).
Owned-reference resolution then equates the updated reference's current and
future. This is not lifetime-end resolution, nor resolution of its referent.
The current-update equation is what connects the original input's final value
to that of the actual reborrow. The existing array-unsize helper preserves both
views under its inherited external-body summary.

Fresh native-drop SST/AIR shows exactly that reborrow -> unsize -> native
constructor chain. The owned iterator is **already resolved** by the compiler.
The remaining missing equation is not fixed by resolving it a second time.
The opaque iterator's prophetic remaining references do not inherit resolution
through that spec observer. The `opaque-resolved` proof passes; the assertion
in `remaining-resolved` fails. `owner-only` versus `owner-not-referent`, and the
checked `referent-later-write-v2` caller, make the general owned-value/referent
distinction executable. A blanket observer-resolution rule would be invalid.

The narrow FnMut preservation rule in ast_to_sst uses supported closure capture
semantics; it is separate inherited TCB. No analogous native assumption is
introduced from a callee name/hash or from a no-store observation.

The borrowed reference's runtime address/provenance is absent from the backend
C/F encoding. `MutRef::unpack(r).ptr` is **not** the missing physical bridge.
`pack-null-views-v2` checks a spec-only repack with null pointer and unchanged
current/future observations. It constructs no executable reference or token.
`unpack-nonnull` fails, and the separate `exec-pointer-origin` probe uses a real
executable `&mut [u8;128]` input with no extra premise. A source-valid reference
is not a source theorem relating that ghost field to its runtime pointer.

Actual native `NonNull::from_mut` rejects as unsupported type/function. The
first reference-to-raw cast, explicit raw borrow, and raw dereference also
reject at the frontend. The inherited raw_ptr interface instead requires a
non-Copy PointsTo permission with pointer agreement and initialization; the
`missing-resource` precondition control distinguishes that resource requirement
from an SMT semantic negative. No conversion from the original input reference
has been obtained. The old tracked entry beside actual iter_mut still rejects
with E0499 (`second-borrow`).

## Checked partial connection, with its trust made explicit

`constructor-summary` checks the current/future connection for the SAME actual
native constructor operand and the original executable array, for every valid
input reference, without supplied permission or second mutable borrow. It uses
the inherited **assumed native constructor summary**, not a proof of its body.
Its returned iterator's 128 prophetic item observations have the exact current
and final array equations. This is an ordinary abstract-summary diagnostic,
not native heap/provenance interpretation or a native no-write theorem.

`expose-exit-v2` checks, using an actual constructor call and only ghost
observations, that equality of all those items' current/future values would
imply final(original array)=initial(original array). It has no new requires,
asserts no missing resolution premise, supplies no authority, and returns no
ghost reference carrier. The implication is deliberately conditional. It
isolates the pending-reference exit obligation rather than closing it.

No new interpreter, supplied-owner calculus, primitive semantics, runtime
replacement, ghost custodian or arbitrary allocation model is built here.
The earlier 115/0 authored interpreter remains read-only. None of its model
transitions is promoted to native source authority. All 101 full-writer native
effect obligations remain open; this lane discharges zero of them.

## Precise blocked obligation

The original Rust reference type already requires input validity; no new user
requires is appropriate. What is missing is a proved compiler/source execution
judgment interpreting that valid runtime reference as the SAME subobject/loan,
with initialized bytes, live containing allocation, provenance and authority,
and a primitive/native simulation that preserves it and couples exit bytes to
F(original reference). Every array subobject of a larger allocation must be
covered. The full writer must permit disjoint pad/key ranges in one allocation
and arbitrary key length, including 0 and >128.

The inspected pre-erasure borrowck evidence binds one exclusive loan 0/region3,
source `(*_1)` -> `_4` -> unsized `_3` -> native constructor `_2`. Its static
loan-issued fact supplies no runtime memory interpretation. That phase also
contains FakeRead and an unwind cleanup/resume block absent from the selected
monomorphized trace. Source-phase simulation and exception freedom are proof
obligations, not consequences of matching identities. The exact judgment and
primitive/call/drop/cleanup/offset/release requirements are in the separate
core proposal. No existing theorem found here discharges them. A new native
assumption would enlarge trust and violate the active goal, so no such rule is
implemented or published.

The unchanged native-drop diagnostic remains **0 verified / 1 error**. The
unchanged full writer remains **1 verified / 1 error**, with both XOR prefix
and unchanged suffix and the full original domain retained. These are unproved
positives under inherited contracts, not Rust counterexamples. The canonical
varint 6/0 result is inherited; it is not rerun or newly claimed by this lane.

## Replay, trust and retained trials

`final-cases.json` freezes 20 boundary/reference/resource cases; the additional
`exec-origin-cases.json` isolates the real executable-input pointer gap.
`results.json` binds durable receipts and exact body inventories/outcomes.
All new proof/app runs use --no-cheating, full default body/lifetime/trait/WF/
mode/prophecy checks, explicit exec/proof body identities and exactly zero
inferred helper identities. Shared mandatory soundness/maximality/exactness
gates are called (vacuously for zero helpers). Python controllers use guarded
receipts and require gates under `python3 -B -O`; Python asserts are not gates.

The runtime is an independent immutable byte COPY of authorized
parent-shared68-bundle-676b99bf, with matching compiler/macros/vstd and solver
SHA e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765.
It is the authorized baseline, not a claim to execute the parent's new ea897a9b
release. No core source changed, so no new compiler/vstd build is necessary.
Inherited vstd 2226/0 and its 83 admissions are not re-proved or reduced. Existing
Rust/Verus frontend/reference semantics, inherited reference/array/slice/native
iterator/raw_ptr specifications, solver and proof backend remain trust boundaries.
The pure C/F and raw-pointer permission models have no newly proved adequacy
link between them. Independent parent review remains pending.

Retained excluded trials are classified in results.json: an early diagnostic
substring mismatch; returning prophetic remaining items through Ghost (frontend
rejection); missing old/final disambiguation (frontend); and a caller value
assertion before providing its local helper's checked reference-preservation
contract (ordinary failed development assertion). None counts as accepted
semantic or native evidence. The corrected candidates have separate pretrial
freezes. Semantic negative, resource rejection, frontend/lifetime rejection and
unproved-positive counts are kept distinct. No timeout or resource exhaustion
is accepted as semantic evidence.

From the repository root, with fresh persistent private output paths:

```sh
python3 -B -O source/tools/research/native_heap_lowering/copy_bundle.py \
  --destination /tmp/fresh-reference-bundle
python3 -B -O source/tools/research/native_reference_adequacy/replay.py \
  --bundle /tmp/fresh-reference-bundle \
  --freeze source/tools/research/native_reference_adequacy/final-freeze.json \
  --cases source/tools/research/native_reference_adequacy/final-cases.json \
  --output /tmp/fresh-reference-boundary
python3 -B -O source/tools/research/native_reference_adequacy/replay.py \
  --bundle /tmp/fresh-reference-bundle \
  --freeze source/tools/research/native_reference_adequacy/exec-origin-freeze.json \
  --cases source/tools/research/native_reference_adequacy/exec-origin-cases.json \
  --output /tmp/fresh-reference-origin
```

Raw binaries, compiler logs, SST/AIR, bundle copies and receipts remain under
`/tmp/native-reference-adequacy-2e339ec3/`. No production integration or complete
native success is justified by these results.
