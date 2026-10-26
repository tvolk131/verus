use vstd::prelude::*;

#[verus_infer]
pub fn debit(balance: u64, amount: u64) -> Option<u64> {
    balance.checked_sub(amount)
}

#[verus_infer]
pub fn fee(amount: u64) -> u64 { amount / 100 }

// None and Err are ordinary outcomes; they do not make the input unsafe.
#[verus_infer]
fn total(amount: u64, fee: u64) -> Result<u64, u8> {
    match amount.checked_add(fee) { None => Err(1), Some(total) => Ok(total) }
}

#[verus_infer]
fn required_debit(balance: u64, total: u64) -> Result<u64, u8> {
    match debit(balance, total) { None => Err(2), Some(remaining) => Ok(remaining) }
}

#[verus_infer]
pub fn validated_transfer(balance: u64, amount: u64) -> Result<u64, u8> {
    let total = total(amount, fee(amount))?;
    let remaining = required_debit(balance, total)?;
    Ok(remaining)
}

#[verus_infer]
pub fn checked_transfer(balance: u64, amount: u64) -> Option<u64> {
    let total = amount.checked_add(fee(amount))?;
    Some(debit(balance, total)?)
}

verus! {
    // Application intent remains an explicit property of the composed contract.
    pub fn conservation(balance: u64, amount: u64) -> (result: Result<u64, u8>)
        ensures match result {
            Ok(remaining) => remaining as int + amount as int + amount as int / 100 == balance as int,
            Err(_) => true,
        },
    { validated_transfer(balance, amount) }
}

fn main() {
    assert_eq!(conservation(1000, 100), Ok(899));
    assert_eq!(conservation(0, 100), Err(2));
    assert_eq!(conservation(u64::MAX, u64::MAX), Err(1));
    assert_eq!(conservation(0, 0), Ok(0));
    assert_eq!(checked_transfer(1000, 100), Some(899));
    assert_eq!(checked_transfer(0, 100), None);
    assert_eq!(checked_transfer(u64::MAX, u64::MAX), None);
    println!("Verus Infer enum examples passed");
}
