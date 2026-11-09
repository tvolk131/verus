use vstd::prelude::*;
verus! {
pub fn write_return(buf: &mut [u16], count: usize, value: u16) -> (result: &[u16])
    requires 1 <= count <= old(buf).len(),
    ensures
        result.len() == count,
        forall|j: int| 0 <= j < result.len() ==> result@[j] == value,
{
    let mut i: usize = 0;
    for b in it: buf.iter_mut()
        invariant_except_break i == it.index(),
        invariant
            i < count,
            count <= it.seq().len(),
            forall|j: int| 0 <= j < it.index() ==> *final(it.seq()[j]) == value,
        ensures
            i + 1 == count,
            forall|j: int| 0 <= j < count ==> *final(it.seq()[j]) == value,
    {
        *b = value;
        if i + 1 == count { break; }
        i += 1;
    }
    &buf[0..=i]
}
}
