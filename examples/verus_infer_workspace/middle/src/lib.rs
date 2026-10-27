use vstd::prelude::*;

#[verus_infer]
pub fn increment_twice(x: u64) -> u64 {
    infer_base::increment(infer_base::increment(x))
}

#[verus_infer(precision = "sound")]
pub fn fee_with_one(amount: u8) -> u8 { infer_base::fee(amount) + 1 }

#[verus_infer(precision = "sound")]
pub fn shifted_signed_scale(amount: i8) -> i8 { infer_base::signed_scale(amount) + 1 }

// Error 1: total would overflow. Error 2: insufficient funds.
#[verus_infer]
fn total(amount: u64, fee: u64) -> Result<u64, u8> {
    match amount.checked_add(fee) { None => Err(1), Some(total) => Ok(total) }
}

#[verus_infer]
pub fn debit_with_fee(balance: u64, amount: u64) -> Result<u64, u8> {
    let total = total(amount, infer_base::exact_fee(amount))?;
    Ok(infer_base::required_debit(balance, total)?)
}

#[verus_infer]
pub fn array_charge(fees: &[u64; 4], tier: usize, amount: u64) -> u64 {
    amount + infer_base::fee_at(fees, tier)
}

#[verus_infer]
pub fn uniform_fee(fee: u64, tier: usize) -> Option<u64> { infer_base::repeated_fee(fee, tier) }

#[verus_infer]
pub fn signed_settlement(balance: i64, delta: i64) -> Option<i64> {
    Some(infer_base::apply_delta(balance, delta)?)
}

#[verus_infer]
pub fn slice_charge(fees: &[u64], tier: usize, amount: u64) -> Option<u64> {
    Some(amount.checked_add(infer_base::slice_fee(fees, tier)?)?)
}

#[verus_infer]
pub fn counted_twice(x: u64, n: u64) -> u64 {
    infer_base::counted_add(infer_base::counted_add(x, n), n)
}

#[verus_infer]
pub fn counted_usize_twice(x: usize, n: usize) -> usize {
    infer_base::counted_usize(infer_base::counted_usize(x, n), n)
}

#[verus_infer]
pub fn offset(base: isize, delta: isize) -> Option<isize> {
    Some(infer_base::checked_offset(base, delta)?)
}

#[verus_infer(precision = "sound")]
pub fn shifted_size(value: usize) -> usize { infer_base::coarse_size(value) + 1 }

#[verus_infer]
pub fn masked_increment(value: usize) -> usize { infer_base::mask_bits(value + 1) }

#[verus_infer]
pub fn checked_wrapping_offset(value: isize, delta: isize) -> Option<isize> {
    infer_base::wrapping_offset(value, delta).checked_neg()
}

#[verus_infer]
pub fn capped_offset(value: isize, multiplier: isize) -> isize {
    infer_base::capped_offset(value, multiplier)
}

#[verus_infer]
pub fn credited_twice(x: u64, n: u64, step: u64) -> u64 {
    infer_base::counted_credit(infer_base::counted_credit(x, n, step), n, step)
}

#[verus_infer]
pub fn debited(x: u64, n: u64, step: u64) -> u64 {
    infer_base::counted_debit(x, n, step)
}

#[verus_infer]
pub fn payload_pair(input: Option<u8>) -> [u8; 2] { infer_base::payload_pair(input) }

#[verus_infer]
pub fn selected_read(values: &[u8], input: Result<u8, u8>) -> Result<u8, u16> {
    let index = infer_base::widened_index(input)?;
    Ok(infer_base::window_read(values, index as usize, values.len(), 0))
}
