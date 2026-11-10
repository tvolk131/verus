use vstd::prelude::*;
verus! {
pub fn enter<'a>(buf: &'a mut [u8;128]) -> (out: Tracked<&'a mut [u8]>)
    ensures (out@)@ == old(buf)@, final(out@)@ == final(buf)@,
{
    let tracked r: &mut [u8] = &mut *buf;
    Tracked(r)
}
pub proof fn borrow_slot<'a>(tracked buf: &'a mut [u8], i: int) -> (tracked out: &'a mut u8)
    requires 0 <= i < buf.len(),
    ensures *out == old(buf)@[i], final(buf)@ == old(buf)@.update(i,*final(out)),
{ vstd::slice::tracked_borrow_mut(buf,i) }
}
