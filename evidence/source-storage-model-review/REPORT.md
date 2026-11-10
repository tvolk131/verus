# Independent storage MODEL review

Accepted at **MODEL-only scope**, with no blocking finding. Base `bc2a0b37`;
early typed scope/interface commit `4b8998f4`. Reviewed delivered
`e77da206`/`ade18e67` and current replay policy `4aefbe9d`/`58a779e2`.
Only this new evidence directory changes. No native acceptance, publication,
full-goal completion, engine/vstd/global edits, installation, Rust rebuild,
subdelegation, push or PR. Historical checkout/source-span paths were treated
as labels; no live parent/sibling checkout was inspected.

The unchanged domain remains every-u64 canonical encoding into arbitrary
initialized `[u8;10]`, and the independently frozen exact HMAC pad-XOR body
with arbitrary initialized `[u8;128]`, all key lengths (including0 and >128),
XOR prefix AND unchanged suffix. Same actual checked automation, normal
checking/fidelity and combined independent acceptance are still required.
Canonical6/0 and full HMAC1/1 unproved are inherited, not rerun. Native
correspondences0; all44 minimal operations+3 phases and101 writer effects
remain open. Publication=false; goal_complete=false.

## Authentication and replay

`hashes.json` authenticates REPORT, results, pins, README, integration-results,
integration freeze, candidates, controller and scope helper to their delivered
Git objects and frozen SHA256 values. All mathematical candidates are unchanged.
The historical final freeze equals trial007; its declaration lists equal the
current freeze. The only frozen-input changes are the controller and replacing
the historical whole-TARGET pin with the frozen goal-scope helper.

Historical worker receipt (authenticated):
`/tmp/native-source-storage-e77da206-trial007/receipt.json`, SHA256
`184b009ecbfa6a343e5183dce5e2df638bf906592c25b979bab5bd877a222b64`.
Parent proof receipt:
`/tmp/verus-infer-buffer-cycle-20261109/parent-storage_scope_fix-001/proofs/receipt.json`,
SHA256 `47f3ebf95acf6e15d3a6b17deabb0103902d48ac62d05a5a7e2fa8079ff8cd1a`.
Its manager receipt also authenticates the same proof receipt; SHA256
`03f7e40a44dbebf5f66f7a38e144118902e579686485dab77ee63b62fe438f58`.

Fresh private receipts:

- `/tmp/native-source-storage-model-review-bc2a0b37-001/receipt.json`
  (`55d1395eb57159e6afd0f81d2cf9f6a90b8e24b67e6016be05f020995ee8cb64`):
  independent preflight/replay/comparison gates.
- `/tmp/native-source-storage-model-review-bc2a0b37-001/replay/receipt.json`
  (`2799d7b04baa46372cebf15dd2bff533788d69658d535f1f749e5262c098f252`):
  all eight normal Coq commands succeeded.
- `/tmp/native-source-storage-model-review-bc2a0b37-001/preflight.json`:
  source/controller/support/tools/dependencies pinned before proof trials;
  1,983 guarded inputs, with full initial metadata and content hashes.
- `/tmp/native-source-storage-model-review-bc2a0b37-001/source-audit.json`:
  separate guarded identity audit of all44 extracted graph rows and20 storage
  sites, including the incoming local1 type and root sizes16/16/8. Every
  supported storage site targets a local other than0/1. Identity is not simulation.

The preflight requires authentic baseline-ready AND successful named
`rocq-toolchain/durable-build-2/receipt.json` before compilation. It verifies
VerusBelt `115a618dd35ff8337112cf2081a7f5391d3973d5`,523 sources/284 objects,
Coq8.20.0/OCaml4.14.2 private tools, runner and loadpaths. GMP manifest SHA256
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`
and both archives match. All1,121 Coq source/object/plugin dependencies match map
`14c4b43b1eaa1ee216c2aedd34303a3d28a536682a8efdc1758355481bf457fa`.
Eight official Rust source files match exact rustc commit
`58cf8f9c35b355199727627d07ae34a4e2846d67` and pinned upstream URLs/hashes.
Those implementation sources guide interpretation; no Rust semantics theorem
was imported. Shared files remained read-only; compilation used private copies.

## Objects, declarations and trust

Normal coqc compiled Interface/Graph/Composition, then Storage/Sites/Prefix and
the declaration-free inspection module. Full recursive `coqchk -silent -o`
checked all three new modules and the inspection module and recursively their
imports, with no exclusions. Set is predicative, rewrite rules disallowed,
zero axioms, no type-in-type, unsafe fixpoints or assumed positivity.

All12 mathematical `.v`/`.vo` hashes match BOTH worker and parent: the three
storage modules and three constructor dependency modules. Inspection `.v`/`.vo`
also match. The142 individually checked declarations comprise Storage103,
Sites6, Prefix33:75 explicit declarations plus67 constructors/eliminators/
projections. The inventory equals Coq Print Module discovery, the frozen source
inventory, both receipts, results and TRUST_INVENTORY. Each Print Assumptions
output is exactly `Closed under the global context`; zero Program obligations.
`ASSUMPTIONS.json` lists every name, empty assumptions and statement hash;
full Check/Print outputs remain in the named replay's `proofs/inventory/`.
No new mathematical proof or axiom was authored in this review.

The raw controller retains input, private-source, compiled-object, inventory and
final guards. Acceptance uses explicit `require` exceptions, works with Python
`-O`, and uses no runtime assert. There were no failed review trials. Coq Redirect
emits output-directory warnings during inspection; all command exit codes are0.
Worker development failures remain historical development evidence, not semantic
negatives or Rust safety results. Raw logs were not committed.

## Mathematical premises and interpretation boundary

- `root_entry` fixes locals0..4: return local0 Immediate Uninit, local1 exactly
  `RefValue r`, locals2/3/4 Dead, control root bb0[0]. H, its supplied value,
  caller cells, r (including allocation/offset/tag), private stack, fresh id and
  prior trace are otherwise arbitrary. No byte-zero, base-offset, empty-history,
  ownership, permission, defined-execution or no_unwind premise appears.
- `first_three_steps`/`prefix_execution` give existence of all three continuing
  steps. `prefix_all_enabled_outcomes` separately quantifies over EVERY outcome
  of each of those exact states/events; each must be the indicated Continue.
  This follows the deterministic operational_step definition, not a selection of
  successful native paths. No theorem covers all future enabled Rust outcomes.
- `prefix_exact_control/locals` reach bb0[3] with locals2/3/4 live uninitialized,
  local0 uninitialized and actual evaluated local1 still exactly r. Caller cells
  and original operand are unchanged, including arbitrary signed interior
  offsets. `prefix_original_bytes` additionally requires only the stated initial
  nth_error-to-InitByte lookup for A; it preserves those lookups for any list A.
  It does not require or prove native validity or a byte-range input relation.
- Prefix history retains arbitrary incoming H while appending local-write hooks
  2/3/4 to the trace; total model history `(H,trace)` changes. The prefix leaves
  private storage unchanged because these initial locals are Dead. This is not
  evidence of Miri hook success/effects or of every lawful Rust history.
- Generic live/dead/execute frame laws condition on a returned Some/Advanced
  successor. All such successors preserve caller cells, stored original operand
  and H. Non-target locals are preserved; the20 supported sites never target
  local1. Fault/Unsupported results contain no successor state, and cannot be
  counted as a normal identity step. At the prefix sites all enabled outcomes
  are additionally proved continuing, so progress is not conditional there.
- Repeated StorageLive resets immediate contents or, given a live indirect
  backing block, resets and retires OLD private storage. Repeated StorageDead of
  an already dead non-return local is pointwise observation-preserving. Invalid
  indirect backing can fault. Caller/Private are disjoint constructors; retired
  blocks keep bytes/alignment in a dead tombstone and become inaccessible.
  Neither StorageDead nor retirement executes referent Drop or resolves loans.
- `materialize_uninit` requires the target to be Immediate Uninit and the chosen
  fresh private id absent. It explicitly allocates UninitByte cells; this is not
  zero initialization or a general initialized spill/address-taking interpreter.
  Abstract model size/alignment/id fields do not establish lawful Rust layouts.
- Root StorageDead progress at bb0[5],bb1[0],bb1[2] assumes exact current control
  and an immediate target local. Backing release laws additionally assume a
  matching live indirect block. None proves reachability past the unsupported
  reborrow/unsize/call. At bb0[3] execution is explicitly Unsupported.
  All other unimplemented reference/raw/address/retag/cleanup operations remain
  unsupported. operational_step encodes coverage/control failure as
  InvalidControl and backing failure as InvalidStorage; these are MODEL
  diagnoses, not a judgment of Rust UB.

The existing source_semantics record is a signature; the partial prefix supplies
compatible carriers/projections/steps, not an actual Rust instance or a
Composition.source_entry_certificate. Any actual interpretation must independently
map EVERY lawful original input and real local/control state to this model,
preserve the same evaluated incoming reference, its signed-offset caller cells,
provenance/ancestor/protector histories, and relate immediate/indirect storage
using an injective private allocation mapping disjoint from caller storage.
It must account for hook/allocation/deallocation effects and failures, and prove
simulation and caller observation over ALL enabled real outcomes. Separate
reborrow, activation, unsize, raw/reference validity, retag, cleanup/control,
ordinary native load/store, original-reference C/F and final observation laws
remain open. Neither the closed model proof nor source identity discharges any
of these. No assumed source safety, permissions, no_unwind or C/F bridge was added.

## Current goal-scope replay policy

The frozen helper hashes exactly the bytes from `## Required property and domain`
through the completion criteria, before `## Coordination`. The required pin is
`11014c3ac73ddf13c464769f5d0d1cbd24598ab52777718d31872c70b3b8f7b0`.
It rejects missing/duplicate/inverted markers, changed requirements and incorrect
freeze/snapshot hashes. The entire current TARGET is separately retained in the
initial ArtifactGuard and checked before/after commands and at completion.
Text outside the scoped section is guarded for run stability, not historically
frozen by that section hash. In particular, this helper is not independent proof
of the earlier HMAC freeze or authorization to change its domain. Documentary
progress changes outside the section need not invalidate mathematical replay.
Fourteen pure in-memory tests checked this policy and kernel-summary rejection;
no source/test filesystem mutation or race/substitution experiment was performed.

Reproduce with fresh output paths (no server, extraction or installation):

```sh
python3 -B -O evidence/source-storage-model-review/review.py \
  --output /tmp/native-source-storage-model-review-UNUSED
python3 -B -O evidence/source-storage-model-review/source_audit.py \
  --output /tmp/native-source-storage-model-review-UNUSED/source-audit.json
```

`commands.json` retains exact executed commands/cwds and exit codes;
`results.json`, `hashes.json` and `ASSUMPTIONS.json` provide compact acceptance,
identity and complete declaration evidence. The integration lead retains all
native-status and publication decisions.
