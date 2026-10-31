// rust_verify/tests/example.rs expect-warnings
use vstd::prelude::*;

#[verus_infer]
pub fn pack(value: u32, mask: u32, shift: u32) -> u32 { (value & mask) << shift }

#[verus_infer]
pub fn invert(value: isize) -> isize { !value }

#[verus_infer]
pub fn wrapping_total(value: i64, delta: i64) -> i64 { value.wrapping_add(delta) }

#[verus_infer]
pub fn capped_product(value: i128, multiplier: i128) -> i128 {
    value.saturating_mul(multiplier)
}

#[verus_infer]
pub fn checked_adjust(value: u64, delta: i64) -> Option<u64> {
    value.checked_sub_signed(delta)
}

#[verus_infer]
pub fn checked_negate(value: isize) -> Option<isize> { value.checked_neg() }

#[verus_infer(precision = "sound")]
pub fn coarse_mask(value: u32) -> u32 { pack(value, 255, 2) }

verus! {
    pub fn application_properties() {
        assert(((10u32 & 15u32) << 2) == 40) by(bit_vector);
        let r = pack(10, 15, 2); assert(r == 40);
        let r = wrapping_total(i64::MAX, 1); assert(r == i64::MIN);
        let r = capped_product(i128::MIN, -1); assert(r == i128::MAX);
        let r = checked_adjust(0, -1); assert(r == Some(1u64));
        let r = checked_adjust(u64::MAX, -1); assert(r == None);
        let r = checked_negate(isize::MIN); assert(r == None);
    }
}

fn main() {
    application_properties();
    assert_eq!(pack(10, 15, 2), 40);
    assert_eq!(pack(u32::MAX, u32::MAX, 1), u32::MAX - 1);
    assert_eq!(invert(0), -1);
    assert_eq!(wrapping_total(i64::MAX, 1), i64::MIN);
    assert_eq!(capped_product(i128::MIN, -1), i128::MAX);
    assert_eq!(checked_adjust(0, -1), Some(1));
    assert_eq!(checked_negate(isize::MIN), None);
    assert_eq!(coarse_mask(10), 40);
    println!("Verus Infer integer-operation examples passed");
}
