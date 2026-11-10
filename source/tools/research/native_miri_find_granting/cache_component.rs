use vstd::prelude::*;
use std::num::NonZero;
use std::ops::Range;
verus! {
// Exact Item representation; tag/permission methods are not used by cache add.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Item(u64);
const CACHE_LEN: usize = 32;
#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorTag(NonZero<u64>);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]
pub enum Permission { Unique, SharedReadWrite, SharedReadOnly, Disabled }
const PERM_MASK: u64 = 0x3 << 61;
const PERM_SHIFT: u64 = 61;
spec fn decode(p: u64) -> Permission {
    if p == 0 { Permission::Unique } else if p == 1 { Permission::SharedReadWrite }
    else if p == 2 { Permission::SharedReadOnly } else { Permission::Disabled }
}
impl Permission {
    const UNIQUE: u64 = 0;
    const SHARED_READ_WRITE: u64 = 1;
    const SHARED_READ_ONLY: u64 = 2;
    const DISABLED: u64 = 3;
    fn from_bits(perm: u64) -> (r: Self)
        requires perm <= 3,
        ensures r == decode(perm),
    {
        match perm {
            Self::UNIQUE => Permission::Unique,
            Self::SHARED_READ_WRITE => Permission::SharedReadWrite,
            Self::SHARED_READ_ONLY => Permission::SharedReadOnly,
            Self::DISABLED => Permission::Disabled,
            _ => unreachable!(),
        }
    }
}
impl Item {
    spec fn permission(self) -> Permission { decode((self.0 & PERM_MASK) >> PERM_SHIFT) }
    fn perm(self) -> (r: Permission)
        ensures r == self.permission(),
    {
        proof {
            assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
            let w = self.0;
            assert((w & 0x6000000000000000u64) >> 61u64 <= 3u64) by(bit_vector);
        }
        Permission::from_bits((self.0 & PERM_MASK) >> PERM_SHIFT)
    }
}
pub struct Stack {
    borrows: Vec<Item>,
    unknown_bottom: Option<BorTag>,
    cache: StackCache,
    unique_range: Range<usize>,
}
impl Stack {
    fn new(item: Item) -> (r: Self)
        ensures r.borrows@ == seq![item], r.unknown_bottom.is_none(),
            cache_consistent(r.cache, r.borrows@),
            r.unique_range.start == 0,
            r.unique_range.end == (if item.permission() == Permission::Unique { 1usize } else { 0usize }),
    {
        Stack {
            borrows: vec![item],
            unknown_bottom: None,
            cache: StackCache { idx: [0; CACHE_LEN], items: [item; CACHE_LEN] },
            unique_range: if item.perm() == Permission::Unique { 0..1 } else { 0..0 },
        }
    }
}
struct StackCache {
    items: [Item; CACHE_LEN],
    idx: [usize; CACHE_LEN],
}
spec fn cache_consistent(cache: StackCache, borrows: Seq<Item>) -> bool {
    forall|j: int| 0 <= j < CACHE_LEN ==> cache.idx[j] < borrows.len()
        && cache.items[j] == borrows[cache.idx[j] as int]
}
spec fn add_effect(before: StackCache, after: StackCache, idx: usize, item: Item) -> bool {
    &&& after.items[0] == item
    &&& after.idx[0] == idx
    &&& forall|j: int| 1 <= j < CACHE_LEN ==> after.items[j] == before.items[j - 1]
        && after.idx[j] == before.idx[j - 1]
}
impl StackCache {
    fn add(&mut self, idx: usize, item: Item)
        ensures add_effect(*old(self), *final(self), idx, item),
    {
        self.items.copy_within(0..CACHE_LEN - 1, 1);
        self.items[0] = item;
        self.idx.copy_within(0..CACHE_LEN - 1, 1);
        self.idx[0] = idx;
    }
}
proof fn add_effect_preserves_consistency(
    before: StackCache, after: StackCache, borrows: Seq<Item>, idx: usize, item: Item,
)
    requires cache_consistent(before, borrows),
        idx < borrows.len(), item == borrows[idx as int],
        add_effect(before, after, idx, item),
    ensures cache_consistent(after, borrows),
{
    assert forall|j: int| 0 <= j < CACHE_LEN implies
        after.idx[j] < borrows.len() && after.items[j] == borrows[after.idx[j] as int] by {
        if j == 0 { } else { assert(0 <= j - 1 < CACHE_LEN); }
    }
}
}
