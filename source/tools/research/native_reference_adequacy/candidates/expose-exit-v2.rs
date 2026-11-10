use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
// No input premise; these are observations of the actual constructor operand.
// The final implication deliberately leaves the native exit premise unproved.
pub fn expose_exit(buf: &mut [u8;128]) {
    let ghost before = buf@;
    let ghost future = final(buf)@;
    let it = buf.iter_mut();
    let ghost items = it.remaining();
    proof {
        assert(items.len() == 128);
        assert(forall|i:int| 0 <= i < 128 ==> *(#[trigger] items[i]) == before[i]);
        assert(forall|i:int| 0 <= i < 128 ==> *final(#[trigger] items[i]) == future[i]);
        assert((forall|i:int| 0 <= i < 128 ==> *final(#[trigger] items[i]) == *items[i])
            ==> future == before);
    }
}
}
