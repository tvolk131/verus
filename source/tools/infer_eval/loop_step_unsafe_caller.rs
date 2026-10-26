use vstd::prelude::*;
#[verus_infer]
fn debit(x:u8,n:u8,step:u8)->u8 {
    let mut r=x;let mut remaining=n;
    while remaining>0 {r-=step;remaining-=1;}r
}
verus! {fn caller(){let _=debit(254,128,2);}}
fn main(){}
