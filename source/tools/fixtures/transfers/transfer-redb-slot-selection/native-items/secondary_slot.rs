    pub(super) fn secondary_slot(&self) -> &TransactionHeader {
        &self.transaction_slots[self.primary_slot ^ 1]
    }
