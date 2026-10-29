//! Run: verus verus_infer_binary.rs --no-cheating
use vstd::prelude::*;

#[verus_verify]
#[derive(Copy, Clone)]
pub struct Count(pub usize);

#[verus_infer]
impl core::ops::Mul<usize> for Count {
    type Output = Self;
    fn mul(self, rhs: usize) -> Self { Count(self.0 * rhs) }
}
#[verus_infer]
impl core::ops::Div<usize> for Count {
    type Output = Self;
    fn div(self, rhs: usize) -> Self { Count(self.0 / rhs) }
}
#[verus_infer]
impl core::ops::Add for Count {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Count(self.0 + rhs.0) }
}

#[verus_infer]
pub fn quorum(n: &Count) -> Count { (*n * 2) / 3 + Count(1) }

verus! {
fn coverage(n: &Count) -> (result: Count)
    requires n.0 <= usize::MAX / 2,
    ensures result.0 == (2 * n.0) / 3 + 1,
{ quorum(n) }
}
fn main() {}
