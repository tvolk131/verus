use vstd::prelude::*;
#[verus_infer(precision_rlimit=0.000001)]
fn pair(value: u8) -> [u8; 2] { [value, value + 1] }
fn main(){}
