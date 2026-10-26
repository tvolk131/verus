use vstd::prelude::*;

#[verus_infer]
pub fn fee_at(fees: &[u64; 4], tier: usize) -> u64 {
    fees[tier]
}

#[verus_infer]
pub fn charge(fees: &[u64; 4], tier: usize, amount: u64) -> u64 {
    amount + fee_at(fees, tier)
}

#[verus_infer]
pub fn checked_charge(fees: &[u64; 4], tier: usize, amount: u64) -> Option<u64> {
    if tier < fees.len() { amount.checked_add(fee_at(fees, tier)) } else { None }
}

#[verus_infer]
pub fn uniform_quote(fee: u64, tier: usize, amount: u64) -> Option<u64> {
    let fees = [fee; 4];
    checked_charge(&fees, tier, amount)
}

verus! {
    // Prove an application property using the inferred interface.
    pub fn quote(fees: &[u64; 4], tier: usize, amount: u64) -> (result: Option<u64>)
        ensures match result {
            Some(total) => tier < 4
                && total as int == amount as int + fees[tier as int] as int
                && total >= amount,
            None => true,
        },
    { checked_charge(fees, tier, amount) }
}

fn main() {
    let fees = [0, 2, 5, 10];
    assert_eq!(quote(&fees, 2, 100), Some(105));
    assert_eq!(quote(&fees, 0, u64::MAX), Some(u64::MAX));
    assert_eq!(quote(&fees, 3, u64::MAX), None);
    assert_eq!(quote(&fees, 4, 100), None);
    assert_eq!(quote(&fees, usize::MAX, 100), None);
    assert_eq!(uniform_quote(7, 3, 100), Some(107));
    assert_eq!(uniform_quote(7, 4, 100), None);
    println!("Verus Infer array examples passed");
}
