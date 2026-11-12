// Checked admission of every frozen geometric state into the broader operational domain.
verus! {
spec fn width(o: int) -> int decreases o { if o <= 0 { 1 } else { 2 * width(o-1) } }
spec fn geometry(a: BuddyAllocator) -> bool {
    representation(a) &&
    forall|o: int| 0 <= o <= a.max_order ==> order_len(a,o) == a.len as int / width(o)
}
spec fn flag(a: BuddyAllocator, o: int, p: int) -> bool { physical_bit(a.free@[o].heights@.last(),p) }
#[verifier::opaque]
spec fn valid(a: BuddyAllocator) -> bool {
    geometry(a) &&
    // Two zero bits must never cover the same order-zero page.
    (forall|o: int, k: int, p: int, q: int| 0 <= o < k <= a.max_order &&
        0 <= p < order_len(a,o) && 0 <= q < order_len(a,k) && !flag(a,o,p) && !flag(a,k,q)
        ==> (p+1)*width(o) <= q*width(k) || (q+1)*width(k) <= p*width(o)) &&
    // Existing in-range buddies cannot both be represented free below the maximum.
    (forall|o: int, p: int| 0 <= o < a.max_order && 0 <= p && 2*p+1 < order_len(a,o)
        ==> #[trigger] flag(a,o,2*p) || flag(a,o,2*p+1))
}
proof fn valid_admission(a: BuddyAllocator)
    requires valid(a),
    ensures layout(a),
{
    reveal(valid);
    geometry_to_layout(a);
}
proof fn order_half(a: BuddyAllocator, o: int)
    requires layout(a), 0 <= o < a.max_order,
    ensures order_len(a,o+1) == order_len(a,o)/2,
{}
proof fn width_positive(o: int)
    requires 0 <= o,
    ensures 0 < width(o),
    decreases o,
{
    if o > 0 { width_positive(o-1); }
}
proof fn geometry_to_layout(a: BuddyAllocator)
    requires geometry(a),
    ensures layout(a),
{
    assert(order_len(a,0) == a.len);
    assert forall|o: int| 0 <= o < a.max_order implies #[trigger] order_len(a,o+1) == order_len(a,o)/2 by {
        width_positive(o);
        assert(width(o+1) == 2 * width(o));
        vstd::arithmetic::div_mod::lemma_div_denominator(a.len as int,width(o),2);
    }
}
}
