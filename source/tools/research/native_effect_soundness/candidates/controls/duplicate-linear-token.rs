use vstd::prelude::*;
use vstd::raw_ptr::{PointsTo,ptr_mut_ref};
#[path="../v8/laws.rs"] mod laws;
use laws::*;
verus! {
pub proof fn bad(tracked loan:Loan) -> (tracked pair:(Loan,Loan))
{ (loan,loan) }
}
