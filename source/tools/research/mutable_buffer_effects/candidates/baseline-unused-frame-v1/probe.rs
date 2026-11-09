use vstd::prelude::*;
verus! {
pub fn unused_iterator(buf: &mut [u8])
    ensures final(buf)@ == old(buf)@,
{
    let it = buf.iter_mut();
}
}
