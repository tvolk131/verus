use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap,value:u8)
 requires valid_input(b,h), value != b.initial[127],
 ensures release(enter(b,h,0)).bytes == b.initial.update(127,value),
{ entry_sound(b,h,0); empty_key_frame(b,h); }
}
