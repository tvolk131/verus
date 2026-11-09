use vstd::prelude::*;
verus! {
#[verifier::infer]
fn select_mask(bit: usize) -> u64 {
    1u64 << (bit as u64)
}
fn check(bit: usize) requires bit < 64, {
    let r = select_mask(bit);
    assert(r == 1u64 << (bit as u64));
}
}
fn main() {}
