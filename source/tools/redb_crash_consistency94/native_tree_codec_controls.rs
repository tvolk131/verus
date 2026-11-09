use vstd::prelude::*;
use pagecodec94::*;
use btreecodec94::*;

verus! {

pub fn reject_changed_root_checksum(header: BtreeHeader)
    requires canonical_page(header.root),
{
    let result = native_header_roundtrip(header);
    assert(result.checksum == header.checksum ^ 1);
}

pub fn reject_changed_root_length(header: BtreeHeader)
    requires canonical_page(header.root),
{
    let result = native_header_roundtrip(header);
    assert(result.length == header.length ^ 1);
}

pub fn reject_unrestricted_page_roundtrip() {
    let page = PageNumber { region: 0, page_index: 1, page_order: 20 };
    let result = PageNumber::from_le_bytes(page.to_le_bytes());
    proof {
        vstd::bytes::lemma_auto_spec_u64_to_from_le_bytes();
        assert(vstd::bytes::spec_u64_from_le_bytes(
            vstd::bytes::spec_u64_to_le_bytes(encoded_page_word(page))) == encoded_page_word(page));
        assert(0x000F_FFFFu64 >> 20 == 0) by (bit_vector);
        let page_index = page.page_index;
        let region = page.region;
        let page_order = page.page_order;
        assert(((0x000F_FFFFu64 & page_index as u64)
            | ((0x000F_FFFFu64 & region as u64) << 20)
            | ((0b0001_1111u64 & page_order as u64) << 59)) >> 59 == 20)
            by (bit_vector) requires page_index == 1, region == 0, page_order == 20;
    }
    assert(result.page_order == 20);
    assert(result.page_index == 0);
    assert(result == page);
}

}
