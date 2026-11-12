// Four-level interior client retained unproved: ordinary and Infer resource limits.
verus! {
#[verifier::rlimit(20)]
fn interior_split_merge(a: &mut BuddyAllocator)
    requires valid(*old(a)), old(a).len == 130, old(a).max_order == 7,
        flags(*old(a))[3].contains(5), flags(*old(a))[3].contains(4),
        flags(*old(a))[4].contains(2), flags(*old(a))[4].contains(3),
        flags(*old(a))[5].contains(1), flags(*old(a))[5].contains(0),
        flags(*old(a))[6].contains(0), flags(*old(a))[6].contains(1),
        !flags(*old(a))[7].contains(0),
    ensures layout(*final(a)), flags(*final(a)) == flags(*old(a)),
{
    let ghost before = *a;
    proof { valid_admission(*a); reveal_with_fuel(alloc_trace,5); reveal_with_fuel(free_trace,5); }
    let success = a.record_alloc_inner(5,3);
    assert(success);
    let merged_order = a.free_inner(5,3);
    assert(merged_order == 7);
    assert(flags(*a) =~= flags(before));
}
}
