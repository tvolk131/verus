# Independent pair/shared-reader MODEL review

Recommendation: accept the exact two MODEL libraries. Fresh normal compilation,
full recursive kernel checking and final authentication passed, with replay
identities recorded in `results.json`. No blocking mathematical or
controller finding was identified in the reviewed versions. Acceptance covers
pair setup18 at `27c0422f` and shared reader19 at `bef470ad` individually.

The early typed milestone is commit `2452f43d`. Only this new evidence directory
was changed. Sources, controllers, shared support, foundation, Rust bodies and
domains were left untouched. The review used its own prepared checkout/history
and the specifically pinned completed libraries; historical checkout paths in
receipts were treated as labels. No live parent/sibling checkout or root replay
was inspected. There were no subdelegates, installations, Rust builds, excluded
filesystem experiments, substitutions, pushes or PRs.

## Authentication and fresh verification

The exact completed receipts authenticated against the user-provided and
committed identities are:

- Pair: `/tmp/verus-infer-buffer-cycle-20261109/pair-input-final023/receipt.json`,
  SHA256 `f57ba4ea66e1722586aafb1b92c4e9a6946183ae00fe974176baae620600ed40`.
- Shared: `/tmp/shared-byte-read-trial-022/receipt.json`, SHA256
  `d434bc73bacd2335313b73001436696cf8a305930fa9fa30eb1c79b014311b69`.

`review.py prepare` first compared every tracked file in each target directory
with its exact requested implementation commit. It authenticated all frozen
inputs, every returned `.v` and `.vo`, the generated assumption sources/objects,
and the complete individual assumptions against committed results. Its
independent comment-aware source inventory enumerates 18 pair declarations
(5 definitions, 13 proved lemmas/theorems) and 19 reader declarations
(3 definitions, 3 proved instances, 13 proved lemmas). Constructors, projections
and generated obligations: zero. `Existing Instance Outer` merely registers an
existing section parameter and introduces no constant. `Typeclasses Opaque`
affects elaboration, not kernel verification. Generated Assumptions modules also
contain the controllers' trivial `Goal True` boundary markers, authenticated by
their complete retained source and object hashes and checked recursively.

`pretrial.json` sealed this evidence before the first normal compilation trial.
Both original final freezes were reused unchanged. Fresh outputs, raw logs,
objects and auxiliary files are private at
`/tmp/pair-shared-reader-review-xDbehgWZ/{pair,shared}`. The unmodified controllers
run normal `coqc -q` on every candidate module and the generated complete
assumptions inventory, then full recursive `coqchk -silent -o`, including the
Assumptions module, with no admission/exclusion flags.

Both replays returned zero for every compile, inventory and kernel command.
All 16 source/object files (8 sources and 8 objects) are byte-identical to the
completed worker artifacts, including both generated Assumptions modules.
Fresh pair receipt SHA256:
`9c33aecb66467cb929a5ab87ca39053d6fa8af66ac593b3f0a0f272f93529b1d`.
Fresh shared receipt SHA256:
`5cae394e8b1a66983f68fe375b103643e19ff4a39f7e7ee8b955e63ed665fb69`.
Both full recursive kernel summary SHA256:
`960298c45af69cab79feb24986db3c1625fa949b59775c1fd8c11bf410bdb526`.
Pair/shared individual-assumption map SHA256, respectively:
`aa7d4fa5a47e7361125889bc803bfe22a85566b15518c19f670790e266644d31`,
`2abab32814951af7737dcb324c7aad1e79dc7b6667b7f1090bfd11416163f7d9`.
Map hashes use sorted-key compact JSON; raw assumption-log hashes are separately
recorded in `results.json`. Pair guarded 878 input artifacts; shared guarded866.

No normal Coq trial failed. Before the pretrial freeze, initial review-script
preparation used the wrong receipt field name (`returncode` instead of
`exit_code`) and was corrected. The early commit also needed command-local Git
author settings. Neither setup event is counted as semantic or proof evidence.

The controllers require the pinned readiness gate **and** successful durable
baseline receipt, foundation115a618d, all 523 source hashes and 284 object hashes,
the exact runner/prover/opam identities, and GMP manifest SHA256
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
Readiness SHA256 is
`d5462c63b81a87382fca07b17085cbc006a784e6a96de167aad5103f3b626007`;
durable baseline receipt SHA256 is
`f3490fd0b0eae837194503b04fb95bd0a2881938c0db26362875e07d2306e13e`.
They also require the shared support/parser files in the exact frozen input
set, and authenticate prerequisite receipts and each dependency source/object.
Pair's historical heap replay and shared's GMP-positive heap replay contain
identical pinned heap proof objects; each receipt is separately authenticated.
No shared build is performed. The final proof-success flag follows all guards.

Per-constant assumptions are retained, not replaced by a union: pair has nine
closed constants, three using functional extensionality, and six using functional
extensionality plus inherited `Eq_rect_eq`; reader has two closed constants and
seventeen using functional extensionality. Both recursive dependency summaries
also report inherited `Eq_rect_eq`. No new principle, axiom, primitive, unsafe
mode or admission was introduced. `statement-inventory.json` and `results.json`
retain the full individual maps; `results.json` pins their canonical hashes,
fresh receipts, assumption stdout, recursive summaries, sources and objects.

## Resource and statement audit

- **One heap and typing context:** pair `setup.v:96` fixes arbitrary resource
  postcondition `Φ` in `Outer`. At line117 exactly one `TypeG Σ Outer ...` is
  constructed. Both reference constructors, Share and callback use it. The
  public wrapper at line160 uses an explicit WP instance and `subst Outer`,
  and eliminates actual final `heap_ctx σ` plus `Φ Outer v`. There is no assumed
  final-memory observation or extra physical-permission input at public entry.
- **Real checked Share:** pair `setup.v:62` invokes baseline
  `type_share_instr` at line81. The unique key is constructed internally; the
  resulting shared tctx is fixed to the exact location/bytes via actual prophecy
  observation satisfaction. It is not a new ownership primitive or trusted
  read summary. The pad is constructed after Share with its retained ticket.
- **Linear separation:** pair `setup.v:105` returns distinct ordinary Lpad/Lkey
  components, each constructed once from its fresh lifetime allocation. The
  recombination lemma at line28 is a separating equivalence, not duplication.
  `shared_key_aliases` duplicates only the persistent shared view/time receipt.
- **Exact domains and frame:** `raw_pair_payload` separates the two exact raw
  intervals and `h ∖ (padmap ∪ keymap)`. `controls.v:10` characterizes disjointness
  solely by actually accessed addresses. No different-block/base-only premise
  exists. Arbitrary lengths cover pads10/128 and keys0/below/equal/above128.
  Same-block gap and arbitrary strict-interior controls are proved, including
  arbitrary prefix/middle/suffix and disjoint outside map. Empty key `q=p` is
  an executed adequate setup control at line81, with no cell access for the key.
  The returned outside resources yield an actual final outside observation;
  neither still-borrowed input interval is falsely claimed recovered.
- **Physical indexed SC read:** reader `shared_read.v:12` is exactly existing
  shared `ty_own`. `bytes_index_guard` at line29 derives the cell at `q+ₗi`
  from the whole physical bytes guard; the induction preserves the exact index.
  `shared_key_read` at line66 requires `i<n`, finds the matching list entry,
  derives its range from the prepared bytes type, composes genuine E/L lifetime
  aliveness with the physical guard, weakens its mask and invokes baseline
  `wp_read_sc_guarded_cells` at line89. The expression is precisely one
  `Read ScOrd #(q+ₗi)`. No exclusive key vector, caller guard, unconditional first
  cell access or extra disjointness premise is assumed.
- **Frame, alias and costs:** arbitrary `Frame : iProp Σ` is returned unchanged
  with ordinary L and the same capability/view. `shared_read_same_alias` checks
  a same-key alias in Frame. Generic finite lengths, list lookup and the explicit
  129/index128 specialization are covered; empty keys have no valid index.
  Ghost-guard preparation costs `g+1` and no program step; reusable physical read
  costs `g+1`; direct view read splits exactly `2*(g+1)` into those two budgets.
  These are sufficient budgets, without a minimality claim.
- **Session restoration and closure:** `session.v:35` prepares from ordinary
  E/L, uses baseline `llctx_interp_make_guarded`, and chooses both abstract R/G
  as separate copies of `fractional.half γ`, not unequal token types. The G
  copy guards L for mutable access while R supports the physical shared read.
  Every read returns R, capability and arbitrary Frame. Closing uses both halves
  at line73 to restore ordinary L; no half is returned. The persistent reader
  still has an explicit R premise at lines13/25. Its persistence supplies no
  post-closure R, so closure does not leave a usable reader through this API.
  The existential factory is not permission to choose `R=True`. Timelessness
  is not persistence. No client ownership/guard/context/credit precondition is
  added beyond the exact advertised MODEL interfaces.
- **Actual same-heap observation:** `observation.v:12` takes genuine `heap_ctx σ`
  and calls baseline `atomic_read` on its authoritative non-atomic map at line37.
  It restores the same σ, L, capability and Frame, and concludes an actual lookup
  `σ.1 !! (q+ₗi) = Some (RSt r,#z)` with existential r and the exact ranged list
  entry. It neither assumes r=0 nor manufactures an exclusive vector.

The supporting baseline rules were read from the authenticated build:
`typing/borrow.v:111`, `typing/shr_bor.v:13`,
`typing/lft_contexts.v:113,358`, `lang/lifting.v:423`,
`util/non_atomic_cell_map.v:1288`, and `guarding/lib/fractional.v`.

## Actions and limits for root

No implementation fix is requested. When composing the libraries, qualify both
`ModelPairInput.setup.shared_key_view` and
`ModelSharedByteRead.shared_read.shared_key_view`; their short names denote
different interfaces. The merged library commits do not yet contain a joint
consumer, so no existing combined theorem is certified here.

Eliminating the pair view supplies **existential synchronized d** and a receipt,
`∃ d, ⧖d ∗ ty_own ... d d ...`. The reader accepts that view at `g=d`, but its
preparation/read budget then depends on this existential index. Nothing in the
reviewed public pair theorem fixes it to 0 or 1, or proves a fixed combined
budget from the initial callback credit k. This is an integration boundary, not
a defect in either advertised individual statement. A new implementation that
normalizes the handoff needs its own exact freeze, proof and review. This review
does not borrow results from that ongoing work.

Acceptance is exactly these MODEL libraries. No combined XOR loop, public
combined application theorem, original-parent final recovery, Rust execution,
provenance, retagging, allocation history, source typing or C-F correspondence
is proved here. Actual native correspondences remain0; all44 minimal +3 phase
obligations and101 writer effects remain open. Full frozen HMAC1/1 is inherited
and was not rerun or strengthened. Publication=false; the overall goal remains
incomplete.

Replay from the repository root with
`python3 -B evidence/pair-shared-reader-review/replay.py`. This allocates a new
private output root, checks the frozen review inputs, runs both unchanged
controllers and authenticates the fresh outputs. For the already completed
review outputs, rerun authentication only with
`python3 -B evidence/pair-shared-reader-review/review.py finish --private /tmp/pair-shared-reader-review-xDbehgWZ`.
