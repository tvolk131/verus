use vstd::prelude::*;
use vstd::bytes::*;

verus! {

pub fn decode_u16(bytes: [u8; 2]) -> (result: u16)
    ensures result == (bytes@[0] as u16) | ((bytes@[1] as u16) << 8),
{
    proof { lemma_u16_from_le_bytes_expansion(bytes@); }
    u16::from_le_bytes(bytes)
}

pub fn decode_u32(bytes: [u8; 4]) -> (result: u32)
    ensures result == (bytes@[0] as u32) | ((bytes@[1] as u32) << 8)
        | ((bytes@[2] as u32) << 16) | ((bytes@[3] as u32) << 24),
{
    proof { lemma_u32_from_le_bytes_expansion(bytes@); }
    u32::from_le_bytes(bytes)
}

pub fn encode_u16(value: u16) -> (result: [u8; 2])
    ensures result@ == seq![(value & 0xff) as u8, ((value >> 8) & 0xff) as u8],
{
    proof { lemma_u16_to_le_bytes_expansion(value); }
    value.to_le_bytes()
}

pub fn encode_u32(value: u32) -> (result: [u8; 4])
    ensures result@ == seq![(value & 0xff) as u8, ((value >> 8) & 0xff) as u8,
        ((value >> 16) & 0xff) as u8, ((value >> 24) & 0xff) as u8],
{
    proof { lemma_u32_to_le_bytes_expansion(value); }
    value.to_le_bytes()
}

}
