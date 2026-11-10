use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap)
 requires valid_input(b,h),
 ensures can_resolve(enter(b,h,0),127),
{ entry_sound(b,h,0); }
}
