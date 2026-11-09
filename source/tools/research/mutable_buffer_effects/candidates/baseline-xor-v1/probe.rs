use vstd::prelude::*;
verus! {
pub fn xor_prefix(buf: &mut [u8], count: usize, mask: u8)
    requires 1 <= count <= old(buf).len(),
    ensures
        final(buf).len() == old(buf).len(),
        forall|j: int| 0 <= j < count ==> final(buf)@[j] == old(buf)@[j] ^ mask,
{
    let mut i: usize = 0;
    for b in it: buf.iter_mut()
        invariant_except_break i == it.index(),
        invariant
            i < count,
            count <= it.seq().len(),
            forall|j: int| 0 <= j < it.index() ==> *final(it.seq()[j]) == *it.seq()[j] ^ mask,
        ensures
            forall|j: int| 0 <= j < count ==> *final(it.seq()[j]) == *it.seq()[j] ^ mask,
    {
        *b ^= mask;
        if i + 1 == count { break; }
        i += 1;
    }
}
pub fn xor_all(buf: &mut [u8], mask: u8)
    ensures
        final(buf).len() == old(buf).len(),
        forall|j: int| 0 <= j < old(buf).len() ==> final(buf)@[j] == old(buf)@[j] ^ mask,
{
    for b in it: buf.iter_mut()
        invariant
            forall|j: int| 0 <= j < it.index() ==> *final(it.seq()[j]) == *it.seq()[j] ^ mask,
    {
        *b ^= mask;
    }
}
}
