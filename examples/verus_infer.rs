use vstd::prelude::*;

#[verus_infer]
pub fn compute(x: u64) -> u64 {
    (x + 10) * 2
}

#[verus_infer]
pub fn debit(balance: u64, amount: u64) -> u64 {
    balance - amount
}

#[verus_infer]
pub fn abs_diff(a: u64, b: u64) -> u64 {
    if a >= b {
        a - b
    } else {
        b - a
    }
}

#[verus_infer]
pub fn fee(amount: u64) -> u64 {
    amount / 100
}

// A second inference layer composes verified summaries of both helpers.
#[verus_infer]
pub fn inferred_compute_then_debit(x: u64) -> u64 {
    debit(compute(x), 10)
}

// A third layer demonstrates inference through an already composed summary.
#[verus_infer]
pub fn net_amount(x: u64) -> u64 {
    inferred_compute_then_debit(x)
}

verus! {

// The application property uses the top-level inferred interface. None of
// the three executable layers needs handwritten requires or ensures.
pub fn compute_then_debit(x: u64) -> (result: u64)
    requires
        x <= 9_223_372_036_854_775_797,
    ensures
        result as int == (x as int + 10) * 2 - 10,
{
    net_amount(x)
}

// A conditional guarantee follows from the base contract for fee.
pub fn small_fee(amount: u64) -> (result: u64)
    requires
        amount < 100,
    ensures
        result == 0,
{
    fee(amount)
}

pub fn difference(a: u64, b: u64) -> (result: u64)
    ensures
        result == if a >= b {
            a - b
        } else {
            b - a
        },
{
    abs_diff(a, b)
}

} // verus!
fn main() {
    assert_eq!(compute_then_debit(5), 20);
    assert_eq!(small_fee(99), 0);
    assert_eq!(difference(3, 10), 7);
    println!("Verus Infer examples passed");
}
