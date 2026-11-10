# Checked pair setup to reusable reader handoff

All 16 declarations normally compile and pass full recursive `coqchk` without
exclusion/admission flags. `results.json` records the final receipt, source,
object, assumptions and kernel-log hashes. `development.json` retains identities
of every pretrial freeze and failed development receipt; failures are excluded
as semantic negatives. The early typed milestone is commit `ab8471ae` (trial
002); the constructive core milestone is `4316f273` (trial 010). Final sources
and all verification controllers change only this directory.

`ModelPairInput.setup.shared_key_view` remains exactly its existential tctx.
`handoff.pair_view_raw_exact` converts it exactly to the raw shared predicate at
fresh synchronized d/g and proves no bound on that witness. The successful
handoff instead constructs stronger information before existential packaging.

`fixed.model_key_share_fixed` specializes the constructive core of the existing
`type_share_instr`: it consumes `model_reference`'s internally allocated unique
key and invokes actual `resolve_uniq_body` at the retained payload d=0,g=0.
The returned genuine lifetime guard plus the unique reference's base guard
construct actual `ty_own (&shr{κ} (bytes_ty keylen))` at d=1,g=0. The byte range
used to construct its persistent component is the original input range
hypothesis, not a new application condition. The proof executes actual `Share`
(the existing `Skip;; Skip` expression) using normal WP steps. It does not
invoke the black-box `type_share_instr` theorem, postulate a replacement typed
rule, recover an owner from a shared alias, or assume a depth/credit bound.
`fixed.fixed_view_pair_context` also packages the exact original pair tctx with
witness 1/1, preserving the separate persistent raw 1/0 view.

`setup.raw_pair_reader_context_wp` derives both references from `raw_pair_payload`
and `bytes_reference_construct`, retaining original setup `Skip;; Skip;; Share;;
e`. It allocates one TypeG over the same Outer and two nonempty lifetimes. Its
initial model budget is `2+k`: one credit for lifetime-context allocation, one
for reader preparation, exactly k retained for the callback. The first setup
Skip supplies cumulative tickets and the persistent time receipt for reference
construction; the second supplies the key ticket. The existing resolution
lemma accounts for its timed credit generation from the constructed unique
body. No external receipt or credit source is introduced. The public
`setup.raw_pair_reader_observed` supplies `2+k` inside initialized observation
adequacy, so this change adds no credit, owner, guard, points-to or domain premise
to the original application.

The callback gets padRef/Lpad, the exact outside fragments, exact pair tctx,
fixed Cap and persistent key_reader, Timeless linear R/G, G's guard of Lkey,
and the R/G closure to ordinary Lkey. One indexed read costs one credit. The
parent can thread `R * loopcredits` as Rin, keep G and closure outside the mutable
pad callback, and join returned R/G afterwards. No parent Rin/Rout interface or
loop implementation changes here. Full caller-selected resource-valued Φ stays
in the same Outer.

`whole_key.whole_key_heap_observe` accepts genuine heap_ctx sigma, ordinary Lkey,
Cap and arbitrary separating Frame. With explicitly reserved `(g+1)*keylen`
credits it returns the identical heap_ctx sigma, Lkey, Cap and Frame, proving
for every original `key !! i = Some z` that sigma's actual cell at q+i is
`Some (RSt r, #z)` for some r, and z is a byte. It needs neither exclusive key
vector nor zero read-state counters. It is a fancy-update observation, with no
program Read execution. Its prefix induction has a no-cell-access zero case;
`controls.whole_key_empty_no_access` derives the empty case directly with no
credits or cell observer. Use it after joining R/G to Lkey. At the constructed
g=0, a full key observation reserves exactly keylen credits as a sufficient
budget; no minimality claim is made.

Controls cover exact list-index reads, duplicated persistent shared aliases,
empty key q=p, disjoint interiors of one allocation (p=base+1 and
q=base+1+padlen+gap), and an actual first-byte SC read supplied only by the raw
initial heap. The latter keeps G outside the read and checks R/G restoration
afterwards. Generic setup supports every finite key/pad length, empty pad and
key, arbitrary valid disjoint interiors and different allocations; it imposes
no allocation-base or nonempty-key restriction.

The controller authenticates completed PairInput18, SharedByteRead19 and every
exact pinned transitive receipt/source/object, plus baseline readiness and its
successful durable receipt. Foundation commit is
`115a618dd35ff8337112cf2081a7f5391d3973d5`: 523 sources, 284 objects; GMP manifest
SHA256 `332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
All retained inputs, compiler/runner identities and generated proof artifacts
are guarded through recursive checking. The final freeze predates compilation.
Every declaration and instance is inventoried; the only existing instance
registration is contextual Outer. No new inductives, records, constructors or
obligations occur. Only inherited FE/Eq_rect_eq appear in the full kernel
closure; no new axioms, admissions, summaries, primitives or unsafe modes.
Raw logs and objects stay in private `/tmp/model-pair-reader-trial-*` directories.

Reproduce with a fresh output directory:

```
python3 -B source/tools/research/model_pair_reader/check.py \
  --freeze source/tools/research/model_pair_reader/final-freeze.json \
  --output /tmp/model-pair-reader-replay-UNUSED
```

The handoff depth and credit claims are proved, with no remaining first sequent
for a missing bound. Parent mutable-loop integration and independent review
remain outstanding. This is MODEL initialization and observation only, not a
Rust execution/provenance/retag/C-F proof. Rust bodies and domains are untouched;
actualnative=0; all 44 minimal, 3 phase and 101 writer effects remain open; full
HMAC stays inherited 1/1 (not rerun); publication=false; goal_complete=false.
