use vstd::prelude::*;
use vstd::mut_ref::*;
verus! {
// This spec-only pack is NOT an executable reference or a physical permission.
pub open spec fn repack_null<'a>(r: &'a mut [u8;128]) -> &'a mut [u8;128] {
    MutRef { ptr: vstd::raw_ptr::ptr_null_mut::<[u8;128]>(), ..MutRef::unpack(r) }.pack()
}
pub proof fn pack_null_views(r: &mut [u8;128])
    ensures
        MutRef::unpack(repack_null(r)).ptr@.addr == 0,
        *repack_null(r) == *r,
        *final(repack_null(r)) == *final(r),
{ broadcast use group_mut_ref_axioms; }
}
