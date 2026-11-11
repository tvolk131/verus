use vstd::prelude::*;
verus! {
fn required_words(elements: u32) -> (result: usize)
    ensures result == (elements as nat + 63) / 64,
{
        let words = elements.div_ceil(64);
        words as usize
    }
}
fn main() {}
