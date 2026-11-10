use vstd::prelude::*;
use vstd::mut_ref::*;
verus! {
// Real executable array input, no extra premise. The ghost unpack pointer
// has no established relation to this executable reference's runtime address.
pub fn exec_pointer_origin(buf: &mut [u8;128])
    ensures MutRef::unpack(old(buf)).ptr@.addr != 0,
{ proof { broadcast use group_mut_ref_axioms; } }
}
