// Appended to the unchanged accepted R2 manual provider; original accessor body.
verus! {
struct BuddyAllocator {
    free: Vec<BtreeBitmap>,
    len: u32,
    max_order: u8,
}
impl BuddyAllocator {
    fn get_order_free_mut(&mut self, order: u8) -> (r: &mut BtreeBitmap)
        requires order < old(self).free@.len(),
        ensures *r == old(self).free@[order as int],
            final(self).free@ == old(self).free@.update(order as int, *final(r)),
            final(self).len == old(self).len,
            final(self).max_order == old(self).max_order,
    {
        &mut self.free[order as usize]
    }
}
}
