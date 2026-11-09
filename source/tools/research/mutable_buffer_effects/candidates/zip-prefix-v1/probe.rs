use vstd::prelude::*;
verus! {
pub fn zip_xor_prefix(buf: &mut [u8], key: &[u8])
    ensures
        final(buf).len() == old(buf).len(),
        forall|j: int| 0 <= j < old(buf).len() && j < key.len() ==>
            final(buf)@[j] == old(buf)@[j] ^ key@[j],
{
    for (b_i, b_h) in it: buf.iter_mut().zip(key)
        invariant
            forall|j: int| #![auto] 0 <= j < it.index() ==>
                *final(it.seq()[j].0) == (*it.seq()[j].0 ^ *it.seq()[j].1),
    {
        *b_i ^= *b_h;
    }
}
}
