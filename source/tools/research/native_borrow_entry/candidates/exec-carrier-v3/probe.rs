use vstd::prelude::*;
verus! {
// Actual executable reference; this field retains Rust's exclusive borrow.
pub struct ArrayBorrow<'a> { pub array: &'a mut [u8; 128] }
pub fn enter<'a>(buf: &'a mut [u8; 128]) -> (out: ArrayBorrow<'a>)
    ensures *out.array == *old(buf), *final(out.array) == *final(buf),
{ ArrayBorrow { array: buf } }
pub fn release<'a>(entry: ArrayBorrow<'a>) -> (out: &'a mut [u8; 128])
    ensures *out == *old(entry.array), *final(out) == *final(entry.array),
{ entry.array }
pub fn split<'a>(entry: ArrayBorrow<'a>, mid: usize) -> (out: (&'a mut [u8], &'a mut [u8]))
    requires mid <= 128,
    ensures
        out.0@ == old(entry.array)@.subrange(0, mid as int),
        out.1@ == old(entry.array)@.subrange(mid as int, 128),
        final(entry.array)@ == final(out.0)@ + final(out.1)@,
{ entry.array.split_at_mut(mid) }
}
