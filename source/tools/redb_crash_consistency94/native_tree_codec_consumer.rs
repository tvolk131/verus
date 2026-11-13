use vstd::prelude::*;
use pagecodec94::*;
use btreecodec94::*;

verus! {

pub fn imported_native_root_roundtrip(header: BtreeHeader) -> (result: BtreeHeader)
    requires canonical_page(header.root),
    ensures result.root == header.root, result.checksum == header.checksum, result.length == header.length,
{
    native_header_roundtrip(header)
}

pub fn imported_native_page_roundtrip(page: PageNumber) -> (result: PageNumber)
    requires canonical_page(page),
    ensures result == page,
{
    canonical_page_roundtrip(page)
}

}
