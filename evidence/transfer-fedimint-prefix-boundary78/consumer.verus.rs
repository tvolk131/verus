use vstd::prelude::*;
use std::fmt::Debug;
use std::ops::Range;
use prefix_provider::*;

verus! {

pub fn imported_keys<Inner: Debug, Other>(database: &PrefixDatabase<Inner>,
    transaction: &PrefixDatabaseTransaction<Other>, suffix: &[u8])
    -> (keys: (Vec<u8>, Vec<u8>))
    ensures keys.0@ == prefix_view(database.prefix@, suffix@),
        keys.1@ == prefix_view(transaction.prefix@, suffix@),
{
    (database.get_full_key(suffix), transaction.get_full_key(suffix))
}

pub fn imported_range<Inner>(transaction: &PrefixDatabaseTransaction<Inner>, range: Range<&[u8]>)
    -> (result: Range<Vec<u8>>)
    ensures result.start@ == prefix_view(transaction.prefix@, range.start@),
        result.end@ == prefix_view(transaction.prefix@, range.end@),
{
    transaction.get_full_range(range)
}

pub fn imported_separation<Inner, Other>(left: &PrefixDatabaseTransaction<Inner>,
    right: &PrefixDatabaseTransaction<Other>, first: &[u8], second: &[u8])
    -> (keys: (Vec<u8>, Vec<u8>))
    requires prefix_incomparable(left.prefix@, right.prefix@),
    ensures keys.0@ != keys.1@,
{
    let first_key = left.get_full_key(first);
    let second_key = right.get_full_key(second);
    proof { disjoint_namespaces(left.prefix@, right.prefix@, first@, second@); }
    (first_key, second_key)
}

}
