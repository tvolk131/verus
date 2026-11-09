pub struct GlobalDBTxAccessToken(u32);

struct PrefixDatabaseTransaction<Inner> {
    inner: Inner,
    global_dbtx_access_token: Option<GlobalDBTxAccessToken>,
    prefix: Vec<u8>,
}

impl<Inner> PrefixDatabaseTransaction<Inner> {
    fn get_full_key(&self, key: &[u8]) -> Vec<u8> {
        let mut full_key = self.prefix.clone();
        full_key.extend_from_slice(key);
        full_key
    }
}

pub trait IDatabaseTransactionOpsCore: MaybeSend {
    async fn raw_insert_bytes(
        &mut self,
        key: &[u8],
        value: &[u8],
    ) -> DatabaseResult<Option<Vec<u8>>>;
}

impl<Inner> IDatabaseTransactionOpsCore for PrefixDatabaseTransaction<Inner> {
    async fn raw_insert_bytes(
        &mut self,
        key: &[u8],
        value: &[u8],
    ) -> DatabaseResult<Option<Vec<u8>>> {
        let key = self.get_full_key(key);
        self.inner.raw_insert_bytes(&key, value).await
    }
}
