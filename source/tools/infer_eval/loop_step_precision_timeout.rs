use vstd::prelude::*;
#[verus_infer(precision_rlimit=0.000001)]
fn credit(x:u8,n:u8,step:u8)->u8 {
    let mut r=x;let mut remaining=n;
    while remaining>0 {r+=step;remaining-=1;}r
}
fn main(){}
