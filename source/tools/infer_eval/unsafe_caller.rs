use vstd::prelude::*;
#[verus_infer]
pub fn debit(balance: u8, amount: u8) -> u8 { balance - amount }
verus! { fn caller() { let _ = debit(0, 1); } }
