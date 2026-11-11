# F2 result: full selected quorum policy proved with ordinary manual corollaries

The selected complete NumPeers policy and all registered quorum/honest-margin consequences are proved in the extracted verification fixture. This is an ordinary source-connected result, not a new Infer improvement or a native/whole-consensus theorem. No engine/vstd edits were made. Baseline Infer removes **zero** local proof steps. Parent must establish matching R2 demand before engineering a common layer.

## Freeze, fidelity and specification correction

Early freeze commit: `52063df6`, before any solver run. Starting checkout: `f870ea939f3727912f64e62ec0a2c2f6f21a4994`. Replacement was already registered in `0bd216ba`/`f870ea93`. `component.rs`, `fidelity.json` and `FREEZE.md` remain byte-identical to that early commit. Six executable method texts (five policy methods plus original From<usize>) are byte-identical to authenticated pinned peer_id.rs. Private tuple visibility is preserved. No formula, domain or target property was changed.

The first baseline exposed an **incomplete specification adapter**: original From<usize> inherits the existing Verus FromSpec contract. Merely supplying a count postcondition did not discharge that inherited contract. `adapter.rs` adds the existing `FromSpecImpl<usize>` implementation, with `obeys_from_spec=true` and a **closed** spec conversion `NumPeers(value)`. The unchanged original From body then proves the inherited equation. These are two erased specification definitions, not a new trusted external body, std axiom, assumed callee summary, field exposure or automation gain. Both uncorrected and corrected baselines are retained. No correction to the authored quorum properties or source bodies occurred.

Original Debug/Eq/ordering derives and unrelated serde/encoding/collection context are omitted; original Copy/Clone retained. Derived Clone is checked and inventoried; no comparison authority is used. `peer_ids` and its u16 cast are excluded. A public **closed** `count` observation connects public contracts to the private field; external clients never reveal it or access the field.

## Domain and achieved property

From/total include every usize, including0. max_evil, one_honest, threshold and degree require n>0, exactly the nonempty/subtraction-safety domain. There is no small-federation/u16/power-of-two bound. All positive representable usize counts are covered by the machine-method proof, including usize::MAX in the ordinary import client.

Authored semantics: n guardians, `f=(n-1)/3` with mathematical positive-domain division, `q=n-f`. Exact original safe machine results establish `3*f<n<=3*f+3`, `1<=q<=n`, h=f+1, d=q-1. Arithmetic in contracts is mathematical, not an added executable calculation. All five policy calculations and their checked callee-summary propagation are automatic with ordinary Verus under these authored contracts.

Explicit finite-set semantic assumptions: U:Set<usize>, |U|=n>0; A,B subsets of U; |A|,|B|>=q. `quorum_intersection` proves |A intersect B|>f. For Bad subset U with |Bad|<=f, `quorum_has_honest` proves A is not a subset of Bad and contains a nonfaulty member. `one_honest_margin` proves a chosen subset of size>=f+1 contains a nonfaulty member. The universe, fault bound, quorum interpretation and connection between concrete counts and set cardinalities stay authored assumptions. No crypto/BFT theorem follows without additional semantic work.

The verifier used its default Either32Or64 architecture model (own accepted source rust_to_vir.rs:559 and vir/prelude.rs). Native compilation was x86_64-unknown-linux-gnu using pinned Rust1.99.1; this is a conditional 32/64 Verus proof plus a native build, **not** a native equivalence or execution proof.

## Baselines, inventories and costs

Every application invocation used --no-cheating and ordinary lifetime/trait/WF/mode/body/erasure checking. No new inference annotation or synthetic helper was inserted. All runs have **zero inferred helper identities**. Thus exactness/maximality/soundness helper checks have an empty candidate set, not disabled checks. Goals-enabled successful runs publish an interface; all goals-enabled failures suppress it and remove the export.

| Fixture | Ordinary / --infer-goals | Full checked inventory | Compiler wall seconds ordinary / infer | Verification ms ordinary / infer |
|---|---|---|---|---|
| Frozen, missing FromSpec adapter | Both fail: original From inherited spec plus three corollaries | 0 certified (8 individually verified,4 failed functions) | 1.074 / 1.723 | 212 / 216 |
| Corrected empty-slot baseline | Both fail: three semantic corollaries | 0 certified (9 individually verified,3 failed functions) | 1.126 / 1.641 | 190 / 196 |
| Corrected arithmetic-policy diagnostic, no corollaries | Both pass | 8 exec,0 proof | 1.858 / 1.804 | 102 / 97 |
| Final full manual proof | Both pass | 8 exec,3 proof | 0.962 / 1.596 | 113 / 113 |
| Same full source repeated | Both pass | 8 exec,3 proof | 1.009 / 1.563 | 115 / 116 |
| Ordinary separate-crate import client | Pass | 2 exec,0 proof | 0.918 | 68 |

The 8 checked exec identities are NumPeers::{from,total,max_evil,one_honest,threshold,degree,clone} and policy_client. The three checked proof identities are quorum_intersection, quorum_has_honest and one_honest_margin. Closed count and FromSpec definitions plus fault/quorum are specification definitions, not executable proofs. Backend verified totals9/12 additionally include the generated tuple constructor; scope inventory does not list that constructor as a separate checked native body. Failed whole-crate runs certify **zero** bodies despite per-function solver successes; no full proof is claimed for them.

Cache was private and initially absent, and remained empty: no inference candidates generated cache entries. Repeated runs establish same-source stability, not an Infer cache hit or speedup. Final positive replay and semantic/safety controls ran under Python -B -O; all23 selected outcomes were subsequently accepted under -B -O with explicit exceptions, exact helper/body/failing-function inventories and healthy dependency checks. RESULTS.json records per-run hashes, costs and receipt locations. All32 compiler invocations including exploration took49.453 compiler-wall seconds; pin hashing/copying and Python setup are outside that number. No compiler build or broad matrix was run.

## Literal proof burden and necessity

Frozen baseline has three explicitly unsolved empty proof slots; it is not a zero-cost proof. The successful final manual variant fills exactly those slots from `manual.json`:

- quorum_intersection: one `lemma_set_intersect_union_lens(a,b)` call and one `lemma_len_subset(a.union(b),u)` call.
- quorum_has_honest: one `if a.subset_of(bad)` block with one `lemma_len_subset(a,bad)` call.
- one_honest_margin: one `if chosen.subset_of(bad)` block with one `lemma_len_subset(chosen,bad)` call.

Total: **3 authored theorem proof bodies,4 existing generic set-lemma calls,2 conditional blocks,0 local assertions,0 new auxiliary lemmas,0 arithmetic lemma calls,0 loop invariants,0 snapshots,0 reveal/fuel/solver directives/broadcasts**. No theorem or target result is assumed. An initial additional union-subset assertion proved unnecessary when omitted and was removed; it is not charged in the final burden. Omitting each of the four remaining calls separately makes exactly its enclosing theorem fail at its intended postcondition; all policy/conversion dependencies pass their individual checks. This demonstrates necessity in this proof, not global proof minimality.

Existing vstd helpers are imported from the accepted authenticated vstd artifact, not newly axiomatized or re-proved in this fixture. Full identities: `vstd::set_lib::lemma_set_intersect_union_lens<usize>` and `vstd::set_lib::lemma_len_subset<usize>`. The former is a proved broadcast function in group_set_properties; the latter is an ordinary proved subset-to-length lemma. Defaults are untouched; no suppression of existing insert/remove cardinality facts and no unneeded broadcast is used.

The seven assertions in policy_client are explicit semantic observation tests; its six exec calls exercise From and every original method. Imported admission_observation has2 exec calls (From, threshold),2 semantic theorem calls in one proof block,2 guard branches and2 result assertions. boundary_observations has4 exec calls and1 result assertion. These clients add no mechanical set lemmas. A final ordinary replay also guards the generated client source and imported .vir/.rlib bytes with ArtifactGuard; both provider and imported client pass. The2 FromSpec definitions are counted separately as erased conversion adaptation; the count/fault/quorum definitions and six exec contracts are authored semantic specification work. Mechanical work after accepted baseline Infer remains **4 calls+2 conditionals**, identical to ordinary manual proof. New automation removed nothing.

## Real admission connection, clients and controls

Pinned actual consumers: Keychain::is_complete, keychain.rs:129-134, rejects `partial.iter().count()<threshold`; ConsensusEngine::validate_signed_session_outcome, engine.rs:742-759, rejects `signatures.len()!=threshold`. Entire registered containing source files are authenticated and privately copied. The external ordinary client imports the successfully verified ordinary provider, constructs peers through original checked From, calls original threshold, uses those two count guards and derives intersection/honesty under explicit set/count hypotheses. It also proves zero construction/total observation and checks policy_client(1) and policy_client(usize::MAX). This is a numeric admission observation wrapper; neither actual NodeMap iteration nor cryptography nor complete source caller implementation is verified.

Semantic controls preserve healthy original methods and manual lemmas:

- Strengthening intersection to `>f+1` fails only quorum_intersection, ordinary/infer. Counterexample: n=1, U=A=B={0}, f=0, intersection size1.
- Weakening fault budget to |Bad|<=q fails only quorum_has_honest, ordinary/infer. Counterexample: n=1, A=Bad=U={0}.
- Removing only max_evil's n>0 restriction (retaining its exact executable body and exact result specification) fails ordinary/infer with arithmetic underflow at `(self.total()-1)`. Zero is constructible and is the unsafe case. This is a **safety-negative**, not a semantic counterexample and not an assertion that construction rejects zero.
- Four individual call omissions are mechanical proof ablations, not altered semantic properties or weakened helper contracts.

All failed goals-enabled runs have no published interface/export; no frontend, resource or safety failure is counted as a semantic negative. Ordinary goals-disabled --export can leave a native .vir even after proof failure; its Infer interface_written flag is false. This preexisting behavior is disclosed, not promoted as certification; only a successful checked ordinary provider is imported. The initial runner wrongly equated native export with Infer publication and rejected its own first receipt. That receipt is retained, the runner check corrected, and original baseline rerun. Two Python syntax-error starts before ablations invoked no compiler. Setup failures and all exploratory receipts remain recorded in RESULTS.json.

## Value checkpoint / handoff

See VALUE_CHECKPOINT.md in this directory. Concrete F2 gap is typed finite `Set<usize>` cardinality assistance: inclusion-exclusion and subset length monotonicity under actual quorum/fault hypotheses. The remaining conditions are mechanical proof structure, not new semantic inputs. Arithmetic and field/callee propagation already prove automatically, so proposing them as an F2 gain would be false. Parent must compare these exact obligations against actual R2 proof work; R2's already-default insert/remove laws or an unrelated synthetic set theorem cannot establish causal cross-project reuse.

No foundation/blocker chain was started; the conversion issue was resolved with the existing checked trait-extension mechanism, and direct corollary proof needed only existing lemmas. No blocker approached45 active minutes. Stop here before engine/vstd engineering. Whole selected F2 property achieved; shared automation milestone remains unachieved. Prior F0/R1/F1 results and files are unchanged and unproved, HMAC/native remains deferred and unachieved.

## Replay

From own checkout, use Python -B -O and a fresh /tmp output for each invocation of `replay.py`. The default scenario is the immutable original baseline. Use `--scenario corrected_baseline`, `--scenario policy`, or `--scenario manual --clients --warm`; use intersection_negative, fault_negative and zero for controls. `--scenario manual --omit union_lens|union_subset|honest_subset|margin_subset --mode ordinary` reproduces individual call omissions. Every run authenticates/copies the accepted eight-file bundle and registered clean pinned sources into private output, checks original six body hashes, uses ArtifactGuard/LoggedCommands/replay_receipt and rejects unexpected semantic outcomes. Do not use ignored target binaries. Raw sources, MIR/AIR, outputs, binaries and logs remain private /tmp artifacts.
