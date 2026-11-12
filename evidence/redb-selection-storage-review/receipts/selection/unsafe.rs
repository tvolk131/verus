use vstd::prelude::*;
#[verus_verify(infer_derive(PartialEq,PartialOrd,Ord))]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TransactionId(pub u64);
verus! {

// Opaque nominal identities only. No method specifications, views or field assumptions.
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);

const TRANSACTION_SIZE: usize = 128;
type Checksum = u128;
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PageNumber {
    pub region: u32,
    pub page_index: u32,
    pub page_order: u8,
}

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

use std::io;
use core::panic;
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
pub struct UnrepairedDatabaseHeader {
    pub inner: DatabaseHeader,
    pub primary_corrupted: bool,
    pub secondary_corrupted: bool,
}
impl UnrepairedDatabaseHeader {
    pub fn select_primary_slot(&mut self) -> Result<bool>
requires true,
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
    pub fn finalize_transaction_slots(self) -> (r: Result<DatabaseHeader>) 
ensures r.is_Err() == self.primary_corrupted,
    match r {
        Ok(header) => header == self.inner,
        Err(StorageError::Corrupted(msg)) => msg@ == "Primary commit slot is corrupted"@,
        _ => false,
    },
{
        if self.primary_corrupted {
            return Err(StorageError::Corrupted(
                "Primary commit slot is corrupted".to_string(),
            ));
        }
        Ok(self.inner)
    }
}


pub open spec fn selection_errors(h: UnrepairedDatabaseHeader) -> bool {
    h.primary_corrupted && (h.inner.two_phase_commit || h.secondary_corrupted)
}
pub open spec fn selection_switches(h: UnrepairedDatabaseHeader) -> bool {
    !h.inner.two_phase_commit && !h.secondary_corrupted &&
        (h.primary_corrupted ||
        h.inner.transaction_slots[(h.inner.primary_slot ^ 1) as int].transaction_id.0 >
            h.inner.transaction_slots[h.inner.primary_slot as int].transaction_id.0)
}

}
