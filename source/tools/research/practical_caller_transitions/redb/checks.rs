verus! {
// An actual multiword/partial-boundary state shape, with repeated operations.
// Preconditions describe arbitrary checked trees in this state; no constructors are assumed.
fn boundary_repeated_operations(a: &mut BuddyAllocator)
    requires layout(*old(a)), old(a).len == 130, old(a).max_order == 7,
        flags(*old(a))[0].contains(128), flags(*old(a))[0].contains(129),
        flags(*old(a))[1].contains(64),
    ensures layout(*final(a)), flags(*final(a)) == flags(*old(a)), sizes(*final(a)) == sizes(*old(a)),
{
    let ghost before = *a;
    proof { reveal_with_fuel(free_trace,3); reveal_with_fuel(alloc_trace,4); }
    let rejected = a.record_alloc_inner(129,0);
    assert(!rejected);
    let rejected_again = a.record_alloc_inner(129,0);
    assert(!rejected_again);
    let first = a.free_inner(129,0);
    assert(first == 0);
    let merged = a.free_inner(128,0);
    assert(merged == 1);
    let allocated = a.record_alloc_inner(128,0);
    assert(allocated);
    let allocated_again = a.record_alloc_inner(129,0);
    assert(allocated_again);
    assert(flags(*a) =~= flags(before));
}
// Restrict this example to the frozen real allocator invariant; operations themselves
// are checked on the broader domain. Intermediate canonical validity remains open.
fn valid_boundary_scenario(a: &mut BuddyAllocator)
    requires valid(*old(a)), old(a).len == 130, old(a).max_order == 7,
        flags(*old(a))[0].contains(128), flags(*old(a))[0].contains(129),
        flags(*old(a))[1].contains(64),
    ensures layout(*final(a)), flags(*final(a)) == flags(*old(a)),
{
    proof { valid_admission(*a); }
    boundary_repeated_operations(a);
}
// A supported interior-order example in the same eight-order, multiword domain.
// The larger four-level client remains separately unproved, not silently replaced.
fn interior_one_split_merge(a: &mut BuddyAllocator)
    requires valid(*old(a)), old(a).len == 130, old(a).max_order == 7,
        flags(*old(a))[3].contains(5), flags(*old(a))[3].contains(4),
        !flags(*old(a))[4].contains(2), flags(*old(a))[4].contains(3),
    ensures layout(*final(a)), flags(*final(a)) == flags(*old(a)),
{
    let ghost before = *a;
    proof {
        valid_admission(*a);
        order_half(*a,0); order_half(*a,1); order_half(*a,2); order_half(*a,3);
        assert(order_len(*a,0) == 130);
        assert(order_len(*a,1) == 65);
        assert(order_len(*a,2) == 32);
        assert(order_len(*a,3) == 16);
        assert(order_len(*a,4) == 8);
        reveal_with_fuel(alloc_trace,2);
        reveal_with_fuel(free_trace,2);
    }
    let success = a.record_alloc_inner(5,3);
    assert(success);
    let merged_order = a.free_inner(5,3);
    assert(merged_order == 4);
    assert forall|o: int| 0 <= o < flags(before).len() implies flags(*a)[o] == flags(before)[o] by {
        assert(flags(*a)[o] =~= flags(before)[o]);
    }
    assert(flags(*a) =~= flags(before));
}
#[cfg(negative_rejection)]
fn wrong_rejection(a: &mut BuddyAllocator)
    requires layout(*old(a)), old(a).max_order < 20,
{
    let r = a.record_alloc_inner(0,21);
    assert(r);
}
#[cfg(negative_merge)]
fn wrong_merge(a: &mut BuddyAllocator)
    requires layout(*old(a)), old(a).len == 130, old(a).max_order == 7,
        flags(*old(a))[0].contains(128), flags(*old(a))[0].contains(129),
        flags(*old(a))[1].contains(64),
{
    proof { reveal_with_fuel(free_trace,3); }
    a.free_inner(129,0);
    let order = a.free_inner(128,0);
    assert(order == 0);
}
#[cfg(negative_safety)]
fn invalid_free(a: &mut BuddyAllocator)
    requires layout(*old(a)), old(a).max_order == 7,
{
    a.free_inner(0,8);
}
}
