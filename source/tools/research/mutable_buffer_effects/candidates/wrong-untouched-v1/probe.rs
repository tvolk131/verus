use vstd::prelude::*;
verus! {
pub fn changed_slot_is_not_unchanged(buf: &mut [u8], index: usize, value: u8)
    requires index < old(buf).len(),
    ensures final(buf)@ == old(buf)@,
{
    buf[index] = value;
}
}
