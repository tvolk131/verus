use vstd::prelude::*;
verus! {
// Actual executable reference; this field retains Rust's exclusive borrow.
pub struct ArrayBorrow<'a> { pub array: &'a mut [u8; 128] }
pub fn enter<'a>(buf: &'a mut [u8; 128]) -> (out: ArrayBorrow<'a>)
    ensures *out.array == *old(buf), *final(out.array) == *final(buf),
{ ArrayBorrow { array: buf } }
pub fn release<'a>(entry: ArrayBorrow<'a>) -> (out: &'a mut [u8; 128])
    ensures *out == *entry.array, *final(out) == *final(entry.array),
{ entry.array }
}
