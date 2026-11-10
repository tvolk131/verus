use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;

verus! {
fn make<'a>(other: &'a mut [u8; 10]) -> (r: core::slice::IterMut<'a, u8>)
    ensures IteratorSpec::decrease(&r) is Some,
{
    other.iter_mut()
}

fn client(buf: &mut [u8; 10], other: &mut [u8; 10], key:&[u8]) {
    let _unused = buf.iter_mut().zip(key);
    for b in make(other) { *b = 1; }
}
}
