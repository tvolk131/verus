use vstd::prelude::*;
use provider::{TransactionId, UnrepairedDatabaseHeader, Result};

verus! {
#[verifier::infer]
pub fn ordering(first: TransactionId, second: TransactionId) -> bool {
    first > second
}

pub fn imported_selection(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(),
    ensures UnrepairedDatabaseHeader::selection_post(*old(header), *final(header), result),
{
    header.select_primary_slot()
}

pub fn imported_order_goal(first: TransactionId, second: TransactionId) -> (result: bool)
    ensures result == (first.0 > second.0),
{
    ordering(first, second)
}
}
