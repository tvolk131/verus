use vstd::prelude::*;
verus! {
pub fn opaque_resolved(buf: &mut [u8;128]) {
    let it = buf.iter_mut();
    proof { assert(has_resolved(it)); }
}
}
