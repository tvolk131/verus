pub(super) struct UnrepairedDatabaseHeader {
    inner: DatabaseHeader,
    primary_corrupted: bool,
    secondary_corrupted: bool,
}
