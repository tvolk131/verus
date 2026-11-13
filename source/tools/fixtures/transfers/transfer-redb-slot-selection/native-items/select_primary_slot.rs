    fn select_primary_slot(&mut self) -> Result<bool> {
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
