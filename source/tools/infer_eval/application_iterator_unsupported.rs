use vstd::prelude::*;
#[verus_infer] pub fn checksum(bytes:&[u8])->u8{bytes.iter().fold(0u8,|sum,value|sum.wrapping_add(*value))}
