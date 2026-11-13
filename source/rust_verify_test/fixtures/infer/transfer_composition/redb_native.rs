use vstd::prelude::*;
use std::{io, panic};
verus! {
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExLocation<'a>(std::panic::Location<'a>);
}
#[verus_verify(infer_derive(PartialEq, PartialOrd, Ord))]
#[derive(Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct TransactionId(pub u64);
verus! {
pub type Checksum = u128;
const TRANSACTION_SIZE: usize = 128;
#[derive(Debug)]
#[non_exhaustive]
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
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PageNumber {
    pub region: u32,
    pub page_index: u32,
    pub page_order: u8,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
pub struct UnrepairedDatabaseHeader {
    pub inner: DatabaseHeader,
    pub primary_corrupted: bool,
    pub secondary_corrupted: bool,
}

impl DatabaseHeader {
    pub open spec fn representation_valid(self) -> bool { self.primary_slot < 2 }
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
    pub open spec fn representation_valid(self) -> bool { self.inner.primary_slot < 2 }
    pub open spec fn selector_safe(self) -> bool {
        self.inner.two_phase_commit || self.primary_corrupted || self.inner.primary_slot < 2
    }
    pub open spec fn ids_secondary_newer(self) -> bool {
        self.inner.transaction_slots@[(self.inner.primary_slot ^ 1) as int].transaction_id.0
        > self.inner.transaction_slots@[self.inner.primary_slot as int].transaction_id.0
    }
    pub open spec fn select_secondary(self) -> bool {
        !self.inner.two_phase_commit && !self.secondary_corrupted
        && (self.primary_corrupted || self.ids_secondary_newer())
    }
    pub open spec fn selection_post(before: Self, after: Self, result: Result<bool>) -> bool {
        after.primary_corrupted == before.primary_corrupted
        && after.secondary_corrupted == before.secondary_corrupted
        && DatabaseHeader::frame(before.inner, after.inner)
        && after.inner.primary_slot == if before.select_secondary() {
            before.inner.primary_slot ^ 1
        } else { before.inner.primary_slot }
        && if before.inner.two_phase_commit && before.primary_corrupted {
            match result {
                Err(StorageError::Corrupted(message)) => message@ == "Primary is corrupted despite 2-phase commit"@,
                _ => false,
            }
        } else if !before.inner.two_phase_commit && before.primary_corrupted && before.secondary_corrupted {
            match result {
                Err(StorageError::Corrupted(message)) => message@ == "Both commit slots are corrupted"@,
                _ => false,
            }
        } else { result == Ok(!before.select_secondary()) }
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
        requires (self.primary_slot ^ 1) < 2,
        ensures *result == self.transaction_slots@[(self.primary_slot ^ 1) as int],
    {
        &self.transaction_slots[self.primary_slot ^ 1]
    }
    pub fn swap_primary_slot(&mut self)
        ensures final(self).primary_slot == old(self).primary_slot ^ 1,
            DatabaseHeader::frame(*old(self), *final(self)),
    {
        self.primary_slot ^= 1;
    }
}
impl UnrepairedDatabaseHeader {
    pub fn select_primary_slot(&mut self) -> (result: Result<bool>)
        requires old(self).selector_safe(),
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

use core::mem::size_of;
#[derive(Debug)]
#[non_exhaustive]
pub enum DatabaseError {
    /// The Database is already open. Cannot acquire lock.
    DatabaseAlreadyOpen,
    /// Another process has the database open with a version of redb that implements a different
    /// generation of the multi-process locking protocol
    LockProtocolMismatch,
    /// [`crate::RepairSession::abort`] was called or repair was aborted for another reason (such as the database being read-only).
    RepairAborted,
    /// The database file is in an old file format and must be manually upgraded
    UpgradeRequired(u8),
    /// A transaction is still in-progress
    TransactionInProgress,
    /// Error from underlying storage
    Storage(StorageError),
}
impl vstd::std_specs::convert::FromSpecImpl<StorageError> for DatabaseError {
    open spec fn obeys_from_spec() -> bool { true }
    open spec fn from_spec(error: StorageError) -> Self { Self::Storage(error) }
}
impl From<StorageError> for DatabaseError {
    fn from(err: StorageError) -> (result: DatabaseError)
        ensures result == DatabaseError::Storage(err),
    {
        DatabaseError::Storage(err)
    }
}
impl PageNumber {
    #[verifier::infer]
    pub const fn serialized_size() -> usize {
        8
    }
}
impl BtreeHeader {
    #[verifier::infer]
    pub const fn serialized_size() -> usize {
        PageNumber::serialized_size() + size_of::<Checksum>() + size_of::<u64>()
    }
}
const USER_ROOT_NON_NULL_OFFSET: usize = size_of::<u8>();
const SYSTEM_ROOT_NON_NULL_OFFSET: usize = USER_ROOT_NON_NULL_OFFSET + size_of::<u8>();
const _UNUSED_OFFSET: usize = SYSTEM_ROOT_NON_NULL_OFFSET + size_of::<u8>();
const PADDING: usize = 4;

const USER_ROOT_OFFSET: usize = _UNUSED_OFFSET + size_of::<u8>() + PADDING;
const SYSTEM_ROOT_OFFSET: usize = USER_ROOT_OFFSET + BtreeHeader::serialized_size();
const _UNUSED2_OFFSET: usize = SYSTEM_ROOT_OFFSET + BtreeHeader::serialized_size();
const TRANSACTION_ID_OFFSET: usize = _UNUSED2_OFFSET + BtreeHeader::serialized_size();
const TRANSACTION_LAST_FIELD: usize = TRANSACTION_ID_OFFSET + size_of::<u64>();


#[verifier::infer]
fn get_u64(data: &[u8]) -> u64 {
    u64::from_le_bytes(data[..size_of::<u64>()].try_into().unwrap())
}
impl TransactionId {
#[verifier::infer]
    pub fn new(value: u64) -> TransactionId {
        Self(value)
    }
}

pub open spec fn le64_at(data: Seq<u8>, offset: int) -> u64 {
    (data[offset] as u64) | ((data[offset + 1] as u64) << 8)
    | ((data[offset + 2] as u64) << 16) | ((data[offset + 3] as u64) << 24)
    | ((data[offset + 4] as u64) << 32) | ((data[offset + 5] as u64) << 40)
    | ((data[offset + 6] as u64) << 48) | ((data[offset + 7] as u64) << 56)
}
impl TransactionHeader {
    fn from_id_tail(data: &[u8], version: u8, user_root: Option<BtreeHeader>,
        system_root: Option<BtreeHeader>, corrupted: bool) -> (result: Result<(Self, bool), DatabaseError>)
        requires data.len() >= TRANSACTION_LAST_FIELD,
            corrupted ==> data.len() >= TRANSACTION_SIZE,
        ensures match result {
            Ok((slot, bad)) => bad == corrupted && slot.version == version
                && slot.user_root == user_root && slot.system_root == system_root
                && slot.transaction_id.0 == le64_at(data@, TRANSACTION_ID_OFFSET as int)
                && if corrupted { match slot.corrupt_bytes {
                    Some(bytes) => bytes@ == data@.subrange(0, TRANSACTION_SIZE as int),
                    None => false,
                } } else { slot.corrupt_bytes.is_none() },
            Err(_) => false,
        },
    {
        let transaction_id = TransactionId::new(get_u64(&data[TRANSACTION_ID_OFFSET..]));

        let result = Self {
            version,
            user_root,
            system_root,
            transaction_id,
            corrupt_bytes: if corrupted {
                Some(data[..TRANSACTION_SIZE].try_into().unwrap())
            } else {
                None
            },
        };

        Ok((result, corrupted))
    }
}

impl UnrepairedDatabaseHeader {
    fn assemble_physical_slots(primary_slot: usize, recovery_required: bool, two_phase_commit: bool,
        primary_unflushed: bool, page_size: u32, region_header_pages: u32, region_max_data_pages: u32,
        full_regions: u32, trailing_data_pages: u32, slot0: TransactionHeader, slot0_corrupted: bool,
        slot1: TransactionHeader, slot1_corrupted: bool) -> (result: Result<Self, DatabaseError>)
        ensures result.is_ok(),
            result->Ok_0.inner.primary_slot == primary_slot,
            result->Ok_0.inner.two_phase_commit == two_phase_commit,
            result->Ok_0.inner.recovery_required == recovery_required,
            result->Ok_0.inner.primary_unflushed == primary_unflushed,
            result->Ok_0.inner.page_size == page_size,
            result->Ok_0.inner.region_header_pages == region_header_pages,
            result->Ok_0.inner.region_max_data_pages == region_max_data_pages,
            result->Ok_0.inner.full_regions == full_regions,
            result->Ok_0.inner.trailing_partial_region_pages == trailing_data_pages,
            result->Ok_0.inner.transaction_slots@[0] == slot0,
            result->Ok_0.inner.transaction_slots@[1] == slot1,
            result->Ok_0.primary_corrupted == if primary_slot == 0 { slot0_corrupted } else { slot1_corrupted },
            result->Ok_0.secondary_corrupted == if primary_slot == 0 { slot1_corrupted } else { slot0_corrupted },
    {
        let (primary_corrupted, secondary_corrupted) = if primary_slot == 0 {
            (slot0_corrupted, slot1_corrupted)
        } else {
            (slot1_corrupted, slot0_corrupted)
        };

        Ok(Self {
            inner: DatabaseHeader {
                primary_slot,
                recovery_required,
                two_phase_commit,
                primary_unflushed,
                page_size,
                region_header_pages,
                region_max_data_pages,
                full_regions,
                trailing_partial_region_pages: trailing_data_pages,
                transaction_slots: [slot0, slot1],
            },
            primary_corrupted,
            secondary_corrupted,
        })
    }

    fn select_after_layout_context(mut self, layout_stale: bool) -> (result: Result<(DatabaseHeader, bool)>)
        requires self.selector_safe(),
        ensures match result {
            Ok((after, clean)) =>
                UnrepairedDatabaseHeader::selection_post(self,
                    UnrepairedDatabaseHeader { inner: after, primary_corrupted: self.primary_corrupted,
                        secondary_corrupted: self.secondary_corrupted }, Ok(!self.select_secondary()))
                && clean == (!self.select_secondary() && !layout_stale),
            Err(error) => UnrepairedDatabaseHeader::selection_post(self, self, Err(error)),
        },
    {
        let kept_primary = self.select_primary_slot()?;
        Ok((self.inner, kept_primary && !layout_stale))
    }
}

spec fn decoded_tail_frame(after: TransactionHeader, before: TransactionHeader, data: Seq<u8>, corrupted: bool) -> bool {
    after.version == before.version && after.user_root == before.user_root
    && after.system_root == before.system_root
    && after.transaction_id.0 == le64_at(data, TRANSACTION_ID_OFFSET as int)
    && if corrupted { match after.corrupt_bytes {
        Some(bytes) => bytes@ == data.subrange(0, TRANSACTION_SIZE as int),
        None => false,
    } } else { after.corrupt_bytes.is_none() }
}
pub open spec fn physical_bad(primary: usize, bad0: bool, bad1: bool) -> bool {
    if primary == 0 { bad0 } else { bad1 }
}
spec fn byte_selection_moves(frame: DatabaseHeader, data0: Seq<u8>, data1: Seq<u8>, bad0: bool, bad1: bool) -> bool {
    !frame.two_phase_commit
    && !(if frame.primary_slot == 0 { bad1 } else { bad0 })
    && (physical_bad(frame.primary_slot, bad0, bad1)
        || if frame.primary_slot == 0 {
            le64_at(data1, TRANSACTION_ID_OFFSET as int) > le64_at(data0, TRANSACTION_ID_OFFSET as int)
        } else {
            le64_at(data0, TRANSACTION_ID_OFFSET as int) > le64_at(data1, TRANSACTION_ID_OFFSET as int)
        })
}
fn decoded_id_selection_client(data0: &[u8], data1: &[u8], frame: DatabaseHeader,
    bad0: bool, bad1: bool, layout_stale: bool) -> (result: Result<(DatabaseHeader, bool), DatabaseError>)
    requires data0.len() >= TRANSACTION_LAST_FIELD, data1.len() >= TRANSACTION_LAST_FIELD,
        bad0 ==> data0.len() >= TRANSACTION_SIZE,
        bad1 ==> data1.len() >= TRANSACTION_SIZE,
        frame.two_phase_commit || physical_bad(frame.primary_slot, bad0, bad1) || frame.primary_slot < 2,
    ensures match result {
        Ok((after, clean)) =>
            decoded_tail_frame(after.transaction_slots@[0], frame.transaction_slots@[0], data0@, bad0)
            && decoded_tail_frame(after.transaction_slots@[1], frame.transaction_slots@[1], data1@, bad1)
            && after.recovery_required == frame.recovery_required
            && after.two_phase_commit == frame.two_phase_commit
            && after.primary_unflushed == frame.primary_unflushed
            && after.page_size == frame.page_size
            && after.region_header_pages == frame.region_header_pages
            && after.region_max_data_pages == frame.region_max_data_pages
            && after.full_regions == frame.full_regions
            && after.trailing_partial_region_pages == frame.trailing_partial_region_pages
            && after.primary_slot == if byte_selection_moves(frame, data0@, data1@, bad0, bad1) {
                frame.primary_slot ^ 1
            } else { frame.primary_slot }
            && clean == (!byte_selection_moves(frame, data0@, data1@, bad0, bad1) && !layout_stale),
        Err(DatabaseError::Storage(StorageError::Corrupted(message))) =>
            if frame.two_phase_commit {
                physical_bad(frame.primary_slot, bad0, bad1)
                && message@ == "Primary is corrupted despite 2-phase commit"@
            } else {
                bad0 && bad1 && message@ == "Both commit slots are corrupted"@
            },
        Err(_) => false,
    },
{
    let (slot0, slot0_corrupted) = TransactionHeader::from_id_tail(data0,
        frame.transaction_slots[0].version, frame.transaction_slots[0].user_root,
        frame.transaction_slots[0].system_root, bad0)?;
    let (slot1, slot1_corrupted) = TransactionHeader::from_id_tail(data1,
        frame.transaction_slots[1].version, frame.transaction_slots[1].user_root,
        frame.transaction_slots[1].system_root, bad1)?;
    let unrepaired = UnrepairedDatabaseHeader::assemble_physical_slots(frame.primary_slot,
        frame.recovery_required, frame.two_phase_commit, frame.primary_unflushed, frame.page_size,
        frame.region_header_pages, frame.region_max_data_pages, frame.full_regions,
        frame.trailing_partial_region_pages, slot0, slot0_corrupted, slot1, slot1_corrupted)?;
    let selected = unrepaired.select_after_layout_context(layout_stale)?;
    Ok(selected)
}
}

use std::fmt::{Debug, Formatter};
impl Debug for PageNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "r{}.{}/{}",
            self.region, self.page_index, self.page_order
        )
    }
}
