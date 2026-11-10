use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked mut c:Carrier) -> (tracked loan:Loan)
 requires c.wf(), c.physical.index == 0, 0 < c.physical.length,
 ensures loan.token.ptr() == c.entry.reads[0],
{ issue(&mut c) }
}
