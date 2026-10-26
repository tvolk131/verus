use vstd::prelude::*;

#[verus_infer]
pub fn adjust(balance: i64, delta: i64) -> i64 { balance + delta }

#[verus_infer]
pub fn reverse(delta: i64) -> i64 { -delta }

#[verus_infer]
pub fn share(total: i64, count: i64) -> i64 { total / count }

#[verus_infer]
pub fn remainder(total: i64, count: i64) -> i64 { total % count }

#[verus_infer]
pub fn checked_share(total: i64, count: i64) -> Option<i64> {
    Some(total.checked_div(count)?)
}

#[verus_infer]
pub fn apply_adjustments(balance: i64, changes: &[i64; 2]) -> Option<i64> {
    let balance = balance.checked_add(changes[0])?;
    Some(balance.checked_add(changes[1])?)
}

#[verus_infer]
pub fn low_byte(value: i16) -> i8 { value as i8 }

verus! {
    pub fn adjustment_property(balance: i64, delta: i64) -> (result: i64)
        requires i64::MIN <= balance as int + delta as int <= i64::MAX,
        ensures result as int == balance as int + delta as int,
    { adjust(balance, delta) }

    pub fn reversal_property(delta: i64) -> (result: i64)
        requires delta != i64::MIN,
        ensures result as int == -(delta as int),
    { reverse(delta) }

    pub fn conservation(balance: i64, changes: &[i64; 2]) -> (result: Option<i64>)
        ensures match result {
            Some(value) => value as int == balance as int + changes[0] as int + changes[1] as int,
            None => true,
        },
    { apply_adjustments(balance, changes) }
}

fn main() {
    assert_eq!(adjustment_property(i64::MIN, 1), i64::MIN + 1);
    assert_eq!(reversal_property(i64::MAX), i64::MIN + 1);
    assert_eq!(share(-7, 3), -2);
    assert_eq!(remainder(-7, 3), -1);
    assert_eq!(checked_share(i64::MIN, -1), None);
    assert_eq!(checked_share(1, 0), None);
    assert_eq!(checked_share(i64::MIN, 1), Some(i64::MIN));
    assert_eq!(conservation(100, &[-20, 5]), Some(85));
    assert_eq!(conservation(i64::MAX, &[1, -1]), None);
    assert_eq!(conservation(i64::MIN, &[-1, 1]), None);
    assert_eq!(low_byte(-129), 127);
    assert_eq!(low_byte(128), -128);
    println!("Verus Infer signed integer examples passed");
}
