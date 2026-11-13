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
        }
    }
}
