use vstd::prelude::*;
verus! {
pub fn enter<'a>(buf: &'a mut [u8;128]) -> (out: Tracked<&'a mut [u8;128]>)
    ensures *(out@) == *old(buf), *final(out@) == *final(buf),
{
    let tracked r = &mut *buf;
    Tracked(r)
}
}
