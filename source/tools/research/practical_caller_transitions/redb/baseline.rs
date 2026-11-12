use vstd::prelude::*;
verus! {
// All items/contracts private: original private fields never become public-open.
struct BtreeBitmap {
    heights: Vec<U64GroupedBitmap>,
}
struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}
spec fn words(n: u32) -> int { (n as int + 63) / 64 }
spec fn member(w: u64, b: int) -> bool { (w & (1u64 << (b as u64))) != 0 }
spec fn physical_bit(t: U64GroupedBitmap, j: int) -> bool { member(t.data@[j / 64], j % 64) }
spec fn layer_wf(t: U64GroupedBitmap) -> bool {
    words(t.len) <= t.data@.len() &&
    forall|j: int| t.len <= j < words(t.len) * 64 ==> physical_bit(t, j)
}
spec fn shape(t: Seq<U64GroupedBitmap>) -> bool {
    0 < t.len() <= u32::MAX && t[0].len <= 64 &&
    (forall|h: int| 0 <= h < t.len() ==> layer_wf(t[h])) &&
    (forall|h: int| 0 <= h < t.len()-1 ==> #[trigger] t[h].len == words(t[h+1].len))
}
spec fn edge(t: Seq<U64GroupedBitmap>, h: int, j: int) -> bool {
    physical_bit(t[h], j) == (t[h+1].data@[j] == u64::MAX)
}
spec fn tree_wf(t: Seq<U64GroupedBitmap>) -> bool {
    shape(t) && (forall|h: int, j: int| 0 <= h < t.len()-1 && 0 <= j < t[h].len ==> edge(t,h,j))
}
spec fn ancestor(i: int, distance: int) -> int
    decreases distance
{
    if distance <= 0 { i } else { ancestor(i / 64, distance-1) }
}
spec fn storage_frame(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>) -> bool {
    a.len() == b.len() && forall|h: int| 0 <= h < a.len() ==>
        a[h].len == b[h].len && a[h].data@.len() == b[h].data@.len()
}
spec fn path_frame(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, i: int) -> bool {
    storage_frame(a,b) && forall|h: int, j: int| 0 <= h < a.len() && 0 <= j < a[h].data@.len()*64 &&
        j != ancestor(i, a.len()-1-h) ==> physical_bit(b[h],j) == physical_bit(a[h],j)
}
spec fn allocated(t: Seq<U64GroupedBitmap>) -> Set<int> {
    Set::<int>::range(0, t.last().len as int).filter(|j: int| physical_bit(t.last(),j))
}
spec fn count(t: Seq<U64GroupedBitmap>) -> nat { allocated(t).len() }
spec fn mutation(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, i: u32, value: bool) -> bool {
    tree_wf(b) && path_frame(a,b,i as int) && physical_bit(b.last(),i as int) == value &&
    allocated(b) == (if value { allocated(a).insert(i as int) } else { allocated(a).remove(i as int) }) &&
    count(b) as int == count(a) as int + (if value && !physical_bit(a.last(),i as int) { 1int }
        else if !value && physical_bit(a.last(),i as int) { -1int } else { 0int })
}
// State immediately after a single leaf edit: only its parent edge may be stale.
spec fn pending(t: Seq<U64GroupedBitmap>, i: u32) -> bool {
    shape(t) && i < t.last().len &&
    (forall|h: int, j: int| 0 <= h < t.len()-1 && 0 <= j < t[h].len &&
        !(h == t.len()-2 && j == i as int / 64) ==> edge(t,h,j))
}
spec fn word_change(a: U64GroupedBitmap, b: U64GroupedBitmap, i: u32, value: bool) -> bool {
    b.len == a.len && b.data@.len() == a.data@.len() && layer_wf(b) &&
    b.data@ == a.data@.update(i as int / 64, if value {
        a.data@[i as int / 64] | (1u64 << ((i as int % 64) as u64))
    } else { a.data@[i as int / 64] & !(1u64 << ((i as int % 64) as u64)) }) &&
    (forall|j: int| 0 <= j < a.data@.len()*64 ==> physical_bit(b,j) == (if j == i as int { value } else { physical_bit(a,j) }))
}
proof fn word_bits(x: u64, p: u64, j: u64)
    requires p < 64, j < 64,
    ensures
        member(x | (1u64 << p), j as int) == (j == p || member(x, j as int)),
        member(x & !(1u64 << p), j as int) == (j != p && member(x, j as int)),
        member(u64::MAX, j as int),
{
    assert((p < 64 && j < 64) ==> (((x | (1u64 << p)) & (1u64 << j) != 0) ==
        (j == p || (x & (1u64 << j) != 0)))) by(bit_vector);
    assert(((x & !(1u64 << p)) & (1u64 << j) != 0) ==
        (j != p && (x & (1u64 << j) != 0))) by(bit_vector);
    assert(j < 64 ==> (u64::MAX & (1u64 << j)) != 0) by(bit_vector);
}
proof fn word_frame(a: U64GroupedBitmap, b: U64GroupedBitmap, i: u32, value: bool)
    requires layer_wf(a), i < a.len,
        b.len == a.len,
        b.data@ == a.data@.update(i as int / 64, if value {
            a.data@[i as int / 64] | (1u64 << ((i as int % 64) as u64))
        } else { a.data@[i as int / 64] & !(1u64 << ((i as int % 64) as u64)) }),
    ensures word_change(a,b,i,value),
{
    assert(0 <= i as int / 64 < a.data@.len());
    assert forall|j: int| 0 <= j < a.data@.len()*64 implies
        physical_bit(b,j) == (if j == i as int { value } else { physical_bit(a,j) }) by {
        if j / 64 == i as int / 64 {
            word_bits(a.data@[j/64], (i as int % 64) as u64, (j%64) as u64);
            assert((j % 64 == i as int % 64) == (j == i as int));
        }
    }
    assert forall|j: int| b.len <= j < words(b.len)*64 implies physical_bit(b,j) by {
        assert(j != i as int);
        assert(physical_bit(a,j));
    }
}

proof fn layer_edit(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, h: int, i: u32, value: bool)
    requires shape(a), 0 <= h < a.len(), i < a[h].len,
        b == a.update(h,b[h]), word_change(a[h],b[h],i,value),
    ensures shape(b), storage_frame(a,b),
        forall|k: int| 0 <= k < a.len() && k != h ==> b[k] == a[k],
        forall|k: int, j: int| 0 <= k < a.len() && 0 <= j < a[k].data@.len()*64 &&
            !(k == h && j == i as int) ==> physical_bit(b[k],j) == physical_bit(a[k],j),
{
    assert forall|k: int| 0 <= k < a.len() implies layer_wf(b[k]) by {
        if k != h { assert(b[k] == a[k]); }
    }
    assert forall|k: int| 0 <= k < a.len() implies
        b[k].len == a[k].len && b[k].data@.len() == a[k].data@.len() by {
        if k != h { assert(b[k] == a[k]); }
    }
    assert forall|k: int| 0 <= k < b.len()-1 implies
        #[trigger] b[k].len == words(b[k+1].len) by {
        assert(b[k].len == a[k].len);
        assert(b[k+1].len == a[k+1].len);
    }
}
proof fn leaf_edit(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, i: u32, value: bool)
    requires tree_wf(a), i < a.last().len,
        b == a.update(a.len()-1,b.last()), word_change(a.last(),b.last(),i,value),
    ensures pending(b,i), path_frame(a,b,i as int),
{
    layer_edit(a,b,a.len()-1,i,value);
    assert forall|h: int, j: int| 0 <= h < b.len()-1 && 0 <= j < b[h].len &&
        !(h == b.len()-2 && j == i as int / 64) implies edge(b,h,j) by {
        assert(b[h] == a[h]);
        if h != b.len()-2 { assert(b[h+1] == a[h+1]); }
        else { assert(b[h+1].data@[j] == a[h+1].data@[j]); }
        assert(edge(a,h,j));
    }
}
proof fn clear_not_full(t: U64GroupedBitmap, i: u32)
    requires i < t.len, layer_wf(t), !physical_bit(t,i as int),
    ensures t.data@[i as int / 64] != u64::MAX,
{
    word_bits(t.data@[i as int / 64], (i as int % 64) as u64, (i as int % 64) as u64);
}
proof fn finish_mutation(a: Seq<U64GroupedBitmap>, m: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, i: u32, value: bool)
    requires tree_wf(a), i < a.last().len, path_frame(a,m,i as int),
        word_change(a.last(),m.last(),i,value),
        tree_wf(b), path_frame(m,b,i as int), b.last() == m.last(),
    ensures mutation(a,b,i,value),
{
    broadcast use vstd::set_lib::range_set_properties;
    assert forall|h: int, j: int| 0 <= h < a.len() && 0 <= j < a[h].data@.len()*64 &&
        j != ancestor(i as int,a.len()-1-h) implies physical_bit(b[h],j) == physical_bit(a[h],j) by {
        assert(physical_bit(m[h],j) == physical_bit(a[h],j));
        assert(physical_bit(b[h],j) == physical_bit(m[h],j));
    }
    assert(allocated(b) =~= (if value { allocated(a).insert(i as int) } else { allocated(a).remove(i as int) }));
}
spec fn pending_at(t: Seq<U64GroupedBitmap>, h: int, e: int) -> bool {
    shape(t) && 0 <= h < t.len()-1 && 0 <= e < t[h].len &&
    forall|k: int, j: int| 0 <= k < t.len()-1 && 0 <= j < t[k].len &&
        !(k == h && j == e) ==> edge(t,k,j)
}
proof fn propagation(a: Seq<U64GroupedBitmap>, b: Seq<U64GroupedBitmap>, h: int, e: u32, value: bool)
    requires pending_at(a,h,e as int), value == (a[h+1].data@[e as int] == u64::MAX),
        b == a.update(h,b[h]), word_change(a[h],b[h],e,value),
    ensures h == 0 ==> tree_wf(b),
        h > 0 ==> pending_at(b,h-1,e as int / 64),
        shape(b), storage_frame(a,b), b.last() == a.last(),
        forall|k: int, j: int| 0 <= k < a.len() && 0 <= j < a[k].data@.len()*64 &&
            !(k == h && j == e as int) ==> physical_bit(b[k],j) == physical_bit(a[k],j),
{
    layer_edit(a,b,h,e,value);
    assert forall|k: int, j: int| 0 <= k < b.len()-1 && 0 <= j < b[k].len &&
        !(k == h-1 && j == e as int / 64) implies edge(b,k,j) by {
        if k == h {
            assert(b[k+1] == a[k+1]);
            if j == e as int { assert(physical_bit(b[k],j) == value); }
            else { assert(edge(a,k,j)); }
        } else if k+1 == h {
            assert(b[k] == a[k]);
            assert(b[k+1].data@[j] == a[k+1].data@[j]);
            assert(edge(a,k,j));
        } else {
            assert(b[k] == a[k]); assert(b[k+1] == a[k+1]); assert(edge(a,k,j));
        }
    }
}
proof fn ancestor_next(i: int, d: int)
    requires 0 <= i, 0 <= d,
    ensures ancestor(i,d+1) == ancestor(i,d)/64,
    decreases d,
{
    reveal_with_fuel(ancestor,2);
    if d > 0 { ancestor_next(i/64,d-1); }
}

impl BtreeBitmap {
fn get_level(&self, i: u32) -> (r: &U64GroupedBitmap)
    requires i < self.heights@.len(), self.heights@.len() <= u32::MAX,
    ensures *r == self.heights@[i as int],
{
        assert!(i < self.get_height());
        &self.heights[i as usize]
    }
fn get_height(&self) -> (r: u32)
    requires self.heights@.len() <= u32::MAX,
    ensures r == self.heights@.len(),
{
        self.heights.len().try_into().unwrap()
    }
fn set(&mut self, i: u32)
    requires tree_wf(old(self).heights@), i < old(self).heights@.last().len,
    ensures mutation(old(self).heights@, final(self).heights@, i, true),
{
        let ghost before = self.heights@;
        let full = self.get_level_mut(self.get_height() - 1).set(i);
        let ghost middle = self.heights@;
        proof { leaf_edit(before,middle,i,true); }
        self.update_to_root(i, full);
        proof { finish_mutation(before,middle,self.heights@,i,true); }
    }
fn clear(&mut self, i: u32)
    requires tree_wf(old(self).heights@), i < old(self).heights@.last().len,
    ensures mutation(old(self).heights@, final(self).heights@, i, false),
{
        let ghost before = self.heights@;
        self.get_level_mut(self.get_height() - 1).clear(i);
        let ghost middle = self.heights@;
        proof { leaf_edit(before,middle,i,false); clear_not_full(middle.last(),i); }
        self.update_to_root(i, false);
        proof { finish_mutation(before,middle,self.heights@,i,false); }
    }
fn get_level_mut(&mut self, i: u32) -> (r: &mut U64GroupedBitmap)
    requires i < old(self).heights@.len(), old(self).heights@.len() <= u32::MAX,
    ensures *r == old(self).heights@[i as int],
        final(self).heights@ == old(self).heights@.update(i as int, *final(r)),
{
        assert!(i < self.get_height());
        &mut self.heights[i as usize]
    }
fn update_to_root(&mut self, i: u32, mut full: bool)
    requires pending(old(self).heights@, i),
        full == (old(self).heights@.last().data@[i as int / 64] == u64::MAX),
    ensures tree_wf(final(self).heights@),
        path_frame(old(self).heights@, final(self).heights@, i as int),
        final(self).heights@.last() == old(self).heights@.last(),
{
        let ghost start = self.heights@;
        if self.get_height() == 1 {
            return;
        }

        let mut parent_height = self.get_height() - 2;
        let mut parent_entry = i / 64;
        proof { reveal_with_fuel(ancestor,2); }
        loop
            invariant_except_break
                pending_at(self.heights@, parent_height as int, parent_entry as int),
                storage_frame(start,self.heights@),
                path_frame(start,self.heights@,i as int),
                self.heights@.last() == start.last(),
                parent_entry as int == ancestor(i as int,start.len()-1-parent_height as int),
                full == (self.heights@[parent_height as int+1].data@[parent_entry as int] == u64::MAX),
            ensures
                tree_wf(self.heights@),
                path_frame(start,self.heights@,i as int),
                self.heights@.last() == start.last(),
            decreases parent_height,
        {
            let ghost step = self.heights@;
            let ghost was_full = full;
            full = if full {
                self.get_level_mut(parent_height).set(parent_entry)
            } else {
                self.get_level_mut(parent_height).clear(parent_entry);
                false
            };

            proof {
                propagation(step,self.heights@,parent_height as int,parent_entry,was_full);
                if !was_full { clear_not_full(self.heights@[parent_height as int],parent_entry); }
                assert forall|h: int, j: int| 0 <= h < start.len() && 0 <= j < start[h].data@.len()*64 &&
                    j != ancestor(i as int,start.len()-1-h) implies physical_bit(self.heights@[h],j) == physical_bit(start[h],j) by {
                    assert(physical_bit(step[h],j) == physical_bit(start[h],j));
                }
            }
            if parent_height == 0 {
                break;
            }
            proof { ancestor_next(i as int,start.len()-1-parent_height as int); }
            parent_height -= 1;
            parent_entry /= 64;
        }
    }
}
impl U64GroupedBitmap {
fn data_index_of(bit: u32) -> (r: (usize, usize))
    ensures r.0 == bit as int / 64, r.1 == bit as int % 64, r.1 < 64,
{
        ((bit as usize) / 64, (bit as usize) % 64)
    }
fn select_mask(bit: usize) -> (r: u64)
    requires bit < 64,
    ensures r == (1u64 << (bit as u64)),
{
        1u64 << (bit as u64)
    }
fn len(&self) -> (r: u32)
    ensures r == self.len,
{
        self.len
    }
fn get(&self, bit: u32) -> (r: bool)
    requires layer_wf(*self), bit < self.len,
    ensures r == physical_bit(*self, bit as int),
{
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let group = self.data[index];
        group & U64GroupedBitmap::select_mask(bit_index) != 0
    }
fn set(&mut self, bit: u32) -> (r: bool)
    requires layer_wf(*old(self)), bit < old(self).len,
    ensures word_change(*old(self), *final(self), bit, true),
        r == (final(self).data@[bit as int / 64] == u64::MAX),
{
        let ghost before = *self;
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let mut group = self.data[index];
        group |= Self::select_mask(bit_index);
        self.data[index] = group;

        proof { word_frame(before, *self, bit, true); }
        group == u64::MAX
    }
fn clear(&mut self, bit: u32)
    requires layer_wf(*old(self)), bit < old(self).len,
    ensures word_change(*old(self), *final(self), bit, false),
{
        let ghost before = *self;
        assert!(bit < self.len, "{bit} must be less than {}", self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        self.data[index] &= !Self::select_mask(bit_index);
        proof { word_frame(before, *self, bit, false); }
    }
}
}

verus! {
struct BuddyAllocator {
    free: Vec<BtreeBitmap>,
    len: u32,
    max_order: u8,
}
spec fn width(o: int) -> int decreases o { if o <= 0 { 1 } else { 2 * width(o-1) } }
spec fn order_len(a: BuddyAllocator, o: int) -> int { a.free@[o].heights@.last().len as int }
spec fn flag(a: BuddyAllocator, o: int, p: int) -> bool { physical_bit(a.free@[o].heights@.last(),p) }
spec fn representation(a: BuddyAllocator) -> bool {
    a.max_order <= 20 && a.free@.len() == a.max_order as int + 1 &&
    (forall|o: int| 0 <= o <= a.max_order ==> tree_wf(a.free@[o].heights@))
}
spec fn geometry(a: BuddyAllocator) -> bool {
    representation(a) &&
    forall|o: int| 0 <= o <= a.max_order ==> order_len(a,o) == a.len as int / width(o)
}
spec fn valid(a: BuddyAllocator) -> bool {
    geometry(a) &&
    // Two zero bits must never cover the same order-zero page.
    (forall|o: int, k: int, p: int, q: int| 0 <= o < k <= a.max_order &&
        0 <= p < order_len(a,o) && 0 <= q < order_len(a,k) && !flag(a,o,p) && !flag(a,k,q)
        ==> (p+1)*width(o) <= q*width(k) || (q+1)*width(k) <= p*width(o)) &&
    // Existing in-range buddies cannot both be represented free below the maximum.
    (forall|o: int, p: int| 0 <= o < a.max_order && 0 <= p && 2*p+1 < order_len(a,o)
        ==> flag(a,o,2*p) || flag(a,o,2*p+1))
}
spec fn free_pages(a: BuddyAllocator) -> Set<int> {
    Set::new(|x: int| 0 <= x < a.len && exists|o: int, p: int| 0 <= o <= a.max_order &&
        0 <= p < order_len(a,o) && !flag(a,o,p) && p*width(o) <= x < (p+1)*width(o))
}
spec fn requested(p: u32, o: u8) -> Set<int> {
    Set::new(|x: int| p as int * width(o as int) <= x < (p as int+1)*width(o as int))
}
spec fn legal_block(a: BuddyAllocator, p: u32, o: u8) -> bool {
    o <= a.max_order && p < order_len(a,o as int)
}
spec fn legal_free(a: BuddyAllocator, p: u32, o: u8) -> bool {
    legal_block(a,p,o) && requested(p,o).disjoint(free_pages(a))
}
spec fn geometry_frame(a: BuddyAllocator, b: BuddyAllocator) -> bool {
    a.len == b.len && a.max_order == b.max_order && a.free@.len() == b.free@.len() &&
    forall|o: int| 0 <= o < a.free@.len() ==> storage_frame(a.free@[o].heights@,b.free@[o].heights@)
}
impl BtreeBitmap {
fn get(&self, i: u32) -> (r: bool)
    requires tree_wf(self.heights@), i < self.heights@.last().len,
    ensures r == physical_bit(self.heights@.last(),i as int),
{
        self.get_level(self.get_height() - 1).get(i)
    }
fn len(&self) -> (r: u32)
    requires tree_wf(self.heights@),
    ensures r == self.heights@.last().len,
{
        self.get_level(self.get_height() - 1).len()
    }
}
fn next_higher_order(page_number: u32) -> (r: u32)
 ensures r == page_number as int / 2, {
    page_number / 2
}
fn buddy_page(page_number: u32) -> (r: u32)
 ensures r == (page_number ^ 1), {
    page_number ^ 1
}
impl BuddyAllocator {
fn get_order_free_mut(&mut self, order: u8) -> (r: &mut BtreeBitmap)
    requires representation(*old(self)), order <= old(self).max_order,
    ensures *r == old(self).free@[order as int],
        final(self).len == old(self).len, final(self).max_order == old(self).max_order,
        final(self).free@ == old(self).free@.update(order as int,*final(r)), {
        &mut self.free[order as usize]
    }
fn record_alloc_inner(&mut self, page_number: u32, order: u8) -> (r: bool)
    requires valid(*old(self)),
    ensures valid(*final(self)), geometry_frame(*old(self),*final(self)),
        r == (legal_block(*old(self),page_number,order) && requested(page_number,order).subset_of(free_pages(*old(self)))),
        r ==> free_pages(*final(self)) == free_pages(*old(self)).difference(requested(page_number,order)),
        !r ==> *final(self) == *old(self),
    decreases old(self).max_order as int + 1 - order as int, {
        // Marking a page that is already allocated walks up the orders looking for a free parent to
        // split, so running off the top is how a duplicate or overlapping page number shows up
        if order > self.max_order {
            return false;
        }
        let allocator = self.get_order_free_mut(order);
        if page_number >= allocator.len() {
            return false;
        }
        if allocator.get(page_number) {
            // Need to split parent page
            let upper_page = next_higher_order(page_number);
            if !self.record_alloc_inner(upper_page, order + 1) {
                return false;
            }
            let allocator = self.get_order_free_mut(order);

            let (free1, free2) = (upper_page * 2, upper_page * 2 + 1);
            debug_assert!(free1 == page_number || free2 == page_number);
            if free1 == page_number {
                allocator.clear(free2);
            } else {
                allocator.clear(free1);
            }
        } else {
            allocator.set(page_number);
        }

        true
    }
fn free_inner(&mut self, page_number: u32, order: u8) -> (r: u8)
    requires valid(*old(self)), legal_free(*old(self),page_number,order),
    ensures valid(*final(self)), geometry_frame(*old(self),*final(self)),
        free_pages(*final(self)) == free_pages(*old(self)).union(requested(page_number,order)),
        order <= r <= old(self).max_order,
        !flag(*final(self),r as int,page_number as int / width(r as int-order as int)),
    decreases old(self).max_order as int - order as int, {
        if order == self.max_order {
            let allocator = self.get_order_free_mut(order);
            allocator.clear(page_number);
            return order;
        }

        let allocator = self.get_order_free_mut(order);
        let buddy = buddy_page(page_number);
        if buddy >= allocator.len() || allocator.get(buddy) {
            allocator.clear(page_number);
            order
        } else {
            // Merge into higher order page
            allocator.set(buddy);
            self.free_inner(next_higher_order(page_number), order + 1)
        }
    }
}
}
