# Ordinary-access MODEL review

**Finding for root: no correctness defect found in the exact nine-declaration
component at `8438803e`.** Fresh normal compilation, all nine closed assumption
reports, an exhaustive compiled-module inventory, and full recursive kernel
checking succeeded. This conclusion applies only to the conditional MODEL facts
below. Publication is false and the Infer goal remains incomplete.

The early scope milestone is commit `2cb4dd51`; `scope.json` records the exact
reviewed commit, source/freeze hashes and two-definition/seven-lemma inventory.
Every tracked file in `source/tools/research/model_non_atomic_access/` was checked
byte-for-byte against `8438803e`. No component, Rust, metadata, production,
baseline, compiler, vstd or shared-helper file was edited.

## Authentication and fresh checks

The component's `source/tools/research/model_non_atomic_access/results.json`
authenticates the completed
`/tmp/verus-infer-buffer-cycle-20261109/parent-non-atomic-access-006/proofs/receipt.json`
with SHA-256
`0d141bdad5ff51bffb3346f4639e437404c63f29fa0605a182b122911935a1a4`.
The review authenticated all **849 recorded input hashes**, all four recorded
source/object products, both object pins in `results.json`, the completed
assumption output for every declaration, and the completed kernel context.
Historical checkout paths in that receipt were treated as labels and mapped
into this checkout; they were never opened at their historical locations.

The successful private review is
`/tmp/ordinary-access-model-review-t3-9fdf220c-trial002/`.
`results.json` in this evidence directory records its compact receipts, product
and raw-log hashes. Raw sources, objects, logs, and full identity snapshots stay
private. The candidate is SHA-256
`203d94195fef1a08d55d2fa335e57f0ce03bec3bfd3fe8400ef07c0dd9777df1`;
the original `final-freeze.json` remained unchanged throughout.

The replay required both authenticated readiness and the successful baseline
receipt for foundation `115a618dd35ff8337112cf2081a7f5391d3973d5` under
`verus-belt-build-115a618d`: 523 pinned sources and 284 pinned objects, the
reviewed tool hashes, all three imported completed libraries' source/object and
receipt identities, and GMP manifest SHA-256
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
Only `rocq-toolchain/run-rocq` and its pinned normal `coqc`/`coqchk` were used.
The baseline was neither rebuilt nor changed.

Before each review attempt, the wrapper froze its own inputs and its
declaration-free `Inventory.v`; the unchanged component controller also guarded
the original final freeze. It ran normal `coqc -q`, individual `Print Assumptions`
for all nine candidate declarations, and recursive `coqchk -silent -o`. A second
normal compilation printed the entire candidate module and all nine types. A
second full recursive check included the candidate, assumption harness and
inventory module. No `-admit`, `-norec`, exclusions or unsafe flags were used.
All 872 guarded original review inputs, including receipts, sources, objects
and tools, and the separately guarded fresh sources and objects, were checked
unchanged through the final kernel check. Both fresh `access.vo` and
`Assumptions.vo` hashes exactly match the authenticated completed objects.

Both source and compiled-module inventories contain exactly two definitions
and seven lemmas. There are no added candidate instances, constructors,
obligations, helper declarations, axioms, primitives or trusted summaries.
All nine candidate assumption sets are empty. The recursive dependency context
contains only inherited functional extensionality and `Eq_rect_eq`; that context
is not being represented as axiom-free. `Print Module` renders opaque proved
lemmas as `Parameter`; their individual closed assumption reports and successful
kernel checks establish that these are checked theorems, not newly postulated
axioms.

## Operational and WP review

The following are direct source/proof observations from `candidates/access.v`
and the authenticated foundation, not negative Rust test outcomes.

| Candidate declaration | Verified scope |
| --- | --- |
| `silent_step` (line 8) | Exactly `@language.prim_step lrust_lang ... [] ... []`, with no observations or forks; it is not an invented transition relation. |
| `sc_read_one_step` (line 11) | One actual `ReadScS` lifted by `base_prim_step`, with arbitrary `RSt r` and unchanged state. |
| `na_read_two_steps` (line 19) | Two actual phases `ReadNa1S`/`ReadNa2S`; arbitrary initial `r`; intermediate `RSt (S r)`; insertion cancellation and the original lookup restore the entire original state. |
| `sc_write_one_step` (line 33) | One `WriteScS`, requiring `RSt 0`; stores `v`, returns poison and retains the other state component. |
| `na_write_two_steps` (line 43) | Two actual write phases requiring the same `RSt 0` premise, passing through `(WSt, old)`, then ending at exactly the SC final update. |
| `na_read_phase_changes` (line 60) | Lookup/injection and arithmetic prove that the required intermediate update is unequal to the original state, for every initial `r`. |
| `na_write_readers_blocked` (line 71) | Inversion excludes every head step of the fully evaluated ordinary write when the cell has `RSt (S r)`. |
| `cell_assignment_na` (line 76) | Explicit ordinary read, unchanged `byte_xor`, then ordinary write to the same `p`. |
| `wp_cell_assignment_na` (line 83) | Conditional framed WP with existing `p ↦ #a`, byte bounds for `a,b`, arbitrary `R`, and the existing `lrustGS` context. Returns poison with `p ↦ #(scalar_xor8 a b)`, the same `R` and byte range. |

I read the actual foundation `lang/lang.v:91` state definition,
`lang/lang.v:261`–`296` head rules and `lang/lang.v:635` language construction,
plus Iris `program_logic/ectx_language.v:127`/`:181`/`:323` and
`ectxi_language.v:163` that connect those head rules to `prim_step`.
The two primitive read/write results describe isolated finite sequences. They
do not quantify over arbitrary interleavings or equate SC and ordinary programs.
The same-cell final-update comparison for writes uses their common `RSt 0`
premise. All other heap entries and the Boolean state component are preserved.

I also read Iris `program_logic/language.v:224`: a `pure_step` requires every
actual step to retain its state. Together with the actual first-phase rule,
`na_read_phase_changes` therefore prevents inferring first-phase state identity
or purity from the isolated read's restored endpoint. It is a concrete
nonidentity lemma, not a newly proved general `¬ PureExec` theorem. The active
reader lemma is stated precisely for `head_step`; its literal address and value
operands have no inner redex that could bypass the unavailable write head rule.
No whole-assignment purity claim is justified or made.

The pinned `lang/notation.v:30` and `:46` expand `!` and `<-` to `Na1Ord`.
The WP proof explicitly binds `Read Na1Ord`, uses `wp_read_na`, applies the
unchanged `wp_scalar_xor8` to the unchanged `byte_xor`, and uses `wp_write_na`
on that same owned cell. The frame is retained by `iFrame`; expression return
is poison, while the cell receives the scalar XOR result. The independently
defined bits/Boolean-XOR/word-value scalar and its range law are imported from
the authenticated `native_xor.v:24`/`:63`/`:108`; no scalar or byte operation was
changed for this proof.

For credit accounting I read `lang/lifting.v:493`, `:530`–`:580`, `:638` and
`lang/heap.v:1612`–`:1660`. SC's public store rule requires `£1`; the ordinary
public rule requires only later cell ownership. Its guarded rule takes
`£(3*d)`, instantiated at `d=0` using `lc_zero`, not a positive credit created
from nothing. `heap_write_na` opens the `WSt` phase and exposes a closing
continuation requiring `£1`; the actual ordinary lifting proof receives
`credit` after phase one and passes it to `Hσclose` before phase two. The model's
existing WP step-credit mechanism, including `lrustGS_irisGS` in
`lang/lifting.v:22`, supplies this accounting. The candidate invents no credit
source and requests zero **external input store credit**. It neither proves a
minimal total budget nor identifies external credits with zero internal work.

## Failures and procedural caveats

The component's five older failed attempts remain ordinary proof development
and elaboration failures, exactly as recorded in its `development.json`. Their
diagnostics are not semantic controls, and they were not rerun.

This review's first fresh component replay itself passed normal compilation,
all nine closed assumption reports and recursive checking. Its supplementary
inventory compilation then failed because a parent-directory `-Q` mapping
overrode the candidate's nested namespace. That review-harness setup failure is
retained in the private `trial001` directory and compactly in `attempts.json`.
The wrapper was corrected to map a sibling inventory directory and the entire
review was rerun in fresh `trial002`; candidate sources and final freeze were
never changed. Failed setup is not semantic evidence.

At initial instruction discovery, `find .. -name AGENTS.md -print` was too broad
for the requested checkout boundary and was interrupted after returning no
output. This is a procedural deviation: it could enumerate directory metadata
outside this checkout; no file contents or semantic evidence were obtained.
Subsequent discovery stayed in this checkout or named ancestor instruction
files. The first scope commit also failed for missing local Git author
identity; retry used per-command `-c` identity only, with no global configuration
change. Neither administrative event is proof evidence.

## Limits and reproduction

No excluded live parent reader/loop replay, unfinished full-writer contents,
parent/sibling checkout contents, filesystem race/substitution experiment or
Rust test was used. No subdelegates, installs, shared builds, pushes or PRs were
performed. The only repository writes are this assigned new evidence directory.

This review supplies **no source/compiler/MIR/provenance/retag/C/F simulation**,
native input interpretation, ancestor-history interpretation, termination or
concurrent-program equivalence. Initialized MODEL resources do not acquire a
real Rust input or provenance meaning from these lemmas. Cell permission is an
explicit internal MODEL premise to be derived by future setup; it is not an
added requirement for the frozen Rust application. The original Rust body and
domain, metadata and default production behavior are unchanged. Native
correspondences stay **0**, all **44+3/101** obligations remain open;
canonical **6/0** and full HMAC **1/1** are inherited and were not rerun.
Publication remains false and the goal incomplete.

From an otherwise prepared checkout, use a new nonexistent private directory:

```sh
python3 -B evidence/ordinary-access-model-review/review.py \
  --output /tmp/ordinary-access-model-review-NEW
```

This authenticates the named completed artifacts, uses the unchanged
`source/tools/research/model_non_atomic_access/final-freeze.json`, and fails
closed while retaining any setup/development failure receipt privately.
