use vstd::prelude::*;
use vstd::bytes::*;
use expansion94::*;

verus! {

pub fn wrong_u16_endian() {
    let decoded = decode_u16([0x34u8, 0x12u8]);
    assert(decoded == 0x3412u16);
}

pub fn wrong_u32_endian() {
    let decoded = decode_u32([0x78u8, 0x56u8, 0x34u8, 0x12u8]);
    assert(decoded == 0x78563412u32);
}

pub fn wrong_encode_order() {
    let encoded = encode_u32(0x12345678u32);
    assert(encoded@ == seq![0x12u8, 0x34u8, 0x56u8, 0x78u8]);
}

pub proof fn missing_width_premise() {
    lemma_u32_from_le_bytes_expansion(seq![0u8, 1u8, 2u8]);
}

}
