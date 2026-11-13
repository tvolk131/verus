pub(super) struct DatabaseHeader {
    primary_slot: usize,
    pub(super) recovery_required: bool,
    pub(super) two_phase_commit: bool,
    pub(super) primary_unflushed: bool,
    page_size: u32,
    region_header_pages: u32,
    region_max_data_pages: u32,
    full_regions: u32,
    trailing_partial_region_pages: u32,
    transaction_slots: [TransactionHeader; 2],
}
