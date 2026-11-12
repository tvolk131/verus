// Optional converse diagnostic: ordinary checked; Infer resource failure.
// Not part of the delivered source-domain admission proof.
verus! {
#[verifier::rlimit(20)]
proof fn layout_length(a: BuddyAllocator, o: int)
    requires layout(a), 0 <= o <= a.max_order,
    ensures order_len(a,o) == a.len as int / width(o),
    decreases o,
{
    if o > 0 {
        layout_length(a,o-1);
        width_positive(o-1);
        assert(width(o) == 2 * width(o-1));
        vstd::arithmetic::div_mod::lemma_div_denominator(a.len as int,width(o-1),2);
    }
}
proof fn layout_to_geometry(a: BuddyAllocator)
    requires layout(a),
    ensures geometry(a),
{
    assert forall|o: int| 0 <= o <= a.max_order implies order_len(a,o) == a.len as int / width(o) by {
        layout_length(a,o);
    }
}
}
