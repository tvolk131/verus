# Checked native byte-iterator certificate requirements

This is an implementation target, not an accepted bridge. The frozen full
HMAC byte-writer property/domain in `transfer-freeze.json` remains required.
The [native-source investigation](../native_iterator_refinement/REPORT.md)
establishes useful source facts and a checked abstract countermodel; it does not
prove correspondence to the native implementation or the full writer theorem.

## Parent-checked extraction milestone

The diagnostic extractor in `f76fe2a6` was rebuilt and replayed from the parent
checkout under Python `-B -O`. It reproduces 25 distinct concrete instances,
401 statements/terminators, 31 resolved call/drop edges and layouts for 163
locals; every resolved edge is present in the inventory and no work item remains
pending. The pointer-distance intrinsic and two unavailable panic bodies remain
explicit semantic blockers. `native_bridge_proved` is false. This establishes
an executable native call inventory, not operation checking, ownership
conservation, a native correspondence theorem or the full writer theorem.

Raw parent receipt:
`/tmp/verus-infer-buffer-cycle-20261109/parent-native-extraction-e4011e16/receipt.json`.

The [subsequent typed-checker milestone](../native_iterator_certificate/REPORT.md#second-executable-milestone)
(`42d1c255`) extends the inventory to 32 instances and 408 operations, including
scope-exit drop queries. Its value diagnostics and two scalar VCs are conditional;
all native ownership/refinement obligations remain open and publication always
rejects. The parent replay is recorded in
`/tmp/verus-infer-buffer-cycle-20261109/parent-native-checker-42d1c255/receipt.json`.
The smaller [constructor/drop inventory](native-drop-inventory.json) has six
instances, 44 operations and no extraction blockers. This is a separate no-write
diagnostic, not a replacement for the full writer or a native frame proof.

The subsequent research preserves checked executable/reference carriers
(`native_borrow_entry`, `98f428fa`) and 18 conditional linear-resource proofs
(`native_effect_soundness`, `20973453`). A tracked slice can be obtained from
the executable array, but retaining it beside actual `iter_mut()` rejects with
E0499. It exposes no raw-pointer permission or provenance. The next link is
one ownership interpretation carried with that same native constructor operand,
followed by checked pointer Offset/raw-reference, resolution and helper effects.
No actual native correspondence or full writer theorem has been established;
all 101 native effect obligations remain open. The native-body lowering must
avoid a second live Rust borrow and cannot mint a ghost permission or assert a
final/current frame. Conditional token-map reconstruction remains separate from
reconstructing the original executable array's future values.

The [current-runtime parent replay](parent-native-foundations.json) reproduces
17 expected outcomes on `676b99bf`: resource laws 18/0 and 12 distinct controls,
tracked-slice entry 2/0, executable entry/release/splitting 3/0, the second-borrow
E0499 rejection, and the native constructor/drop frame's 0/1 failure. That last
case is an unproved positive under inherited contracts, not a Rust counterexample.
This replay discharges no native effect or source-correspondence obligation.

The [source-derived fragment interpreter](../native_heap_lowering/REPORT.md)
checks a generated trace for the minimal constructor/drop graph and a generic
write-footprint frame in explicit authored semantics. Its metadata is a copyable
observation, not physical authority. Original-reference entry, native primitive
simulation and final-array observation adequacy remain unproved; publication
still rejects. These model proofs do not change the native obligation counts.

## Required source connection

Start from the actual unchanged local writer and resolve the monomorphized
Rust 1.99.1 IterMut<u8>/Iter<u8>/Zip path and target layout. Enumerate the actual
constructor, specialized next/indexed getters, relevant pointer operations,
constants and drop behavior. Retain typed operation/control-flow evidence,
source/instance fingerprints and a complete reachable-call inventory.

Names and hashes bind an identity; neither proves an effect. The checker must
validate the operations and transitions against a supported typed semantics,
rejecting unknown calls, writes, pointer origins, layouts and exceptional paths.
Do not interpret a hand-authored value-level trace as the native source.

The real specialized Zip advances its own index and uses indexed getters;
physical inner cursors need not advance. Logical projections must account for
that index. Identifying a spec projection with the raw inner field is insufficient.

## Ownership and proof obligations

The u8/128 case is sufficient initially; it retains every byte value and every
key length, including zero and lengths greater than 128. A checked certificate
must establish at least:

1. The original exclusive array borrow supplies all 128 owned slots, with the
   exact initial values and valid pointer origins/bounds. No permission may be
   invented from a duplicable spec observation.
2. A successful native indexed yield transfers exactly the corresponding slot
   and immutable key observation. Bounds, pointer arithmetic, alias separation,
   current/future values and eventual resolution must be justified.
3. All unissued slots remain owned through every actual source transition.
   The truncated logical pair sequence alone is not an ownership inventory.
4. A None transition preserves those slots, including empty-key behavior;
   native drop has the checked effect used by the certificate. Account for
   specialization and any side-effect flags rather than assuming cursor behavior.
5. Resolving all issued loans and releasing the retained tail reconstructs the
   original array borrow. This derives the unchanged suffix and output views;
   it is not an axiom stating the requested postcondition.
6. The actual source-to-certificate refinement and the value/ownership theorem
   are checked. An unrelated safe iterator model, finite native tests, a source
   hash allowlist or an assumed external contract is not that refinement.

The existing ordinary verifier/SMT and typed memory/borrow semantics remain the
checking boundary. Any new lowering/checking rule needs explicit soundness
obligations, narrow eligibility, independent review and false-certificate controls.
Do not silently add a trusted native resolution/frame fact or bypass ordinary
checking. Keep the original executable body as input; proof translation must
carry its correspondence rather than substituting an indexed algorithm.

## Staged implementation and acceptance

An initial standalone extractor/checker may live in
`source/tools/research/native_iterator_certificate/`. It must not install facts
in production. Its first milestone is executable typed-source extraction and
conservative rejection of unsupported effects. A subsequent milestone must
prove conservation/refinement obligations under normal checking. These are
partial infrastructure milestones, not a completed native bridge.

Tests must reject an omitted/changed tail write, wrong origin/index, lost slot,
incorrect Zip projection, unexamined helper/drop effect, layout/type mismatch,
forged source binding and incomplete/failed/resource-limited proof. Test source
variants independently of fingerprint mismatch so identity is not mistaken for
semantic checking.

Production integration is allowed only after the native correspondence and
full original writer theorem are established with the stated checks. Bind
certificates to source, type, layout, dependency and proof identities and preserve
failed-proof publication barriers. The parent owns policy/cache/schema changes.
Shared automation, cold/warm/import/adversarial replays, source fidelity and a
fresh independent review remain necessary for the overall active goal.
