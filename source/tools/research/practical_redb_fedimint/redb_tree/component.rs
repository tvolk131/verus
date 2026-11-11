use vstd::prelude::*;
verus! {
// All items/contracts private: original private fields never become public-open.
struct BtreeBitmap { heights: Vec<U64GroupedBitmap> }
struct U64GroupedBitmap { len: u32, data: Vec<u64> }
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
    (forall|h: int| 0 <= h < t.len()-1 ==> t[h].len == words(t[h+1].len))
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
    Set::new(|j: int| 0 <= j < t.last().len && physical_bit(t.last(),j))
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
        let full = self.get_level_mut(self.get_height() - 1).set(i);
        self.update_to_root(i, full);
    }
fn clear(&mut self, i: u32)
    requires tree_wf(old(self).heights@), i < old(self).heights@.last().len,
    ensures mutation(old(self).heights@, final(self).heights@, i, false),
{
        self.get_level_mut(self.get_height() - 1).clear(i);
        self.update_to_root(i, false);
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
        if self.get_height() == 1 {
            return;
        }

        let mut parent_height = self.get_height() - 2;
        let mut parent_entry = i / 64;
        loop {
            full = if full {
                self.get_level_mut(parent_height).set(parent_entry)
            } else {
                self.get_level_mut(parent_height).clear(parent_entry);
                false
            };

            if parent_height == 0 {
                break;
            }
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
        assert!(bit < self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        let mut group = self.data[index];
        group |= Self::select_mask(bit_index);
        self.data[index] = group;

        group == u64::MAX
    }
fn clear(&mut self, bit: u32)
    requires layer_wf(*old(self)), bit < old(self).len,
    ensures word_change(*old(self), *final(self), bit, false),
{
        assert!(bit < self.len, "{bit} must be less than {}", self.len);
        let (index, bit_index) = Self::data_index_of(bit);
        self.data[index] &= !Self::select_mask(bit_index);
    }
}
}
fn main() {}
