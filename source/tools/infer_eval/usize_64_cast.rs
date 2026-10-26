use vstd::prelude::*;
verus! { global size_of usize == 8; }
#[verus_infer]
fn cast(x: u64) -> usize { x as usize }
verus! { fn caller() { assert((4294967297u64 as usize) == 4294967297) by(bit_vector); let r = cast(4294967297); assert(r == 4294967297); } }
