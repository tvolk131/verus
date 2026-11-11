# S1: bounded checked Set-pair assistance (local proposal)

This implements only parent-authorized S1. Parent selection, fresh independent
implementation review and combined acceptance remain required. The original
HMAC/native objective is deferred and unachieved; F0/R1/F1 remain unsupported and
in the denominator. S1 is a small R2 improvement and cannot complete the practical
goal or establish substantially reduced overall R2 proof burden.

## Mechanism and boundary

`vstd::set_lib::lemma_set_pair_observations<A>` ordinarily proves exactly C0's
conditional subset-length bound, inclusion-exclusion equation, and ordinary Set
equality iff memberwise equality. Its body reuses the existing proved lemmas and
inherited Set extensionality. No axiom, external body, stronger application
contract, installed-contract rewrite or new Rust semantics was added. Its
standalone body passes `--no-cheating`; the matching full NORMAL vstd build passes
2251/0. Existing vstd admissions are inherited and not passed no-cheating.

One generic VIR pass selects ordinary proof functions with immutable Spec
parameters and unit result; it skips infer, external/proxy, unsafe/async,
prophecy-dependent and termination-suppressed signatures. Only supplied
requires/ensures and source-visible ordinary spec definitions supply observations.
Actual term/type substitution preserves compact Set-returning calls, so
`allocated(t)` never expands into filter closure internals. Recursive definitions,
binder capture, unknown roots and exhausted node/definition/term/pair budgets fail
closed. Limits are4096 nodes,32 expanded definitions,8 typed terms,32 ordered
non-reflexive same-type pairs. One step maps an observed intersection to its
ordinary same-typed union for the F2 universe observation. No project or semantic
symbol recognizer exists. No range enabler is added: the range broadcast was
already redundant in accepted defaults and its removal earns zero gain.

Pre-mode frontend Move/Copy reads are accepted only after checking their roots
are immutable Spec parameters; generated copies become Spec reads. Original
reads stay byte-identical. No future/old/local/return/bound name becomes an input.
Library calls use a closed vstd identity/mode/type catalog. The original complete
Function is independently reconstructed by stripping only the exact legal prefix
calls and compared byte-for-byte, including contracts, executable effects,
invariants/decreases, metadata and embedded type/implementation paths. Generated
Expr/Place/read/statement IDs are fresh against ALL preceding current/imported
function ASTs and all earlier functions decorated by this pass. Later passes must
reserve from S1's output; see REPLAY_S1.md. `body_rechecks` requests FULL WF/mode
checking and refreshed resolution through the existing driver. Reporting is an
observation, not a proof fact or inferred-helper/cache receipt.

TCB remains the existing frontend, VIR/lifetime/trait/WF/mode and ordinary proof
checking pipeline, pinned Z3, and inherited vstd semantics/admissions. The selector
is untrusted proposal generation. The new helper is proved ordinary library code;
no new trust-bearing contract or admission is introduced.

Provisional epochs: schema39/policy77/cache80, skipping deferred76/79. No VIR
serializer/schema change. Export/import artifacts are freshly rebuilt.

## Literal application cost and removals

Reduced application freezes precede assisted trials in `e51548fa`; exact SHA pins
are in S1_FIXTURE_FREEZE.json. No authored checked-pair helper or calls remain in
them. Original lane manual/freeze/replay/report and failed-case files remain
unchanged. Registered project source files match all five selection hashes.

| Site | Author work removed | Still authored/charged | Generated ordinary calls |
| --- | --- | --- | --- |
| R2 finish_mutation | One necessary allocated-set extensional equality assertion | All8 mechanical helpers,15 original lemma call sites,9 exec proof blocks,9 snapshots,6 entry invariants, all frame/ancestor transport and other assertions |20 calls over5 terms, only in finish_mutation |
| F2 three corollaries | All4 original set-lemma calls and2 conditional blocks | Original semantics, finite-universe/fault/count assumptions,3 empty ordinary corollary bodies, checked FromSpec adaptation |20 intersection +6 honest +6 margin calls |

The redundant R2 range broadcast is omitted but uncharged. Existing default
cardinality/insert/remove algebra is not new authority. F2 numeric arithmetic and
FromSpec adaptation earn zero reduction. New reusable library proof: one ordinary
generic body, charged separately. No native allocator/consensus implementation,
whole-project, cryptographic, crash-consistency or financial correctness claim.

## Validation and retained evidence

Private root `/tmp/practical-set-s1-3b931894`; replay instructions in REPLAY_S1.md.
The final bundle receipt pins the exact runtime, source and verified library.
Full AIR145 and VIR163 unit tests pass. Six focused structural tests cover
catalog/signature rejection, pure roots, typed dedup, budgets/cycles/hidden
observations/capture, full Function projection, loop/effect/decrease/metadata
mutations, embedded datatype implementation paths, duplicate/misplaced calls and
ID collisions across repeated decorations. Compiler/app trials and their exact
outcomes/costs are recorded in RESULTS_S1.json and raw guarded receipts.

Accepted75 baseline: reduced R2 fails only finish_mutation24/1; reduced F2 fails
only the three corollaries9/3, with goals off and on. Private narrow ablation
disables only S1 preparation while keeping all other profiles/library enabled:
exactly the same failures, with healthy dependencies and zero pair observations.
The ablation's one-line pipeline mutation is retained privately with exact
before/disabled/restored hashes; the original pipeline bytes were restored.

The standalone helper passes1/0 no-cheating. Two full normal library builds each
pass2251/0; final runtime/source hashes identify the matching production boundary.
Zero inferred-helper inventory is explicit for all R2/F2 outcomes. Repeated trials
are repeat verification, not evidence of cached helper reuse or a speedup.

Final cold/repeat: R2 25/0 and F2 12/0. Providers/imported callers: R2 27/0
and3/0, F2 12/0 and2/0 (ordinary and goals-enabled consumers). All25 final
outcomes match expectations, including strict set false claims/callers,
R2 leaf/count/frame, F2 intersection/fault/zero-safety, scalar precision/safety,
stale-export removal and policy-version rejection. Unsupported mutable proof
signatures remain undecorated. Negative dependency breakdowns stay healthy.

Costs on this private debug runtime: cold/repeat R2 23.87/21.71 seconds;
F2 18.61/13.12 seconds. These are verification trials with ordinary generated
calls, not cached helper hits or a speedup claim. Per-run SMT/verification times,
full command timings and exact counts are retained in RESULTS_S1.json/raw receipts.

Earlier setup/build/frontend and unsuccessful selector attempts are retained,
classified separately in RESULTS_S1.json. In particular: bwrap loopback failure;
private Git refs reconstruction; missing rustup/toolchain/bootstrap/ghost flags;
initial ReadKind compile/test-fixture mistakes; premature ablation setup; initial
selector trials with zero calls; temporary diagnostics. None is semantic proof
or a healthy semantic negative. No resource failure is relabelled as proof.

Stop after S1. No frame/loop/semantic next layer is authorized here. Parent owns
selection/combined acceptance, including any future S2 composition.

Formatting check: `git diff --check bfdf5883` reports six trailing-space blank
lines in the already-frozen reduced proof slots. Their bytes were deliberately
retained after trials; this formatting finding is not a verification result.
