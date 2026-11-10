use vstd::prelude::*;
verus! {
pub fn coercion(buf: &mut [u8;128])
    ensures *final(buf) == *old(buf),
{ let _r: &mut [u8] = buf; }
}
