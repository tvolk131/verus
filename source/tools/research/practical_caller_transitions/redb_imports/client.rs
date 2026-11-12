use vstd::prelude::*;
use r3_public::{BuddyAllocator, free_pages, record_alloc_semantic, requested, valid, width, width_positive};

verus! {
pub fn repeated_allocation(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: (bool, bool))
    requires valid(*old(a)),
    ensures valid(*final(a)), !(r.0 && r.1),
{
    let first = record_alloc_semantic(a, p, o);
    let ghost middle = *a;
    let second = record_alloc_semantic(a, p, o);
    proof {
        if first && second {
            width_positive(o as int);
            broadcast use vstd::set_lib::range_set_properties;
            let x = p as int * width(o as int);
            vstd::arithmetic::mul::lemma_mul_is_distributive_add_other_way(width(o as int), p as int, 1);
            assert(requested(p, o).contains(x));
            assert(free_pages(middle).contains(x));
            assert(!free_pages(middle).contains(x));
        }
    }
    (first, second)
}

#[cfg(negative_value)]
pub fn wrong_simultaneous_success(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: bool)
    requires valid(*old(a)),
    ensures r,
{
    let attempts = repeated_allocation(a, p, o);
    attempts.0 && attempts.1
}

#[cfg(negative_safety)]
pub fn unchecked_allocator(a: &mut BuddyAllocator, p: u32, o: u8) -> bool {
    record_alloc_semantic(a, p, o)
}
}
