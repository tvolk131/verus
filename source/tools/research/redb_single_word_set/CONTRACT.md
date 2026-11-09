# Frozen stateful target (before proof attempts)

Upstream redb c82ab865bb362bc4af067d332adc44978ff64813,
`src/tree_store/page_store/bitmap.rs`: U64GroupedBitmap, data_index_of,
select_mask, get, set. Exact items and hashes are in original/ and freeze.json.
No application implementation is assumed. Only set is the transition; get is
an observation. No alloc search, clear, resize, BtreeBitmap summary propagation,
transaction, durability, concurrency or crash-recovery claim is intended.

## Supported domain and authored policy

Single word: data.len() == 1, 1 <= len <= 64. Every physical padding bit
len <= j < 64 is allocated (set). Logical bit argument satisfies bit < len.
Every logical allocation pattern is allowed, including empty, full, and
already-allocated target; no freshness precondition. This is an explicitly
bounded leaf slice, selected before any solver attempt, not the full allocator.

Abstract view: Seq<bool> of length len with entry j equal to
(data[0] & (1u64 << j)) != 0. Count is the mathematical number of true entries
(vstd Seq::count_prefix with identity predicate). Invariant is the supported
domain plus allocated padding. No stored count is added to the implementation.

Desired set postcondition: invariant preserved; len and physical word count
unchanged; abstract view equals old view updated at bit to true; all other
logical memberships unchanged; count increases by 1 exactly when old target
was free, otherwise unchanged. Return is true iff all logical pages are
allocated. get returns the corresponding abstract membership without mutation.

## Adaptation rules and baseline

Retain exact executable bodies of all selected items. Add Verus wrapper,
semantic definitions, requires/ensures and named return binders. No change of
Vec representation, assignment, helper algorithm or arithmetic. Visibility can
be widened in an extraction; report any frontend adaptation separately.
Initial baseline contains no proof blocks/lemmas, reveal or solver annotations;
run with --infer-goals only (plus output/resource settings). Record all failing
obligations or frontend blockers. A later assisted fixture may add erased proof
code but must retain original executable statements. An explicitly weaker
comparison fixture cannot satisfy this frozen target.

Semantic policy is the allocation/update/count goal. Representation laws are
bit membership and allocated padding. Mechanical work is finite bit reasoning,
sequence extensionality and count-of-update. Trusted imports: Rust/Verus
integer/Vec specs and SMT/kernel/standard vstd proof infrastructure, not redb
state-update summaries. Native finite runs are supporting erasure checks only.
