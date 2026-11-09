# First native redb cell: decision and bounded next action

## Decision

**All 18 cells OPEN; no acceptance or new proof credit.** Strongest component
evidence is reviewed header selection plus conditional reencoding/interrupted
writeback, not an established nearest final cell. No inspected record supplies
complete native operation/history simulation, accepted storage/resource boundary
and final acceptance support. There is no defensible percentage, ETA or proof
that lookup/one-phase is cheapest; this is not demonstrably one helper/probe away.

**Recommended first-cell target: two-phase acknowledged durability.** This is a
priority, NOT a proven smallest remaining proof.
It exploits the preparation barrier without one-phase's checksum-integrity burden.
Valid roots, buffers/barriers/errors, reclamation and restart remain missing;
this cannot be ranked objectively closer than recovery or error correctness.

## Evidence credit, narrowly stated

* Selector review: six bounded source-corresponding bodies; old/prepared selected
  SLOT payload, not a whole header/database. Parse/validity/corruption/integrity
  premises are supplied. Publication's four native statements omit actual I/O.
* Reencoding review: unbounded-finite SAME-target writeback, starting with an
  encoded dirty header at one accepted length, fixed geometry/slots/mode/flags.
  Valid parsed normal slots can contain padding that encoding zeroes/rehashes:
  parse success is NOT encoded-origin. Flags are preserved, not cleared; restart
  must reparse corruption flags relative to the current primary. Native retry,
  availability and mixed histories do not follow. No historical controls rerun.
* Leaf100 repairs partition proof without narrowing inputs; admission101 derives
  supplied-page admission; completion103 independently checks conditional
  branch/leaf association. None proves native fetch identity, reachable intervals,
  arbitrary trees, native lifecycle/mutation/reclamation or persistence. Its 38
  outcomes are not 38 native functions.
* Cycle1 is real but hypothetical one-token/two-slot IR transport. Authority is
  supplied, not produced from Rust; operations are not native heap/Arc/Drop/loan
  semantics. Partial E1 and E2-E6 remain open; narrow hardening is not a producer.
* Native106 lacks frozen-engine build/source correspondence and original unsized
  Arc lowering. Its inspected build compiled vstd with existing rust_verify, NOT
  rust_verify itself. Source inventories do not prove compiled-binary identity.
  Receipt uncertainty and native resource semantics are independent obligations.

## Dependency cuts

Here `N` is the authenticated recovered redb source root:
`/private/tmp/verus-crash-consistency94-20261105/native-leaf-lookup-007/recovered-native/redb/src`.
Recovered bytes do not repair path/Git continuity.

```
original SCOPE / S0 / exact candidate + reviewed trust + runtime/source binding
                              |
native operation/resource/allocator invariants -- native storage simulation
                              |                    |
                valid root/history relation -- commit + acknowledgement CFG
                              |                    |
                       actual parse/select/recover/restart
                              |
                    ONE property/history theorem
                              |
          controls + regressions + imports + independent FINAL review
```

Necessary cuts:

1. **Native boundary:** checked bodies or individually reviewed bound primitives;
   caller premise establishment and resource authority, not application axioms.
   S0 whole-native/full-scope/import/export barriers remain.
2. **Storage simulation:** application buffers versus backend-visible versus
   durable bytes; partial/out-of-order persistence; whole-byte selector; write
   frames; sync/errors; length/growth/shrink; locks/poisoning. Power loss differs
   from process-only restart.
3. **History invariant:** establish validity initially ONCE; preserve valid roots,
   geometry, allocation/reclamation and recoverability through the native required
   operations/errors. No fresh helper validity assumptions. Acknowledged history
   survives later operations/overwrites; do not freeze every header byte forever.
4. **Recovery:** actual parse and policy, encoded-origin or a sound generalization
   of the retry theorem, native flags/cache/length/error effects, allocator rebuild,
   operational validity and conditional eventual success after crashes stop.
5. **Acceptance:** exact source/runtime/cfg/primitive bindings, burden inventory,
   fresh imports where applicable, semantic/crash/error controls, ordinary
   regressions and independent final review. Compiled-binary binding remains open.

Additional row burdens:

| Row | What the strongest nearby evidence still does not prove |
|---|---|
| Crash atomicity | Native root denotes a legal complete history prefix, not just an old/new slot. |
| Acknowledged durability | Public Ok follows the barrier; updates/reclamation/recovery preserve acknowledged history. |
| Recoverable-state preservation | Native allocation/free/growth/shrink preserve recoverable roots/pages, not a supplied protected set. |
| Recovery, including interruption | Real parse/rewrite/flags/allocator/cache establish operational state and retry premises, not perpetual refusal. |
| Error correctness | Public error/abort/refusal/poison outcomes, including after publication; Err is not rollback. |
| Logical KV refinement | All required native operations refine maps over arbitrary supported trees/bytes, preserving unrelated entries across restart. |

**One-phase:** validation coverage and scope-permitted no-undetected relevant
corruption, not universal checksum injectivity. **Two-phase:** preparation/order
and surviving roots, no unnecessary checksum premise. **Mixed:** arbitrary finite
switches, whole-byte mode/primary coherence, both predecessor modes. Both policies
in individual calls do not prove history induction.

## Is Arc immutable-view production a universal prerequisite?

**Not for every protocol lemma, nor necessarily as that implementation mechanism.
No inspected existing interface discharges it for a complete native cell.**

* Header/order proofs can use checked mathematical interfaces without immutable
  Arc fetch. Single-process `write_header` immediately uses a WritablePage mutable
  loan/copy, not `PageImpl::memory`. Diagnostic107-D1 is not a necessary ordering
  prerequisite for every source-connected protocol lemma.
* A real modular checked native contract could discharge a caller dependency
  without reimplementing its producer. But current leaf/branch supplied slices
  establish content predicates only; cycle1 supplies hypothetical authority; the
  reviewed primitive laws are proposals, not adopted bindings. None links the
  actual receiver, buffer identity/generation, live loan, page metadata and fetch.
  A borrowed-slice substitution or equal Seq bytes cannot prove that dependency.
* Complete rows retain the original operation set and S0. Actual immutable page
  reads and mutable/lifecycle dependencies therefore need a checked semantic
  route or an already legitimate reviewed boundary; this assessment found no
  ready route, not that a bespoke Arc producer is the only possible architecture.
  S0 does not require every std feature: bind required dependencies or prove
  genuine unreachability. No silent S1 or growth/shrink/reclamation exclusions.

Independent semantic gaps: weak-CAS progress, panic hooks/runtime, allocator
effects and final drop/glue. Bind needed instances or prove exclusion; receipts
and Arc diagnostics do not settle them.

## ONE smallest complete milestone

**M1: accepted native acknowledged-durability / two-phase cell**, for one exact
candidate and all finite two-phase Immediate histories under the UNCHANGED SCOPE.
For every actual successful public commit, every permitted subsequent required
operation/error/crash and eventual successful recovery preserves an abstract
history containing that acknowledged transaction, with legitimate overwrites.
Quantify over supported sizes/keys/values, both crash kinds and interrupted
recovery; validity only initially. Include reachable growth/shrink/allocation and
errors; no hidden assumptions, S0 selection or conditional API contracts.

Closure needs the cuts and independent FINAL cell acceptance. Supporting
preservation/atomicity lemmas do not close their rows. Target only, not acceptance.

## ONE implementation task and authority

**Implement an outcome-complete, source-connected native commit-to-storage
simulation boundary**, rather than another leaf fixture or frontend audit.
Anchors: N/tree_store/page_store/page_manager.rs:1797 (`commit`), :1617
(`write_header`), :879 (`read_header`); N/transactions.rs:1752 (public commit),
:2061 (`durable_commit`), :2139 (memory commit and postpublication epilogue);
N/tree_store/page_store/cached_file.rs:490 and :596 (buffer flush then sync).
Use `source/tools/redb_crash_consistency94/header_order_model.rs:6` and :38,
`commit_order_model.rs:25` and :39, plus the reviewed selector/reencoding contracts
as conditional lemma interfaces, not as native oracles.

Proof scope: actual CFG, mode, both full-header writes/barriers, shrink/resize,
locks/cache and postpublication fallible work through public Ok. Track success,
partial failure and crash successors; never erase intervening native operations.
Restrict the history mode to two-phase legitimately, not the required operations.

Premises: established operation invariant and real storage/resource contracts;
prepared reachable payload, no outstanding mutable access, exact encoding/length/
frame, barriers/outcomes. These native premises are NOT all established. The
selected-byte protection theorem cannot assume native writes/reclamation obey it.

Semantic control: remove/delay FINAL successful sync but retain public success
on a lawful nonempty image. A permitted power-loss choice must refute the unchanged
claim; restore identical positive. Later errors must allow fully persisted but
unacknowledged completion, not rollback. This tests acknowledgement placement.

**NO-GO now:** this assessment authorizes reading/reporting, not implementation;
native resource/storage bindings are not accepted. Request ONE separately
budgeted isolated candidate for this exact simulation, pinned inputs and explicit
run/build limits. No parent edits, new laws/producers, admission/export change or
automatic follow-on audit. Bounded acceptance: outcome-complete conditional
simulation with explicit undischarged premises, OR localized unsupported-boundary
result and stop; neither is M1. Needed primitive/trust/producer authority requires
a separate decision, not a hidden requires clause. Diagnostic107-D1 remains
separately unapproved; this proposal does not launch or replace it.

## Authentication, limits and accounting

Start 2026-11-08 03:05:20 UTC (November 7, 21:05:20 America/Chicago); discovery
cutoff 03:12:20; deadline 03:15:20 UTC. INPUTS.json records all 33 used inputs,
781418 bytes, exact SHA256/sizes. Primary hashes preceded substantive reads;
manifest creation followed some initial reads and matched earlier hashes. Scoped
AGENTS checked; native AGENTS hashed before reading, added to manifest afterward.
No broad tmp searches/copies. Endpoints do not certify transient nonmutation.

Five used native .rs hashes match recovery review pins; this does not authenticate
whole archives/compiler closure. Old GOAL/STATUS/TAKEOVER review drift remains
qualified, not new drift. All current inputs checked at exit; see EXIT.json.

Zero builds/tests/verifier/solver/compiler/native runs, network, Git writes,
source edits, delegation, trust/admission/export changes or proof credit. One
read-only git status. Node scratch mkdir denied EPERM; permitted shell scratch
creation succeeded without authority expansion. No prohibited work resumed.
EXIT.json retains final byte counts/free disk/hashes; scratch is assessment only.

**Action boundary: authorize the precise native protocol/storage milestone or
hold. Do not commission more unrelated helper counts or an automatic audit chain.
All eighteen final cells remain OPEN.**
