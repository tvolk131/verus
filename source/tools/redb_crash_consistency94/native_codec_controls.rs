use vstd::prelude::*;
use codec94::*;

verus! {

pub fn reject_u32_changed_word() {
    let value = u32_roundtrip(17);
    assert(value == 18);
}

pub fn reject_u128_changed_word() {
    let value = u128_roundtrip(u128::MAX);
    assert(value == 0);
}

pub fn reject_changed_u32_byte(bytes: [u8; 4]) {
    let mut result = u32_byte_roundtrip(bytes);
    result[0] ^= 1;
    assert(result == bytes);
}

pub fn reject_changed_u128_byte(bytes: [u8; 16]) {
    let mut result = u128_byte_roundtrip(bytes);
    result[15] ^= 128;
    assert(result == bytes);
}

pub fn reject_short_native_u32_slice() {
    let _result = get_u32(&[0u8, 0u8, 0u8]);
}

pub fn reject_short_native_u64_slice() {
    let _result = get_u64(&[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8]);
}

}
