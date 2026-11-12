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
// Operational checkpoint: exact stored-bit transitions, not the interval theorem.
spec fn order_len(a: BuddyAllocator, o: int) -> int { a.free@[o].heights@.last().len as int }
spec fn representation(a: BuddyAllocator) -> bool {
    a.max_order <= 20 && a.free@.len() == a.max_order as int + 1 &&
    (forall|o: int| 0 <= o <= a.max_order ==> tree_wf(a.free@[o].heights@))
}
spec fn layout(a: BuddyAllocator) -> bool {
    representation(a) && order_len(a,0) == a.len &&
    forall|o: int| 0 <= o < a.max_order ==> #[trigger] order_len(a,o+1) == order_len(a,o)/2
}
spec fn sizes(a: BuddyAllocator) -> Seq<int> {
    Seq::new(a.free@.len(), |o: int| order_len(a,o))
}
spec fn flags(a: BuddyAllocator) -> Seq<Set<int>> {
    Seq::new(a.free@.len(), |o: int| allocated(a.free@[o].heights@))
}
spec fn state_eq(a: BuddyAllocator, b: BuddyAllocator) -> bool {
    a.len == b.len && a.max_order == b.max_order && a.free@ == b.free@
}
spec fn geometry_frame(a: BuddyAllocator, b: BuddyAllocator) -> bool {
    a.len == b.len && a.max_order == b.max_order && a.free@.len() == b.free@.len() &&
    forall|o: int| 0 <= o < a.free@.len() ==> storage_frame(a.free@[o].heights@,b.free@[o].heights@)
}
spec fn alloc_trace(bits: Seq<Set<int>>, ns: Seq<int>, p: int, o: int, m: int) -> (Seq<Set<int>>,bool)
    decreases m+1-o
{
    if o > m || p >= ns[o] { (bits,false) }
    else if bits[o].contains(p) {
        let up = alloc_trace(bits,ns,p/2,o+1,m);
        if !up.1 { (bits,false) }
        else { (up.0.update(o,up.0[o].remove(if p%2 == 0 {p+1} else {p-1})),true) }
    } else { (bits.update(o,bits[o].insert(p)),true) }
}
spec fn free_trace(bits: Seq<Set<int>>, ns: Seq<int>, p: int, o: int, m: int) -> (Seq<Set<int>>,int)
    decreases m-o
{
    let buddy = if p%2 == 0 {p+1} else {p-1};
    if o >= m || buddy >= ns[o] || bits[o].contains(buddy) {
        (bits.update(o,bits[o].remove(p)),o)
    } else { free_trace(bits.update(o,bits[o].insert(buddy)),ns,p/2,o+1,m) }
}
proof fn state_transport(a: BuddyAllocator, b: BuddyAllocator)
    requires layout(a), state_eq(a,b),
    ensures layout(b), geometry_frame(a,b), sizes(b) == sizes(a), flags(b) == flags(a),
{
    assert(representation(b));
    assert(sizes(b) =~= sizes(a));
    assert(flags(b) =~= flags(a));
    assert forall|o: int| 0 <= o < b.max_order implies #[trigger] order_len(b,o+1) == order_len(b,o)/2 by {
        assert(order_len(a,o+1) == order_len(a,o)/2);
    }
}
proof fn edit_frame(a: BuddyAllocator, b: BuddyAllocator, o: int, p: u32, value: bool)
    requires layout(a), 0 <= o <= a.max_order, p < order_len(a,o),
        a.len == b.len, a.max_order == b.max_order, a.free@.len() == b.free@.len(),
        b.free@ == a.free@.update(o,b.free@[o]),
        mutation(a.free@[o].heights@,b.free@[o].heights@,p,value),
    ensures layout(b), geometry_frame(a,b), sizes(b) == sizes(a),
        flags(b) == flags(a).update(o, if value { flags(a)[o].insert(p as int) } else { flags(a)[o].remove(p as int) }),
{
    assert forall|k: int| 0 <= k < a.free@.len() implies
        tree_wf(b.free@[k].heights@) && storage_frame(a.free@[k].heights@,b.free@[k].heights@) by {
        if k != o { assert(b.free@[k] == a.free@[k]); }
    }
    assert(representation(b));
    assert(sizes(b) =~= sizes(a));
    assert forall|k: int| 0 <= k < b.max_order implies #[trigger] order_len(b,k+1) == order_len(b,k)/2 by {
        assert(order_len(a,k+1) == order_len(a,k)/2);
        assert(sizes(a)[k] == sizes(b)[k]);
        assert(sizes(a)[k+1] == sizes(b)[k+1]);
    }
    assert(order_len(b,0) == order_len(a,0));
    assert(flags(b) =~= flags(a).update(o,if value { flags(a)[o].insert(p as int) } else { flags(a)[o].remove(p as int) }));
}
proof fn frame_trans(a: BuddyAllocator, b: BuddyAllocator, c: BuddyAllocator)
    requires geometry_frame(a,b), geometry_frame(b,c),
    ensures geometry_frame(a,c),
{
    assert forall|o: int| 0 <= o < a.free@.len() implies storage_frame(a.free@[o].heights@,c.free@[o].heights@) by {
        assert(storage_frame(a.free@[o].heights@,b.free@[o].heights@));
        assert(storage_frame(b.free@[o].heights@,c.free@[o].heights@));
    }
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
 ensures r == page_number as int / 2,
{
    page_number / 2
}
fn buddy_page(page_number: u32) -> (r: u32)
 ensures r == (page_number ^ 1), r as int == (if page_number%2 == 0 {page_number as int+1} else {page_number as int-1}),
{
    proof { assert((page_number ^ 1) == (if page_number%2 == 0 {page_number+1} else {page_number-1})) by(bit_vector); }
    page_number ^ 1
}
impl BuddyAllocator {
fn get_order_free_mut(&mut self, order: u8) -> (r: &mut BtreeBitmap)
    requires representation(*old(self)), order <= old(self).max_order,
    ensures *r == old(self).free@[order as int],
        final(self).len == old(self).len, final(self).max_order == old(self).max_order,
        final(self).free@ == old(self).free@.update(order as int,*final(r)),
{
        &mut self.free[order as usize]
    }
fn record_alloc_inner(&mut self, page_number: u32, order: u8) -> (r: bool)
    requires layout(*old(self)),
    ensures layout(*final(self)), geometry_frame(*old(self),*final(self)), sizes(*final(self)) == sizes(*old(self)),
        (flags(*final(self)),r) == alloc_trace(flags(*old(self)),sizes(*old(self)),page_number as int,order as int,old(self).max_order as int),
        !r ==> state_eq(*old(self),*final(self)),
        r ==> order <= old(self).max_order && page_number < order_len(*old(self),order as int),
        forall|o: int| 0 <= o < order as int && o < old(self).free@.len() ==> final(self).free@[o] == old(self).free@[o],
    decreases old(self).max_order as int + 1 - order as int,
{
        let ghost start = *self;
        proof { reveal(alloc_trace); }
        // Marking a page that is already allocated walks up the orders looking for a free parent to
        // split, so running off the top is how a duplicate or overlapping page number shows up
        if order > self.max_order {
            return false;
        }
        let allocator = self.get_order_free_mut(order);
        if page_number >= allocator.len() {
            proof {
                assert(*allocator == start.free@[order as int]);
                assert(self.free@ =~= start.free@);
                assert(state_eq(start,*self));
                state_transport(start,*self);
            }
            return false;
        }
        if allocator.get(page_number) {
            // Need to split parent page
            let upper_page = next_higher_order(page_number);
            proof {
                assert(*allocator == start.free@[order as int]);
                assert(self.free@ =~= start.free@);
                assert(state_eq(start,*self));
                state_transport(start,*self);
            }
            if !self.record_alloc_inner(upper_page, order + 1) {
                proof { state_transport(start,*self); }
                return false;
            }
            let ghost mid = *self;
            let allocator = self.get_order_free_mut(order);

            let (free1, free2) = (upper_page * 2, upper_page * 2 + 1);
            debug_assert!(free1 == page_number || free2 == page_number);
            if free1 == page_number {
                allocator.clear(free2);
            } else {
                allocator.clear(free1);
            }
            proof {
                edit_frame(mid,*self,order as int,(if page_number%2 == 0 {page_number+1} else {page_number-1}) as u32,false);
                frame_trans(start,mid,*self);
            }
        } else {
            allocator.set(page_number);
            proof { edit_frame(start,*self,order as int,page_number,true); }
        }

        true
    }
fn free_inner(&mut self, page_number: u32, order: u8) -> (r: u8)
    requires layout(*old(self)), order <= old(self).max_order, page_number < order_len(*old(self),order as int),
    ensures layout(*final(self)), geometry_frame(*old(self),*final(self)), sizes(*final(self)) == sizes(*old(self)),
        (flags(*final(self)),r as int) == free_trace(flags(*old(self)),sizes(*old(self)),page_number as int,order as int,old(self).max_order as int),
        order <= r <= old(self).max_order,
        forall|o: int| 0 <= o < order as int ==> final(self).free@[o] == old(self).free@[o],
    decreases old(self).max_order as int - order as int,
{
        let ghost start = *self;
        proof { reveal(free_trace); }
        if order == self.max_order {
            let allocator = self.get_order_free_mut(order);
            allocator.clear(page_number);
            proof { edit_frame(start,*self,order as int,page_number,false); }
            return order;
        }

        let allocator = self.get_order_free_mut(order);
        let buddy = buddy_page(page_number);
        if buddy >= allocator.len() || allocator.get(buddy) {
            allocator.clear(page_number);
            proof { edit_frame(start,*self,order as int,page_number,false); }
            order
        } else {
            // Merge into higher order page
            allocator.set(buddy);
            proof { edit_frame(start,*self,order as int,buddy,true); }
            let ghost mid = *self;
            self.free_inner(next_higher_order(page_number), order + 1)
        }
    }
}
}

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
    ensures interval(p,o).subset_of(Set::<int>::range(0,n)), 0 <= p*width(o), (p+1)*width(o) <= n,
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
    vstd::arithmetic::mul::lemma_mul_nonnegative(p%2,w);
    vstd::arithmetic::mul::lemma_mul_nonnegative(1-p%2,w);
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
    if !bs[o].contains(p) { assert(free_cover(bs,ns,p,o,(o,p).0,(o,p).1)); }
    else if o == 0 {
        block_point(p,o);
        let x=p*width(o);
        assert(pages(bs,ns,n).contains(x));
        page_witness(bs,ns,n,x);
        let (k,q)=choose|z: (int,int)| clear_at(bs,ns,z.0,z.1) && inside(x,z.1,z.0);
        lift_point_cover(p,o,q,k,x);
        assert(free_cover(bs,ns,p,o,(k,q).0,(k,q).1));
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
            assert(encloses(q,k,2*p,o-1));
            assert(inside(x,q,k));
            assert(interval(2*p,o-1).contains(x));
            assert(interval(p,o).contains(x));
            assert(inside(x,p,o));
            lift_point_cover(p,o,q,k,x);
            assert(free_cover(bs,ns,p,o,(k,q).0,(k,q).1));
        } else {
            same_order_cover(2*p,q,o-1);
            assert(clear_at(bs,ns,o-1,2*p));
            assert(interval(2*p+1,o-1).subset_of(pages(bs,ns,n)));
            cover_ancestor(bs,ns,n,m,2*p+1,o-1);
            let (l,r)=choose|z: (int,int)| free_cover(bs,ns,2*p+1,o-1,z.0,z.1);
            if l >= o {
                let x=(2*p+1)*width(o-1);
                block_point(2*p+1,o-1);
                assert(encloses(r,l,2*p+1,o-1));
                assert(inside(x,r,l));
                assert(interval(2*p+1,o-1).contains(x));
                assert(interval(p,o).contains(x));
                assert(inside(x,p,o));
                lift_point_cover(p,o,r,l,x);
                assert(free_cover(bs,ns,p,o,(l,r).0,(l,r).1));
            } else {
                same_order_cover(2*p+1,r,o-1);
                assert(bs[o-1].contains(2*p) || bs[o-1].contains(2*p+1));
                assert(false);
            }
        }
    }
}
proof fn legal_from_end(ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires model_shape(ns,n,m), 0 <= o <= m, 0 <= p, (p+1)*width(o) <= n,
    ensures model_legal(ns,m,p,o),
{
    width_positive(o);
    vstd::arithmetic::div_mod::lemma_div_is_ordered((p+1)*width(o),n,width(o));
    vstd::arithmetic::div_mod::lemma_div_multiples_vanish(p+1,width(o));
    assert((p+1)*width(o) == width(o)*(p+1)) by(nonlinear_arith);
}
proof fn parent_whole(bs: Seq<Set<int>>, ns: Seq<int>, n: int, m: int, p: int, o: int)
    requires canonical(bs,ns,n,m), model_legal(ns,m,p,o), bs[o].contains(p), interval(p,o).subset_of(pages(bs,ns,n)),
    ensures o < m, model_legal(ns,m,p/2,o+1), interval(p/2,o+1).subset_of(pages(bs,ns,n)),
{
    broadcast use vstd::set_lib::range_set_properties;
    cover_ancestor(bs,ns,n,m,p,o);
    let (k,q)=choose|z: (int,int)| free_cover(bs,ns,p,o,z.0,z.1);
    if k == o { same_order_cover(p,q,o); assert(false); }
    block_point(p,o); parent_geometry(p,o);
    lift_point_cover(p/2,o+1,q,k,p*width(o));
    block_in_bounds(ns,n,m,q,k);
    legal_from_end(ns,n,m,p/2,o+1);
    represented_subset(bs,ns,n,m,q,k);
    assert forall|x: int| interval(p/2,o+1).contains(x) implies pages(bs,ns,n).contains(x) by {
        assert(interval(q,k).contains(x));
    }
}
proof fn alloc_trace_frame(bs: Seq<Set<int>>, ns: Seq<int>, p: int, o: int, m: int)
    requires 0 <= p, 0 <= o, 0 <= m, bs.len() == m+1, ns.len() == bs.len(),
    ensures alloc_trace(bs,ns,p,o,m).0.len() == bs.len(),
        forall|k: int| 0 <= k < o && k < bs.len() ==> alloc_trace(bs,ns,p,o,m).0[k] == bs[k],
    decreases m+1-o,
{
    reveal(alloc_trace);
    if o <= m && p < ns[o] && bs[o].contains(p) {
        alloc_trace_frame(bs,ns,p/2,o+1,m);
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
{
    broadcast use vstd::set_lib::range_set_properties;
    reveal(alloc_trace);
    if o > m || p >= ns[o] {} else if !bs[o].contains(p) {
        remove_block(bs,ns,n,m,p,o); represented_subset(bs,ns,n,m,p,o);
    } else {
        alloc_interpret(bs,ns,n,m,p/2,o+1);
        let up=alloc_trace(bs,ns,p/2,o+1,m);
        if !up.1 {
            if interval(p,o).subset_of(pages(bs,ns,n)) { parent_whole(bs,ns,n,m,p,o); assert(false); }
        } else {
            alloc_trace_frame(bs,ns,p/2,o+1,m);
            model_half(ns,n,m,o);
            let buddy=sibling(p);
            assert(model_legal(ns,m,buddy,o));
            assert(sibling(buddy) == p);
            parent_partition(ns,n,m,p,o);
            assert(interval(buddy,o).disjoint(pages(up.0,ns,n))) by {
                assert forall|x: int| interval(buddy,o).contains(x) implies !pages(up.0,ns,n).contains(x) by {}
            }
            assert(up.0[o] == bs[o]);
            add_block(up.0,ns,n,m,buddy,o);
            let next=up.0.update(o,up.0[o].remove(buddy));
            assert(interval(p,o).subset_of(pages(bs,ns,n)));
            assert(pages(next,ns,n) =~= pages(bs,ns,n).difference(interval(p,o)));
        }
    }
}
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
        r ==> free_pages(*final(a)).len() as int + width(o as int) == free_pages(*old(a)).len() as int,
        r ==> used_pages(*final(a)).len() as int == used_pages(*old(a)).len() as int + width(o as int),
{
    let ghost before=*a;
    proof { valid_admission(before); actual_model(before); }
    let r=a.record_alloc_inner(p,o);
    proof {
        alloc_interpret(flags(before),sizes(before),before.len as int,before.max_order as int,p as int,o as int);
        model_actual(before,*a);
        if r {
            interval_cardinality(p as int,o as int);
            assert(free_pages(before).intersect(requested(p,o)) =~= requested(p,o));
            vstd::set_lib::lemma_set_difference_len(free_pages(before),requested(p,o));
            page_partition(before); page_partition(*a);
        }
    }
    r
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
