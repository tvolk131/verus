use vstd::prelude::*;

// Dynamic lengths stay symbolic; this function requires tier < fees.len().
#[verus_infer]
pub fn fee_at(fees: &[u64], tier: usize) -> u64 { fees[tier] }

// Guarded reads accept every index, including on an empty slice.
#[verus_infer]
pub fn get_fee(fees: &[u64], tier: usize) -> Option<u64> {
    if tier < fees.len() { Some(fee_at(fees, tier)) } else { None }
}

#[verus_infer]
pub fn checked_charge(fees: &[u64], tier: usize, amount: u64) -> Option<u64> {
    Some(amount.checked_add(get_fee(fees, tier)?)?)
}

#[verus_infer]
pub fn signed_delta(values: &[i64], index: usize, delta: i64) -> Option<i64> {
    if index >= values.len() { None } else { values[index].checked_add(delta) }
}

verus! {
    pub fn lookup_property(fees: &[u64], tier: usize) -> (result: Option<u64>)
        ensures result == if tier < fees@.len() { Some(fees@[tier as int]) } else { None },
    { get_fee(fees, tier) }

    pub fn charge_property(fees: &[u64], tier: usize, amount: u64) -> (result: Option<u64>)
        ensures match result {
            Some(total) => tier < fees@.len()
                && total as int == amount as int + fees@[tier as int] as int,
            None => true,
        },
    { checked_charge(fees, tier, amount) }
}

fn main() {
    let fees = [2u64, 5, 10];
    assert_eq!(lookup_property(&fees, 1), Some(5));
    assert_eq!(lookup_property(&[], 0), None);
    assert_eq!(lookup_property(&fees, usize::MAX), None);
    assert_eq!(charge_property(&fees, 2, 100), Some(110));
    assert_eq!(charge_property(&[u64::MAX], 0, 1), None);
    assert_eq!(signed_delta(&[i64::MIN], 0, -1), None);
    assert_eq!(signed_delta(&[i64::MIN], 0, 1), Some(i64::MIN + 1));
    assert_eq!(lookup_property(&[7u64; 300], 299), Some(7));
    println!("Verus Infer slice examples passed");
}
