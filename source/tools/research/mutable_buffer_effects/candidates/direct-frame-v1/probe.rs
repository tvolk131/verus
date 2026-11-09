use vstd::prelude::*;
verus! {
pub fn direct_reborrow(buf: &mut [u8])
    ensures final(buf)@ == old(buf)@,
{
    let borrow = &mut *buf;
}
pub fn direct_slot(buf: &mut [u8], index: usize, value: u8)
    requires index < old(buf).len(),
    ensures final(buf)@ == old(buf)@.update(index as int, value),
{
    buf[index] = value;
}
}
