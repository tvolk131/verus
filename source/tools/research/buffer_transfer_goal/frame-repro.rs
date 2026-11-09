use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
use core::slice::IterMut;
verus! {
// Diagnostic only. Not the frozen writer theorem, no restrictions on its domain.
pub fn unused_iter(ipad: &mut [u8;128])
    ensures final(ipad)@ == old(ipad)@,
{
    let _iter = ipad.iter_mut();
}
// Proposed typed dependency, intentionally NOT assumed. Current model trial.
pub proof fn iter_drop_frame<'a, T>(iter: IterMut<'a,T>)
    requires has_resolved(iter),
    ensures forall|j: int| #![auto] 0 <= j < iter.remaining().len() ==>
        *final(iter.remaining()[j]) == *iter.remaining()[j],
{}
}
fn main() {}
