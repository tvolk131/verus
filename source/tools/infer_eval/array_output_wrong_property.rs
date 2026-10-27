use vstd::prelude::*;
#[verus_infer]
fn pair(value: u8) -> [u8; 2] { [value, value + 1] }
verus! {fn caller(){let p=pair(254);assert(p[1]==254);}}
fn main(){}
