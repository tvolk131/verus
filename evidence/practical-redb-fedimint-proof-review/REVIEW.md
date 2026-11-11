# Independent practical R2/F2 checkpoint review

**Accept both scoped ordinary manual application proofs at 4ac48a9a. Accept C0 as concrete shared checked-observation demand after the user's supplemental evidence, not as an Infer improvement. The practical automation goal remains unachieved.** No source/domain/property or production change is made by this review. One nonblocking measurement correction is required: R2's explicit range-set broadcast is redundant in the accepted runtime.

## Finding: do not charge a redundant broadcast as necessary work

`source/tools/research/practical_redb_fedimint/VALUE_CHECKPOINT.md:36` says the R2 range/filter update equality *needs* a range-set broadcast. `redb_tree/VALUE.md:9` charges it together with the extensional assertion; supplemental `common_set/PAIR_PROTOTYPE.md` lists its removal as an expected gain. The literal inventory is accurate, but necessity is overstated.

Fresh ordinary omission of **only** `broadcast use vstd::set_lib::range_set_properties;` from the unchanged selected R2 manual proof passes 25 verified queries/0 errors, with exactly 12 checked executable and8 checked proof bodies, no inferred helpers. Source projection and original contracts/specs pass the existing R2 fidelity checks. Receipt: `receipts/R2-omit-range-broadcast.json`, SHA256 `6e3fd865b14fc2c653c333f38e24841e6cf8c428d1e5444247f782f76e29d887`; private output `/tmp/redb-r2-REVIEW-4ac48a9a-omit-range-broadcast`. Accepted Git f4415999 already includes `range_set_properties` in `group_set_lib_default`, itself in `group_vstd_default`.

Parent correction: retain “one literal broadcast” in the historical annotation inventory, distinguish it from necessary work, and never credit removing it to new automation. The extensional equality remains a real demand; the existing count ablations and fresh C0 weak-summary control support it. This finding does not invalidate the application proof or the common Set-pair interface. No conclusion about necessity of the two facade broadcasts was tested.

## Source, domain, mathematics and modularity

R2's existing replay authenticates the completed private snapshot's eight accepted tools+receipt and both clean source files against `private.json`; it checks all12 executable projections against exact original item offsets/hashes. I inspected those projections, genuine private `heights: Vec<U64GroupedBitmap>` and `len:u32/data:Vec<u64>` fields, original sources and concrete consumers. Only proof annotations are erased by the narrowly defined projection. Added named results, visibility/import context and Verus contracts are disclosed. Initial, corrected and manual fixture contracts/specs remain aligned, apart from proof triggers. No runtime algorithm is replaced.

The invariant allows any nonempty representable-height tree, arbitrary multiword levels, root logical length<=64, covered storage, allocated padding in relevant words, parent logical length=ceil(child logical length/64), and parent membership=child-word fullness. There is no fixed height, fresh target or one-word assumption. Zero logical lengths remain valid; no target index exists then. Extra storage words are unconstrained and framed. `word_change` frames the complete word vector; `path_frame` frames every stored physical bit outside the ancestor path, with all lengths unchanged. `tree_wf` restores every summary edge. `mutation` gives exact logical leaf insert/remove and count delta conditional on prior membership, including repeated set/clear. Count excludes padding and spare words. Initialization, resize, serialization, search and concrete counting implementations are outside scope.

R2 dependencies are checked and acyclic: word bit laws -> word frames -> original scalar mutations; layer/leaf edits -> pending-edge facts; propagation plus ancestor recursion -> original update_to_root; checked transitions plus finite-set observation -> final set/clear composition. Recursive proofs have decreases; there is no target theorem assumed as a callee. `pending`, `pending_at`, `ancestor`, frame predicates, fuel, triggers and transport assertions are charged mechanical support, not extra semantic authority. Shape/storage/hierarchy and membership meaning are authored semantics, not automatically discovered invariants.

F2's six exact original method texts (From and five policy methods) match pinned peer_id.rs. Private tuple visibility and public closed count observation remain; Copy/Clone are retained and Clone is checked. Omitted Debug/comparison derives, serde/collection context and peer_ids are not granted authority. FromSpecImpl supplies existing erased trait-extension definitions; the original From body proves the inherited specification. The spec constructor directly uses the genuine tuple field, without recursion through From or a new trusted contract.

From/total cover all usize including0. The subtraction-dependent methods require only n>0, covering the entire positive domain including usize::MAX. No u16/small-federation bound is introduced. Arithmetic contracts use mathematical integers. The default verifier architecture is Either32Or64; native compilation here is x86_64 with Rust1.99.1, not a native equivalence theorem or execution on32-bit hardware.

Independently, f=floor((n-1)/3) gives 3f<n<=3f+3, q=n-f, h=f+1 and d=q-1. For finite A,B subsets of an n-member U with sizes>=q, inclusion-exclusion and |A union B|<=n give |A intersect B|>=n-2f>f. A quorum has size>f, and a selected f+1 set also cannot be contained in a faulty set of size<=f. These conclusions require the explicit finite-set/universe/count/fault hypotheses. They do not establish NodeMap distinctness/count semantics, cryptographic validity or a full consensus invariant. Arithmetic and original scalar callee propagation already work automatically.

The reviewed projections use ordinary backend/vstd specifications and checked modular preconditions. No new assume/admit/external body, trusted law, narrowed domain or lifetime/trait/WF/mode/body/erasure bypass appears. This is source-connected verification within the accepted Verus trust boundary; it is not whole-project or general native-Rust certification.

## Fresh application outcomes and cost

`MANIFEST.json` records every command, exact engine, raw log hashes, full failing identities, body counts, empty helper/goal-assistance inventories and immutable receipt copies. All existing replays ran under Python -B -O with explicit-exception gates, ArtifactGuard, LoggedCommands and replay_receipt. Runtime receipt `761ae34bc811fe116ddd23d6347d57bef96223dd208837b05acaa744591d2e05` pins engine f4415999/schema39/policy75/cache78 and all eight tools. No ignored target binary, compiler/vstd rebuild or network access was used.

R2's11 expected outcomes completed in28.788 compiler-wall seconds:

- Eligible ordinary/infer baseline: each8 verified queries,6 errors across exactly BtreeBitmap::{set,clear,update_to_root} and U64GroupedBitmap::{set,clear}; zero certified bodies. These are body failures, not frontend/resource negatives.
- Manual ordinary/infer cold/infer warm: each25/0, exactly12 exec+8 proof bodies;2.221/2.903/2.901 seconds. Identical annotations, no generated helper or goal assistance.
- Leaf, repeated-count and nonpath-frame semantic negatives: each25/1, failing only controls::{wrong_set_membership,wrong_repeat_count,wrong_clear_frame}, respectively. Healthy selected methods/helpers remain in the inventory; each --infer-goals --export suppresses its interface. No mixed safety, invariant, frontend or resource failure.
- Compiled/exported provider:27/0,14 exec+8 proof bodies,2.857 seconds, including two checked facade bodies. Separate ordinary/infer clients:3/0, exactly3 exec/0 proof,0.873/1.475 seconds. The provider is full-body verified before import.

F2's17 expected outcomes completed in21.315 compiler-wall seconds:

- Corrected baseline ordinary/infer fails exactly quorum_intersection, quorum_has_honest and one_honest_margin (9/3); all numeric/conversion dependencies individually pass, zero certified bodies for the failed crates.
- Manual ordinary/infer plus unchanged repeats each12/0, exactly8 exec+3 proof. Compiler times ordinary1.194, infer1.704, ordinary repeat1.075, infer repeat1.715 seconds. Ordinary imported client2/0, exactly2 exec/0 proof,0.966 seconds. No inferred helpers.
- Strengthened intersection and relaxed-fault semantic controls fail only their intended theorem, ordinary/infer (11/1); n=1, U=A=B={0} refutes intersection>f+1, and A=Bad=U refutes honesty when |Bad|<=q. Other functions' SMT breakdowns are healthy.
- Zero control fails only NumPeers::max_evil at original total()-1, ordinary/infer (7/1). It is a safety-negative. Constructing zero and observing total zero succeed in the imported positive client. Removing a precondition here does not mean From rejects zero.
- Each of union_lens, union_subset, honest_subset and margin_subset was separately omitted in ordinary mode. Each fails only its enclosing theorem at its postcondition (11/1), all dependencies healthy. This independently confirms necessity in this proof, not universal proof minimality.

All goals-enabled failed exports are absent. F2 ordinary goals-disabled failed --export does leave a native .vir, as disclosed; it has no certification authority and was never imported. Only the successful ordinary provider supplies the F2 import. R2's provider is successful and goals-enabled. Empty helper inventories mean inference precision/exactness obligations have an empty candidate set, not that checks were disabled. Repeats establish stability, not cache-hit/speedup evidence.

The range-broadcast omission adds2.117 seconds, making29 fresh application/review compiler invocations and52.220 seconds. Supplemental C0 adds4 invocations/6.500 seconds, for33 invocations/58.719 seconds total. Costs exclude hashing, copying, Python setup and review reading; no RSS/statistical performance claim. Initial default sandbox failed bwrap loopback; authorized escalated invocations succeeded. One display-only attempt requested absent R2 times-ms data and was corrected; no compiler result/receipt changed. No blocker approached the45-active-minute limit; no prerequisite was opened.

Historical own input hashes match both RESULTS.json deliveries. I authenticated only permitted completed receipts/logs: R2 accepted plus count-ablation/count-broadcast/count-equality and F2 accepted. Their exact hashes/labels are in MANIFEST; no historical worker checkout/cache was read. Historical source paths were rebound to own hash-identical Git files. Supplemental C0 was likewise reconstructed from own sources rather than executing its controller's parent-checkout reads.

## Corrections, remaining manual work and consumer limits

R2 C1 openly fixes finite Set::new returning Option<Set> by range(...).filter(...); this is the intended bounded logical set, not a domain change or automation gain. Mandatory decreases and dimension trigger setup are charged mechanical work. Earlier frontend/resource/setup failures are not semantic controls. F2's missing existing FromSpec adapter is a separate erased-specification correction, not proof reduction or trusted conversion authority.

R2 literal inventory independently matches:8 helpers,15 static calls (including recursion),9 exec-local proof blocks,9 snapshots,37 proof assertions including9 quantified blocks,6 loop-entry predicates and3 exit predicates,3 BV directives,2 fuel directives,2 triggers,1 literal broadcast,1 exec-loop decreases and1 proof-recursion decreases, plus original spec-recursion decreases. Categories overlap. Helpers are word_bits, word_frame, layer_edit, leaf_edit, clear_not_full, finish_mutation, propagation, ancestor_next; call counts respectively2/2/2/2/2/2/1/2. Their bodies and expanded pending_at invariant remain mechanical work. Seven accessors/scalar methods need no extra local blocks. Insert/remove cardinality is already automatic after set equality, with zero local length-lemma calls. The broadcast finding distinguishes literal cost from necessary cost.

F2 retains exactly3 authored theorem bodies filled with4 existing set-lemma calls and2 conditional blocks; zero local assertions/auxiliary lemmas/arithmetic calls/invariants/snapshots/fuel/solver directives/broadcasts. The existing inclusion-exclusion lemma is checked broadcast library proof; subset-to-length is checked ordinary library proof. Both are mechanical. Client semantic observations/theorem applications and the two erased FromSpec definitions remain separately authored work. Infer removes ZERO in both applications.

Pinned actual consumers were inspected: BuddyAllocator record_alloc_inner's allocator.set(page_number), free_inner's allocator.clear(page_number), and BtreeBitmap::alloc's self.set(entry); Keychain::is_complete's count<threshold and ConsensusEngine::validate_signed_session_outcome's len!=threshold. R2 imported wrappers prove conditional membership/count compatibility; they have no constructed tree or complete allocator proof. F2's ordinary imported observation wrapper uses the actual count guards with explicit set/count assumptions. Neither verifies complete source callers, allocation splitting/merging/search, NodeMap/crypto/async consensus or crash consistency.

## Supplemental C0: concrete common demand, bounded future mechanism

User authorized the completed C0 evidence during review. I read committed411e4a97 via own Git, authenticated receipt `3ead0adf8ac24dced937ca483e01c1644efa093ba483dc32bf9112fe678205e4`, and inspected/reconstructed its exact transformations from own unchanged R2 manual/F2 generated manual sources. Completed helper equals the committed pair-prototype.rs byte-for-byte. No live parent file/controller/cache read occurred. `C0_MANIFEST.json`, copied parent documents and both raw receipts preserve the distinction from base4ac48a9a.

Fresh independent C0 replay `/tmp/practical-set-pair-REVIEW-4ac48a9a` reproduces all4 outcomes: R2 26/0 with12 exec+9 proof; F2 13/0 with8 exec+4 proof. Weak extensional summary fails only r2_no_extensional::finish_mutation (25/1); weak cardinality summary fails only the three f2_no_cardinality corollaries (10/3). Helper bodies are unchanged and individually healthy; failed whole crates certify zero bodies. Exact source/contract/proof replacements, native identities, empty inferred helpers and per-function dependency health were checked. Fresh receipt SHA256 `8a641645807a815f53088cd26df204ccc7a900f29eeaa10118b4b5f00f0fb2b1`.

The single interface is ordinary checked `checked_set_pair<A>(s:Set<A>,t:Set<A>)`, without preconditions, yielding:

```
s.subset_of(t) ==> s.len() <= t.len()
s.union(t).len() + s.intersect(t).len() == s.len() + t.len()
(s == t) == (forall|x:A| s.contains(x) == t.contains(x))
```

It is a coherent finite-Set pair observation interface, applicable without project names or new semantic assumptions. The summary controls show causal need for different observations at real unchanged application sites. Identical lemma names or identical consumed conclusions are unnecessary. This **does** justify a concrete bounded common selector proposal; the pre-supplement absence of evidence for one is superseded. It does not show that every emitted fact is needed or that merely combining conjuncts is already an automation improvement.

Exact proposed typed instantiations and literal targets, with no engineering in this checkout:

- R2 Set<int>: pair allocated(b) with `if value { allocated(a).insert(i as int) } else { allocated(a).remove(i as int) }`; remove the one explicit extensional equality assertion in finish_mutation. The prototype instead adds one explicit pair call, so it has no net author-glue reduction there. Do not credit the already-redundant range broadcast.
- F2 Set<usize>: pairs (a,b), (a.union(b),u), (a,bad), (chosen,bad) at their respective theorems; remove4 existing lemma calls+2 conditional blocks. The prototype replaces them with4 explicit pair calls; those calls plus the generic helper's library calls/conditionals/assertion are charged replacement work.
- Future selector may bound collection of pure parameter-rooted Set terms from unchanged contracts and visible specification observations, including the observed conditional term and union closure of intersection operands; propose same-element-type pairs through the checked generic interface. Preserve the full original function projection, normal body/lifetime/trait/WF/mode/erasure checking, exact helper identities and failed-export gates. No project recognizer or contract/executable rewrite.

Stop unless injected checked assistance actually removes those literal author annotations without hidden replacement glue on BOTH frozen applications. Reject gains from already-automatic arithmetic, insert/remove length or redundant broadcasts. Maintain focused ordinary/infer/import, cold/warm, weak-summary and publication checks; fail closed on ineligible/over-budget term collection. Stop each blocker at45 active minutes; no foundations, representation chain or synthetic application fixture. A successful selector would demonstrate limited common causal reuse, but one R2 set assertion alone is **not substantial overall R2 reduction**. Word/nested-Seq frames and ancestor-loop transport still dominate and need a separate concrete value checkpoint, not automatic scope expansion.

## Replay and handoff

From an own hash-identical checkout, use fresh unique /tmp names:

```sh
python3 -B -O source/tools/research/practical_redb_fedimint/redb_tree/replay.py /tmp/redb-r2-REVIEW-UNIQUE
python3 -B -O source/tools/research/practical_redb_fedimint/fedimint_quorum/replay.py --output /tmp/fedimint-f2-REVIEW-UNIQUE-corrected --scenario corrected_baseline
python3 -B -O source/tools/research/practical_redb_fedimint/fedimint_quorum/replay.py --output /tmp/fedimint-f2-REVIEW-UNIQUE-manual --scenario manual --clients --warm
for scenario in intersection_negative fault_negative zero; do
  python3 -B -O source/tools/research/practical_redb_fedimint/fedimint_quorum/replay.py --output /tmp/fedimint-f2-REVIEW-UNIQUE-$scenario --scenario "$scenario"
done
for omit in union_lens union_subset honest_subset margin_subset; do
  python3 -B -O source/tools/research/practical_redb_fedimint/fedimint_quorum/replay.py --output /tmp/fedimint-f2-REVIEW-UNIQUE-omit-$omit --scenario manual --omit "$omit" --mode ordinary
done
```

To reproduce the extra R2 check, copy the original own manual.rs to fresh private output, remove only the single literal broadcast specified above, retain every other byte, call existing R2 fidelity(meta,fixture), and run the argv/environment in receipts/R2-omit-range-broadcast.json with output/cache prefixes rebound. It uses --edition=2024 --no-cheating --output-json --time and the same pinned snapshot runtime. Existing LoggedCommands/replay_receipt/ArtifactGuard were used, not a new evidence framework.

For C0, hash-copy the four completed application/control files authenticated in C0_MANIFEST into fresh private output. Run exact argv/environment from receipts/C0-review.json with private output/cache prefixes rebound, using existing execution/guard utilities. Parent experiment.py is inspected evidence, not an executable replay against its historical root. No access to that root is needed: positive sources reconstruct by the literal annotation replacements above plus the committed helper, weak controls change only its corresponding ensures. Authenticate all inputs first; do not overwrite completed artifacts. Raw logs/binaries/interfaces stay in the named private outputs; immutable receipt bytes are committed here, with log hashes and commands in manifests. No failed ordinary F2 artifact should be imported.

Ordered local evidence commits:7399913b (scope before fresh trials), ef811b86 (fresh application/control receipts and manifest), df8742d6 (authorized supplemental C0 authentication/replay), then the commit containing this verdict. Every commit uses command-local Codex/codex@openai.com identity and changes only this new evidence directory.

R1 div_ceil, F1 saturating_div and F0 map remain unsupported/unproved and counted. Native0/HMAC1/1 and HMAC/native foundations remain deferred/unachieved limits; accepted bundle publication=false and goal_complete=false are unchanged. Application proof acceptance and C0 demand evidence do not imply a whole database/protocol/native theorem, substantial mechanical reduction, a completed Infer gain or global goal completion. No production edits, push/PR, delegates, broader matrix or prerequisite work is authorized by this verdict.
