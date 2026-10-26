use vstd::prelude::*;
#[verus_infer]
pub fn identity(x: u8) -> u8 { x }
verus! { fn caller() { let result = identity(1); assert(result == 2); } }
