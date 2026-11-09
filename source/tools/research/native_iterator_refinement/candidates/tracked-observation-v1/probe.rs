use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
pub fn witness_from_observation(buf: &mut [u8; 128]) {
    let it = buf.iter_mut();
    // A spec sequence is a duplicable observation; it is not linear ownership.
    let tracked refs = it.remaining();
}
}
