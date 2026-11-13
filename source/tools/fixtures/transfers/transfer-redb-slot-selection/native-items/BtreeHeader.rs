pub(crate) struct BtreeHeader {
    pub(crate) root: PageNumber,
    pub(crate) checksum: Checksum,
    pub(crate) length: u64,
}
