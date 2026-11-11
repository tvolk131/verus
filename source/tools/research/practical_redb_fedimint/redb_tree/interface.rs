// Checked testing facade. Original tree/fields/methods remain private.
// This is not an extracted BuddyAllocator body or a new bitmap algorithm.
verus! {
pub struct CheckedBitmap { tree: BtreeBitmap }
impl CheckedBitmap {
    pub closed spec fn valid(&self) -> bool { tree_wf(self.tree.heights@) }
    pub closed spec fn logical_len(&self) -> u32 { self.tree.heights@.last().len }
    pub closed spec fn members(&self) -> Set<int> { allocated(self.tree.heights@) }
    pub closed spec fn allocated_count(&self) -> nat { count(self.tree.heights@) }
    pub fn mark_allocated(&mut self, i: u32)
        requires old(self).valid(), i < old(self).logical_len(),
        ensures final(self).valid(), final(self).logical_len() == old(self).logical_len(),
            final(self).members() == old(self).members().insert(i as int),
            final(self).allocated_count() as int == old(self).allocated_count() as int +
                (if old(self).members().contains(i as int) { 0int } else { 1int }),
    {
        proof { broadcast use vstd::set_lib::range_set_properties; }
        self.tree.set(i);
    }
    pub fn mark_free(&mut self, i: u32)
        requires old(self).valid(), i < old(self).logical_len(),
        ensures final(self).valid(), final(self).logical_len() == old(self).logical_len(),
            final(self).members() == old(self).members().remove(i as int),
            final(self).allocated_count() as int == old(self).allocated_count() as int -
                (if old(self).members().contains(i as int) { 1int } else { 0int }),
    {
        proof { broadcast use vstd::set_lib::range_set_properties; }
        self.tree.clear(i);
    }
}
}
