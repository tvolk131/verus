use vstd::prelude::*;

#[verus_infer]
fn private_increment(x: u64) -> u64 { x + 1 }

#[verus_infer]
pub fn increment(x: u64) -> u64 { private_increment(x) }

// Sound mode retains a useful range even though the input/output relation is lost.
#[verus_infer(precision = "sound")]
pub fn fee(amount: u8) -> u8 { amount / 100 }

#[verus_infer(precision = "sound")]
pub fn signed_scale(amount: i8) -> i8 { amount / 10 }

#[verus_infer]
pub fn debit(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

#[verus_infer]
pub fn required_debit(balance: u64, amount: u64) -> Result<u64, u8> {
    match debit(balance, amount) { None => Err(2), Some(remaining) => Ok(remaining) }
}

#[verus_infer]
pub fn exact_fee(amount: u64) -> u64 { amount / 100 }

#[verus_infer]
pub fn fee_at(fees: &[u64; 4], tier: usize) -> u64 { fees[tier] }

#[verus_infer]
pub fn repeated_fee(fee: u64, tier: usize) -> Option<u64> {
    let fees = [fee; 4];
    if tier < fees.len() { Some(fees[tier]) } else { None }
}

#[verus_infer]
pub fn apply_delta(balance: i64, delta: i64) -> Option<i64> {
    balance.checked_add(delta)
}
