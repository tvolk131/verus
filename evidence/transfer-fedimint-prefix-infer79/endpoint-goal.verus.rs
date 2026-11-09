fn endpoint_transform<Inner>(database: &PrefixDatabaseTransaction<Inner>, bounds: Range<&[u8]>)
    -> (result: Range<Vec<u8>>)
    ensures result.start@ == prefix_view(database.prefix@, bounds.start@),
        result.end@ == prefix_view(database.prefix@, bounds.end@),
{
    database.get_full_range(bounds)
}
