use vstd::prelude::*;
#[verus_infer(precision_rlimit = 0.000001)]
pub fn selected(b: bool) -> u8 { if b { 1 } else { 2 } }
