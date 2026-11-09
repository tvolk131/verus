# CHECKED CONDITIONAL MODEL-ONLY: Fedimint atomic transaction/replay

Date: November 2, 2026. This is an AUTHOR-OWNED abstract protocol proof, NOT
verification or refinement of the original async consensus method, modules,
RocksDB, its Rust/C++ bindings, recovery, storage hardware, or fsync.
The repaired candidate77 baseline still awaits independent acceptance.

## Bounded result

`model.rs` defines a single current-open-session index and one in-flight attempt.
For every finite trace admitted by `step`, from `initial(key, frame)`, checked
induction establishes `valid_state`, key constancy, and the unrelated-state
frame. Recovery has either no marker and no committed effect bundle, or the
matching marker and exactly one matching committed effect bundle. A matching
item AND peer replay returns accepted without changing ANY durable state.
A different item OR peer at that index returns rejected without changing it.
The result also covers committed transactions whose result/ack is not observed:
recovery and matching replay retain exactly the one already committed bundle.

Scope is the full selected authored crate, with 1 checked executable body,
16 checked proof bodies, 15 specification definitions and exactly 18 verified
queries (the recursive specification also has a termination query). These are
model identities, not bodies from Fedimint. There are ZERO inferred functions
and ZERO proved native source correspondences. `replay_decision` is deliberately
authored, not an extracted unchanged native slice. `scope.json` freezes every
function's declaration/body identity and the whole model hash.

## Explicit author-owned boundaries

- **Atomicity:** `commit` selects the OLD entire durable image or the NEW image
  containing BOTH the staged bundle and its marker. `takes_effect=false` models
  an unsuccessful transaction that wrote nothing; `true` models linearization.
  Crashes during commit are represented by selecting either branch and crashing.
  Atomicity is an authored DB abstraction, not a proved RocksDB guarantee.
- **Durability/recovery:** separately, `crash` retains the complete `disk` image
  and discards only volatile phase, request and reply. A linearized commit is
  recoverable even if its result/ack is lost. This requires reliable persistence
  and recovery without losing, fabricating, or tearing committed images. It is
  NOT established by the model or by inspecting `set_sync(true)`. Fsync durability
  alone does NOT imply transaction atomicity. An atomic but non-durable commit
  could disappear on recovery; that transition is intentionally not admitted.
- **Module write confinement:** the module's DB writes are represented by one
  opaque bundle token. Staging changes only volatile phase/request, NEVER disk.
  No module can independently commit, write a different DB, mutate the marker,
  or introduce a durable effect before this transaction commits. A native
  successful attempt need not make exactly one KV write: the sequence counts
  abstract committed processing bundles, NOT RocksDB writes or funds. It is a
  ghost accounting observation, not an invented native database table.
- **Lifetime:** `AcceptedItemKey` is an index, NOT a `(session,index)` native key.
  `key` is fixed during this model's current marker lifetime. Initialization
  means no prior bundle for this selected index in that lifetime. There is no
  marker deletion, session close, index reuse or pruning transition. Native
  `complete_session` DOES delete these markers. Old effects need not disappear.
  Extending beyond that boundary requires a new explicit lifecycle contract;
  no global forever-exactly-once claim follows.
- **Serialization and equality:** at most one attempt is in flight for this
  selected index. Native ordering or conflict validation must justify this
  abstraction; concurrent snapshots are not proved. The payload's `u64` item is
  a symbolic equality-class label, NOT an encoding or hash of `ConsensusItem`.
  Equality reflection for real items/peers and reliable marker reads remain
  unproved. No cryptographic or collision-free assumption is made.
- **Frame:** `disk.frame` represents arbitrary unrelated state outside the
  selected bundle footprint. All model transitions preserve it. No native
  module-footprint or cross-key isolation theorem is claimed.

All of these restrictions occur in checked definitions/transition semantics,
not in `assume`, `external_body`, imported application summaries or an axiom of
the desired conclusion. The positive file has none of those escape hatches.
Ordinary Verus/SMT/specification-library trust remains, including reused sealed
vstd artifacts; this cycle does not freshly verify all vstd.

Phases 0/1/2/3 mean idle, marker read absent, item effects staged, and matching
marker staged. Reply 0/1/2 means unobserved, accepted, rejected. Stage 2 has an
effect WITHOUT a marker only in VOLATILE staging; its recovered disk is empty.
Stage 3 can abort or crash without persisting either. These are intentionally
permissive abstract abort transitions, not exact native control-flow claims.

External commit hooks, notifications, metrics, file/network effects, audits,
panics, rejection details, funding, fees, cryptography, consensus safety and
liveness are excluded. No result here establishes native atomicity or fund safety.

## Counterexample controls

The positive file CHECKS witnesses that durable marker-only and effects-only
writes remain unpaired even under a durability-preserving crash. It also CHECKS
a witness that bypassing dedup and committing twice yields two bundles under
one marker. These weakened transitions are NOT part of `step`.

Fresh compiler controls reject the corresponding false safety claims, acceptance
of a different item or peer, durable leakage before commit, double execution
after a lost ack, and an unchecked u64 increment. Semantic controls require
actual verification errors, not parsing, timeouts, resource failures or panics.

## Runner and legacy export behavior

Run from the repository root, choosing a NEW output directory each time:

```
python3 -B -m unittest discover -s source/tools -p test_fedimint_atomic_replay.py -v
python3 -B -O -m unittest discover -s source/tools -p test_fedimint_atomic_replay.py -v
python3 -B source/tools/check_fedimint_atomic_replay.py --output /private/tmp/atomic-fresh-normal
python3 -B -O source/tools/check_fedimint_atomic_replay.py --output /private/tmp/atomic-fresh-optimized
```

The runner reuses sealed tools77; no Cargo/compiler rebuild or dependency fetch.
It audits all baseline file/source/build/runtime/toolchain/native/guarded-input
ledgers, plus the four locally inspected dependency files, before AND after
replay. Git context is sanitized and optional locks disabled. Native/package,
runtime, identity and parent files remain read-only. Each mode has 11 cases:
two positive default-erasure compiles and nine expected failures. The receipt
binds zero errors, whole selected scope, exact counts/identities, strictly typed
flags, model/body/pin/tool/source hashes, commands, environment and artifacts.

No `#[verifier::infer]`, `--infer-goals`, provider import or candidate cache is
used. Ordinary `.vir` export is legacy behavior: `interface_written` is false
even when the ordinary artifact exists. In particular, a failed proof can still
leave a freshly exported `.vir`; the seeded failure probe records whether the
earlier ordinary export was replaced. We do NOT call this inference-aware
publication, stale-artifact cleanup, safe import/export, or a warm-cache test.
These artifacts are not imported by the proof. Fresh directories and successful
whole-scope verification, not artifact existence, determine positive acceptance.

See `SOURCE_REFINEMENT.md` for inspected steps versus unproved correspondences,
`REPORT.md` for this run's outputs and failed attempts, and the delivery manifest
for the exact incremental reconstruction. Old baseline suites are audited only,
not reported as freshly executed by this cycle.
