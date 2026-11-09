use vstd::prelude::*;
verus! {
pub(crate) struct U64GroupedBitmap {
    len: u32,
    data: Vec<u64>,
}
impl U64GroupedBitmap {
    fn required_words(elements: u32) -> usize {
        let words = elements.div_ceil(64);
        words as usize
    }
}
}
