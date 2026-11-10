use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo, ptr_ref};
verus! {
// Resource negative: a supplied token alone does not imply pointer agreement
// or initialization. This supplies no entry conversion from an executable ref.
pub fn missing_resource(p: *const u8, Tracked(perm): Tracked<PointsTo<u8>>) -> (b:u8) {
    let r = ptr_ref(p, Tracked(&perm));
    *r
}
}
