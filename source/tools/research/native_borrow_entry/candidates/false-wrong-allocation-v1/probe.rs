use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap)
 requires valid_input(b,h),
 ensures wf(State{heap:Heap{origin:Origin{allocation:b.origin.allocation+1,..b.origin},..enter(b,h,0).heap},..enter(b,h,0)}),
{ entry_sound(b,h,0); }
}
