use vstd::prelude::*;
use codec94::*;

verus! {

pub fn imported_u32_codec(value: u32) -> (result: u32)
    ensures result == value,
{
    u32_roundtrip(value)
}

pub fn imported_u128_codec(value: u128) -> (result: u128)
    ensures result == value,
{
    u128_roundtrip(value)
}

pub fn imported_native_u32_read(data: &[u8]) -> (result: u32)
    requires data.len() >= 4,
    ensures result == vstd::bytes::spec_u32_from_le_bytes(data@.subrange(0, 4)),
{
    get_u32(data)
}

}
