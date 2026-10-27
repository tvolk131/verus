use vstd::prelude::*;
#[verus_infer]
fn length(values: &[u8], start: usize, end: usize) -> usize {
    let window = &values[start..end];
    window.len()
}
verus! {fn caller(){let _ = length(&[1,2],2,1);}}
fn main(){}
