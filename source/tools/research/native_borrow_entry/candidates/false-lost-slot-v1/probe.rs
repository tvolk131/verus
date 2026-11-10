use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap)
 requires valid_input(b,h),
 ensures wf(State{heap:Heap{holders:enter(b,h,0).heap.holders.subrange(0,127),..enter(b,h,0).heap},..enter(b,h,0)}),
{ entry_sound(b,h,0); }
}
