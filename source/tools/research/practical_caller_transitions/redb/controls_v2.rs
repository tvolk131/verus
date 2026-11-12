verus! {
#[cfg(v2_negative_effect)]
fn wrong_free_effect(a: &mut BuddyAllocator, p: u32, o: u8)
    requires valid(*old(a)), legal_free(*old(a),p,o),
{
    let ghost before=*a;
    free_semantic(a,p,o);
    assert(free_pages(*a).len() == free_pages(before).len());
}
#[cfg(v2_negative_merge)]
fn wrong_free_merge(a: &mut BuddyAllocator, p: u32, o: u8)
    requires valid(*old(a)), legal_free(*old(a),p,o),
{
    let r=free_semantic(a,p,o);
    assert(r < o);
}
#[cfg(v2_negative_domain)]
fn wrong_free_domain(a: &mut BuddyAllocator)
    requires valid(*old(a)), old(a).max_order == 7,
{
    free_semantic(a,0,8);
}
}
verus! {
#[cfg(v2_negative_alloc_accept)]
fn wrong_alloc_accept(a: &mut BuddyAllocator, p: u32, o: u8)
    requires valid(*old(a)), legal_block(*old(a),p,o),
        !requested(p,o).subset_of(free_pages(*old(a))),
{
    let r=record_alloc_semantic(a,p,o);
    assert(r);
}
#[cfg(v2_negative_alloc_effect)]
fn wrong_alloc_effect(a: &mut BuddyAllocator, p: u32, o: u8)
    requires valid(*old(a)), legal_block(*old(a),p,o), requested(p,o).subset_of(free_pages(*old(a))),
{
    let ghost before=*a;
    let r=record_alloc_semantic(a,p,o);
    assert(free_pages(*a).len() == free_pages(before).len());
}
#[cfg(v2_negative_overlap)]
proof fn wrong_overlap(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int)
    requires canonical(bs,ns,n,m), m > 0, ns[1] > 0, clear_at(bs,ns,0,0),
{
    let next=bs.update(1,bs[1].remove(0));
    assert(canonical(next,ns,n,m));
}
}
