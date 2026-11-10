use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap)
 requires valid_input(b,h),
 ensures wf(State{heap:Heap{holders:issue(enter(b,h,128)).heap.holders.update(1,Holder::Outstanding),..issue(enter(b,h,128)).heap},..issue(enter(b,h,128))}),
{ entry_sound(b,h,128); issue_sound(enter(b,h,128)); }
}
