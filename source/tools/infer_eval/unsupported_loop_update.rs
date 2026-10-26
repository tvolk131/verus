use vstd::prelude::*;
#[verus_infer]
pub fn add(x: u8, n: u8) -> u8 {
    let mut r = x;
    let mut i = 0u8;
    while i < n { r += 2; i += 2; }
    r
}
