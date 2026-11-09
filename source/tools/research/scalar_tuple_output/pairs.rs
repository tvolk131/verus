use vstd::prelude::*;
verus! {
#[verifier::infer]
pub fn data_index_of(bit: u32) -> (usize, usize) {
    ((bit as usize) / 64, (bit as usize) % 64)
}
#[verifier::infer]
fn signed(x: i8, y: i8) -> (i8, i8) { (x / y, x % y) }
#[verifier::infer]
fn slice_pair(xs: &[u8]) -> (u8, u8) { (xs[0], xs[1]) }
// Unrelated case: both arithmetic components have their own eager safety domain.
#[verifier::infer]
fn arithmetic(x: u8, y: u8) -> (u8, u8) { (x + 1, x / y) }
#[verifier::infer]
fn composed(x: u8, y: u8, flag: bool) -> (u8, u8) {
    let pair = arithmetic(x, y);
    if flag { (pair.1, pair.0) } else { pair }
}
#[verifier::infer]
fn branch(x: u8, flag: bool) -> (u8, bool) {
    if flag { (x + 1, true) } else { (x - 1, false) }
}
#[verifier::infer]
fn ignored_left(x: u8, y: u8) -> u8 { let pair = (x / y, x + 1); pair.1 }
#[verifier::infer]
fn ignored_right(x: u8, y: u8) -> u8 { let pair = (x + 1, x / y); pair.0 }
fn check(bit: u32) {
    let pair = data_index_of(bit);
    assert(pair.0 == (bit as usize) / 64 && pair.1 == (bit as usize) % 64);
    let zero = data_index_of(0); assert(zero.0 == 0 && zero.1 == 0);
    let max = data_index_of(u32::MAX); assert(max.0 == 67108863 && max.1 == 63);
    let a = signed(i8::MIN, 1); assert(a.0 == i8::MIN && a.1 == 0);
    let a = slice_pair(&[7u8, 9]); assert(a.0 == 7 && a.1 == 9);
    let a = arithmetic(254, 2); assert(a.0 == 255 && a.1 == 127);
    let a = composed(254, 2, true); assert(a.0 == 127 && a.1 == 255);
    let a = branch(254, true); assert(a.0 == 255 && a.1);
    let a = branch(1, false); assert(a.0 == 0 && !a.1);
}
}
fn main() {}
