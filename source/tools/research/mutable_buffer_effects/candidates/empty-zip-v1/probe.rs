use vstd::prelude::*;
verus! {
pub fn empty_zip(buf: &mut [u8])
    ensures final(buf)@ == old(buf)@,
{
    let key: &[u8] = &[];
    for (b_i, b_h) in buf.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}
}
