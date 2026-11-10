# Frozen caller interface: current Stack lookup and enabled cache

Base 53178f50; Rust/Miri commit 58cf8f9c35b355199727627d07ae34a4e2846d67.
Only this new research directory is editable. No publication or native correspondence.

## Required internal theorem (not yet established)

For arbitrary current Stack, Read or Write, Concrete or Wildcard provenance, and
actual FxHashSet<BorTag> exposed_tags, establish with ordinary checked bodies:

- final(self).borrows equals old(self).borrows and unknown_bottom is unchanged;
- Ok(Some(g)) implies g < final(self).borrows.len() and the decoded permission
  at g grants access; Write implies exactly Unique or SharedReadWrite;
- tagged successful search additionally matches the genuine BorTag;
- wildcard successful search additionally belongs to actual exposed_tags;
- Ok(None) and Err remain outcomes. Wildcard unknown success requires an unknown
  bottom; concrete unknown success requires tag < unknown_limit after no known hit.

Cache mutation is allowed; unique_range remains present and unchanged. Feature
stack-cache=true is mandatory for enabled-cache coverage. Feature
expensive-consistency-checks=false is the primary selection; its debug checker
is inventoried but not silently called/proved. No application input restrictions.

## Minimal cross-lane need, routed before trials

The tag-representation lane owns genuine BorTag(NonZero<u64>), current Item::tag,
and constructor/tag laws. Lookup needs a checked internal Item predicate
`wf_item(i) := (i.0 & TAG_MASK) != 0`, a checked tag body under that predicate,
and tag equality/order correspondence for genuine derived BorTag traits.
No assumed or external-body tag contract is permitted. Until an immutable
milestone is explicitly delivered, only original standard NonZero specs and
literal frontend trials are eligible here.

An independent internal cache predicate is needed:
`len(borrows)>32 ==> forall j in 0..32: idx[j]<len(borrows) &&
items[j]==borrows[idx[j]]`. Item wellformedness is required for live borrows and
cache slots actually decoded. This is a structural relation, not successful
lookup or defined future execution. Relevant actual add/swap operations must
preserve it; global initialization/insert/remove/permission-change preservation
is a separate unproved native invariant obligation if not checked here.

## Source-fidelity rule

Freeze unchanged find_granting, find_granting_tagged, find_granting_cache,
StackCache::add and actual fields/dependencies before trials. Original
iter/enumerate/rev/find_map/FnMut, position, copy_within and swap must first be
tried literally. Unsupported features are frontend evidence, not failed semantic
controls. Any supported translation is separately labeled and must have an
explicit correspondence; indexed replacement is never called unchanged source.
Permission::grants and packed permission decoding may reuse own-checkout accepted
components, with their authored contracts and manual proof inventory disclosed.
FxHashSet membership must be genuine; a simplified set is not eligible.

## Boundaries

Manual ordinary proofs are diagnostic milestones only. Native correspondences=0;
44 minimal + 3 phase / 101 writer effects OPEN; full HMAC 1/1 UNPROVED;
publication=false; goal_complete=false. No source-safety, physical-permission,
control-flow or application acceptance follows from this component work.
