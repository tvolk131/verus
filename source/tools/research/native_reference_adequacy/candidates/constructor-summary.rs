use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
// Total abstract-summary link for the SAME native operand. This uses the
// inherited iter_mut contract; it does NOT verify that native contract's body.
pub fn constructor_summary<'a>(buf: &'a mut [u8;128])
    -> (it: core::slice::IterMut<'a,u8>)
    ensures
        it.remaining().len() == 128,
        forall|i:int| 0 <= i < 128 ==> *(#[trigger] it.remaining()[i]) == old(buf)@[i],
        forall|i:int| 0 <= i < 128 ==> *final(#[trigger] it.remaining()[i]) == final(buf)@[i],
{ buf.iter_mut() }
}
