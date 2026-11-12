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
spec fn edited(bs: Seq<Set<int>>, o: int, p: int, value: bool) -> Seq<Set<int>> {
    bs.update(o,if value {bs[o].insert(p)} else {bs[o].remove(p)})
}
// Explicit membership/witness and frame instantiations, proved from definitions.
proof fn update_clear(bs: Seq<Set<int>>, ns: Seq<int>, o: int, p: int, value: bool)
    requires bs.len() == ns.len(), 0 <= o < bs.len(),
    ensures forall|k: int, q: int| #[trigger] clear_at(edited(bs,o,p,value),ns,k,q) ==
        (0 <= k < bs.len() && 0 <= q < ns[k] &&
            (if k == o && q == p { !value } else { !bs[k].contains(q) })),
{
    assert forall|k: int, q: int| #[trigger] clear_at(edited(bs,o,p,value),ns,k,q) ==
        (0 <= k < bs.len() && 0 <= q < ns[k] &&
            (if k == o && q == p { !value } else { !bs[k].contains(q) })) by {}
}
proof fn page_intro(bs: Seq<Set<int>>, ns: Seq<int>, n: int, x: int, p: int, o: int)
    requires 0 <= x < n, clear_at(bs,ns,o,p), inside(x,p,o),
    ensures pages(bs,ns,n).contains(x),
{
    broadcast use vstd::set_lib::range_set_properties;
    assert(exists|k: int, q: int| clear_at(bs,ns,k,q) && inside(x,q,k));
}
proof fn page_witness(bs: Seq<Set<int>>, ns: Seq<int>, n: int, x: int)
    requires pages(bs,ns,n).contains(x),
    ensures 0 <= x < n, exists|z: (int,int)| clear_at(bs,ns,z.0,z.1) && inside(x,z.1,z.0),
{
    broadcast use vstd::set_lib::range_set_properties;
    assert(exists|k: int, q: int| clear_at(bs,ns,k,q) && inside(x,q,k));
    let k=choose|k: int| #![trigger bs[k]] exists|q: int| clear_at(bs,ns,k,q) && inside(x,q,k);
    let q=choose|q: int| clear_at(bs,ns,k,q) && inside(x,q,k);
    assert(clear_at(bs,ns,(k,q).0,(k,q).1) && inside(x,(k,q).1,(k,q).0));
}
proof fn represented_disjoint(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int, q: int, k: int)
    requires canonical(bs,ns,n,m), clear_at(bs,ns,o,p), clear_at(bs,ns,k,q), o != k || p != q,
    ensures disjoint_blocks(p,o,q,k),
{
    if o == k { same_order_disjoint(p,q,o); }
    else if o < k { assert(disjoint_blocks(p,o,q,k)); }
    else { assert(disjoint_blocks(q,k,p,o)); }
}
proof fn vacant_disjoint(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int, q: int, k: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o), clear_at(bs,ns,k,q),
        interval(p,o).disjoint(pages(bs,ns,n)),
    ensures disjoint_blocks(p,o,q,k),
{
    broadcast use vstd::set_lib::range_set_properties;
    block_in_bounds(ns,n,m,p,o); block_in_bounds(ns,n,m,q,k);
    width_positive(o); width_positive(k);
    if !disjoint_blocks(p,o,q,k) {
        let x=if p*width(o) > q*width(k) {p*width(o)} else {q*width(k)};
        assert((p+1)*width(o) == p*width(o)+width(o)) by(nonlinear_arith);
        assert((q+1)*width(k) == q*width(k)+width(k)) by(nonlinear_arith);
        assert(inside(x,p,o) && inside(x,q,k));
        assert(interval(p,o).contains(x));
        page_intro(bs,ns,n,x,q,k);
        assert(false);
    }
}
// Staged interpretation obligations. Empty bodies are failures, never admissions.
proof fn block_in_bounds(ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires model_shape(ns,n,m), model_legal(ns,m,p,o),
    ensures interval(p,o).subset_of(Set::<int>::range(0,n)),
{
    broadcast use vstd::set_lib::range_set_properties;
    width_positive(o);
    let w=width(o);
    vstd::arithmetic::mul::lemma_mul_inequality(0,p,w);
    vstd::arithmetic::mul::lemma_mul_inequality(p+1,n/w,w);
    vstd::arithmetic::div_mod::lemma_mod_bound(n,w);
    vstd::arithmetic::div_mod::lemma_fundamental_div_mod(n,w);
    assert(w*(n/w) == (n/w)*w) by(nonlinear_arith);
    assert((n/w)*w <= n);
    assert(interval(p,o).subset_of(Set::<int>::range(0,n))) by {
        assert forall|x: int| interval(p,o).contains(x) implies Set::<int>::range(0,n).contains(x) by {}
    }
}
proof fn same_order_disjoint(p: int, q: int, o: int)
    requires 0 <= o, p != q,
    ensures disjoint_blocks(p,o,q,o),
{
    width_positive(o);
    if p < q { vstd::arithmetic::mul::lemma_mul_inequality(p+1,q,width(o)); }
    else { vstd::arithmetic::mul::lemma_mul_inequality(q+1,p,width(o)); }
}
proof fn remove_block(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), clear_at(bs,ns,o,p),
    ensures canonical(bs.update(o,bs[o].insert(p)),ns,n,m),
        pages(bs.update(o,bs[o].insert(p)),ns,n) == pages(bs,ns,n).difference(interval(p,o)),
{
    broadcast use vstd::set_lib::range_set_properties;
    let next=bs.update(o,bs[o].insert(p));
    update_clear(bs,ns,o,p,true);
    assert forall|k: int, l: int, q: int, r: int| 0 <= k < l <= m &&
        clear_at(next,ns,k,q) && clear_at(next,ns,l,r) implies disjoint_blocks(q,k,r,l) by {
        assert(clear_at(bs,ns,k,q) && clear_at(bs,ns,l,r));
        assert(disjoint_blocks(q,k,r,l));
    }
    assert forall|k: int, q: int| 0 <= k < m && 0 <= q && 2*q+1 < ns[k] implies
        #[trigger] next[k].contains(2*q) || next[k].contains(2*q+1) by {
        assert(bs[k].contains(2*q) || bs[k].contains(2*q+1));
    }
    assert forall|x: int| pages(next,ns,n).contains(x) == pages(bs,ns,n).difference(interval(p,o)).contains(x) by {
        if pages(next,ns,n).contains(x) {
            page_witness(next,ns,n,x);
            let (k,q)=choose|z: (int,int)| clear_at(next,ns,z.0,z.1) && inside(x,z.1,z.0);
            assert(clear_at(bs,ns,k,q));
            page_intro(bs,ns,n,x,q,k);
            represented_disjoint(bs,ns,n,m,p,o,q,k);
            assert(!interval(p,o).contains(x));
        } else if pages(bs,ns,n).contains(x) && !interval(p,o).contains(x) {
            page_witness(bs,ns,n,x);
            let (k,q)=choose|z: (int,int)| clear_at(bs,ns,z.0,z.1) && inside(x,z.1,z.0);
            assert(!(k == o && q == p));
            assert(clear_at(next,ns,k,q));
            page_intro(next,ns,n,x,q,k);
        }
    }
    assert(pages(next,ns,n) =~= pages(bs,ns,n).difference(interval(p,o)));
}
proof fn add_block(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o),
        interval(p,o).disjoint(pages(bs,ns,n)),
        o == m || sibling(p) >= ns[o] || bs[o].contains(sibling(p)),
    ensures canonical(bs.update(o,bs[o].remove(p)),ns,n,m),
        pages(bs.update(o,bs[o].remove(p)),ns,n) == pages(bs,ns,n).union(interval(p,o)),
{
    broadcast use vstd::set_lib::range_set_properties;
    let next=bs.update(o,bs[o].remove(p));
    update_clear(bs,ns,o,p,false);
    block_in_bounds(ns,n,m,p,o);
    assert forall|k: int, l: int, q: int, r: int| 0 <= k < l <= m &&
        clear_at(next,ns,k,q) && clear_at(next,ns,l,r) implies disjoint_blocks(q,k,r,l) by {
        if k == o && q == p { vacant_disjoint(bs,ns,n,m,p,o,r,l); }
        else if l == o && r == p { vacant_disjoint(bs,ns,n,m,p,o,q,k); }
        else { assert(clear_at(bs,ns,k,q) && clear_at(bs,ns,l,r)); assert(disjoint_blocks(q,k,r,l)); }
    }
    assert forall|k: int, q: int| 0 <= k < m && 0 <= q && 2*q+1 < ns[k] implies
        #[trigger] next[k].contains(2*q) || next[k].contains(2*q+1) by {
        assert(bs[k].contains(2*q) || bs[k].contains(2*q+1));
        if k == o && (2*q == p || 2*q+1 == p) {
            assert(sibling(p) == (if 2*q == p {2*q+1} else {2*q}));
        }
    }
    assert forall|x: int| pages(next,ns,n).contains(x) == pages(bs,ns,n).union(interval(p,o)).contains(x) by {
        if pages(next,ns,n).contains(x) {
            page_witness(next,ns,n,x);
            let (k,q)=choose|z: (int,int)| clear_at(next,ns,z.0,z.1) && inside(x,z.1,z.0);
            if !(k == o && q == p) { assert(clear_at(bs,ns,k,q)); page_intro(bs,ns,n,x,q,k); }
        } else if interval(p,o).contains(x) { page_intro(next,ns,n,x,p,o); }
        else if pages(bs,ns,n).contains(x) {
            page_witness(bs,ns,n,x);
            let (k,q)=choose|z: (int,int)| clear_at(bs,ns,z.0,z.1) && inside(x,z.1,z.0);
            assert(clear_at(next,ns,k,q)); page_intro(next,ns,n,x,q,k);
        }
    }
    assert(pages(next,ns,n) =~= pages(bs,ns,n).union(interval(p,o)));
}
proof fn flag_membership(a: BuddyAllocator, o: int, p: int)
    requires representation(a), 0 <= o <= a.max_order, 0 <= p < order_len(a,o),
    ensures flags(a)[o].contains(p) == flag(a,o,p),
{
    broadcast use vstd::set_lib::range_set_properties;
}
proof fn actual_pages(a: BuddyAllocator)
    requires geometry(a),
    ensures pages(flags(a),sizes(a),a.len as int) == free_pages(a),
{
    broadcast use vstd::set_lib::range_set_properties;
    assert forall|x: int| pages(flags(a),sizes(a),a.len as int).contains(x) == free_pages(a).contains(x) by {
        if pages(flags(a),sizes(a),a.len as int).contains(x) {
            page_witness(flags(a),sizes(a),a.len as int,x);
            let (o,p)=choose|z: (int,int)| clear_at(flags(a),sizes(a),z.0,z.1) && inside(x,z.1,z.0);
            flag_membership(a,o,p);
            assert(exists|k: int, q: int| 0 <= k <= a.max_order && 0 <= q < order_len(a,k) && !flag(a,k,q) &&
                q*width(k) <= x < (q+1)*width(k));
        } else if free_pages(a).contains(x) {
            let o=choose|k: int| #![trigger a.free@[k]] exists|q: int| 0 <= k <= a.max_order && 0 <= q < order_len(a,k) &&
                !flag(a,k,q) && q*width(k) <= x < (q+1)*width(k);
            let p=choose|q: int| 0 <= o <= a.max_order && 0 <= q < order_len(a,o) && !flag(a,o,q) &&
                q*width(o) <= x < (q+1)*width(o);
            flag_membership(a,o,p);
            assert(clear_at(flags(a),sizes(a),o,p));
            page_intro(flags(a),sizes(a),a.len as int,x,p,o);
        }
    }
    assert(pages(flags(a),sizes(a),a.len as int) =~= free_pages(a));
}
proof fn frame_geometry(a: BuddyAllocator, b: BuddyAllocator)
    requires geometry(a), geometry_frame(a,b), representation(b),
    ensures geometry(b), sizes(b) == sizes(a),
{
    assert(sizes(b) =~= sizes(a));
    assert forall|o: int| 0 <= o <= b.max_order implies order_len(b,o) == b.len as int/width(o) by {
        assert(order_len(a,o) == a.len as int/width(o));
        assert(storage_frame(a.free@[o].heights@,b.free@[o].heights@));
    }
}
proof fn actual_model(a: BuddyAllocator)
    requires valid(a),
    ensures geometry(a), canonical(flags(a),sizes(a),a.len as int,a.max_order as int),
        pages(flags(a),sizes(a),a.len as int) == free_pages(a),
{
    reveal(valid);
    assert(model_shape(sizes(a),a.len as int,a.max_order as int));
    assert forall|o: int, k: int, p: int, q: int| 0 <= o < k <= a.max_order &&
        clear_at(flags(a),sizes(a),o,p) && clear_at(flags(a),sizes(a),k,q) implies disjoint_blocks(p,o,q,k) by {
        flag_membership(a,o,p); flag_membership(a,k,q);
        assert((p+1)*width(o) <= q*width(k) || (q+1)*width(k) <= p*width(o));
    }
    assert forall|o: int, p: int| 0 <= o < a.max_order && 0 <= p && 2*p+1 < sizes(a)[o] implies
        #[trigger] flags(a)[o].contains(2*p) || flags(a)[o].contains(2*p+1) by {
        flag_membership(a,o,2*p); flag_membership(a,o,2*p+1);
        assert(flag(a,o,2*p) || flag(a,o,2*p+1));
    }
    actual_pages(a);
}
proof fn model_actual(a: BuddyAllocator, b: BuddyAllocator)
    requires geometry(a), geometry_frame(a,b), representation(b),
        canonical(flags(b),sizes(a),a.len as int,a.max_order as int),
    ensures valid(b), pages(flags(b),sizes(a),a.len as int) == free_pages(b),
{
    reveal(valid);
    frame_geometry(a,b);
    assert forall|o: int, k: int, p: int, q: int| 0 <= o < k <= b.max_order &&
        0 <= p < order_len(b,o) && 0 <= q < order_len(b,k) && !flag(b,o,p) && !flag(b,k,q) implies
        (p+1)*width(o) <= q*width(k) || (q+1)*width(k) <= p*width(o) by {
        flag_membership(b,o,p); flag_membership(b,k,q);
        assert(clear_at(flags(b),sizes(a),o,p) && clear_at(flags(b),sizes(a),k,q));
        assert(disjoint_blocks(p,o,q,k));
    }
    assert forall|o: int, p: int| 0 <= o < b.max_order && 0 <= p && 2*p+1 < order_len(b,o) implies
        #[trigger] flag(b,o,2*p) || flag(b,o,2*p+1) by {
        flag_membership(b,o,2*p); flag_membership(b,o,2*p+1);
        assert(flags(b)[o].contains(2*p) || flags(b)[o].contains(2*p+1));
    }
    actual_pages(b);
}
proof fn model_half(ns: Seq<int>, n: int, m: int, o: int)
    requires model_shape(ns,n,m), 0 <= o < m,
    ensures ns[o+1] == ns[o]/2,
{
    width_positive(o);
    assert(width(o+1) == 2*width(o));
    vstd::arithmetic::div_mod::lemma_div_denominator(n,width(o),2);
}
proof fn parent_partition(ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires model_shape(ns,n,m), model_legal(ns,m,p,o), o < m, sibling(p) < ns[o],
    ensures model_legal(ns,m,p/2,o+1),
        interval(p/2,o+1) == interval(p,o).union(interval(sibling(p),o)),
        interval(p,o).disjoint(interval(sibling(p),o)),
{
    broadcast use vstd::set_lib::range_set_properties;
    model_half(ns,n,m,o); width_positive(o);
    let w=width(o);
    assert(width(o+1) == 2*w);
    assert(p == 2*(p/2)+p%2);
    assert(0 <= p%2 < 2);
    assert(sibling(p) >= 0 && sibling(p) != p);
    same_order_disjoint(p,sibling(p),o);
    if p%2 == 0 {
        assert(p*w == (p/2)*(2*w)) by(nonlinear_arith)
            requires p == 2*(p/2);
        assert((p+2)*w == (p/2+1)*(2*w)) by(nonlinear_arith)
            requires p == 2*(p/2);
    } else {
        assert((p-1)*w == (p/2)*(2*w)) by(nonlinear_arith)
            requires p == 2*(p/2)+1;
        assert((p+1)*w == (p/2+1)*(2*w)) by(nonlinear_arith)
            requires p == 2*(p/2)+1;
    }
    assert((p+1)*w == p*w+w) by(nonlinear_arith);
    assert((p+2)*w == p*w+2*w) by(nonlinear_arith);
    assert((p-1)*w == p*w-w) by(nonlinear_arith);
    assert forall|x: int| interval(p/2,o+1).contains(x) == interval(p,o).union(interval(sibling(p),o)).contains(x) by {}
    assert(interval(p/2,o+1) =~= interval(p,o).union(interval(sibling(p),o)));
    assert(interval(p,o).disjoint(interval(sibling(p),o)));
}
proof fn represented_subset(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), clear_at(bs,ns,o,p),
    ensures interval(p,o).subset_of(pages(bs,ns,n)),
{
    broadcast use vstd::set_lib::range_set_properties;
    block_in_bounds(ns,n,m,p,o);
    assert forall|x: int| interval(p,o).contains(x) implies pages(bs,ns,n).contains(x) by {
        page_intro(bs,ns,n,x,p,o);
    }
}
proof fn ancestor_quotient(p: int, d: int)
    requires 0 <= p, 0 < d,
    ensures (p/2)/width(d-1) == p/width(d),
{
    width_positive(d-1);
    assert(width(d) == 2*width(d-1));
    vstd::arithmetic::div_mod::lemma_div_denominator(p,2,width(d-1));
}
proof fn canonical_buddy(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), clear_at(bs,ns,o,p),
    ensures o == m || sibling(p) >= ns[o] || bs[o].contains(sibling(p)),
{
    if o < m && sibling(p) < ns[o] {
        assert(0 <= p%2 < 2);
        assert(p == 2*(p/2)+p%2);
        assert(2*(p/2)+1 < ns[o]);
        assert(bs[o].contains(2*(p/2)) || bs[o].contains(2*(p/2)+1));
    }
}
proof fn interval_cardinality(p: int, o: int)
    requires 0 <= o,
    ensures interval(p,o).len() as int == width(o),
{
    width_positive(o);
    assert((p+1)*width(o) == p*width(o)+width(o)) by(nonlinear_arith);
    broadcast use vstd::set_lib::range_set_properties;
}
spec fn used_pages(a: BuddyAllocator) -> Set<int> { Set::<int>::range(0,a.len as int).difference(free_pages(a)) }
proof fn page_partition(a: BuddyAllocator)
    ensures free_pages(a).disjoint(used_pages(a)),
        free_pages(a).union(used_pages(a)) == Set::<int>::range(0,a.len as int),
        free_pages(a).len() + used_pages(a).len() == a.len,
{
    broadcast use vstd::set_lib::range_set_properties;
    assert(free_pages(a).subset_of(Set::<int>::range(0,a.len as int)));
    assert(free_pages(a).union(used_pages(a)) =~= Set::<int>::range(0,a.len as int));
    vstd::set_lib::lemma_set_disjoint_lens(free_pages(a),used_pages(a));
}
// Allocation completeness: canonical whole-block coverage has a free ancestor.
spec fn encloses(q: int, k: int, p: int, o: int) -> bool {
    q*width(k) <= p*width(o) && (p+1)*width(o) <= (q+1)*width(k)
}
spec fn free_cover(bs: Seq<Set<int>>, ns: Seq<int>, p: int, o: int, k: int, q: int) -> bool {
    o <= k && clear_at(bs,ns,k,q) && encloses(q,k,p,o)
}
proof fn block_point(p: int, o: int)
    requires 0 <= o,
    ensures inside(p*width(o),p,o),
{
    width_positive(o);
    assert((p+1)*width(o) == p*width(o)+width(o)) by(nonlinear_arith);
}
proof fn parent_geometry(p: int, o: int)
    requires 0 <= p, 0 <= o,
    ensures encloses(p/2,o+1,p,o),
{
    width_positive(o);
    let w=width(o);
    assert(width(o+1) == 2*w);
    assert(p == 2*(p/2)+p%2 && 0 <= p%2 < 2);
    assert(p*w == (p/2)*(2*w)+(p%2)*w) by(nonlinear_arith)
        requires p == 2*(p/2)+p%2;
    assert((p/2+1)*(2*w) == (p+1)*w+(1-p%2)*w) by(nonlinear_arith)
        requires p == 2*(p/2)+p%2;
    if p%2 == 0 { assert((p%2)*w == 0 && (1-p%2)*w == w); }
    else { assert((p%2)*w == w && (1-p%2)*w == 0); }
}
proof fn dyadic_nested(p: int, o: int, q: int, k: int)
    requires 0 <= p, 0 <= q, 0 <= o <= k,
    ensures disjoint_blocks(p,o,q,k) || encloses(q,k,p,o),
    decreases k-o,
{
    if o == k { if p != q { same_order_disjoint(p,q,o); } }
    else {
        parent_geometry(p,o);
        dyadic_nested(p/2,o+1,q,k);
    }
}
proof fn lift_point_cover(p: int, o: int, q: int, k: int, x: int)
    requires 0 <= p, 0 <= q, 0 <= o <= k, inside(x,p,o), inside(x,q,k),
    ensures encloses(q,k,p,o),
{
    dyadic_nested(p,o,q,k);
}
proof fn same_order_cover(p: int, q: int, o: int)
    requires 0 <= o, encloses(q,o,p,o),
    ensures p == q,
{
    block_point(p,o);
    if p != q { same_order_disjoint(p,q,o); }
}
proof fn cover_ancestor(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o), interval(p,o).subset_of(pages(bs,ns,n)),
    ensures exists|z: (int,int)| free_cover(bs,ns,p,o,z.0,z.1),
    decreases o,
{
    broadcast use vstd::set_lib::range_set_properties;
    if !bs[o].contains(p) { assert(free_cover(bs,ns,p,o,o,p)); }
    else if o == 0 {
        block_point(p,o);
        let x=p*width(o);
        assert(pages(bs,ns,n).contains(x));
        page_witness(bs,ns,n,x);
        let (k,q)=choose|z: (int,int)| clear_at(bs,ns,z.0,z.1) && inside(x,z.1,z.0);
        lift_point_cover(p,o,q,k,x);
        assert(free_cover(bs,ns,p,o,k,q));
    } else {
        model_half(ns,n,m,o-1);
        assert(model_legal(ns,m,2*p,o-1) && model_legal(ns,m,2*p+1,o-1));
        parent_partition(ns,n,m,2*p,o-1);
        assert(interval(2*p,o-1).subset_of(pages(bs,ns,n)));
        cover_ancestor(bs,ns,n,m,2*p,o-1);
        let (k,q)=choose|z: (int,int)| free_cover(bs,ns,2*p,o-1,z.0,z.1);
        if k >= o {
            let x=2*p*width(o-1);
            block_point(2*p,o-1);
            assert(inside(x,p,o) && inside(x,q,k));
            lift_point_cover(p,o,q,k,x);
            assert(free_cover(bs,ns,p,o,k,q));
        } else {
            same_order_cover(2*p,q,o-1);
            assert(clear_at(bs,ns,o-1,2*p));
            assert(interval(2*p+1,o-1).subset_of(pages(bs,ns,n)));
            cover_ancestor(bs,ns,n,m,2*p+1,o-1);
            let (l,r)=choose|z: (int,int)| free_cover(bs,ns,2*p+1,o-1,z.0,z.1);
            if l >= o {
                let x=(2*p+1)*width(o-1);
                block_point(2*p+1,o-1);
                assert(inside(x,p,o) && inside(x,r,l));
                lift_point_cover(p,o,r,l,x);
                assert(free_cover(bs,ns,p,o,l,r));
            } else {
                same_order_cover(2*p+1,r,o-1);
                assert(bs[o-1].contains(2*p) || bs[o-1].contains(2*p+1));
                assert(false);
            }
        }
    }
}
#[cfg(full_semantics)]
proof fn alloc_interpret(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), 0 <= p, 0 <= o,
    ensures ({let t=alloc_trace(bs,ns,p,o,m);
        canonical(t.0,ns,n,m) &&
        t.1 == (model_legal(ns,m,p,o) && interval(p,o).subset_of(pages(bs,ns,n))) &&
        (t.1 ==> pages(t.0,ns,n) == pages(bs,ns,n).difference(interval(p,o))) &&
        (!t.1 ==> t.0 == bs) }),
    decreases m+1-o,
{}
#[cfg(any(full_semantics,free_trial))]
proof fn free_interpret(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o), interval(p,o).disjoint(pages(bs,ns,n)),
    ensures ({let t=free_trace(bs,ns,p,o,m);
        canonical(t.0,ns,n,m) && o <= t.1 <= m &&
        pages(t.0,ns,n) == pages(bs,ns,n).union(interval(p,o)) &&
        clear_at(t.0,ns,t.1,p/width(t.1-o)) &&
        interval(p,o).subset_of(interval(p/width(t.1-o),t.1)) }),
    decreases m-o,
{
    broadcast use vstd::set_lib::range_set_properties;
    reveal(free_trace);
    let buddy=sibling(p);
    if o == m || buddy >= ns[o] || bs[o].contains(buddy) {
        add_block(bs,ns,n,m,p,o);
        assert(width(0) == 1);
        assert(clear_at(bs.update(o,bs[o].remove(p)),ns,o,p));
    } else {
        assert(clear_at(bs,ns,o,buddy));
        remove_block(bs,ns,n,m,buddy,o);
        represented_subset(bs,ns,n,m,buddy,o);
        parent_partition(ns,n,m,p,o);
        let next=bs.update(o,bs[o].insert(buddy));
        assert(interval(p/2,o+1).disjoint(pages(next,ns,n))) by {
            assert forall|x: int| interval(p/2,o+1).contains(x) implies !pages(next,ns,n).contains(x) by {}
        }
        free_interpret(next,ns,n,m,p/2,o+1);
        let t=free_trace(next,ns,p/2,o+1,m);
        ancestor_quotient(p,t.1-o);
        assert(pages(t.0,ns,n) =~= pages(bs,ns,n).union(interval(p,o)));
        assert(interval(p,o).subset_of(interval(p/width(t.1-o),t.1))) by {
            assert forall|x: int| interval(p,o).contains(x) implies interval(p/width(t.1-o),t.1).contains(x) by {}
        }
    }
}
#[cfg(full_semantics)]
fn record_alloc_semantic(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: bool)
    requires valid(*old(a)),
    ensures valid(*final(a)), geometry_frame(*old(a),*final(a)),
        r == (legal_block(*old(a),p,o) && requested(p,o).subset_of(free_pages(*old(a)))),
        r ==> free_pages(*final(a)) == free_pages(*old(a)).difference(requested(p,o)),
        !r ==> state_eq(*old(a),*final(a)),
{
    a.record_alloc_inner(p,o)
}
#[cfg(any(full_semantics,free_trial))]
fn free_semantic(a: &mut BuddyAllocator, p: u32, o: u8) -> (r: u8)
    requires valid(*old(a)), legal_free(*old(a),p,o),
    ensures valid(*final(a)), geometry_frame(*old(a),*final(a)),
        free_pages(*final(a)) == free_pages(*old(a)).union(requested(p,o)),
        o <= r <= old(a).max_order,
        !flag(*final(a),r as int,p as int/width(r as int-o as int)),
        requested(p,o).subset_of(interval(p as int/width(r as int-o as int),r as int)),
        r == old(a).max_order || sibling(p as int/width(r as int-o as int)) >= order_len(*final(a),r as int) ||
            flag(*final(a),r as int,sibling(p as int/width(r as int-o as int))),
        free_pages(*final(a)).len() as int == free_pages(*old(a)).len() as int + width(o as int),
        used_pages(*final(a)).len() as int + width(o as int) == used_pages(*old(a)).len() as int,
{
    let ghost before=*a;
    proof { valid_admission(before); actual_model(before); }
    let r=a.free_inner(p,o);
    proof {
        free_interpret(flags(before),sizes(before),before.len as int,before.max_order as int,p as int,o as int);
        model_actual(before,*a);
        let q=p as int/width(r as int-o as int);
        flag_membership(*a,r as int,q);
        actual_model(*a);
        canonical_buddy(flags(*a),sizes(*a),a.len as int,a.max_order as int,q,r as int);
        if r < a.max_order && sibling(q) < order_len(*a,r as int) { flag_membership(*a,r as int,sibling(q)); }
        interval_cardinality(p as int,o as int);
        vstd::set_lib::lemma_set_disjoint_lens(free_pages(before),requested(p,o));
        page_partition(before); page_partition(*a);
    }
    r
}
}
