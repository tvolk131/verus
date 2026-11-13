    pub(super) fn primary_slot(&self) -> &TransactionHeader {
        &self.transaction_slots[self.primary_slot]
    }
