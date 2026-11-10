use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked mut c:Carrier) -> (tracked out:Carrier)
 requires c.wf(), c.physical.index < 128,
 ensures out.wf(),
{ let ghost i=c.physical.index; let tracked lost=c.retained.tracked_remove(i); c }
}
