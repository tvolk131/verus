use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked c:Carrier)
 requires c.wf(), !c.pending.is_empty(),
{ let tracked all=reconstruct(c); }
}
