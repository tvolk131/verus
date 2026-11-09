use vstd::prelude::*;
verus! {
#[verifier::infer]
fn data_index_of(bit: u32) -> (usize, usize) {
    ((bit as usize) / 64, (bit as usize) % 64)
}
fn check(bit: u32) {
    let r = data_index_of(bit);
    assert(r.0 == (bit as usize) / 64 && r.1 == (bit as usize) % 64);
}
}
fn main() {}
