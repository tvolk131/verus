use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked mut c:Carrier,tracked loan:Loan)
 requires c.wf(), loan.wf(), loan.entry != c.entry, c.pending.contains(loan.index),
{ resolve(&mut c,loan); }
}
