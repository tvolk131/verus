use vstd::prelude::*;
use core::mem::size_of;
use pagecodec94::*;
use vstd::std_specs::clone::CloneIdentity;

verus! {

pub type Checksum = u128;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BtreeHeader {
    pub root: PageNumber,
    pub checksum: Checksum,
    pub length: u64,
}

impl CloneIdentity for BtreeHeader {
    proof fn lemma_clone_identity(original: Self, cloned_header: Self)
        ensures original == cloned_header,
    {
    }
}

impl BtreeHeader {
    pub const fn serialized_size() -> (result: usize)
        ensures result == 32,
    {
        PageNumber::serialized_size() + size_of::<Checksum>() + size_of::<u64>()
    }

    pub fn from_le_bytes(bytes: [u8; Self::serialized_size()]) -> (result: Self)
        ensures canonical_page(result.root),
            result.root.page_order == (vstd::bytes::spec_u64_from_le_bytes(bytes@.subrange(0, 8)) >> 59) as u8,
            result.root.page_index as u64 == (vstd::bytes::spec_u64_from_le_bytes(bytes@.subrange(0, 8))
                & (0x000F_FFFFu64 >> result.root.page_order)),
            result.root.region == ((vstd::bytes::spec_u64_from_le_bytes(bytes@.subrange(0, 8)) >> 20)
                & 0x000F_FFFFu64) as u32,
            result.checksum == vstd::bytes::spec_u128_from_le_bytes(bytes@.subrange(8, 24)),
            result.length == vstd::bytes::spec_u64_from_le_bytes(bytes@.subrange(24, 32)),
    {
        let root =
            PageNumber::from_le_bytes(bytes[..PageNumber::serialized_size()].try_into().unwrap());
        let mut offset = PageNumber::serialized_size();
        let checksum = Checksum::from_le_bytes(
            bytes[offset..(offset + size_of::<Checksum>())]
                .try_into()
                .unwrap(),
        );
        offset += size_of::<Checksum>();
        let length = u64::from_le_bytes(
            bytes[offset..(offset + size_of::<u64>())]
                .try_into()
                .unwrap(),
        );

        Self {
            root,
            checksum,
            length,
        }
    }

    pub fn to_le_bytes(self) -> (result: [u8; Self::serialized_size()])
        ensures result@.subrange(0, 8) == vstd::bytes::spec_u64_to_le_bytes(encoded_page_word(self.root)),
            result@.subrange(8, 24) == vstd::bytes::spec_u128_to_le_bytes(self.checksum),
            result@.subrange(24, 32) == vstd::bytes::spec_u64_to_le_bytes(self.length),
    {
        let mut result = [0; Self::serialized_size()];
        result[..PageNumber::serialized_size()].copy_from_slice(&self.root.to_le_bytes());
        result[PageNumber::serialized_size()
            ..(PageNumber::serialized_size() + size_of::<Checksum>())]
            .copy_from_slice(&self.checksum.to_le_bytes());
        result[(PageNumber::serialized_size() + size_of::<Checksum>())..]
            .copy_from_slice(&self.length.to_le_bytes());

        result
    }
}

pub fn native_header_roundtrip(header: BtreeHeader) -> (result: BtreeHeader)
    requires canonical_page(header.root),
    ensures result.root == header.root, result.checksum == header.checksum, result.length == header.length,
{
    proof {
        canonical_page_word_roundtrip(header.root);
        vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
        vstd::bytes::lemma_auto_spec_u128_to_from_le_bytes();
        assert(vstd::bytes::spec_u128_to_le_bytes(header.checksum).len() == 16);
        assert(vstd::bytes::spec_u64_from_le_bytes(
            vstd::bytes::spec_u64_to_le_bytes(encoded_page_word(header.root))) == encoded_page_word(header.root));
        assert(vstd::bytes::spec_u128_from_le_bytes(
            vstd::bytes::spec_u128_to_le_bytes(header.checksum)) == header.checksum);
        assert(vstd::bytes::spec_u64_from_le_bytes(vstd::bytes::spec_u64_to_le_bytes(header.length)) == header.length);
    }
    BtreeHeader::from_le_bytes(header.to_le_bytes())
}

}
