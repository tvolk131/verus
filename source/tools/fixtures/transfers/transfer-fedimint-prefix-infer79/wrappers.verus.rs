#[verifier::infer]
pub fn key_contents(prefix: &Vec<u8>, key: &[u8]) -> Vec<u8> {
    let database = PrefixDatabaseTransaction {
        inner: 0u8,
        global_dbtx_access_token: None,
        prefix: prefix.clone(),
    };
    database.get_full_key(key)
}

#[verifier::infer]
pub fn range_contents(prefix: &Vec<u8>, bounds: Range<&[u8]>) -> Range<Vec<u8>> {
    let database = PrefixDatabaseTransaction {
        inner: 0u8,
        global_dbtx_access_token: None,
        prefix: prefix.clone(),
    };
    database.get_full_range(bounds)
}
