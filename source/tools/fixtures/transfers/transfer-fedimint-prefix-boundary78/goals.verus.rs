pub fn same_prefix_keys<Inner: Debug>(database: &PrefixDatabase<Inner>, left: &[u8], right: &[u8])
    -> (keys: (Vec<u8>, Vec<u8>))
    ensures keys.0@ == prefix_view(database.prefix@, left@),
        keys.1@ == prefix_view(database.prefix@, right@),
        (keys.0@ == keys.1@) == (left@ == right@),
{
    let first = database.get_full_key(left);
    let second = database.get_full_key(right);
    proof {
        if first@ == second@ {
            suffix_cancellation(database.prefix@, left@, right@);
        }
    }
    (first, second)
}

pub fn separate_namespace_keys<Inner, Other>(left: &PrefixDatabaseTransaction<Inner>,
    right: &PrefixDatabaseTransaction<Other>, first: &[u8], second: &[u8])
    -> (keys: (Vec<u8>, Vec<u8>))
    requires prefix_incomparable(left.prefix@, right.prefix@),
    ensures keys.0@ == prefix_view(left.prefix@, first@),
        keys.1@ == prefix_view(right.prefix@, second@), keys.0@ != keys.1@,
{
    let first_key = left.get_full_key(first);
    let second_key = right.get_full_key(second);
    proof { disjoint_namespaces(left.prefix@, right.prefix@, first@, second@); }
    (first_key, second_key)
}
