// Version 2: interpretations of the checked stored-bit traces.
// All executable dependencies are the unchanged manual.rs + domain.rs.
verus! {
spec fn inside(x: int, p: int, o: int) -> bool {
    p*width(o) <= x < (p+1)*width(o)
}
spec fn interval(p: int, o: int) -> Set<int> {
    Set::<int>::range(p*width(o),(p+1)*width(o))
}
spec fn disjoint_blocks(p: int, o: int, q: int, k: int) -> bool {
    (p+1)*width(o) <= q*width(k) || (q+1)*width(k) <= p*width(o)
}
spec fn model_shape(ns: Seq<int>, n: int, m: int) -> bool {
    0 <= n <= u32::MAX && 0 <= m <= 20 && ns.len() == m+1 &&
    forall|o: int| 0 <= o <= m ==> #[trigger] ns[o] == n/width(o)
}
spec fn clear_at(bs: Seq<Set<int>>, ns: Seq<int>, o: int, p: int) -> bool {
    0 <= o < bs.len() && 0 <= p < ns[o] && !bs[o].contains(p)
}
spec fn canonical(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int) -> bool {
    model_shape(ns,n,m) && bs.len() == ns.len() &&
    (forall|o: int, k: int, p: int, q: int| 0 <= o < k <= m &&
        clear_at(bs,ns,o,p) && clear_at(bs,ns,k,q) ==> disjoint_blocks(p,o,q,k)) &&
    (forall|o: int, p: int| 0 <= o < m && 0 <= p && 2*p+1 < ns[o]
        ==> #[trigger] bs[o].contains(2*p) || bs[o].contains(2*p+1))
}
spec fn pages(bs: Seq<Set<int>>, ns: Seq<int>, n: int) -> Set<int> {
    Set::<int>::range(0,n).filter(|x: int| exists|o: int, p: int|
        clear_at(bs,ns,o,p) && inside(x,p,o))
}
spec fn model_legal(ns: Seq<int>, m: int, p: int, o: int) -> bool {
    0 <= o <= m && 0 <= p < ns[o]
}
spec fn sibling(p: int) -> int { if p%2 == 0 {p+1} else {p-1} }
// Frozen original meanings, copied rather than reinterpreted through the model.
spec fn free_pages(a: BuddyAllocator) -> Set<int> {
    Set::<int>::range(0,a.len as int).filter(|x: int| exists|o: int, p: int| 0 <= o <= a.max_order &&
        0 <= p < order_len(a,o) && !flag(a,o,p) && p*width(o) <= x < (p+1)*width(o))
}
spec fn requested(p: u32, o: u8) -> Set<int> {
    Set::<int>::range(p as int * width(o as int), (p as int+1)*width(o as int))
}
spec fn legal_block(a: BuddyAllocator, p: u32, o: u8) -> bool {
    o <= a.max_order && p < order_len(a,o as int)
}
spec fn legal_free(a: BuddyAllocator, p: u32, o: u8) -> bool {
    legal_block(a,p,o) && requested(p,o).disjoint(free_pages(a))
}
// Staged interpretation obligations. Empty bodies are failures, never admissions.
proof fn block_in_bounds(ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires model_shape(ns,n,m), model_legal(ns,m,p,o),
    ensures interval(p,o).subset_of(Set::<int>::range(0,n)),
{}
proof fn same_order_disjoint(p: int, q: int, o: int)
    requires 0 <= o, p != q,
    ensures disjoint_blocks(p,o,q,o),
{}
proof fn remove_block(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), clear_at(bs,ns,o,p),
    ensures canonical(bs.update(o,bs[o].insert(p)),ns,n,m),
        pages(bs.update(o,bs[o].insert(p)),ns,n) == pages(bs,ns,n).difference(interval(p,o)),
{}
proof fn add_block(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o),
        interval(p,o).disjoint(pages(bs,ns,n)),
        o == m || sibling(p) >= ns[o] || bs[o].contains(sibling(p)),
    ensures canonical(bs.update(o,bs[o].remove(p)),ns,n,m),
        pages(bs.update(o,bs[o].remove(p)),ns,n) == pages(bs,ns,n).union(interval(p,o)),
{}
proof fn actual_model(a: BuddyAllocator)
    requires valid(a),
    ensures canonical(flags(a),sizes(a),a.len as int,a.max_order as int),
        pages(flags(a),sizes(a),a.len as int) == free_pages(a),
{}
proof fn model_actual(a: BuddyAllocator, b: BuddyAllocator)
    requires geometry(a), geometry_frame(a,b), representation(b),
        canonical(flags(b),sizes(a),a.len as int,a.max_order as int),
    ensures valid(b), pages(flags(b),sizes(a),a.len as int) == free_pages(b),
{}
proof fn alloc_interpret(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), 0 <= p, 0 <= o,
    ensures ({let t=alloc_trace(bs,ns,p,o,m);
        canonical(t.0,ns,n,m) &&
        t.1 == (model_legal(ns,m,p,o) && interval(p,o).subset_of(pages(bs,ns,n))) &&
        (t.1 ==> pages(t.0,ns,n) == pages(bs,ns,n).difference(interval(p,o))) &&
        (!t.1 ==> t.0 == bs) }),
    decreases m+1-o,
{}
proof fn free_interpret(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o), interval(p,o).disjoint(pages(bs,ns,n)),
    ensures ({let t=free_trace(bs,ns,p,o,m);
        canonical(t.0,ns,n,m) && o <= t.1 <= m &&
        pages(t.0,ns,n) == pages(bs,ns,n).union(interval(p,o)) &&
        clear_at(t.0,ns,t.1,p/width(t.1-o)) &&
        interval(p,o).subset_of(interval(p/width(t.1-o),t.1)) }),
    decreases m-o,
{}
fn record_alloc_semantic(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: bool)
    requires valid(*old(a)),
    ensures valid(*final(a)), geometry_frame(*old(a),*final(a)),
        r == (legal_block(*old(a),p,o) && requested(p,o).subset_of(free_pages(*old(a)))),
        r ==> free_pages(*final(a)) == free_pages(*old(a)).difference(requested(p,o)),
        !r ==> state_eq(*old(a),*final(a)),
{
    a.record_alloc_inner(p,o)
}
fn free_semantic(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: u8)
    requires valid(*old(a)), legal_free(*old(a),p,o),
    ensures valid(*final(a)), geometry_frame(*old(a),*final(a)),
        free_pages(*final(a)) == free_pages(*old(a)).union(requested(p,o)),
        o <= r <= old(a).max_order,
        !flag(*final(a),r as int,p as int/width(r as int-o as int)),
{
    a.free_inner(p,o)
}
}
