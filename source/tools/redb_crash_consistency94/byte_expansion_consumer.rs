use vstd::prelude::*;
use expansion94::*;

verus! {

pub fn literal_decoding() {
    let small = decode_u16([0x34u8, 0x12u8]);
    assert((0x34u16 | (0x12u16 << 8)) == 0x1234u16) by (bit_vector);
    assert(small == 0x1234u16);
    let wide = decode_u32([0x78u8, 0x56u8, 0x34u8, 0x12u8]);
    assert((0x78u32 | (0x56u32 << 8) | (0x34u32 << 16) | (0x12u32 << 24))
        == 0x12345678u32) by (bit_vector);
    assert(wide == 0x12345678u32);
    let zero = decode_u32([0u8, 0u8, 0u8, 0u8]);
    assert((0u32 | (0u32 << 8) | (0u32 << 16) | (0u32 << 24)) == 0u32) by (bit_vector);
    assert(zero == 0u32);
    let maximum = decode_u32([0xffu8, 0xffu8, 0xffu8, 0xffu8]);
    assert((0xffu32 | (0xffu32 << 8) | (0xffu32 << 16) | (0xffu32 << 24))
        == u32::MAX) by (bit_vector);
    assert(maximum == u32::MAX);
}

pub fn literal_encoding() {
    let small = encode_u16(0x1234u16);
    assert((0x1234u16 & 0xff) as u8 == 0x34u8
        && ((0x1234u16 >> 8) & 0xff) as u8 == 0x12u8) by (bit_vector);
    assert(small@ == seq![0x34u8, 0x12u8]);
    let wide = encode_u32(0x12345678u32);
    assert((0x12345678u32 & 0xff) as u8 == 0x78u8
        && ((0x12345678u32 >> 8) & 0xff) as u8 == 0x56u8
        && ((0x12345678u32 >> 16) & 0xff) as u8 == 0x34u8
        && ((0x12345678u32 >> 24) & 0xff) as u8 == 0x12u8) by (bit_vector);
    assert(wide@ == seq![0x78u8, 0x56u8, 0x34u8, 0x12u8]);
    let zero = encode_u32(0u32);
    assert((0u32 & 0xff) as u8 == 0u8 && ((0u32 >> 8) & 0xff) as u8 == 0u8
        && ((0u32 >> 16) & 0xff) as u8 == 0u8
        && ((0u32 >> 24) & 0xff) as u8 == 0u8) by (bit_vector);
    assert(zero@ == seq![0u8, 0u8, 0u8, 0u8]);
    let maximum = encode_u32(u32::MAX);
    assert((u32::MAX & 0xff) as u8 == 0xffu8 && ((u32::MAX >> 8) & 0xff) as u8 == 0xffu8
        && ((u32::MAX >> 16) & 0xff) as u8 == 0xffu8
        && ((u32::MAX >> 24) & 0xff) as u8 == 0xffu8) by (bit_vector);
    assert(maximum@ == seq![0xffu8, 0xffu8, 0xffu8, 0xffu8]);
}

}
