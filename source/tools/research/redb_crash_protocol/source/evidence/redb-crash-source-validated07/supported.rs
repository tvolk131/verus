use vstd::prelude::*;
verus! {
const TRANSACTION_SIZE: usize = 128;
type Checksum = u128;
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PageNumber {
    pub region: u32,
    pub page_index: u32,
    pub page_order: u8,
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TransactionId(u64);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BtreeHeader {
    pub root: PageNumber,
    pub checksum: Checksum,
    pub length: u64,
}
#[derive(Clone)]
pub struct TransactionHeader {
    pub version: u8,
    pub user_root: Option<BtreeHeader>,
    pub system_root: Option<BtreeHeader>,
    pub transaction_id: TransactionId,
    // When present, the original on-disk bytes of a slot whose checksum did not verify. They are
    // written back verbatim, so a slot that failed verification keeps its invalid checksum instead
    // of being re-serialized as if it were valid. `None` once the slot holds a new commit.
    pub corrupt_bytes: Option<[u8; TRANSACTION_SIZE]>,
}
#[derive(Clone)]
pub struct DatabaseHeader {
    pub primary_slot: usize,
    pub recovery_required: bool,
    pub two_phase_commit: bool,
    pub primary_unflushed: bool,
    pub page_size: u32,
    pub region_header_pages: u32,
    pub region_max_data_pages: u32,
    pub full_regions: u32,
    pub trailing_partial_region_pages: u32,
    pub transaction_slots: [TransactionHeader; 2],
}
impl DatabaseHeader {
    pub fn primary_slot(&self) -> (r: &TransactionHeader) 
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2, self.primary_slot != (self.primary_slot ^ 1),
ensures *r == self.transaction_slots[self.primary_slot as int],
{
        &self.transaction_slots[self.primary_slot]
    }
    pub fn secondary_slot(&self) -> (r: &TransactionHeader) 
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2, self.primary_slot != (self.primary_slot ^ 1),
ensures *r == self.transaction_slots[(self.primary_slot ^ 1) as int],
{
        &self.transaction_slots[self.primary_slot ^ 1]
    }
    pub fn published_slot(&self) -> (r: &TransactionHeader) 
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2, self.primary_slot != (self.primary_slot ^ 1),
ensures *r == self.transaction_slots[(if self.primary_unflushed { self.primary_slot ^ 1 } else { self.primary_slot }) as int],
{
        if self.primary_unflushed {
            self.secondary_slot()
        } else {
            self.primary_slot()
        }
    }
    pub fn swap_primary_slot(&mut self) 
requires old(self).primary_slot < 2, (old(self).primary_slot ^ 1) < 2, old(self).primary_slot != (old(self).primary_slot ^ 1),
ensures final(self).primary_slot == old(self).primary_slot ^ 1,
final(self).transaction_slots == old(self).transaction_slots,
final(self).recovery_required == old(self).recovery_required,
final(self).two_phase_commit == old(self).two_phase_commit,
final(self).primary_unflushed == old(self).primary_unflushed,
final(self).page_size == old(self).page_size,
final(self).region_header_pages == old(self).region_header_pages,
final(self).region_max_data_pages == old(self).region_max_data_pages,
final(self).full_regions == old(self).full_regions,
final(self).trailing_partial_region_pages == old(self).trailing_partial_region_pages,
{
        self.primary_slot ^= 1;
    }
    pub fn write_secondary_slot(
        &mut self,
        transaction_id: TransactionId,
        user_root: Option<BtreeHeader>,
        system_root: Option<BtreeHeader>,
    ) 
requires old(self).primary_slot < 2, (old(self).primary_slot ^ 1) < 2, old(self).primary_slot != (old(self).primary_slot ^ 1),
ensures final(self).primary_slot == old(self).primary_slot,
final(self).transaction_slots[old(self).primary_slot as int] == old(self).transaction_slots[old(self).primary_slot as int],
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].transaction_id == transaction_id,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].user_root == user_root,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].system_root == system_root,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].corrupt_bytes == None,
final(self).transaction_slots[(old(self).primary_slot ^ 1) as int].version == old(self).transaction_slots[(old(self).primary_slot ^ 1) as int].version,
final(self).recovery_required == old(self).recovery_required,
final(self).two_phase_commit == old(self).two_phase_commit,
final(self).primary_unflushed == old(self).primary_unflushed,
final(self).page_size == old(self).page_size,
final(self).region_header_pages == old(self).region_header_pages,
final(self).region_max_data_pages == old(self).region_max_data_pages,
final(self).full_regions == old(self).full_regions,
final(self).trailing_partial_region_pages == old(self).trailing_partial_region_pages,
{
        let slot = &mut self.transaction_slots[self.primary_slot ^ 1];
        slot.transaction_id = transaction_id;
        slot.user_root = user_root;
        slot.system_root = system_root;
        slot.corrupt_bytes = None;
    }
}
}

verus! {
pub proof fn slot_domain(p: usize)
    requires p < 2,
    ensures (p ^ 1) < 2, p != (p ^ 1),
{
    assert((p ^ 1) < 2 && p != (p ^ 1)) by(bit_vector)
        requires p < 2;
}
pub fn publication_witness(prior_mode: bool, bytes: [u8; 128]) -> (r: DatabaseHeader)
    ensures r.primary_slot == 1, r.two_phase_commit == prior_mode,
        r.transaction_slots[0].corrupt_bytes == Some(bytes),
        r.transaction_slots[1].corrupt_bytes == None,
{
    let old_slot = TransactionHeader { version: 3, user_root: None, system_root: None,
        transaction_id: TransactionId(0), corrupt_bytes: Some(bytes) };
    let spare_slot = TransactionHeader { version: 3, user_root: None, system_root: None,
        transaction_id: TransactionId(0), corrupt_bytes: None };
    let mut header = DatabaseHeader { primary_slot: 0, recovery_required: true,
        two_phase_commit: prior_mode, primary_unflushed: false, page_size: 4096,
        region_header_pages: 0, region_max_data_pages: 1024, full_regions: 0,
        trailing_partial_region_pages: 1, transaction_slots: [old_slot, spare_slot] };
    proof { slot_domain(0); }
    header.write_secondary_slot(TransactionId(1), None, None);
    header.swap_primary_slot();
    proof { slot_domain(1); }
    let published = header.published_slot();
    assert(published.transaction_id == TransactionId(1));
    header.primary_unflushed = true;
    let published_old = header.published_slot();
    assert(published_old.corrupt_bytes == Some(bytes));
    header
}
}
