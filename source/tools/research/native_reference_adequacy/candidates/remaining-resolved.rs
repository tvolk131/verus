use vstd::prelude::*;
use vstd::std_specs::iter::IteratorSpec;
verus! {
pub fn remaining_resolved(buf: &mut [u8;128]) {
    let it = buf.iter_mut();
    proof {
        assert(has_resolved(it));
        assert(has_resolved(it.remaining()[0]));
    }
}
}
