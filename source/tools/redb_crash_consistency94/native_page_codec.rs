use vstd::prelude::*;

verus! {

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PageNumber {
    pub region: u32,
    pub page_index: u32,
    pub page_order: u8,
}

pub open spec fn encoded_page_word(page: PageNumber) -> u64 {
    (0x000F_FFFFu64 & (page.page_index as u64))
        | ((0x000F_FFFFu64 & (page.region as u64)) << 20)
        | ((0b0001_1111u64 & (page.page_order as u64)) << 59)
}

pub open spec fn canonical_page(page: PageNumber) -> bool {
    page.page_order <= 31 && page.region <= 0x000F_FFFFu32
        && page.page_index as u64 <= (0x000F_FFFFu64 >> page.page_order)
}

impl PageNumber {
    pub const fn serialized_size() -> (result: usize)
        ensures result == 8,
    {
        8
    }

    pub fn to_le_bytes(self) -> (result: [u8; 8])
        ensures result@ == vstd::bytes::spec_u64_to_le_bytes(encoded_page_word(self)),
    {
        let mut temp = 0x000F_FFFF & u64::from(self.page_index);
        temp |= (0x000F_FFFF & u64::from(self.region)) << 20;
        temp |= (0b0001_1111 & u64::from(self.page_order)) << 59;
        temp.to_le_bytes()
    }

    pub fn from_le_bytes(bytes: [u8; 8]) -> (result: Self)
        ensures canonical_page(result),
            result.page_order == (vstd::bytes::spec_u64_from_le_bytes(bytes@) >> 59) as u8,
            result.page_index as u64 == (vstd::bytes::spec_u64_from_le_bytes(bytes@)
                & (0x000F_FFFFu64 >> result.page_order)),
            result.region == ((vstd::bytes::spec_u64_from_le_bytes(bytes@) >> 20) & 0x000F_FFFFu64) as u32,
    {
        let temp = u64::from_le_bytes(bytes);
        proof {
            assert(temp >> 59 <= 31) by (bit_vector);
        }
        let order = (temp >> 59) as u8;
        proof {
            assert(temp & (0x000F_FFFFu64 >> order) <= 0x000F_FFFFu64 >> order)
                by (bit_vector);
            assert(0x000F_FFFFu64 >> order <= 0x000F_FFFFu64) by (bit_vector);
            assert((temp >> 20) & 0x000F_FFFFu64 <= 0x000F_FFFFu64) by (bit_vector);
        }
        let index = u32::try_from(temp & (0x000F_FFFF >> order)).unwrap();
        let region = ((temp >> 20) & 0x000F_FFFF) as u32;

        Self {
            region,
            page_index: index,
            page_order: order,
        }
    }
}

pub proof fn canonical_page_word_roundtrip(page: PageNumber)
    requires canonical_page(page),
    ensures (encoded_page_word(page) >> 59) as u8 == page.page_order,
        encoded_page_word(page) & (0x000F_FFFFu64 >> page.page_order) == page.page_index as u64,
        ((encoded_page_word(page) >> 20) & 0x000F_FFFFu64) as u32 == page.region,
{
    let region = page.region;
    let page_index = page.page_index;
    let page_order = page.page_order;
    assert(page_order <= 31 && region <= 0x000F_FFFFu32
        && page_index as u64 <= 0x000F_FFFFu64 >> page_order);
    assert({
        let encoded = (0x000F_FFFFu64 & (page_index as u64))
            | ((0x000F_FFFFu64 & (region as u64)) << 20)
            | ((0b0001_1111u64 & (page_order as u64)) << 59);
        (encoded >> 59) as u8 == page_order
            && encoded & (0x000F_FFFFu64 >> page_order) == page_index as u64
            && ((encoded >> 20) & 0x000F_FFFFu64) as u32 == region
    }) by (bit_vector)
        requires page_order <= 31, region <= 0x000F_FFFFu32,
            page_index as u64 <= 0x000F_FFFFu64 >> page_order;
}

pub fn canonical_page_roundtrip(page: PageNumber) -> (result: PageNumber)
    requires canonical_page(page),
    ensures result == page,
{
    proof {
        canonical_page_word_roundtrip(page);
        vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
        assert(vstd::bytes::spec_u64_from_le_bytes(
            vstd::bytes::spec_u64_to_le_bytes(encoded_page_word(page))) == encoded_page_word(page));
    }
    PageNumber::from_le_bytes(page.to_le_bytes())
}

}
