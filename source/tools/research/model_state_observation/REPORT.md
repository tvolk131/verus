# Checked MODEL final-state observation

This lane delivers an actual final-heap adequacy rule. It is a mathematical
MODEL prerequisite, not Rust entry, provenance, native simulation, full-goal
acceptance or publication. Only this directory was changed. No installs,
subagents, pushes/PRs, shared baseline changes or filesystem experiments.

The pretrial typed interface was committed at `15867032`. Each trial additionally
freezes exact statements, proofs, controller and trust/support inputs before
compilation; all failed source copies, freezes and logs remain indexed in
`development.json`. Failed scripts are development only. The countertheorems
below are ordinary compiled Qed proof terms.

## Adequacy and actual state

`observation.initialized_interval_adequacy` accepts arbitrary finite initial map
h, expression e, mathematical location p, natural length n, initial credits k
and semantic relation `P : val -> list val -> Prop`. Its genuine WP obligation,
universally quantified over the internally allocated lrustGS, is:

```coq
initial_fragments h -∗ £k -∗ time_ctx -∗
WP e {{ v, ∃ vs : list val, ⌜length vs = n ∧ P v vs⌝ ∗ p ↦∗ vs }}
```

It proves ordinary `adequate NotStuck e (initialized_state h)` with postcondition:

```coq
λ v σ, ∃ vs, length vs = n ∧ P v vs ∧
  ∀ (i : nat) w, vs !! i = Some w →
    (snd <$> σ.1) !! (p +ₗ i) = Some w
```

The returned list is the SAME witness in the semantic predicate, length and
actual final lookups. `heap_values` is exactly `snd <$> σ.1`, not a chosen heap
or new validity predicate. Initial allRSt0/global-lock=false and empty freeability
are fresh MODEL initialization conventions. No final RSt0 or unlocked-state
condition is required. No allocation base, zero initialization, supplied owner,
permission, lifetime or freeability premise. Full points-to resources have their
usual exclusive separating alias obligations; those are WP proof obligations,
not extra application premises. Arbitrary n includes 0, 10 and 128; arbitrary
values include every initialized byte vector. Explicit length prevents an empty
returned resource from satisfying a positive target length.

`initialized_interval_frame_adequacy` additionally observes every actual final
key/value of a returned exact finite frame f. The input-specific
`initialized_interior_observation` requires ONLY pure original raw-map per-index
lookups for an arbitrary input list, supplies both interval and exact
`h minus interval_map p input` internally, and derives actual final values for
that entire original outside map. It applies at every mathematical offset,
including strict interiors of larger same-block intervals with nonempty prefix
and suffix, and retains all unrelated map entries. It does not assert equality
of the whole final heap: additional allocations elsewhere are allowed. No claim
about finite native addresses or provenance follows from mathematical locations.

The private state observation is justified by the existing
`points_to_heap_agree` on `heap_ctx`'s actual `non_atomic_map heap_name σ.1`.
`fragments_observe` and `interval_observe` erase only lock bookkeeping.
The generic `initialized_observation_adequacy` accepts an arbitrary resource
post Φ and a proved observer over that same actual heap_ctx. The interval/frame
rules discharge that observer internally; no desired-final-state premise is
exposed by those specializations.

The strong adequacy proof allocates exactly one HeapGS and passes its SAME
LRustGS to the callback, state interpretation and final observer. It uses the
existing finite initializer, normal time initialization, atomic pool/counter,
and empty freeability authority. The final state interpretation is the ordinary
`heap_ctx σ ∗ time_interp (n+1)`. It examines the singleton initial thread's
`from_option Φ True (to_val e')`, applying the observer if that thread is a value;
it never requires the rest of the pool to be values. Its conclusion covers
EVERY reachable main-return configuration, including those with live forks,
and NotStuck for every thread. It is safety/partial correctness, not termination
or a claim that forked threads cannot run after main return. The observed values
hold at all reachable configurations with main returned because returned full
resources remain in the initial thread's postcondition.

## Executed instances and controls

`instances.read_interval_final`, `write_interval_final`, and
`write_read_interval_final` execute actual model SC reads/stores at p. They return
all cells of arbitrary `first::tail` plus the exact outside frame. The read
observes the original first value. The store returns poison and observes
`written::tail`; store/read returns the written value and the same updated list.
The tail and the entire original outside frame are observed in actual final
memory. Inputs are arbitrary raw maps and values, not initialized-zero heaps.

The read needs k=0; SC store needs k=1 for the existing write rule; store/read
also needs k=1 and includes its visible sequencing beta step. No setup Skip is
inserted by this lane. The adequacy wrapper internally supplies
`advance_credits 4 = 210` for normal `time_init`, in addition to callback k,
just as the pinned initializer does; ordinary step-later accounting is inherited
from strong adequacy. There is no runtime reinitialization step.

`store_execution` is an explicit one-step operational execution proof, lifted
from existing WriteScS to rtc erased_step. `same_return_different_memory` proves
two such executions have identical poison return and different actual final
values. `value_only_does_not_imply_unchanged` proves value-only adequate NotStuck
for a changing store AND refutes adequate NotStuck with an unchanged-cell
postcondition, using the witnessed reachable final state and adequate_result.
`returned_value_cannot_disagree` proves resource/state disagreement impossible;
`explicit_length_refuses_empty` proves the positive-length empty-payload control.
These are model-program diagnostics, never replacement Rust writers or full-goal
proofs. No operational semantics was modified.

## Replay and trust

Run with a fresh output directory:

```sh
python3 -B -O source/tools/research/model_state_observation/check.py \
  --freeze source/tools/research/model_state_observation/final-freeze.json \
  --output /tmp/fresh-model-state-observation
```

`results.json` records the final successful receipt, exact source and object
hashes and complete per-constant assumptions. All 20 new definitions/lemmas/
theorems are individually inventoried; all are closed under the global context.
No Program declarations or generated obligations are introduced. Normal coqc
and recursive coqchk include all three modules plus the generated assumption
inventory. The inherited foundation kernel inventory permits only dependent
functional extensionality and Eq_rect_eq; no new axiom, admission, unsafe mode,
type-in-type, assumed positivity or rewrite rule.

The small controller guards retained JSON snapshots before interpreting them,
authenticates baseline-ready AND successful durable-build-2, all 523 source/284
object pins, tools and loadpaths, and explicitly compares the retained GMP
manifest hash to authenticated foundation-baseline.toolchain.GMP_packages.
It authenticates the corrected `gmp-pin-heap-positive` receipt and exact compiled
prerequisite artifacts. Copied source identity is checked through kernel checking;
all final source/product/input guards run before success. Prover cwd is private.
No mutation/concurrency/source-substitution experiment was run.

## Remaining composition and source gaps

The committed ModelInputContext context/composition/instances were read. Its
public `raw_model_context_adequacy` only concludes pure φ; that theorem cannot
export final resources for this observation rule. The minimal additional
interface is a **WP-level context setup lemma** in a fixed outer lrustGS,
parametric in arbitrary `Φ : val -> iProp Σ`: initialize the type/lifetime/
prophecy/uniqueness contexts wrapping that EXACT outer heap instance, give the
callback the existing reference, exact frame, contexts and £k, and conclude
`WP (Skip;;e) {{Φ}}` from original raw-map fragments, time_ctx and £(1+k).
The callback's resource Φ must survive unchanged rather than be reduced to a
pure proposition or tied to a second heap. This is a proposed integration
signature, not a theorem claimed here. The now completed parent typed-input
replay is available but is not imported by the independent observation proof.

Its existing composition retains frame, llctx, future/current observation,
prophecy relation and lifetime-end restoration continuation. Resolving a loan
alone does not recover the full physical points-to interval. Actual loan-end
recovery and a WP callback returning those resources are still required before
that composition can feed this rule. This lane neither assumes that recovery
nor edits the other lane.

Fresh model context creation does not identify an original runtime loan, incoming
Rust typing/provenance/ancestor state, native MIR/phase/exit simulation or Verus
C/F observation import. All 44 minimal simulations +3 phases /101 effects remain
open; native=0, publication=false. Canonical every-u64/arbitrary-[10] remains 6/0;
full frozen HMAC prefix AND untouched suffix over arbitrary-[128] and all valid
keys, including 0 and >128, remains 1/1. The original bodies, properties, authored
semantic-input requirements and domains are unchanged. This prerequisite cannot
complete the goal alone. Parent integration and new independent review required.
