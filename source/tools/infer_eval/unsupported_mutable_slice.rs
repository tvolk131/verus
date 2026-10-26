use vstd::prelude::*;
#[verus_infer]
pub fn first(values: &mut [u8]) -> u8 { values[0] }
