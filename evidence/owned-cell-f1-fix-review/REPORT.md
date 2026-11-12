# Independent F1 skipped-body correction recheck

**Request changes to complete F1 admission.** The exact `const_body` / `reveal_fn`
finding61c39752 is **closed** by9cc0f65c: the checks run on each item and every
local ancestor, with the expected precise early refusal. However, new independent
finding **F1-FIX-RECHECK-1** below leaves the broader “no admitted frontend-skipped
bodies” requirement **open**. No current native proof exploit is demonstrated.
Every experimental request still refuses. Keep the complete stage inactive.

Frozen root ad51b8c25f59900a6f79f9694a363024419cb12c; correction
9cc0f65c5e8ebb75f6858475277991e3bfd940d7; parent validation6f0b3b4c4
(full identity in AUTHENTICATION.json). Early scope/authentication commit09dec7606.
Only this NEW evidence directory was edited. No production edits, subdelegation,
live actor/parent/sibling reads, shared mutable caches/outputs or remote publication.
Required exact ancestor-instruction search found no AGENTS.md; all four required
frozen goal/worker/acceptance documents were read and hashed.

## F1-FIX-RECHECK-1: other frontend directives still count as ordinary checked bodies

**P1 for complete F1 acceptance.** `checked_owned_cell_admission.rs:639` excludes
the corrected two flags and external/trusted attributes, but not `reveal_group`
or `item_broadcast_use`. Its whole-unit function/const inventory subsequently
counts these owners as admitted bodies.

A minimal, independently successful default-off witness is:

```rust
use vstd::prelude::*;
#[verifier::verify]
#[verus::internal(reveal_group)]
pub fn skipped() {}
```

With the corrected authenticated bundle, ordinary and Infer feature-on runs report
`inactive: F1 admission passed; functions=1, bodies=1, headers=0; ...`.
Expected: an early skip/directive-specific admission refusal, with no later
admission message. Default-off, both modes succeed with `verified=0`, empty
`func-details`, zero checked exec/proof bodies, and a selected native inventory
entry `body_status="selected_item_without_translated_body"`. In contrast, removing
the marker verifies the unit function: `verified=1`, one checked exec body.

This follows the actual frontend: `rust_to_vir_func.rs:2008` converts the body to
a reveal-group declaration and returns `Ok(None)` at2036 instead of emitting an
ordinary Function. Its validation of reveal statements is not ordinary function
body verification. This is a coverage/inventory mismatch, not a claim that an
arbitrary false contract passes the reveal-group validator.

A second valid witness is:

```rust
use vstd::prelude::*;
#[verifier::verify]
#[verus::internal(item_broadcast_use)]
const K: () = {};
```

Both feature-on modes admit `functions=0, bodies=1, headers=0`. Both default-off
modes succeed with `verified=0` and zero checked bodies. `rust_to_vir.rs:97` and177
consume this as a module directive, with no ordinary const Function.

Exact commands, sources, stdout/stderr and result inventories are private under
`/tmp/owned-cell-f1-fix-review-ad51b8c2/coverage/`; `coverage.py` reproduces the
matrix. Its receipt hash is in RESULTS.json. Each call uses the private bundle's
`verus`, the exact `input.rs`, `--crate-type=lib --crate-name=coverage
--edition=2024 --no-cheating --compile --output-json -o <private .rlib>`, with
`--experimental-owned-unsafe-cell` and/or `--infer-goals` as recorded. No positive
skip evidence here depends on a frontend failure or panic.

Smallest actionable correction: extend the existing item/all-local-ancestor
refusal predicate to exclude these directive/non-Function markers. Also close
its source-visible special-const omissions `size_of_global` and
`structural_const_wrapper` (`rust_to_vir.rs:94,179`), conservatively within the same
closed grammar. Add ordinary/Infer precise early negatives and preserve unmarked
scalar/header and false-contract admission controls. Do not modify default-off
translation. Before future activation, reconcile admitted owners with actual
translated/checked Functions; an attribute list alone is not that reconciliation.
No production fix is made by this review.

The two special-const variants were independently admitted, but their default-off
raw fixtures do **not** establish successful unchecked verification: four
structural-wrapper runs (+/- client) and two plain raw-const controls panic in
THIR/erasure initialization; two malformed size_of fixtures give `invalid global
size_of`. These eight outcomes are excluded from positive skip/semantic evidence.
The two default-off reveal-group false-contract fixtures likewise give a frontend
reveal-group error, not a postcondition failure or active exploit. All raw failures
remain preserved. The valid empty reveal-group and broadcast witnesses above are
sufficient for the finding independently of those exclusions.

## Corrected finding and independent execution

All **10 focused tests /166 verifier invocations pass** with the original test
bodies and oracles. The standalone std-based harness removes only the unused
`rustc_private` feature and adjusts the frozen include path; a minimal
`InferCrates` tempdir/path shim replaces the broad common module. Its serde_json
1.0.151/tempfile3.27.0 dependencies come from the authenticated offline bootstrap.
The exact source transformation is checked by summarize.py. No broad compiler
or vstd build, no parent build cache, and no replacement receipt framework.
Execution and stability checks reuse `infer_experiment` and `infer_eval_support`.

* The complete old116 admission +32 inherited calls remain, plus16 corrected
  early-marker negatives and2 unmarked false-contract controls. Original
  option/selection/import/export/Cargo/parse invalidation controls pass.
* **84 additional corrected-runtime calls:**24 byte-for-byte original finding/
  control witnesses across ordinary/Infer and flag on/off;44 marker probes across
  deep module ancestors, crate inner attrs, unused nested items, marked outer
  functions, macro-generated item/module, cfg_attr, const, mixed attributes,
  scalar-spec and real verus! forms;8 normal macro/header/const controls;
  8 explicitly exploratory adjacent-directive cases. All76 preclassified calls
  meet their oracle; the8 exploratory observations are retained without pass
  claims. All52 marked additional requests reject early with the precise reason,
  empty proof/helper/export inventories and no later-admission message.
* **24 independently executed pre-fix calls** after authenticating all8 old tools:
  old feature-on witnesses reach later refusal; all12 default-off comparisons
  match the corrected runtime on stable result/body/contract/inventory fields.
  No timing or source-path differences are hidden as semantic differences.
* **32 additional corrected coverage confirmations** distinguish valid default-off
  skips from checked unit controls, ordinary frontend failures and panics.
  Counts and exclusions are explicit in RESULTS.json.

Total: **306 verifier invocations**, including the old comparator. The focused
suite also compiles its imported fixture once with native rustc; that is neither
a verifier invocation nor a native proof. All corrected experimental invocations
refuse with zero requested/generated helpers, checked exec/proof credit, and
`interface_written=false` wherever JSON is requested. Direct experimental output
directories contain no emitted `.rlib` or `.vir`; original stale-output controls
also pass. The frozen trio retains3 functions /6 bodies /3 canonical headers.
False predicates without skip markers remain valid F1 admission inputs and reach
the later inactive refusal; default-off they fail the actual postcondition.

## Source, authentication and cost

Audited the exact delta, item/parent walk, whole-crate definitions and body-owner
inventory, unvisited nested-body refusal, canonical-header ownership, native
preflight/caller ordering, and ordinary frontend skip/directive paths. All
functions/consts call checked_item before insertion. The walk continues through
`opt_parent` to the local crate owner; verified mode never short-circuits the skip
checks. An empty marked module with no body is not invented as an omitted body.
The source fix has no default-off translation changes. The admitted private
inventory still has only the unconditional later-refusal consumer; it carries no
native proof, field or export authority. SOURCE_HASHES.json binds audited files
to exact frozen-root blobs, and no outside production edits occurred.

Corrected snapshot receipt SHA256:
`9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f`.
New rust_verify SHA256:
`d48a95360484cda8128131b6fd33548de2a454d48c03bf1a23e66d48806c0dd3`.
Both corrected source hashes and the complete8-tool map are in AUTHENTICATION.json.
All8 tools were authenticated **before** private copying and rechecked afterward;
the7 supporting artifacts match the frozen F1/preflight identities. Old comparator
receipt SHA256 `51ae6b41380ac4bdd8dd362889a4818df7b352d86aa032d306055377a6b98bf1`;
old verifier `812e3e94c9994039371b8624dfa9646dd5468242f034be917cd285b6f1c1ad88`.
Normal vstd2257/0 is inherited, not rebuilt or independently reverified.

Focused command51.377s (includes5.89s harness compilation); additional corrected
probes83.124s, old comparator79.069s, coverage79.588s. Nested verifier costs are
recorded separately, not added to the enclosing focused command. Concurrent
command sums are resource costs, not elapsed wall time. RESULTS.json records the
conservative wall/active upper bound from16:56 UTC, including setup/builds/waits;
it is below45 minutes. No CPU/dollar meter. Sandbox startup initially failed
before command execution; escalated workspace commands were approved. All
completed logs/binaries/repeated sources and the full RAW_HASHES.json manifest
remain private under the review's /tmp; compact committed receipts reference
paths, hashes, precise argv and costs. No thousands-file evidence copy.

R1 native54 MIR+4 signature/region mutation controls remain **historical**, not
independently executed here. F2/F3, native Function/place/read/writeback
correspondence, private final-input-state query, compiler field authority and
original-call erasure remain unimplemented/unaccepted. This review provides no
full-stage semantic acceptance. CP1=false, CP2=false, source_coupling_proved=false;
native=0, publication=false, goal_complete=false. Full REDB Mutex/cache/
publication/recovery remains unproved; HMAC/native **DEFERRED AND UNACHIEVED**.
