use vstd::prelude::*;

verus! {

pub fn pair_count_roundtrip(value: u16) -> (result: u16)
    ensures result == value,
{
    proof {
        vstd::bytes::lemma_auto_spec_u16_to_from_le_bytes();
        assert(vstd::bytes::spec_u16_to_le_bytes(value).len() == 2);
        assert(vstd::bytes::spec_u16_from_le_bytes(vstd::bytes::spec_u16_to_le_bytes(value)) == value);
    }
    u16::from_le_bytes(value.to_le_bytes())
}

pub fn pair_count_byte_roundtrip(bytes: [u8; 2]) -> (result: [u8; 2])
    ensures result == bytes,
{
    proof { vstd::bytes::lemma_auto_spec_u16_to_from_le_bytes(); }
    u16::from_le_bytes(bytes).to_le_bytes()
}

}
