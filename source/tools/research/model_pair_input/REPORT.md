# Raw model heap to mutable pad and actual shared key

The implementation constructs both references from exact initialized raw-map
fragments in one fixed `Outer : lrustGS Σ` and one internally allocated `TypeG`.
It executes the existing checked `Share` instruction on the key. The handoff
provides `model_reference pad p tid κpad`, the actual shared-reference tctx at
`((q, repeat [] (length key)), bytes_of_list key)`, separate ordinary `Lpad` and
`Lkey`, the invariant context, setup contexts, remaining callback credits, and
exact `initial_fragments (h ∖ (byte_map p pad ∪ byte_map q key))`.

The early precise interface is commit `8ee89c3b`; normally compiled source and
controller were handed off at `16b332ab`. Final validation identities are in
`results.json`, `final-freeze.json`, and the named private receipt.

## Resource and operational contract

`raw_pair_model_context_wp` preserves arbitrary resource-valued Φ in Outer.
`raw_pair_model_observed` initializes the finite model heap and delivers the
setup resources internally, then eliminates actual post-resources with the
observer in that same Outer. Its WP instance is explicit and its proof uses
`subst Outer`. No owner, PointsTo, extra guard or shared key premise is an input
to the public initialized-heap theorem.

The only physical-domain restriction beyond byte ranges/exact lookups is exact
interval disjointness. `byte_map_disjoint_iff` characterizes it by inequalities
between accessed cell addresses. Thus disjoint strict interiors can use the
same block; an empty key imposes no cell restriction, including q=p. Neither
pad nor key lengths are specialized: arbitrary lists cover pads10/128 and keys
0, below/equal/above128.

The setup program is `Skip;; Skip;; Share;; e`. Its budget is £(1+k): £1 for
`llft_alloc`, and £k delivered untouched to the continuation. First Skip yields
two cumulative tickets: one retained for pad construction, one converted with
`⧖0` to persistent `⧖1`. Second Skip yields a key-construction ticket and a named
unused affine `Spare`. Key construction uses `bytes_reference_construct`; the
unique key is converted to a tctx at depth1 by checked ghost-depth monotonicity.
`type_share_instr` supplies its own operational steps, time receipts, tickets
and credits through `typed_instr_of_skip`. Prophecy validity forces the returned
shared tctx's view to exactly the input location and byte list. Pad construction
uses the retained first ticket after Share. Its exact `model_reference` then
passes directly to the continuation.

Two lifetimes are independently allocated, and each ordinary linear handle is
passed once. `pair_lifetimes_recombine` proves their ordinary context
recombination. Opening the pad can retain Lkey outside the pad guard.
`shared_key_view_own` exposes only ordinary shared `ty_own` and its persistent
time receipt at an existential depth. `shared_key_aliases` proves duplication of
the shared view, with no duplication of either linear lifetime context and no
client-exclusive key payload.

## Validation and replay

The final frozen candidate has 18 declarations: five definitions and thirteen
proved lemmas/theorems. The controller inventories every declaration, prints its
assumptions, rejects new Program definitions/obligations and unchecked constructs,
and runs normal `coqc` followed by full recursive `coqchk -silent -o` without
exclusions. Generated obligations: zero. There are no added axioms or admissions.
Inherited assumptions are reported per constant; the permitted existing library
principles are functional extensionality and `Eq_rect_eq`.

The replay authenticates the mandatory readiness gate and successful baseline
receipt, all523 foundation sources and284 objects, prover/runner identities,
GMP manifest `332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`,
and all prerequisite source/object/receipt identities against committed records.
Dependencies are exactly NativeHeapInitialization via
`mutable_buffer_goal/parent-heap-initialization.json`, NativeSemanticBridge
`semantic-guard-validation-1efbd004/proofs/proofs`, ModelInputContext
`parent-model-input-94b87742/proofs/proofs`, ModelStateObservation trial017, and
ModelInputObservation trial005. Historical paths in receipts are treated as
labels; prerequisite artifacts are resolved only in the explicitly pinned /tmp
libraries. No parent or sibling checkout or shared mutable build is used.
All compiler products, auxiliary files and raw logs stay in private /tmp paths.

```sh
python3 -B source/tools/research/model_pair_input/check.py \
  --freeze source/tools/research/model_pair_input/final-freeze.json \
  --output /tmp/verus-pair-input-replay-UNUSED
```

The output directory must not already exist. `development.json` compactly retains
failed source freezes, receipt identities and diagnostics. Those failures are
proof-development/setup evidence, never semantic rejection. Trial022 checked all
11 core declarations recursively. Final023 also includes all seven controls.
The control executions observe the actual exact outside frame using returned
fragments and the initialized final heap authority. The strict-interior theorem
uses arbitrary nonempty prefix/middle/suffix, arbitrary byte lists and an
arbitrary disjoint outside raw map.

## Limits

This is fresh MODEL initialization, not a Rust incoming-reference, lifetime,
provenance, allocation or ancestor-history interpretation. Shared-read derivation
is independent work and is not imported here. Root remains responsible for the
changing pad's child loan, original-parent recovery and final observer.
The control observer does not claim final physical observations for either
still-borrowed input region; no invented finalheap facts or two-region adapter
are installed.

Actual Rust/native correspondences remain ZERO. All44 minimal simulations,
3 phase obligations and101 writer effects remain open. The established canonical6/0 result is unchanged; its shared observer was causally
redundant. The established full frozen HMAC prefix+suffix result remains1/1 and
was not rerun or extended here.
This MODEL work does not close the native goal. Publication=false.

## Final receipt

Final023 passed normal compilation, all18 declaration assumption inventories,
and recursive kernel checking. Receipt: `/tmp/verus-infer-buffer-cycle-20261109/pair-input-final023/receipt.json`.

Receipt SHA256: `f57ba4ea66e1722586aafb1b92c4e9a6946183ae00fe974176baae620600ed40`.

Freeze SHA256: `20a40977a61680fb1eaec96fb4ddcda9c39a817fec82a7cf3f1bbada1bad4c83`.

- `Assumptions.vo`: `c8d60e9293cf017cfb756f015408d8da083d5940789d94855eb6327e50c628cc`
- `controls.vo`: `46729ceb85fbaeab045cbe1f15ad57cc37239f714ae845295574c701a7e6504b`
- `setup.vo`: `71e173b0fd36ea86a12325f7c0de2d77bd526fadaa6f88afa6ab7ba6e3d3bd95`
