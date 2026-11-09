use vstd::prelude::*;
use std::{io, panic};
#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct TransactionId(pub u64);
verus! {
pub type Checksum = u128;
const TRANSACTION_SIZE: usize = 128;
pub enum StorageError {
    /// The Database is corrupted
    Corrupted(String),
    /// The value being inserted exceeds the maximum of 3GiB
    ValueTooLarge(usize),
    Io(io::Error),
    /// The backend does not support a required storage operation.
    Unsupported,
    PreviousIo,
    DatabaseClosed,
    LockPoisoned(&'static panic::Location<'static>),
}
pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;
pub struct PageNumber {
    pub region: u32,
    pub page_index: u32,
    pub page_order: u8,
}
pub struct BtreeHeader {
    pub root: PageNumber,
    pub checksum: Checksum,
    pub length: u64,
}
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
pub struct UnrepairedDatabaseHeader {
    pub inner: DatabaseHeader,
    pub primary_corrupted: bool,
    pub secondary_corrupted: bool,
}
pub mod slot_bits {
use vstd::prelude::*;

pub broadcast proof fn two_slot_toggle(index: usize)
    requires index < 2,
    ensures (#[trigger] (index ^ 1usize)) < 2, (index ^ 1usize) != index,
{
    assert((index ^ 1usize) < 2) by(bit_vector)
        requires index < 2;
    assert((index ^ 1usize) != index) by(bit_vector);
}
}

broadcast use slot_bits::two_slot_toggle;

impl DatabaseHeader {
    pub open spec fn representation_valid(self) -> bool {
        self.primary_slot < 2
    }

    pub open spec fn frame(before: Self, after: Self) -> bool {
        after.recovery_required == before.recovery_required
        && after.two_phase_commit == before.two_phase_commit
        && after.primary_unflushed == before.primary_unflushed
        && after.page_size == before.page_size
        && after.region_header_pages == before.region_header_pages
        && after.region_max_data_pages == before.region_max_data_pages
        && after.full_regions == before.full_regions
        && after.trailing_partial_region_pages == before.trailing_partial_region_pages
        && after.transaction_slots == before.transaction_slots
    }
}

impl UnrepairedDatabaseHeader {
    pub open spec fn representation_valid(self) -> bool {
        self.inner.representation_valid()
    }

    pub open spec fn selection_success(self) -> bool {
        if self.inner.two_phase_commit {
            !self.primary_corrupted
        } else {
            !self.primary_corrupted || !self.secondary_corrupted
        }
    }

    pub open spec fn secondary_newer(self) -> bool {
        self.inner.transaction_slots@[(self.inner.primary_slot ^ 1) as int].transaction_id.0
            > self.inner.transaction_slots@[self.inner.primary_slot as int].transaction_id.0
    }

    pub open spec fn choose_secondary(self) -> bool {
        !self.inner.two_phase_commit && !self.secondary_corrupted
            && (self.primary_corrupted || self.secondary_newer())
    }

    pub open spec fn slot_admissible(self, index: usize) -> bool {
        (index == self.inner.primary_slot && !self.primary_corrupted)
        || (!self.inner.two_phase_commit
            && index == (self.inner.primary_slot ^ 1) && !self.secondary_corrupted)
    }

    pub open spec fn selection_post(before: Self, after: Self, result: Result<bool>) -> bool {
        after.representation_valid()
        && after.primary_corrupted == before.primary_corrupted
        && after.secondary_corrupted == before.secondary_corrupted
        && DatabaseHeader::frame(before.inner, after.inner)
        && after.inner.primary_slot == if before.choose_secondary() {
            before.inner.primary_slot ^ 1
        } else { before.inner.primary_slot }
        && match result {
            Ok(kept) => before.selection_success() && kept == !before.choose_secondary()
                && before.slot_admissible(after.inner.primary_slot),
            Err(StorageError::Corrupted(message)) => !before.selection_success()
                && message@ == if before.inner.two_phase_commit {
                    "Primary is corrupted despite 2-phase commit"@
                } else { "Both commit slots are corrupted"@ },
            Err(_) => false,
        }
    }
}

impl DatabaseHeader {
    pub fn primary_slot(&self) -> (result: &TransactionHeader)
        requires self.representation_valid(),
        ensures *result == self.transaction_slots@[self.primary_slot as int],
    {
        &self.transaction_slots[self.primary_slot]
    }
    pub fn secondary_slot(&self) -> (result: &TransactionHeader)
        requires self.representation_valid(),
        ensures *result == self.transaction_slots@[(self.primary_slot ^ 1) as int],
    {
        &self.transaction_slots[self.primary_slot ^ 1]
    }
    pub fn swap_primary_slot(&mut self)
        requires old(self).representation_valid(),
        ensures final(self).representation_valid(),
            final(self).primary_slot == old(self).primary_slot ^ 1,
            DatabaseHeader::frame(*old(self), *final(self)),
    {
        self.primary_slot ^= 1;
    }
}
impl UnrepairedDatabaseHeader {
    pub fn select_primary_slot(&mut self) -> (result: Result<bool>)
        requires old(self).representation_valid(),
        ensures UnrepairedDatabaseHeader::selection_post(*old(self), *final(self), result),
    {
        // If the primary was written using 2-phase commit, it's guaranteed to be valid. Don't look
        // at the secondary; even if it happens to have a valid checksum, Durability::Paranoid means
        // we can't trust it
        if self.inner.two_phase_commit {
            if self.primary_corrupted {
                return Err(StorageError::Corrupted(
                    "Primary is corrupted despite 2-phase commit".to_string(),
                ));
            }
            return Ok(true);
        }

        // Pick whichever slot is newer, assuming it has a valid checksum. This handles an edge case
        // where we crash during fsync(), and the only data that got written to disk was the god byte
        // update swapping the primary -- in that case, the primary contains a valid but out-of-date
        // transaction, so we need to load from the secondary instead
        if self.primary_corrupted {
            if self.secondary_corrupted {
                return Err(StorageError::Corrupted(
                    "Both commit slots are corrupted".to_string(),
                ));
            }
            self.inner.swap_primary_slot();
            return Ok(false);
        }

        let secondary_newer =
            self.inner.secondary_slot().transaction_id > self.inner.primary_slot().transaction_id;
        if secondary_newer && !self.secondary_corrupted {
            self.inner.swap_primary_slot();
            return Ok(false);
        }

        Ok(true)
    }
}
pub fn two_phase_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), old(header).inner.two_phase_commit,
    ensures final(header).inner.primary_slot == old(header).inner.primary_slot,
        result.is_ok() == !old(header).primary_corrupted,
        result.is_ok() ==> result->Ok_0,
{
    header.select_primary_slot()
}

pub fn both_corrupt_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        old(header).primary_corrupted, old(header).secondary_corrupted,
    ensures result.is_err(), *final(header) == *old(header),
{
    header.select_primary_slot()
}

pub fn fallback_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        old(header).primary_corrupted, !old(header).secondary_corrupted,
    ensures result.is_ok(), !result->Ok_0,
        final(header).inner.primary_slot == (old(header).inner.primary_slot ^ 1),
{
    header.select_primary_slot()
}

pub fn newer_secondary_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, !old(header).secondary_corrupted,
        old(header).secondary_newer(),
    ensures result.is_ok(), !result->Ok_0,
        final(header).inner.primary_slot == (old(header).inner.primary_slot ^ 1),
{
    header.select_primary_slot()
}

pub fn equal_or_stale_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, !old(header).secondary_newer(),
    ensures result.is_ok(), result->Ok_0,
        final(header).inner.primary_slot == old(header).inner.primary_slot,
{
    header.select_primary_slot()
}

pub fn corrupt_secondary_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, old(header).secondary_corrupted,
    ensures result.is_ok(), result->Ok_0,
        final(header).inner.primary_slot == old(header).inner.primary_slot,
{
    header.select_primary_slot()
}

pub fn selected_observation_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(),
    ensures result.is_ok() == old(header).selection_success(),
        result.is_ok() ==> old(header).slot_admissible(final(header).inner.primary_slot),
        result.is_ok() ==> (result->Ok_0
            == (final(header).inner.primary_slot == old(header).inner.primary_slot)),
        DatabaseHeader::frame(old(header).inner, final(header).inner),
        final(header).primary_corrupted == old(header).primary_corrupted,
        final(header).secondary_corrupted == old(header).secondary_corrupted,
{
    header.select_primary_slot()
}
}
