use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(e:Entry,tracked pad:Map<int,PointsTo<u8>>) -> (tracked c:Carrier)
 requires e.valid(), full_owner(pad,e),
 ensures c.physical.pad_end == e.pad_base@.addr as int+127,
{ accept_owner(e,pad) }
}
