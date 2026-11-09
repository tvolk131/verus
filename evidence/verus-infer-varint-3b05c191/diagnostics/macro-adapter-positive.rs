#![feature(proc_macro_hygiene)]
use vstd::prelude::*;

macro_rules! count_to {
    ($limit:expr, $count:ident) => {
        verus_exec_expr! {
            for index in 0..$limit
                invariant $count == index,
            {
                $count += 1;
            }
        }
    };
}

#[verus_spec(result => ensures result == limit)]
pub fn count(limit: u32) -> u32 {
    let mut result = 0u32;
    count_to!(limit, result);
    result
}
