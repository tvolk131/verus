use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap,value:u8)
 requires valid_input(b,h),
 ensures can_resolve(resolve(issue(enter(b,h,128)),0,value),0),
{ entry_sound(b,h,128); issue_sound(enter(b,h,128)); resolved_loan_cannot_write(issue(enter(b,h,128)),0,value); }
}
