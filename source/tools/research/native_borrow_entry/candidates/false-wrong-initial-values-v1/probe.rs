use vstd::prelude::*;
#[path="../heap-foundation-v5/probe.rs"] mod foundation;
use foundation::*;
verus! {
pub proof fn false_claim(b: Borrow,h: Heap,value:u8)
 requires valid_input(b,h), value != b.initial[0],
 ensures valid_input(b,Heap{bytes:h.bytes.update(0,value),..h}),
{ entry_sound(b,h,0); }
}
