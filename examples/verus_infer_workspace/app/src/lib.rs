use vstd::prelude::*;

#[verus_infer]
pub fn top(x: u64) -> u64 { infer_middle::increment_twice(x) }

verus! {
    pub fn application_property(x: u64) -> (result: u64)
        requires x <= u64::MAX - 2,
        ensures result as int == x as int + 2,
    { top(x) }
}

#[verus_infer(precision = "sound")]
pub fn sound_top(x: u8) -> u8 { infer_middle::fee_with_one(x) }

#[verus_infer(precision = "sound")]
pub fn signed_sound_top(x: i8) -> i8 { infer_middle::shifted_signed_scale(x) }

verus! {
    pub fn approximate_application_property(x: u8) -> (result: u8)
        ensures 1 <= result && result <= 3,
    { sound_top(x) }

    pub fn signed_approximate_property(x: i8) -> (result: i8)
        ensures -11 <= result && result <= 13,
    { signed_sound_top(x) }
}

#[verus_infer]
pub fn validated_transfer(balance: u64, amount: u64) -> Result<u64, u8> {
    Ok(infer_middle::debit_with_fee(balance, amount)?)
}

#[verus_infer]
pub fn array_total(fees: &[u64; 4], tier: usize, amount: u64) -> u64 {
    infer_middle::array_charge(fees, tier, amount)
}

verus! {
    pub fn array_pricing_property(fees: &[u64; 4], tier: usize, amount: u64) -> (result: u64)
        requires tier < fees.len(), amount as int + fees[tier as int] as int <= u64::MAX,
        ensures result as int == amount as int + fees[tier as int] as int,
    { array_total(fees, tier, amount) }

    pub fn uniform_pricing_property(fee: u64, tier: usize) -> (result: Option<u64>)
        ensures result == if tier < 4 { Some(fee) } else { None },
    { infer_middle::uniform_fee(fee, tier) }
}

verus! {
    pub fn conservation(balance: u64, amount: u64) -> (result: Result<u64, u8>)
        ensures match result {
            Ok(remaining) => remaining as int + amount as int + amount as int / 100 == balance as int,
            Err(_) => true,
        },
    { validated_transfer(balance, amount) }
}

#[verus_infer]
pub fn signed_settlement(balance: i64, delta: i64) -> Option<i64> {
    Some(infer_middle::signed_settlement(balance, delta)?)
}

#[verus_infer]
pub fn slice_total(fees: &[u64], tier: usize, amount: u64) -> Option<u64> {
    infer_middle::slice_charge(fees, tier, amount)
}

verus! {
    pub fn slice_pricing_property(fees: &[u64], tier: usize, amount: u64) -> (result: Option<u64>)
        ensures match result {
            Some(total) => tier < fees@.len()
                && total as int == amount as int + fees@[tier as int] as int,
            None => true,
        },
    { slice_total(fees, tier, amount) }
}

verus! {
    pub fn signed_settlement_property(balance: i64, delta: i64) -> (result: Option<i64>)
        ensures match result {
            Some(value) => value as int == balance as int + delta as int,
            None => true,
        },
    { signed_settlement(balance, delta) }
}
