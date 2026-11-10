use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(e:Entry,tracked pad:Map<int,PointsTo<u8>>)
 requires e.valid(), owner_shape(pad,e), !full_owner(pad,e),
{ let tracked c=accept_owner(e,pad); }
}
