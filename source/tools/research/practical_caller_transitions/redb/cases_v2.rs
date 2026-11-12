// This client checks full semantic composition over arbitrary valid geometry,
// including all legal interior orders and ragged-tail requests. It does not
// assert opaque Vec identity or replace the retained V1 four-level diagnostic.
verus! {
#[cfg(full_semantics)]
fn semantic_roundtrip(a: &mut BuddyAllocator, p: u32, o: u8)
    requires valid(*old(a)), legal_block(*old(a),p,o), requested(p,o).subset_of(free_pages(*old(a))),
    ensures valid(*final(a)), free_pages(*final(a)) == free_pages(*old(a)),
        free_pages(*final(a)).len() == free_pages(*old(a)).len(),
{
    let ghost before=*a;
    proof { actual_model(before); }
    let success=record_alloc_semantic(a,p,o);
    assert(success);
    proof {
        actual_model(*a);
        frame_geometry(before,*a);
        assert(legal_block(*a,p,o));
        assert(requested(p,o).disjoint(free_pages(*a)));
        assert(legal_free(*a,p,o));
    }
    free_semantic(a,p,o);
    assert(free_pages(*a) =~= free_pages(before));
}
}
