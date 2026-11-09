use vstd::prelude::*;
use rootcodec94::*;
use vstd::std_specs::clone::CloneIdentity;

verus! {

broadcast use {
    vstd::std_specs::clone::lemma_cloned_identity,
    vstd::array::group_array_axioms,
    vstd::seq::axiom_seq_ext_equal,
};

pub const TRANSACTION_SIZE: usize = 128;
pub const FILE_FORMAT_VERSION3: u8 = 3;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TransactionId(pub u64);

impl CloneIdentity for TransactionId {
    proof fn lemma_clone_identity(original: Self, cloned_id: Self)
        ensures original == cloned_id,
    {
    }
}

impl TransactionId {
    pub fn new(value: u64) -> (result: TransactionId)
        ensures result.0 == value,
    {
        Self(value)
    }

    pub fn raw_id(self) -> (result: u64)
        ensures result == self.0,
    {
        self.0
    }

    pub fn next(self) -> (result: TransactionId)
        requires self.0 < u64::MAX,
        ensures result.0 == self.0 + 1,
    {
        TransactionId(self.0 + 1)
    }

    pub fn increment(&mut self) -> (result: TransactionId)
        requires old(self).0 < u64::MAX,
        ensures final(self).0 == old(self).0 + 1, result == *final(self),
    {
        let next = self.next();
        *self = next;
        next
    }
}

#[derive(Clone)]
pub struct TransactionHeader {
    pub version: u8,
    pub user_root: Option<BtreeHeader>,
    pub system_root: Option<BtreeHeader>,
    pub transaction_id: TransactionId,
    pub corrupt_bytes: Option<[u8; TRANSACTION_SIZE]>,
}

impl TransactionHeader {
    pub open spec fn __verus_clone_relation(&self, result: Self) -> bool {
        result == *self
    }

    pub fn new(transaction_id: TransactionId) -> (result: Self)
        ensures result.version == FILE_FORMAT_VERSION3, result.user_root.is_none(),
            result.system_root.is_none(), result.transaction_id == transaction_id,
            result.corrupt_bytes.is_none(),
    {
        Self {
            version: FILE_FORMAT_VERSION3,
            user_root: None,
            system_root: None,
            transaction_id,
            corrupt_bytes: None,
        }
    }
}

impl CloneIdentity for TransactionHeader {
    proof fn lemma_clone_identity(original: Self, cloned_header: Self)
        ensures original == cloned_header,
    {
    }
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

pub open spec fn header_metadata(header: DatabaseHeader) -> (usize, bool, bool, bool, u32, u32, u32, u32, u32) {
    (header.primary_slot, header.recovery_required, header.two_phase_commit, header.primary_unflushed,
        header.page_size, header.region_header_pages, header.region_max_data_pages,
        header.full_regions, header.trailing_partial_region_pages)
}

pub proof fn other_slot_bounds(primary: usize)
    requires primary < 2,
    ensures primary ^ 1 < 2, primary ^ 1 != primary, (primary ^ 1) ^ 1 == primary,
{
    assert(primary ^ 1 < 2 && primary ^ 1 != primary && (primary ^ 1) ^ 1 == primary)
        by (bit_vector) requires primary < 2;
}

impl DatabaseHeader {
    pub open spec fn __verus_clone_relation(&self, result: Self) -> bool {
        header_metadata(result) == header_metadata(*self)
            && result.transaction_slots@[0] == self.transaction_slots@[0]
            && result.transaction_slots@[1] == self.transaction_slots@[1]
    }

    pub fn primary_slot(&self) -> (result: &TransactionHeader)
        requires self.primary_slot < 2,
        ensures *result == self.transaction_slots@[self.primary_slot as int],
    {
        &self.transaction_slots[self.primary_slot]
    }

    pub fn secondary_slot(&self) -> (result: &TransactionHeader)
        requires self.primary_slot < 2,
        ensures *result == self.transaction_slots@[(self.primary_slot ^ 1) as int],
    {
        proof { other_slot_bounds(self.primary_slot); }
        &self.transaction_slots[self.primary_slot ^ 1]
    }

    pub fn published_slot(&self) -> (result: &TransactionHeader)
        requires self.primary_slot < 2,
        ensures *result == self.transaction_slots@[
            (if self.primary_unflushed { self.primary_slot ^ 1 } else { self.primary_slot }) as int],
    {
        if self.primary_unflushed {
            self.secondary_slot()
        } else {
            self.primary_slot()
        }
    }

    pub fn write_secondary_slot(
        &mut self,
        transaction_id: TransactionId,
        user_root: Option<BtreeHeader>,
        system_root: Option<BtreeHeader>,
    )
        requires old(self).primary_slot < 2,
        ensures header_metadata(*final(self)) == header_metadata(*old(self)),
            final(self).transaction_slots@[final(self).primary_slot as int]
                == old(self).transaction_slots@[old(self).primary_slot as int],
            final(self).transaction_slots@[(final(self).primary_slot ^ 1) as int] == (TransactionHeader {
                version: old(self).transaction_slots@[(old(self).primary_slot ^ 1) as int].version,
                user_root, system_root, transaction_id, corrupt_bytes: None }),
    {
        proof { other_slot_bounds(self.primary_slot); }
        let slot = &mut self.transaction_slots[self.primary_slot ^ 1];
        slot.transaction_id = transaction_id;
        slot.user_root = user_root;
        slot.system_root = system_root;
        slot.corrupt_bytes = None;
    }

    pub fn swap_primary_slot(&mut self)
        requires old(self).primary_slot < 2,
        ensures final(self).primary_slot == old(self).primary_slot ^ 1, final(self).primary_slot < 2,
            final(self).recovery_required == old(self).recovery_required,
            final(self).two_phase_commit == old(self).two_phase_commit,
            final(self).primary_unflushed == old(self).primary_unflushed,
            final(self).page_size == old(self).page_size,
            final(self).region_header_pages == old(self).region_header_pages,
            final(self).region_max_data_pages == old(self).region_max_data_pages,
            final(self).full_regions == old(self).full_regions,
            final(self).trailing_partial_region_pages == old(self).trailing_partial_region_pages,
            final(self).transaction_slots@ == old(self).transaction_slots@,
    {
        proof { other_slot_bounds(self.primary_slot); }
        self.primary_slot ^= 1;
    }
}

pub fn checked_slot_clone(header: &TransactionHeader) -> (result: TransactionHeader)
    ensures result == *header,
{
    header.clone()
}

pub fn checked_database_header_clone(header: &DatabaseHeader) -> (result: DatabaseHeader)
    ensures result == *header,
{
    let result = header.clone();
    proof {
        assert(result.transaction_slots@ =~= header.transaction_slots@);
        assert(result.transaction_slots =~= header.transaction_slots);
    }
    result
}

}
