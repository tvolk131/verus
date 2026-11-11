# Current Miri proof boundary

The branch supports checked direct core bool-to-u64 bodies, tuple-constructor
function items, genuine NonZero equality, and bounded unsigned packing
assistance. The accepted interface uses **schema 39 / policy 72 / cache 75**.
Rebuild compiler/macros/vstd, providers and consumers together; certificates bind
the executing compiler, loaded core body/artifacts, target and dependencies.

[Combined certificate review](parent-certified-foreign-review.json) records
independent acceptance of this bounded capability. [Parent validation](parent-combined72-validation.json)
records the matching runtime and full regression/replay results. Certified
export waits for every required ordinary, inference and auxiliary proof plus
erased Rust compilation. Ordinary imports validate required foreign evidence,
including evidence retained through private-helper pruning and transitive exports.
These are proof-produced interfaces under existing compiler/artifact/model trust;
they are not native refinement certificates.

Per-function [deduplication](parent-packing-dedup-review.json) removes only
identical closed proposals after retaining an ordinary checked copy. No wall-time
speedup is demonstrated.

## What now verifies

The original full Item fixture verifies **132/0 cold/warm**, retaining genuine
private carriers, authored semantic views/contracts and original runtime bodies
and debug assertions. It has no application mechanical proof blocks, theorem
calls, snapshots or proof functions. Its cost includes 102 isolated bitvector
checks and all three fresh full-domain Eq precision checks. The natural packet
writer verifies **31/0**; a separate public-visibility adapter enables an ordinary
imported client. The [arithmetic review](parent-item-packing-review.json) records
source fidelity, ablation, costs and coverage limits.

The constructor contract exposes well-formedness and lower-tag equality. It
does not expose equality between its protector/permission inputs and later
observations. Closed observations remain closed on import; Eq/value compatibility
has an explicitly authored checked broadcast lemma and client import, recorded
in [observation exports](parent-tag-observation-exports.json).

Earlier component results remain bounded evidence: the packed grant-boundary
helper checks **22/0**, and cache initialization/insertion **17/0**. Both require authored
proof assistance and show no causal inference gain. Their
[grant record](parent-miri-grant-component.json) and
[cache record](parent-miri-cache-component.json) retain their exact scope.
They do not establish full lookup or runtime permission grants.

## Next source obligation

The [ordering baseline](parent-nonzero-ordering-baseline.json) distinguishes
passing direct NonZero partial-order contracts from an unproved `Ord::cmp`
contract. Genuine BorTag ordering remains unproved. Its exact MIR exposes the
get-fragment/view and numeric-comparison correspondence needed next; no new
standard contract or representation law has been assumed.

Prove that actual `find_granting(Write, ...) = Ok(Some(g))` preserves borrows,
`unknown_bottom` and `unique_range`, and returns an in-bounds granting item.
Cover the enabled cache, concrete/wildcard/unknown provenance and all error outcomes; do not add a
new restriction on application callers. The
[typed needs](../native_miri_find_granting/NEEDS.json) record reverse-iterator,
hasher/key, Option callback, swap and global cache-preservation gaps. Its tag
constructor/equality entries describe an older checkpoint and are now supported
within the scope above; ordering remains separate.

Use the [compiler-matched source pin](../native_miri_policy_pin/pins.json), Rust
commit 58cf8f9c35b355199727627d07ae34a4e2846d67. Current-source features prevent a
blanket identification with the historical borrow models:

- Wildcard/unknown-bottom paths can return `Ok(None)` without a known granting
  item; reversing only the known vector loses that represented case.
- No-access SRW insertion and unknown-stack forgetting differ from access-then-
  push. A fallible pop visitor can leave a partially changed source stack.
- The low 61-bit Item tag check differs from the wider BorTag counter; its failure
  needs classification, not a tag-budget premise on the original buffer writer.

## Original buffer goal remains open

Canonical varint is **6/0** with authored semantic invariants and no local mechanical
proofs. Shared iterator observations are redundant for that frozen canonical
fixture and necessary only as a group for the separate Zip prefix theorem;
[causal ablation](../shared_observation_ablation/REPORT.md) rules out a claimed
necessary shared step across both full targets.

Full frozen HMAC remains **1/1 UNPROVED** at the unchanged-suffix postcondition.
Native correspondences remain **0**; all 44 minimal + 3 phase simulations and 101
writer-effect obligations remain open. Checked MODEL initialization, resources,
loops, final-heap observation and historical borrow-entry laws are prerequisites,
not actual Rust input/provenance/retag/unwind/current-future correspondence.
Whole-goal `publication=false` and `goal_complete=false` remain in force.
