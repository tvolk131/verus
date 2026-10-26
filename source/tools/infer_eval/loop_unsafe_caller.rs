use vstd::prelude::*;
#[verus_infer]
pub fn add(x: u8, n: u8) -> u8 {
    let mut r = x;
    let mut i = 0u8;
    while i < n { r += 1; i += 1; }
    r
}
verus! { fn caller() { let _ = add(255, 1); } }
