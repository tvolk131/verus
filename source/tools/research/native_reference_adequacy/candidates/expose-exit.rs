use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
// Diagnostic only. Ghost items are observations, never authority.
// No additional input premise and no second Rust mutable borrow.
pub fn expose_exit<'a>(buf: &'a mut [u8;128])
    -> (out: Ghost<Seq<&'a mut u8>>)
    ensures
        out@.len() == 128,
        forall|i:int| 0 <= i < 128 ==> *(#[trigger] out@[i]) == old(buf)@[i],
        forall|i:int| 0 <= i < 128 ==> *final(#[trigger] out@[i]) == final(buf)@[i],
        (forall|i:int| 0 <= i < 128 ==> *final(#[trigger] out@[i]) == *out@[i])
            ==> final(buf)@ == old(buf)@,
{
    let it = buf.iter_mut();
    let ghost items = it.remaining();
    Ghost(items)
}
}
