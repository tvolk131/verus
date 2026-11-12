
use vstd::prelude::*;
verus! {
const TRANSACTION_SIZE: usize = 128;
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct PageNumber {
    pub(crate) region: u32,
    pub(crate) page_index: u32,
    pub(crate) page_order: u8,
}
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct TransactionId(u64);

impl TransactionId {
    pub(crate) fn new(value: u64) -> TransactionId {
        Self(value)
    }

    pub(crate) fn raw_id(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self) -> TransactionId {
        TransactionId(self.0 + 1)
    }

    pub(crate) fn increment(&mut self) -> TransactionId {
        let next = self.next();
        *self = next;
        next
    }
}
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
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2,
ensures *r == self.transaction_slots[self.primary_slot as int],
{
        &self.transaction_slots[self.primary_slot]
    }
    pub(crate) fn secondary_slot(&self) -> (r: &TransactionHeader) 
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2,
ensures *r == self.transaction_slots[(self.primary_slot ^ 1) as int],
{
        &self.transaction_slots[self.primary_slot ^ 1]
    }
    pub(crate) fn published_slot(&self) -> (r: &TransactionHeader) 
requires self.primary_slot < 2, (self.primary_slot ^ 1) < 2,
ensures *r == self.transaction_slots[(if self.primary_unflushed { self.primary_slot ^ 1 } else { self.primary_slot }) as int],
{
        if self.primary_unflushed {
            self.secondary_slot()
        } else {
            self.primary_slot()
        }
    }
    pub(crate) fn swap_primary_slot(&mut self) 
requires old(self).primary_slot < 2, (old(self).primary_slot ^ 1) < 2,
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
requires old(self).primary_slot < 2, (old(self).primary_slot ^ 1) < 2,
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

use core::mem::size_of;
pub(crate) const MAGICNUMBER: [u8; 9] = [b'r', b'e', b'd', b'b', 0x1A, 0x0A, 0xA9, 0x0D, 0x0A];
const GOD_BYTE_OFFSET: usize = MAGICNUMBER.len();
const PAGE_SIZE_OFFSET: usize = GOD_BYTE_OFFSET + size_of::<u8>() + 2; // +2 for padding
const REGION_HEADER_PAGES_OFFSET: usize = PAGE_SIZE_OFFSET + size_of::<u32>();
const REGION_MAX_DATA_PAGES_OFFSET: usize = REGION_HEADER_PAGES_OFFSET + size_of::<u32>();
const NUM_FULL_REGIONS_OFFSET: usize = REGION_MAX_DATA_PAGES_OFFSET + size_of::<u32>();
const TRAILING_REGION_DATA_PAGES_OFFSET: usize = NUM_FULL_REGIONS_OFFSET + size_of::<u32>();
// Formerly the region tracker page
const _UNUSED3_OFFSET: usize = TRAILING_REGION_DATA_PAGES_OFFSET + size_of::<u32>();

const TRANSACTION_0_OFFSET: usize = 64;
const TRANSACTION_1_OFFSET: usize = TRANSACTION_0_OFFSET + TRANSACTION_SIZE;
pub(crate) const DB_HEADER_SIZE: usize = TRANSACTION_1_OFFSET + TRANSACTION_SIZE;

// God byte flags
const PRIMARY_BIT: u8 = 1;
const RECOVERY_REQUIRED: u8 = 2;
const TWO_PHASE_COMMIT: u8 = 4;
// Set by the write that makes a commit the primary in the multi-process modes, and cleared by a
// second write once the first has been flushed. Another process that finds it set serves the
// secondary slot, the previous commit, so no process serves a commit a power loss could still
// lose. It outlives a writer that crashes in that window; the next writer's recovery clears it.
const PRIMARY_UNFLUSHED: u8 = 8;

// Structure of each commit slot
const VERSION_OFFSET: usize = 0;
const USER_ROOT_NON_NULL_OFFSET: usize = size_of::<u8>();
const SYSTEM_ROOT_NON_NULL_OFFSET: usize = USER_ROOT_NON_NULL_OFFSET + size_of::<u8>();
const _UNUSED_OFFSET: usize = SYSTEM_ROOT_NON_NULL_OFFSET + size_of::<u8>();
const PADDING: usize = 4;

const USER_ROOT_OFFSET: usize = _UNUSED_OFFSET + size_of::<u8>() + PADDING;
const SYSTEM_ROOT_OFFSET: usize = USER_ROOT_OFFSET + BtreeHeader::serialized_size();
const _UNUSED2_OFFSET: usize = SYSTEM_ROOT_OFFSET + BtreeHeader::serialized_size();
const TRANSACTION_ID_OFFSET: usize = _UNUSED2_OFFSET + BtreeHeader::serialized_size();
const TRANSACTION_LAST_FIELD: usize = TRANSACTION_ID_OFFSET + size_of::<u64>();

const SLOT_CHECKSUM_OFFSET: usize = TRANSACTION_SIZE - size_of::<Checksum>();


type Checksum = u128;
const FILE_FORMAT_VERSION3: u8 = 3;
impl TransactionId {
    pub(crate) fn raw_id(self) -> u64 {
        self.0
    }
}
impl PageNumber {
    pub(crate) const fn serialized_size() -> usize {
        8
    }
    pub(crate) fn to_le_bytes(self) -> [u8; 8] {
        let mut temp = 0x000F_FFFF & u64::from(self.page_index);
        temp |= (0x000F_FFFF & u64::from(self.region)) << 20;
        temp |= (0b0001_1111 & u64::from(self.page_order)) << 59;
        temp.to_le_bytes()
    }
}
impl BtreeHeader {
    pub(crate) const fn serialized_size() -> usize {
        PageNumber::serialized_size() + size_of::<Checksum>() + size_of::<u64>()
    }
    pub(crate) fn to_le_bytes(self) -> [u8; Self::serialized_size()] {
        let mut result = [0; Self::serialized_size()];
        result[..PageNumber::serialized_size()].copy_from_slice(&self.root.to_le_bytes());
        result[PageNumber::serialized_size()
            ..(PageNumber::serialized_size() + size_of::<Checksum>())]
            .copy_from_slice(&self.checksum.to_le_bytes());
        result[(PageNumber::serialized_size() + size_of::<Checksum>())..]
            .copy_from_slice(&self.length.to_le_bytes());

        result
    }
}
impl TransactionHeader {
    pub(crate) fn to_bytes(&self) -> [u8; TRANSACTION_SIZE] {
        // A slot that failed checksum verification is written back verbatim (see `corrupt_bytes`).
        if let Some(bytes) = self.corrupt_bytes {
            return bytes;
        }
        assert_eq!(self.version, FILE_FORMAT_VERSION3);
        let mut result = [0; TRANSACTION_SIZE];
        result[VERSION_OFFSET] = self.version;
        if let Some(header) = self.user_root {
            result[USER_ROOT_NON_NULL_OFFSET] = 1;
            result[USER_ROOT_OFFSET..(USER_ROOT_OFFSET + BtreeHeader::serialized_size())]
                .copy_from_slice(&header.to_le_bytes());
        }
        if let Some(header) = self.system_root {
            result[SYSTEM_ROOT_NON_NULL_OFFSET] = 1;
            result[SYSTEM_ROOT_OFFSET..(SYSTEM_ROOT_OFFSET + BtreeHeader::serialized_size())]
                .copy_from_slice(&header.to_le_bytes());
        }
        result[TRANSACTION_ID_OFFSET..(TRANSACTION_ID_OFFSET + size_of::<u64>())]
            .copy_from_slice(&self.transaction_id.raw_id().to_le_bytes());
        let checksum = xxh3_checksum(&result[..SLOT_CHECKSUM_OFFSET]);
        result[SLOT_CHECKSUM_OFFSET..(SLOT_CHECKSUM_OFFSET + size_of::<Checksum>())]
            .copy_from_slice(&checksum.to_le_bytes());

        result
    }
}
impl DatabaseHeader {
    pub(crate) fn to_bytes(&self, include_magic_number: bool) -> [u8; DB_HEADER_SIZE] {
        let mut result = [0; DB_HEADER_SIZE];
        if include_magic_number {
            result[..MAGICNUMBER.len()].copy_from_slice(&MAGICNUMBER);
        }
        result[GOD_BYTE_OFFSET] = self.primary_slot.try_into().unwrap();
        if self.recovery_required {
            result[GOD_BYTE_OFFSET] |= RECOVERY_REQUIRED;
        }
        if self.two_phase_commit {
            result[GOD_BYTE_OFFSET] |= TWO_PHASE_COMMIT;
        }
        if self.primary_unflushed {
            result[GOD_BYTE_OFFSET] |= PRIMARY_UNFLUSHED;
        }
        result[PAGE_SIZE_OFFSET..(PAGE_SIZE_OFFSET + size_of::<u32>())]
            .copy_from_slice(&self.page_size.to_le_bytes());
        result[REGION_HEADER_PAGES_OFFSET..(REGION_HEADER_PAGES_OFFSET + size_of::<u32>())]
            .copy_from_slice(&self.region_header_pages.to_le_bytes());
        result[REGION_MAX_DATA_PAGES_OFFSET..(REGION_MAX_DATA_PAGES_OFFSET + size_of::<u32>())]
            .copy_from_slice(&self.region_max_data_pages.to_le_bytes());
        result[NUM_FULL_REGIONS_OFFSET..(NUM_FULL_REGIONS_OFFSET + size_of::<u32>())]
            .copy_from_slice(&self.full_regions.to_le_bytes());
        result[TRAILING_REGION_DATA_PAGES_OFFSET
            ..(TRAILING_REGION_DATA_PAGES_OFFSET + size_of::<u32>())]
            .copy_from_slice(&self.trailing_partial_region_pages.to_le_bytes());
        let slot0 = self.transaction_slots[0].to_bytes();
        result[TRANSACTION_0_OFFSET..(TRANSACTION_0_OFFSET + slot0.len())].copy_from_slice(&slot0);
        let slot1 = self.transaction_slots[1].to_bytes();
        result[TRANSACTION_1_OFFSET..(TRANSACTION_1_OFFSET + slot1.len())].copy_from_slice(&slot1);

        result
    }
}
pub(crate) fn xxh3_checksum(data: &[u8]) -> Checksum {
    hash128_with_seed(data, 0)
}
}
