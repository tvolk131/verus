use vstd::prelude::*;
#[verus_infer]
pub fn first(values: &[u8], index: usize) -> u8 { values[index] }
