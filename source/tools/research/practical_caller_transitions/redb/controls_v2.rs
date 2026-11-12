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
