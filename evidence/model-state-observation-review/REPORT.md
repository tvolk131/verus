Independent formal mathematics review of integrated `91a7a1c1`, at prepared
checkout HEAD `1d380328`.

**Verdict: accept the exact 20-constant MODEL final-state observation
prerequisite. No blocking mathematical or evidence-authentication defect found.**
This is conditional adequacy from the stated, genuine WP proof obligation. It
is not acceptance of a native program theorem, typed-input composition, or the
whole buffer/HMAC goal. No implementation change is requested by this review.

The review independently read `INTERFACE.md`, `FROZEN.md`, `REPORT.md`,
`results.json`, `development.json`, `check.py`, and all three candidate modules
under `source/tools/research/model_state_observation/`. It also read the actual
pinned initializer/partition proofs, relevant framework state interpretation,
adequacy, agreement, operational semantics, SC lifting and time initialization,
and the controller's shared helpers, parser and runner. Only this new evidence
directory was written. No subagents, installs, implementation changes, source
substitution/concurrency experiments, artifact-control drivers, pushes or PRs
were used. No parent replay outputs or parent-checkout source were read.

The new replay ran the unchanged controller with this exact command from the
review checkout:

```sh
python3 -B -O source/tools/research/model_state_observation/check.py \
  --freeze source/tools/research/model_state_observation/final-freeze.json \
  --output "$PWD/evidence/model-state-observation-review/replay" \
  > evidence/model-state-observation-review/replay.stdout \
  2> evidence/model-state-observation-review/replay.stderr
python3 -B evidence/model-state-observation-review/audit.py \
  > evidence/model-state-observation-review/audit.stdout \
  2> evidence/model-state-observation-review/audit.stderr
```

All five prover commands exited 0: ordinary `coqc -q` for observation, instances
and controls (4.18s, 2.49s, 1.92s), ordinary compilation of the generated complete
assumptions inventory (16.50s), and recursive `coqchk -silent -o` on all three
modules and Assumptions (64.43s). Complete argv, private cwd, input snapshots and
results are retained in `replay/receipt.json`. There was no quick compilation or
nonrecursive checker flag. Auxiliary `.vos`/`.vok` files produced by normal
compilation are retained but are not the checked proof objects.

**Mathematical conditions checked.** `observation.v:8-12` defines the observed
map as exactly `snd <$> sigma.1`. This erases lock bookkeeping from the actual
physical state, not a separately supplied model, owner, client observation,
chosen poststate, or assumed validity relation.

At `observation.v:73-86`, arbitrary `h,e,p,n,k` and arbitrary semantic
`P : val -> list val -> Prop` are permitted. The WP must return
`exists vs, pure(length vs = n /\ P v vs) * p |->* vs`. The conclusion uses that
SAME witness for length, semantic P, and every actual final lookup at `p + i`.
There is no list substitution at the observer. The full points-to vector is
required, not just a list assertion, length bound, or bare value postcondition.
At n>0, the explicit equation prevents using the empty points-to vector to
prove a nonempty-buffer claim. At n=0, vacuous interval observation is the
correct mathematical result. No fixed bound, byte-zero, allocation-base or
native pointer assumption restricts n or values; lengths 10 and 128 and all
initialized byte vectors are included.

The generic rule `observation.v:40-71` does expose an observer obligation for an
arbitrary resource postcondition Phi. This is a proof obligation over the
actual same-instance heap authority, rather than an axiom. Crucially, the
interval and frame specializations discharge it internally at lines 83-85 and
101-104; their application premises do not ask the client to prove the desired
final heap relation separately.

`fragments_observe` (`observation.v:16-25`) extracts `non_atomic_map heap_name
sigma.1` from `heap_ctx sigma` and uses the existing
`points_to_heap_agree` (`lrust/util/non_atomic_cell_map.v:1033-1038`). The latter
proves an actual lookup `sigma !! l = Some (st,v)` for some st. Rewriting
`lookup_fmap` therefore produces precisely the observed final value without
assuming final RSt0, an unlocked global state, or any particular final lock
count. `interval_observe` applies this to `interval_map` and its checked lookup
lemma; no abstract-to-physical step is left assumed.

The same heap instance runs throughout `observation.v:48-61`: finite-map
initialization creates vgamma and all initial full fragments; empty freeability
authority is allocated; `time_init` receives `advance_credits 4 = 210`; a full
atomic invariant pool and zero atomic lock counter are allocated. A SINGLE
`HeapGS` is constructed from those names and wrapped by the SINGLE `LRustGS`
given to the callback. Inference of the state interpretation uses that same
instance, with ordinary `heap_ctx sigma * time_interp stepcnt`
(`lrust/lang/lifting.v:13-30`). The initial physical state is the checked
initializer's `(RSt0 <$> h, false)` with empty freeability relation. This is a
MODEL convention, not a claim about an incoming Rust history. Additional k
credits go to the callback; ordinary strong-adequacy step credits account for
execution. No operational initialization step or inserted Skip is claimed.

The strong-adequacy dependency (`lrust/util/adequacy_util.v:199-278`) splits the
reachable pool into the final versions of the initial threads and arbitrary
forked threads. Its callback provides state interpretation at actual sigma2,
`from_option Phi True (to_val e')` for the initial thread, and safety for all
threads. The singleton list destruct at `observation.v:62` constrains the number
of INITIAL threads only; `rest` is arbitrary. For a returned initial thread,
lines 64-67 consume its Phi together with actual final Heap; for a nonvalue,
lines 68-70 prove the result implication vacuously and retain safety. There is
no `Forall is_value rest`, all-threads-return, or termination premise.
`adequate_alt` and `adequate_result` (`iris/program_logic/adequacy.v:280-302`)
quantify over every reachable configuration with a returned main thread, with
any remaining pool, so live forked threads are covered. The top-to-empty fancy
update is used consistently: the generic observer has exactly that mask, and
the specialized pure observations weaken masks legitimately. No invariant
closure or final-state fact is supplied by fiat.

`initialized_interval_frame_adequacy` (`observation.v:88-105`) requires full
returned interval plus full returned exact finite frame f. The original-frame
specialization `initialized_interior_observation` (`107-121`) fixes f to
`h minus interval_map p input`. It asks only pure original per-index lookups
plus the genuine WP proof callback. The checked `interior_extract`
(`NativeHeapInitialization/partition.v:74-80`) supplies the interval and that
entire complement internally. All original keys outside the selected interval
are covered, including a prefix and suffix of the same larger allocation and
unrelated map entries. Since locations are block/integer-offset pairs and p is
unrestricted, strict interiors do not require a base-address premise. There is
no assertion that the entire final domain equals the original domain: fresh
allocations elsewhere remain possible. If output length differs from input
length, the returned interval and fixed original frame must still satisfy the
stated separating resource obligation; this is not a theorem granting arbitrary
resizing through overlapping resources.

**Executed witnesses checked.** `instances.v:7-50` uses the actual SC read,
write and write-then-read expressions, not value-returning stand-ins. Read
returns first and observes `first::tail`. Write returns poison and observes
`written::tail`. Write/read includes the ordinary sequencing beta step,
returns written and observes `written::tail`. Each returns the complete vector
and original frame. Read uses k=0; the SC store rules use k=1. The pinned lifting
rules reduce through real ReadScS/WriteScS, with their actual lookup/update
semantics (`lrust/lang/lang.v:261-283`).

`instances.v:53-72` proves a one-step head reduction, lifts it to an erased
threadpool step and then to rtc. The constructed `stored_state` is justified
by that execution; it is not an assumed reachable final state.
`controls.same_return_different_memory` (`26-37`) instantiates two such steps,
with distinct written values but identical poison return, and proves their
actual final lookups differ. These are two different store expressions, not
a claim of nondeterminism for one fixed expression.
`controls.value_only_does_not_imply_unchanged` (`39-56`) proves value-only
NotStuck adequacy for one changing store using the existing checked initializer
and WP rule, AND refutes unchanged-cell adequacy by applying its hypothetical
`adequate_result` to the explicit execution. The actual stored lookup then
contradicts old != written. These are genuine Qed operational countertheorems.
The other controls (`10-23`) prove actual authority/fragment disagreement is
impossible and positive length cannot equal the empty length. No failed script
is used as a counterexample.

**Inventory and trust authentication.** The complete inventory is in
`audit.json`, with kind, module, name and source line. There are exactly four
Definitions and sixteen Lemmas/Theorems, across the three fixed sources
(9 + 7 + 4). There are no Corollaries, Fixpoints, records/instances, Program
declarations, generated obligations, admitted proofs, Abort, new axioms,
parameters, unsafe computation or kernel mode changes in these new sources.
All twenty are individually printed in `replay/assumptions.stdout`, each
`Closed under the global context`. The generated inventory's trivial anonymous
marker goals are themselves normally compiled and recursively checked; they
are instrumentation, not additional claimed lane theorems. The kernel summary
permits only the inherited dependent functional extensionality and Eq_rect_eq,
with predicative Set, no rewrite rules, no type-in-type, no unsafe fixpoints and
no assumed positivity. Those global imported axioms do not appear among the
assumptions of any of the twenty new declarations.

The worker receipt at `/tmp/verus-infer-buffer-cycle-20261109/model-observation-trial017/receipt.json`
hashes to the user-specified
`7d2620147db8641c926692404be19cfbdaf5431d39bbac99f8d38bee0635aef1`.
Its retained freeze is byte-identical to `final-freeze.json`, SHA256
`9697821f7b7f9e9c235be8758f54d8e87b4862e528cca02971b50f949c3fa3e4`.
All retained source/object and log hashes match `results.json`, and all current
frozen inputs match. All eight replay source/object products match the worker
byte for byte, including these four objects:

| Object | SHA256 |
| --- | --- |
| observation.vo | `aac99a0a57d144b00744b9650a362a4c62c7f76a4dfb0f928def81df9ad3a28e` |
| instances.vo | `a54ced029fe03a71b1a089a70c73e222416128abfa0ec1b01019373bbc6e40f5` |
| controls.vo | `5b4b8c0fea8b9d56a027dc53a34266b3b5d0ccc6446cf28847423dfd37025a23` |
| Assumptions.vo | `f09e563fe4a7698082bb071ce41de3c8bb08780739a4edbe284e799e99ed1137` |

The fresh receipt SHA256 is
`522f3c6831e9144bf54dfca0e81de9350a06c137e2d583126ff3753a225dcc08`.
The replay guarded 838 input identities. Independent `audit.py` does not import
the lane controller or its shared helpers. It separately authenticates the
worker receipt, final freeze, source and log snapshots, same shared dependencies,
ready receipt AND successful durable-build-2 receipt, 523 source pins and the
284-object inventory (map SHA256
`92585333704cfeea68bed68a061df64939d6b5c2c72d1c3398f0680a9cd394c4`),
loadpaths and four tool identities. It authenticates the corrected heap receipt
`a3aea2cd291724a26f76082739455cbd4cfa8b64b3d3dcb465a465ed5c6d739d`
and every retained prerequisite `.v`/`.vo` in that receipt. The GMP manifest
matches the expected authenticated foundation pin
`332763a92d52a35f1623c7383970e6cd290b5010c426ca5bb15b21efc7f0bb37`.

`check.py:23-29` includes both mandatory support files, parser and all trust
records in the freeze. Lines 49-90 authenticate retained JSON bytes and pins
before prover invocation and substantive trust acceptance. `ArtifactGuard` hashes inputs and
compares hashes/stat identities; `LoggedCommands` guards before and after each
prover subprocess. Copied proof sources are additionally pinned at lines
127-136 and retained through the recursive check. Products are snapshotted
before coqchk. At lines 164-168, copied-source, product and input guards all
complete before setting model_proofs_kernel_checked; `replay_receipt` sets
success only on normal exit. The fixed sources were manually inventoried as
well as checked against the declarations regex: this review does not certify
that regex as a complete parser for arbitrary future Coq syntax, nor claim
adversarial transient-substitution resistance outside the excluded experiments.
The ordinary trust boundary remains the authenticated framework, Coq/kernel,
standard libraries, tools and their existing environment.

Every indexed historical receipt/freeze/source/log in `development.json` was
authenticated read-only. Trials 001-003, 005-014 and 015-016 failed ordinary
proof development in observation, instances or controls respectively. All 15
failures are excluded as semantic evidence. Trial004 is a historical successful
preliminary version, not the final result being accepted. Only trial017 and
this fresh matching replay support the final verdict. The review audit itself
initially rejected a path-based comparison of the worker's original external
freeze filename with the new replay's different freeze filename. That local
audit comparison was corrected to compare frozen identity; no prover command
failed and no candidate or baseline source changed. This nonsemantic audit
failure is recorded in `audit-development.json`.

**Outstanding mathematical conditions and scope.** The independently accepted
typed-input26 / earlier review bb38f57a is a given boundary here, not an imported
proof dependency. Its public adequacy only exposes a pure result. Before the
typed input protocol can feed this observer, a resource-valued WP context setup
rule must preserve arbitrary Phi in the SAME outer lrustGS/heap instance and
carry through its setup prefix/credits. Actual lifetime-end recovery must
produce the full physical interval and exact frame at the point where the
observer's postcondition requires them. A continuation promising restoration,
a resolved prophecy or a separate fresh heap instance is insufficient. This
review proves neither of those missing composition conditions.

No native Rust input, incoming owner/loan, lifetime/provenance/ancestor,
phase/control-flow, MIR, primitive or original C/F observation correspondence
is discharged. Actual native count stays 0; all 44 minimal simulations plus 3
phases and all 101 effects stay open. Canonical remains 6/0. The full frozen
HMAC prefix AND untouched suffix remains 1/1 over its unchanged domain: all
valid key lengths, including 0 and >128, and arbitrary pad/input bytes. There
is no domain narrowing, body rewrite, publication authorization or whole-goal
acceptance. `publication=false`.
