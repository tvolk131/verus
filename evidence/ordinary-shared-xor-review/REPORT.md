# Ordinary shared XOR MODEL review

Finding for root: **no correctness defect found in the exact 21-declaration
component at `212044b1551d3e14d9010c063bcdd73af78c71cf`**. The reviewed scope is
`ModelOrdinarySharedXor`, including its fixed pair-session adapter. This finding
requires the successful fresh receipt recorded in `results.json`; it does not
claim the initialized writer/final-observer composition. Native correspondence
remains 0, 44+3/101 obligations remain open, publication is false, and the goal
is incomplete.

The early typed interface and review controller were committed in `990ad032`;
the corrected wrapper is committed in `271f97c8`.
All 14 tracked subject files were compared byte-for-byte with the exact target
commit, including HANDOFF, INTERFACE, results, controllers, candidates, dependency
pins and final freeze. Every file in the unchanged final freeze was also checked
against that commit. Historical checkout paths in receipts were treated as
labels mapped into this prepared checkout, never opened at their old locations.
Only this assigned new evidence directory changed in the repository.

## Authentication and fresh validation

The completed worker receipt is
`/tmp/model-ordinary-shared-xor-trial-008/receipt.json`, SHA256
`79b5452cb7e3dee9cf69ea6582eda68e6fc5ec3c31ab1e37dc6ac38c563b75ab`.
The review authenticated all 2,212 recorded input identities, all ten source/object
products and every recorded raw-log hash. The worker pretrial freeze is identical
to the committed final freeze, SHA256
`9508a7081f3191aef69c82ca90030a294cce7105cb0b7910572da9511c05a6c9`.
The three implementation source hashes are recorded alongside the two generated
inventory source hashes and all fresh object hashes in `results.json`.

The review wrapper freezes its own inputs before each fresh trial, invokes the
unchanged pinned controller with the unchanged final freeze, and retains durable
private failure/success receipts. It opens no historical checkout path from an
imported receipt. The controller checks exact receipt/source/object inventories
for all 13 imported library loadpaths, including the already accepted ordinary
primitive component. That primitive's committed review at `612219cb` was read in
this checkout; its historical sibling checkout and private review trial were
not accessed.

Readiness AND the successful durable baseline receipt are mandatory. The
controller authenticates both through the committed foundation pins, requires
successful version/build/kernel stages, and checks all 523 sources and the exact
284-object map at foundation `115a618dd35ff8337112cf2081a7f5391d3973d5`.
It authenticates the runner, coqc, coqchk and opam tool hashes, all frozen
loadpaths, Coq runtime dependencies and the private GMP package/environment
manifest SHA256
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
Actual hashes are recorded in the private guarded input maps, not inferred from
success booleans. Inherited Coq/OCaml/library overrides are removed; the pinned
`rocq-toolchain/run-rocq` supplies the pinned opam switch and GMP environment.
Nothing was installed or rebuilt in shared storage.

The successful private review is
`/tmp/ordinary-shared-xor-review-e443558a-trial002/`; `results.json` records its
review receipt, child replay receipt and pretrial-freeze SHA256 values. The
computational receipt leaves manual audit pending by design; this final report
and `manual_statement_and_proof_audit_completed` in `results.json` record the
completed manual review. The successful review receipt SHA256 is
`050a7c6cb4628ba8c26995c88077b0d85f196242ccdfdd406e139b850c37d4f8`;
the fresh child replay receipt SHA256 is
`82835c0716751bf69d2474be9be3d75e5d9ba406a71d0fc9f63b4b6bfdbb7619`.
All five fresh object hashes exactly match the authenticated worker objects.
The outer guard retained 2,245 input/product/log identities; the fresh controller
independently retained its 2,212 original inputs and separately guarded all ten
fresh source/object products through full recursive checking.

Fresh normal `coqc -q` compiles the three modules and both inventory modules.
`Print Assumptions` covers every one of the 21 declarations, including the
persistent instance, and all 84 generated marker lemmas. Full recursive
`coqchk -o` checks all five modules and their dependencies, with no admission,
exclusion, `-norec`, quick/opaque compilation or unsafe flags. The complete
kernel inventory contains exactly 105 component/helper constants. Source
inspection finds no added constructors, Program obligations, nested declarations
or other generated implementation constants; the only Existing Instance
registration is the pre-existing `Outer` in the adapter.

Three component declarations (`zip_transform_na`, `shared_xor_na_cost`,
`shared_xor_na_cost_step`) and every inventory helper are closed under the global
context. The other 18 component declarations depend only on inherited dependent
functional extensionality. The recursively checked dependency context additionally
contains inherited `Eq_rect_eq`. No new axiom or primitive is introduced; the
whole dependency context is not axiom-free. Printed proof-term section hashes
can reflect printing-depth limits; full recursive checking supplies the complete
kernel validation. The unchanged controller's guards cover all original inputs
before/after commands and at completion; separate guards retain copied sources,
generated inventories and compiled objects through the final check. The outer
review also retains worker receipt/input/object/log identities through success.

## Statement and proof audit

I read all source statements/proofs and the 84 generated helper statements.
The following inventory describes their exact scope rather than treating the
contract definitions as assumed WP summaries.

| Declaration | Checked meaning |
| --- | --- |
| `reader_na.key_reader_na` | Persistent indexed Na-read interface, taking linear R, arbitrary Frame and 3*(g+1) credits; returns the exact byte, range, same R/Cap/Frame. |
| `key_reader_na_persistent` | Persistence of the reader assertion, not duplication of linear R. |
| `key_reader_na_read` | Eliminates precisely that reader interface at an in-bounds index. |
| `shared_key_read_na` | Derives an actual indexed Na read from ordinary E/L, alive κ and Cap, restoring L. |
| `shared_key_reader_na_session_prepared` | Opens fresh existential R/G from ordinary L and prepared Cap, with no credit premise; provides G's guard and a closure to the original L. |
| `shared_key_reader_na_session` | Calls shared_key_prepare once with g+1 credits, then opens the prepared session. |
| `key_reader_na_aliases` | Duplicates persistent reader/Cap aliases only; no R or exclusive key vector is produced. |
| `key_reader_na_read_close` | Reads, then consumes R/G through the supplied closure to return original L, Cap and Frame. |
| `xor_na.zip_transform_na` | Actual recursive loop value, with only the three memory-order changes described below. |
| `shared_xor_na_cost` | count*(3*(g+1)). |
| `shared_xor_na_cost_step` | Exact successor decomposition into current read cost plus remaining cost. |
| `shared_xor_na_contract` | An iProp definition containing the exact WP and length/suffix/range postcondition, proved below. |
| `wp_shared_transform_loop_na` | Operational induction for an operation satisfying the existing Z.lxor operation spec, arbitrary start/count, original key base and exact transformed cells. |
| `wp_shared_xor_loop_na` | Instantiates the operator with the checked unchanged byte_xor implementation. |
| `shared_xor_na_verified` | Proves the defined contract and its pure facts from bounds and pad-byte range. |
| `wp_shared_xor_na_min` | Exact xor_result, XOR prefix, unchanged suffix, length and byte range at min(length pad,n). |
| `wp_shared_xor_na_128` | Specializes only pad length to 128; key length remains arbitrary. |
| `wp_shared_xor_na_zero` | Separate Cap input; no Reader, pad-byte bound, first-cell ownership or access requirement. Preserves pad/R/Cap/Frame. |
| `adapter.sc_session_to_na` | Consumes old Rsc/Gsc with their closure, then opens a fresh Na session from original L and existing Cap; frames an arbitrary assertion. |
| `raw_pair_reader_na_context_wp` | Preserves the frozen fixed pair-setup callback type except for the Na reader, with same Outer/TypeG, pad/Lpad and exact outside frame. |
| `raw_pair_reader_na_observed` | Uses initialized observation adequacy with internal 2+k credits and an actual resource-to-state observer in the same Outer. |

The direct and session Na-reader proofs unfold the actual shared view, reject
the impossible depth-zero view, retain its physical guard, compose it with the
alive-lifetime guard, and select `(q +ₗ i) ↦[^ []] #z` via `bytes_index_guard`
using the actual list lookup. Mask weakening produces the required naN guard.
Both proofs call the foundation's **`wp_read_na_guarded_cells` at depth g+1**.
No SC black-box WP is used to establish a Na read law. Cap supplies byte range
but does not replace the physical permission derivation.

The foundation rule at `src/lambda_verus/lang/lifting.v:453` takes exactly
`£(3*d)` and returns its original guard token. Its implementation uses both
`ReadNa1S` and `ReadNa2S`; the actual rules in `lang/lang.v:268`–`:279` permit
arbitrary initial reader count, increment during phase one and decrement in
phase two. This is an ordinary read with an intermediate state change, not an
SC atomic read in disguise. The result does not equate concurrent SC/Na
programs or erase their differing intermediate behavior.

Prepared-session allocation uses the existing
`llctx_interp_make_guarded`/fractional allocation to produce fresh halves. It
obtains the alive guard using R while returning R, composes guards, and builds
the Na reader over that exact half. G separately guards the original L. Closure
joins both fresh halves through the existing fractional withdrawal rule to
recover exactly L. The prepared lemma has no `£` premise; only the raw-view
wrapper spends g+1 credits to derive Cap via the authenticated shared preparation
proof. Neither persistent alias theorem duplicates a linear session token.

The loop body is the frozen `zip_transform` expression, byte-for-byte after
whitespace normalization and replacing exactly **two ScOrd reads and one ScOrd
write** with Na1Ord. Guard, parameter list, count test, operator call, store
address, recursion and both pointer increments are unchanged. Baseline notation
maps `!` and `<-` to Na1Ord. Frozen SC body SHA256 is
`6bea58114f4405a61c4c801717075336061569efc93e927ef2ed62679fe0f60e`;
Na body SHA256 is
`6bb03908ac214d7dbf7360bd3e0d99f5aea89c9dad7592ad6a64f6f737acd4ae`.
`source-audit.json` independently records this comparison and exact equality of
both old/new adapter statements after the reader/theorem name substitutions.

The operational proof binds an actual Na pad read, the actual Na key read at
`q +ₗ start`, and an actual Na pad store. It splits only the current pad head,
keeps the tail, reads through the shared key reader without extracting an
exclusive key vector, and applies the existing byte_xor operator theorem.
The induction keeps q fixed and proves
`(q +ₗ start) +ₗ 1 = q +ₗ S start`; it passes S start, the same reader/R/Cap and
remaining credits into the recursive call. Lookup equality ties the returned
key byte to the exact dropped-key head. Recombining the updated pad head gives
`transform_prefix count Z.lxor pad (drop start key)` exactly. R and arbitrary
Frame are carried throughout; G/closure or persistent aliases can be retained
in Frame/outside the loop and used afterward.

The budget splits into current 3*(g+1) and remaining count*3*(g+1).
Owned-pad reads use `wp_read_na` at guard depth zero. The pad store applies the
actual `wp_write_na` with **only PadHead**, no input store credit. Foundation
`lifting.v:638` instantiates its guarded store at depth zero using lc_zero;
`lifting.v:562` uses the credit received from the first runtime phase to close
the heap transition before phase two. Thus zero additional external store credit
is justified by the real WP rule; it does not mean zero internal proof work or
claim a minimal budget. No invented positive credit source is used.

Pure results use the existing exact `shared_xor_result`, `transform_prefix`,
`shared_xor_min_result` and XOR range/prefix/suffix lemmas. Bounds are precisely
count≤length pad and start+count≤n. The min and 128 specializations impose no
key-length upper/lower restriction: key length 0 and >128 are included. Count
zero has its own theorem: it reduces recursion/test/branch and immediately
returns resources, without lookup, pad-head splitting, read, write, Reader or
first-cell ownership. Cap is a separate input. There is no p≠q premise, so p=q
is permitted where the supplied MODEL resources are satisfiable.

The adapter first invokes the old SC closure with **both** Rsc and Gsc to obtain
L, then calls the prepared Na-session lemma. It does not return either old
linear token. A retained persistent SC-reader assertion alone cannot read
without its consumed old R. Fresh existential R/G come from new fractional
allocation; old reader and guard assertions do not provide the withdrawn
ownership. The fixed pair wrapper explicitly frames Lpad, invariant context,
PadRef, outside fragments and all £k through this transition. It uses the same
TG and substitutes the proven SameOuter equality; it does not introduce a
second heap/WP interpretation. At d=1,g=0, the existing prepared Cap is reused
without a third setup credit. The public observed wrapper passes 2+k to the
existing initialized adequacy theorem, which allocates those credits internally.
Its external data/closedness/disjointness/callback/observer hypotheses are the
same frozen model interface with the Na reader substituted. No new caller
permission/credit, trusted application premise or predicted final-state premise
was added. Completing the callback for the full initialized writer/observer
remains root's integration work.

## Procedure, reproduction and limits

All seven worker development failures remain nonsemantic development/setup
history in its committed attempts inventory; they were not rerun or used as
negative semantic evidence. Review trial001 completed all component commands,
including recursive kernel checking, but its outer wrapper then looked for Coq's success line in stdout
instead of stderr and recorded a failure. Only that review check was corrected,
and input/snapshot hash comparisons were made explicit. A completely fresh
trial002 reran normal compilation and recursive checking. Both trials and
the harness correction are retained in `attempts.json` and private durable
receipts; the first wrapper failure is not a semantic negative or an accepted
outer review receipt. Raw compiler/assumption/term/kernel logs and objects remain
private; compact authenticated hashes and commands are committed here.

Reproduce in this prepared checkout with a new nonexistent output directory:

```sh
python3 -B evidence/ordinary-shared-xor-review/review.py \
  --output /tmp/ordinary-shared-xor-review-NEW
python3 -B evidence/ordinary-shared-xor-review/audit.py
```

No implementation, baseline/compiler/vstd/shared helper, frozen Rust body/domain
or other evidence lane was edited. No live parent/sibling checkout, root
full-model-writer trial or unfinished private output was read. No filesystem
race/substitution experiment, subdelegate, installation, shared build, push or
PR was used. An initial guessed baseline source path (`src/lrust`) did not
exist; `_CoqProject` supplied the actual `src/lambda_verus` path. This was a
read-only lookup error and supplied no evidence.

These are conditional partial-correctness MODEL results. There is no native
Rust/MIR/compiler correspondence, native input/provenance/ancestor-history,
retag/C/F simulation, unrestricted concurrent-program equivalence or termination
claim. Canonical 6/0 and HMAC 1/1 are inherited and were not rerun. Native stays
0, minimal/phase obligations 44+3 and writer effects 101 stay open, publication
stays false, and the goal stays incomplete.
