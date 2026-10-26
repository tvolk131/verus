use vstd::prelude::*;

// This deliberately publishes a range, not an exact input/output relationship.
#[verus_infer(precision = "sound")]
pub fn fee(amount: u8) -> u8 {
    amount / 100
}

#[verus_infer(precision = "sound")]
pub fn signed_scale(amount: i8) -> i8 { amount / 10 }

#[verus_infer(precision = "sound")]
pub fn signed_remainder(amount: i8) -> i8 { amount % 10 }

// The callee's range makes addition safe for every i8 input.
#[verus_infer(precision = "sound")]
pub fn shifted_scale(amount: i8) -> i8 { signed_scale(amount) + 1 }

// The optional precision queries have a tiny budget; soundness keeps its normal budget.
#[verus_infer(precision = "sound", precision_rlimit = 0.000001)]
pub fn selected_value(b: bool) -> u8 {
    if b { 1 } else { 2 }
}

verus! {
    pub fn range_property(amount: u8) -> (result: u8)
        ensures result <= 2,
    { fee(amount) }

    pub fn selected_range(b: bool) -> (result: u8)
        ensures 1 <= result && result <= 2,
    { selected_value(b) }

    pub fn signed_range(amount: i8) -> (result: i8)
        ensures -12 <= result && result <= 12,
    { signed_scale(amount) }

    pub fn signed_remainder_range(amount: i8) -> (result: i8)
        ensures -9 <= result && result <= 9,
    { signed_remainder(amount) }

    pub fn shifted_range(amount: i8) -> (result: i8)
        ensures -11 <= result && result <= 13,
    { shifted_scale(amount) }
}

fn main() {
    assert_eq!(range_property(255), 2);
    assert_eq!(selected_range(true), 1);
    assert_eq!(signed_range(-128), -12);
    assert_eq!(signed_range(127), 12);
    assert_eq!(signed_remainder_range(-127), -7);
    assert_eq!(shifted_range(-1), 1);
    assert_eq!(shifted_range(i8::MIN), -11);
    assert_eq!(shifted_range(i8::MAX), 13);
    println!("Verus Infer partial inference examples passed");
}
