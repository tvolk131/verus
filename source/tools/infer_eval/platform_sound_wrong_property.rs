use vstd::prelude::*;
#[verus_infer(precision="sound")] fn scale(x: isize) -> isize { x / 10 }
verus! { fn caller() { let r=scale(100); assert(r==10); } }
