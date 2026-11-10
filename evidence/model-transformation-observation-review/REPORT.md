# Initialized transformation and final observer review

**Verdict: accept the exact seven-declaration conditional one-buffer MODEL
checkpoint. Findings: none.**

Target: `d639f378db2f701ffea26cef3c52950bada392bb`, unchanged trial016 freeze.
Review interface committed early as `68c7c680`. The five-law checkpoint at
`33edad89` (earlier resource/child laws at `b9b0209e`) was independently accepted
at `2de94ded`; its three source modules remain byte-identical. The six-law
checkpoint at `da85900f` adds the unchanged `transform.v`. This review focuses
on `initialized_nested_transform` and `raw_model_transform_observed`, while
reading all seven complete statements and their proofs.

## Evidence and replay

The completed trial016 receipt authenticates through committed
`observation-results.json` as
`76628395e319f769da36a07ce79cc78adb64db9e579f27441cd63a69d9ad1d57`.
The accepted completed six-law receipt authenticates through
`transform-results.json` as
`aa7cd63005acc828564c55ffff72dfeb2c66935b2f091ddadafb82482d0f0121`.
Every recorded proof source/object, the exact declaration inventories, all
per-constant assumption reports and the kernel summaries were independently
checked. Historical repository paths in receipt inputs were treated only as
identifiers: repository-relative names were rebased into our prepared checkout.
No historical parent/sibling workspace was opened.

Mandatory foundation readiness and the successful durable baseline receipt both
authenticate against the committed foundation metadata. Foundation revision is
`115a618dd35ff8337112cf2081a7f5391d3973d5`; all 523 source hashes and all 284
compiled object hashes match. The runner is `rocq-toolchain/run-rocq`; tool
hashes match `baseline-bindings.json`. GMP manifest SHA256 is
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.
All six prerequisite receipt pins and their completed artifacts authenticate;
the prerequisite sources inspected here also match those compiled artifacts.

The unchanged controller performs five normal compilations, one seven-constant
assumptions compilation, and a full recursive `coqchk -silent -o` of the five
modules plus Assumptions. It uses no `-norec`, quick/vos/vok mode or admissions.
Shared sources, foundation, tools, helpers and prerequisite objects stay
read-only. Logs and VO files stay under the private review output.

Fresh completed receipt:
`/tmp/verus-infer-buffer-cycle-20261109/model-transformation-observation-review-d639f378/replay/receipt.json`

SHA256: `ff8997d5fbbfb3aebaa13555f5fef8a244ce79df849bbbdbd3f323b85e001a07`.
All seven stages returned zero. All twelve source/object artifacts, including
Assumptions, match authenticated trial016 byte-for-byte. Its four pre-existing
source proof objects also match the accepted six-law receipt. Every one of the
seven per-constant assumption reports matches, with no extra assumptions.
`audit-results.json` records compact source/object/receipt/log hashes, dependency
pins and exact stage results; `STATEMENTS.md` inventories all full statements.

The committed parser's six `Assumptions` tests passed. They check allowed and
closed reports, unqualified/qualified/indented extra axioms, truncated reports,
missing/duplicate/mislabelled inventory blocks, and unsafe or malformed kernel
summaries. No SourceBindings tests or historical extraction experiments were
run. Source inspection confirms exactly seven declarations and no new
axiom/parameter, admission, Program obligation, constructor, instance, unchecked
computation or incomplete proof.

## Resource and changing-parent audit

`shorter_run_resources` takes arbitrary spatial `Rin`, `Rout`, `Frame` under one
`typeG`. Only `Rin` enters `RunChild`; the ancestor `Frame` and blocked parent
stay outside. `Rout` returns once and is framed through child-end/recovery.
There is no persistence assumption on these resources and no permission copied
from a pure pointer assertion. The persistent lifetime/time/prophecy/unique
contexts are distinct from these spatial resources.

`bytes_vector_mapsto` converts the complete physical byte vector, including the
empty-vector case, into ordinary exclusive interval points-to. Fixed `n` and
`byte_list_length` preserve length and the same location/path shape. In
`byte_child_transform`, old initialized range comes from the opened ghost body;
new range is a premise. The callback receives the actual old interval and
`Rin`, and must return the actual new interval and `Rout`. It is not a contract
for writes fabricated from a read-only lemma.

Opening is paid by exactly one explicit `£1`, consumed by
`lc_fupd_elim_later`. Guard acquisition uses the current local lifetime context.
`uniq_body_acc` provides a linear ticket and closing continuation; both the
physical payload and ticket are returned to that continuation. The same `ξi`,
`di`, `gi`, `idx`, pointer and vector shape are reconstructed with current value
`x1`; `uniq_prvar_app_irrel` preserves the original future. Local context halves
are reunited after closing. `Rout` is retained separately.

`initialized_nested_transform` specializes the changing payload law with
`Rin' = £1 ∗ Rin`. The opening credit is passed to `byte_child_transform`; the
physical callback receives only its `Rin`, so its own write/step costs remain
its responsibility. `byte_list_of_list` identifies the original physical input
exactly with `zs`. `Frame`, `Rout`, typing context and the actual new vector
come from the two existing changing initialized laws. The wrapper applies no
no-write theorem.

`initialized_nested_run` destructs the original `model_reference` once and
retains its `ξi`, `ζi`, `idx`, current-link observation and `Back`. It builds the
original parent typing resource from that ownership and a real Skip time
receipt. Reborrowing records the blocked relation between child future and
original parent future. Child end resolves the updated child, and the combined
observations substitute `x1` into that original blocked relation. Parent
resolution then derives **the original parent's future = x1**. The returned
`Back` is the initializer's restoration capability, not a fresh replacement.

`initialized_nested_payload_run` ends that same fresh parent lifetime with the
retained alive token. Its original `Back` yields the physical vector under a
later. The executed Endlft/Skip transitions discharge this later. Prophecy
equality, variable independence (`bytes_vars_nil`) and
`bytes_observed_injective` identify the restored value as `x1`, then recover
ordinary interval ownership. Nothing asserts that the old `zs` survived a
mutating callback.

All relevant mask/later changes remain explicit: lifetime/unique/proph namespace
inclusions use `solve_ndisj`; opening consumes `£1`; closing uses `iNext` and
`fupd_mask_mono`; local-context restoration returns its guarded halves;
child-end uses the existing receipt-indexed step-fupd rule before resolving,
ending and unblocking; parent end uses `wp_step_fupd`/`wp_mask_mono` and its
real Skip steps. The final observer opens the actual state interpretation and
ends at the `⊤,∅` update. No new mask axiom or uncharged later elimination was
introduced. Normal compilation and recursive kernel checking validate these
transitions against the pinned foundation.

## Public initialized observation

The public theorem has only `typePreG` setup, closedness, old/new byte ranges,
raw initial heap lookups, and the separately proved physical WP premise
**for every `TG : typeG Σ`**. That WP consumes the old points-to vector and
`£k`, returning the new points-to vector. This is an explicit mathematical
condition: the checkpoint supplies a composition theorem, not a concrete
application callback proof. No client ownership, permission, lifetime guard,
prophecy, restoration token or credit premise is appended to the original Rust
application.

The theorem passes `1+k` to `raw_model_context_observed`. Setup supplies its own
one additional lifetime credit, hence `2+k` initial internal credits in all:
one lifetime allocation credit, one payload-opening credit and `k` callback
credits. Step-index/time receipts used by the model remain accounted for by
the existing adequacy/time rules; this budget is not a runtime bound.

The setup's `raw_model_context_wp Outer` constructs
`TG := TypeG Σ Outer ...`, explicitly fixes `Outer` as the heap instance, and
passes `@type_lrustGS Σ TG = Outer`. The observed setup rewrites with this
`SameHeap` before invoking the typed callback. The observer uses the resource
post returned by this exact typed heap against the final `heap_ctx σ` of the
same initialized authority. This avoids observing an unrelated, shadowed heap
instance. The initial fragments are derived by finite-map initialization and
partitioned internally into the input interval and its exact difference map.

Here `Rin = £k`, `Rout = emp`, and
`Frame = initial_fragments (h ∖ interval_map p (LitInt <$> zs))`.
After parent recovery, the public proof returns exactly
`LitInt <$> byte_list (length zs) x1` plus that retained frame. The inherited
observer derives lookup agreement from final heap authority, erasing only lock
bookkeeping via `heap_values σ = snd <$> σ.1`. Thus the post describes actual
final model memory, not a predicted ghost state. It observes every input-length
output cell and every cell in the exact original outside map; it does not assert
that the final domain contains no additional allocations.

The foundation's `adequate_result` permits an arbitrary remaining thread list
`t2` when the main thread returns. The strong observer likewise keeps arbitrary
forked threads, including live ones, while inspecting the actual state.
`adequate_not_stuck` covers all reached threads. A callback that forks must
satisfy the ordinary WP fork obligations and return the exclusive output
ownership; keeping the outside `Frame` spatially unavailable prevents giving
it to a fork. The result does not require all threads to terminate or establish
future quiescence.

## Scope and findings

No mathematical or statement-scope defect found in the reviewed conditional
MODEL composition. The complete assumption inventory is recorded per constant
in `audit-results.json`: `bytes_vector_mapsto` and `byte_child_transform` use
only inherited dependent functional extensionality; each of the other five
uses that plus inherited `Coq.Logic.Eqdep.Eq_rect_eq.eq_rect_eq`. The recursive
kernel summary has exactly this two-axiom inventory, predicative Set, no rewrite
rules, no type-in-type, unsafe (co)fixpoints or assumed positivity. These
inherited axioms are disclosed, not treated as assumption-free proofs.

This is conditional partial-correctness `adequate NotStuck` for an initialized
one-buffer MODEL expression. It proves no termination, native/application
correspondence, incoming Rust reference provenance, native retag/loan/source
phase, or C/F import. It does not instantiate the pair/shared two-region writer,
prove shared-key reuse, or prove full HMAC. The unchanged original executable
sources and domains gain no requirements. Both implementation commits add only
research proof/controller/metadata files; this review changes only its new
evidence directory.

Status remains: **0 new native correspondences; 44 operations + 3 phases and
101 native effects open; HMAC 1/1 open; publication false; goal incomplete**.
Unfinished ordinary-memory model access and active loop/reader-budget work are
explicitly excluded. No root live ordinary-access trial, parent/sibling checkout,
filesystem race/substitution, earlier excluded experiment, Rust/HMAC rebuild,
implementation edit, subdelegation, push or PR was used. Development/script
setup errors are not semantic negatives.
