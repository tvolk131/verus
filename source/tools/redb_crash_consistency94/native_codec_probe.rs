use vstd::prelude::*;
use std::mem::size_of;

verus! {

pub fn get_u32(data: &[u8]) -> (result: u32)
    requires data.len() >= 4,
    ensures result == vstd::bytes::spec_u32_from_le_bytes(data@.subrange(0, 4)),
{
    u32::from_le_bytes(data[..size_of::<u32>()].try_into().unwrap())
}

pub fn get_u64(data: &[u8]) -> (result: u64)
    requires data.len() >= 8,
    ensures result == vstd::bytes::spec_u64_from_le_bytes(data@.subrange(0, 8)),
{
    u64::from_le_bytes(data[..size_of::<u64>()].try_into().unwrap())
}

pub fn u32_roundtrip(value: u32) -> (result: u32)
    ensures result == value,
{
    proof {
        vstd::bytes::lemma_auto_spec_u32_to_from_le_bytes();
        assert(vstd::bytes::spec_u32_to_le_bytes(value).len() == 4);
        assert(vstd::bytes::spec_u32_from_le_bytes(vstd::bytes::spec_u32_to_le_bytes(value)) == value);
    }
    u32::from_le_bytes(value.to_le_bytes())
}

pub fn u128_roundtrip(value: u128) -> (result: u128)
    ensures result == value,
{
    proof {
        vstd::bytes::lemma_auto_spec_u128_to_from_le_bytes();
        assert(vstd::bytes::spec_u128_to_le_bytes(value).len() == 16);
        assert(vstd::bytes::spec_u128_from_le_bytes(vstd::bytes::spec_u128_to_le_bytes(value)) == value);
    }
    u128::from_le_bytes(value.to_le_bytes())
}

pub fn u128_byte_roundtrip(bytes: [u8; 16]) -> (result: [u8; 16])
    ensures result == bytes,
{
    proof { vstd::bytes::lemma_auto_spec_u128_to_from_le_bytes(); }
    u128::from_le_bytes(bytes).to_le_bytes()
}

pub fn u32_byte_roundtrip(bytes: [u8; 4]) -> (result: [u8; 4])
    ensures result == bytes,
{
    proof { vstd::bytes::lemma_auto_spec_u32_to_from_le_bytes(); }
    u32::from_le_bytes(bytes).to_le_bytes()
}

}
