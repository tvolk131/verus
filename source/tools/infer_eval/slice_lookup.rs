use vstd::prelude::*;
#[verus_infer]
pub fn at(values: &[u8], index: usize) -> u8 { values[index] }
