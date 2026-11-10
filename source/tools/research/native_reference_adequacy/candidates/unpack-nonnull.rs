use vstd::prelude::*;
use vstd::mut_ref::*;
verus! {
// Expected unproved: unpack.ptr has no established runtime pointer relation.
pub proof fn unpack_nonnull(r: &mut [u8;128])
    ensures MutRef::unpack(r).ptr@.addr != 0,
{ broadcast use group_mut_ref_axioms; }
}
