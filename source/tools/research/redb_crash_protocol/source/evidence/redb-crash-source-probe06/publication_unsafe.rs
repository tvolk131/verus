use vstd::prelude::*;
verus! {
const TRANSACTION_SIZE: usize = 128;
type Checksum = u128;
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct PageNumber {
    pub(crate) region: u32,
    pub(crate) page_index: u32,
    pub(crate) page_order: u8,
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TransactionId(u64);
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct BtreeHeader {
    pub(crate) root: PageNumber,
    pub(crate) checksum: Checksum,
    pub(crate) length: u64,
}
#[derive(Clone)]
pub(crate) struct TransactionHeader {
    pub(crate) version: u8,
    pub(crate) user_root: Option<BtreeHeader>,
    pub(crate) system_root: Option<BtreeHeader>,
    pub(crate) transaction_id: TransactionId,
    // When present, the original on-disk bytes of a slot whose checksum did not verify. They are
    // written back verbatim, so a slot that failed verification keeps its invalid checksum instead
    // of being re-serialized as if it were valid. `None` once the slot holds a new commit.
    corrupt_bytes: Option<[u8; TRANSACTION_SIZE]>,
}
#[derive(Clone)]
pub(crate) struct DatabaseHeader {
    primary_slot: usize,
    pub(crate) recovery_required: bool,
    pub(crate) two_phase_commit: bool,
    pub(crate) primary_unflushed: bool,
    page_size: u32,
    region_header_pages: u32,
    region_max_data_pages: u32,
    full_regions: u32,
    trailing_partial_region_pages: u32,
    transaction_slots: [TransactionHeader; 2],
}
impl DatabaseHeader {
    pub(crate) fn primary_slot(&self) -> (r: &TransactionHeader) 
requires true,
ensures *r == self.transaction_slots[self.primary_slot as int],
{
        &self.transaction_slots[self.primary_slot]
    }
    pub(crate) fn secondary_slot(&self) -> (r: &TransactionHeader) 
requires true,
ensures *r == self.transaction_slots[(self.primary_slot ^ 1) as int],
{
        &self.transaction_slots[self.primary_slot ^ 1]
    }
    pub(crate) fn published_slot(&self) -> (r: &TransactionHeader) 
requires true,
ensures *r == self.transaction_slots[(if self.primary_unflushed { self.primary_slot ^ 1 } else { self.primary_slot }) as int],
{
        if self.primary_unflushed {
            self.secondary_slot()
        } else {
            self.primary_slot()
        }
    }
    pub(crate) fn swap_primary_slot(&mut self) 
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
    pub(crate) fn write_secondary_slot(
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
