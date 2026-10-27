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

#[verus_infer]
pub fn slice_fee(fees: &[u64], tier: usize) -> Option<u64> {
    if tier < fees.len() { Some(fees[tier]) } else { None }
}

#[verus_infer]
pub fn counted_add(x: u64, n: u64) -> u64 {
    let mut result = x;
    let mut i = 0u64;
    while i < n { result += 1; i += 1; }
    result
}

#[verus_infer]
pub fn counted_usize(x: usize, n: usize) -> usize {
    let mut result = x;
    let mut i = 0usize;
    while i < n { result += 1; i += 1; }
    result
}

#[verus_infer]
pub fn checked_offset(base: isize, delta: isize) -> Option<isize> {
    base.checked_add(delta)
}

#[verus_infer(precision = "sound")]
pub fn coarse_size(value: usize) -> usize { value / 10 }

#[verus_infer]
pub fn mask_bits(value: usize) -> usize { value & 15 }

#[verus_infer]
pub fn wrapping_offset(value: isize, delta: isize) -> isize { value.wrapping_add(delta) }

#[verus_infer]
pub fn capped_offset(value: isize, multiplier: isize) -> isize {
    value.saturating_mul(multiplier)
}

#[verus_infer]
pub fn counted_credit(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut i = 0u64;
    while i < n { result += step; i += 1; }
    result
}

#[verus_infer]
pub fn counted_debit(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut remaining = n;
    while remaining > 0 { result -= step; remaining -= 1; }
    result
}

#[verus_infer]
pub fn payload_pair(input: Option<u8>) -> [u8; 2] {
    let Some(value) = input else { return [0, 0]; };
    [value, value + 1]
}

#[verus_infer]
pub fn widened_index(input: Result<u8, u8>) -> Result<u8, u16> { Ok(input?) }

#[verus_infer]
pub fn window_read(values: &[u8], start: usize, end: usize, index: usize) -> u8 {
    let window = &values[start..end];
    window[index]
}
