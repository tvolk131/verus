use vstd::prelude::*;
verus! { global size_of usize == 4; }
#[verus_infer]
fn cast(x: u64) -> usize { x as usize }
verus! { fn caller() { assert((4294967297u64 as usize) == 1) by(bit_vector); let r = cast(4294967297); assert(r == 1); } }
