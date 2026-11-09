use vstd::prelude::*;
#[verus_infer]
pub fn mask(x: u64) -> u8 { (x & 15) as u8 }
verus! {
pub fn call(x: u64) {
    let r = mask(x);
    assert(r < 16);
    assert(r as u64 == x % 16);
}
}
