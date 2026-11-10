use vstd::prelude::*;
use vstd::mut_ref::*;
verus! {
// A spec representation can carry null. This is never an executable loan.
pub proof fn pack_null_observation(r: &mut [u8;128])
    ensures MutRef::unpack(MutRef {
        ptr: vstd::raw_ptr::ptr_null_mut::<[u8;128]>(),
        ..MutRef::unpack(r)
    }.pack()).ptr == vstd::raw_ptr::ptr_null_mut::<[u8;128]>(),
{ broadcast use group_mut_ref_axioms; }
}
