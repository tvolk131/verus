use vstd::prelude::*;
verus! {
pub fn plain(buf: &mut [u8; 10])
    ensures forall|j: int| #![auto] 0 <= j < 10 ==>
        final(buf)@[j] == (old(buf)@[j] ^ 1u8),
{
    for b in it: buf.iter_mut()
        invariant
            forall|j: int| #![auto] 0 <= j < it.index() ==>
                *final(it.seq()[j]) == (*it.seq()[j] ^ 1u8),
    { *b ^= 1; }
}
}
fn main() {}
