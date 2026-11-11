# Source, assumptions, manual input and trust inventory

| Item | This worker's status |
|---|---|
| Original carrier | Exact pinned declaration, all seven derives, private genuine NonZero<u64> field; declaration SHA in BASELINE.json |
| Runtime derives | Rust1.99.1 original generated bodies unchanged; six method-body hashes, full native MIR hash; Copy/TrivialClone/StructuralPartialEq marker impls have no local comparison bodies |
| Original BorTag Eq | Checked on accepted bundle, one inferred helper, three fresh exact proofs on miss and hit; unrelated CounterKey equivalent also checks |
| Original BorTag ordering | Not checked: macro gate, then separate typed request gate; no certified original ordering VIR body |
| Actual six operator/trait calls | < <= > >= /partial_cmp/cmp authored ordinary clients, genuine borrowed BorTag operands, failed intended postconditions 2/6 |
| Numeric carrier order | Direct genuine NonZero partial-order contracts 5/0; direct cmp numeric contract 0/1; no accepted Ord theorem |
| Exact foreign closure | Seven whole instantiated MIR bodies inventoried privately; includes inlined get body and u64 Cmp; no new foreign certificates |
| Domain | All genuine NonZero<u64>; no tag budget, <=2^61-1 premise, caller restriction or alternative runtime carrier |
| Authored semantic input | Closed value=self.0@; ordinary numeric postconditions on u64 views; diagnostic closed OrdSpecImpl/PartialOrdSpecImpl candidate definitions and selection markers; these stop before body proof |
| Mechanical proof input | No proof blocks, proof functions, lemma calls, snapshots, executable replacement comparison or new application premise in these comparison fixtures |
| Existing standard trust | Opaque NonZero View; existing new/get/equality/partial-order specs; core comparison trait contracts; primitive u64 ordering model; original derive-Clone checking |
| New assumed standard trust | None: no axiom, assume_specification, external-body stub, comparison law or transmute/view equivalence added |
| Existing Eq/value export compatibility | Prior acceptance uses ONE checked broadcast lemma plus client import; not repeated, not called proof-free |
| Ordering exports/imports | No successful BorTag ordering provider exists; no ordering import claimed; future closed-view client requirements remain in PROPOSAL.md |
| Failed export cases | Fresh selected exports/libraries absent for all four failed baseline cases; no claim of a newly exercised stale-output/poisoned-cache matrix |
| Cache | Existing Eq full-domain exact miss/hit rechecks only; no ordering cache result |
| Compiler identity | Accepted eight-file runtime, core/compiler byte guards, actual rustc -vV source pin; diagnostic sidecar never counted as an accepted compiler |
| Ordinary checks | --no-cheating, full selected crate; no disabled WF/mode/lifetime/trait/source/erasure checks; no trusted leaf/body bypass |
| TCB | Accepted Verus/frontend/macros and logical encoding; matching Rust compiler/lifetime checks; inherited vstd contracts and solver |
| Manual evidence work | Authored bounded replay/diagnostic/audit controllers, classified retained failures, compared source/MIR identities and wrote proposal; no engine implementation |
| Regression counts | Parent 717 compiler /149 VIR /6 lib /vstd2250 accepted historically, not rerun; production unchanged |
| Global boundary | Native0; HMAC1/1 UNPROVED; all44 minimal+3 phase/101writer effects OPEN; publication=false; goal_complete=false |

Only this directory changed relative to base131ffc35. No global schema/policy/
cache file, prior application/canonical/HMAC fixture, historical evidence or
standard-library model was edited. Version73/cache76 remains parent-owned.
Independent parent review is required; this is honest blocked source progress.
