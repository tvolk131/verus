use vstd::prelude::*;
use std::ops::Range;
verus! {
pub(crate) struct PageNumber {
    pub(crate) region: u32,
    pub(crate) page_index: u32,
    pub(crate) page_order: u8,
}
impl PageNumber {
#[verifier::infer]
    pub(crate) fn page_size_bytes(&self, page_size: u32) -> u64 {
        let pages = 1u64 << self.page_order;
        pages * u64::from(page_size)
    }
#[verifier::infer]
    pub(crate) fn address_range(
        &self,
        data_section_offset: u64,
        region_size: u64,
        region_pages_start: u64,
        page_size: u32,
    ) -> Range<u64> {
        let regional_start =
            region_pages_start + u64::from(self.page_index) * self.page_size_bytes(page_size);
        debug_assert!(regional_start < region_size);
        let region_base = u64::from(self.region) * region_size;
        let start = data_section_offset + region_base + regional_start;
        let end = start + self.page_size_bytes(page_size);
        start..end
    }
}
fn strict_start(region:u32,first:u32,second:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)->(result:(Range<u64>,Range<u64>))
    requires order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        first as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        second as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        first<second,
        size>0,
    ensures result.0.start<result.1.start,
{
    let a=PageNumber{region,page_index:first,page_order:order}.address_range(offset,region_size,pages_start,size);
    let b=PageNumber{region,page_index:second,page_order:order}.address_range(offset,region_size,pages_start,size);
    (a,b)
}
fn injective_start(region:u32,first:u32,second:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)->(result:(Range<u64>,Range<u64>))
    requires order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        first as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        second as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        first!=second,
        size>0,
    ensures result.0.start!=result.1.start,
{
    let a=PageNumber{region,page_index:first,page_order:order}.address_range(offset,region_size,pages_start,size);
    let b=PageNumber{region,page_index:second,page_order:order}.address_range(offset,region_size,pages_start,size);
    (a,b)
}
fn ordered_nonoverlap(region:u32,first:u32,second:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)->(result:(Range<u64>,Range<u64>))
    requires order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        first as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        second as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        first<second,
        size>0,
    ensures result.0.end<=result.1.start,
{
    let a=PageNumber{region,page_index:first,page_order:order}.address_range(offset,region_size,pages_start,size);
    let b=PageNumber{region,page_index:second,page_order:order}.address_range(offset,region_size,pages_start,size);
    (a,b)
}
fn nonoverlap_all_sizes(region:u32,first:u32,second:u32,order:u8,size:u32,offset:u64,region_size:u64,pages_start:u64)->(result:(Range<u64>,Range<u64>))
    requires order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        first as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + first as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + first as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        order<64,
        ((1u64 << order) as int * size as int)<=u64::MAX,
        second as int * ((1u64 << order) as int * size as int)<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<=u64::MAX,
        (pages_start as int + second as int * ((1u64 << order) as int * size as int))<region_size,
        (region as int * region_size as int)<=u64::MAX,
        (offset as int + (region as int * region_size as int))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))<=u64::MAX,
        ((offset as int + (region as int * region_size as int)) + (pages_start as int + second as int * ((1u64 << order) as int * size as int)))+((1u64 << order) as int * size as int)<=u64::MAX,
        first<second,
    ensures result.0.end<=result.1.start,
{
    let a=PageNumber{region,page_index:first,page_order:order}.address_range(offset,region_size,pages_start,size);
    let b=PageNumber{region,page_index:second,page_order:order}.address_range(offset,region_size,pages_start,size);
    (a,b)
}

fn zero_witness()->(result:(Range<u64>,Range<u64>))
    ensures result.0.start==0,result.0.end==0,result.1.start==0,result.1.end==0,
        result.0.end<=result.1.start,
{
    let a=PageNumber{region:0,page_index:0,page_order:63}.address_range(0,1,0,0);
    let b=PageNumber{region:0,page_index:1,page_order:63}.address_range(0,1,0,0);
    (a,b)
}
fn mixed_witness()->(result:(Range<u64>,Range<u64>))
    ensures result.0.start==0,result.0.end==8192,result.1.start==4096,result.1.end==8192,
        result.0.start<result.1.end && result.1.start<result.0.end,
{
    let a=PageNumber{region:0,page_index:0,page_order:1}.address_range(0,8192,0,4096);
    let b=PageNumber{region:0,page_index:1,page_order:0}.address_range(0,8192,0,4096);
    (a,b)
}
}
