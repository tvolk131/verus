use vstd::prelude::*;
verus! {
pub fn reborrow(buf: &mut [u8;128])
    ensures *final(buf) == *old(buf),
{ let _r = &mut *buf; }
}
