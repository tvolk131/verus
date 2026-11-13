use vstd::prelude::*;
verus! {
global size_of usize == 4;
pub fn generic_mask(bit: usize) -> (result: u64)
    requires bit == 0,
    ensures result == 0,
{
    assert((1u64 << bit) == 1u64) by(bit_vector) requires bit == 0;
    (1u64 << bit) - 1
}
}
