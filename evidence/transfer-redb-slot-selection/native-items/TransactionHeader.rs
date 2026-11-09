pub(super) struct TransactionHeader {
    pub(super) version: u8,
    pub(super) user_root: Option<BtreeHeader>,
    pub(super) system_root: Option<BtreeHeader>,
    pub(super) transaction_id: TransactionId,
    // When present, the original on-disk bytes of a slot whose checksum did not verify. They are
    // written back verbatim, so a slot that failed verification keeps its invalid checksum instead
    // of being re-serialized as if it were valid. `None` once the slot holds a new commit.
    corrupt_bytes: Option<[u8; TRANSACTION_SIZE]>,
}
