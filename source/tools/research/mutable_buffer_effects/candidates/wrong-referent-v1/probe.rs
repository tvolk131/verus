use vstd::prelude::*;
verus! {
pub fn write_prefix(buf: &mut [u8], count: usize, value: u8)
    requires 1 <= count <= old(buf).len(),
    ensures
        final(buf).len() == old(buf).len(),
        forall|j: int| 0 <= j < count ==> final(buf)@[j] == value,
{
    let mut i: usize = 0;
    for b in it: buf.iter_mut()
        invariant_except_break
            i == it.index(),
        invariant
            i < count,
            count <= it.seq().len(),
            forall|j: int| 0 <= j < it.index() ==> *final(it.seq()[j]) == value,
        ensures
            forall|j: int| 0 <= j < count ==> *final(it.seq()[j]) == value,
    {
        *b = value;
        if i + 1 == count { break; }
        i += 1;
    }
}
}

verus! {
pub fn wrong_referent(left: &mut [u8], right: &[u8], count: usize, value: u8)
    requires 1 <= count <= old(left).len(), right.len() > 0,
{
    write_prefix(left, count, value);
    assert(right[0] == value);
}
}
