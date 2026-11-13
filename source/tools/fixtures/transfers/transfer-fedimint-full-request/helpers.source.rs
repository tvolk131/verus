pub const NO_INCOMING_CIRCUIT: (u64, u64) = (0, 0);

impl InterceptPaymentRequest {
    /// The incoming circuit of an intercepted forward, or `None` when the
    /// payment is held by a HOLD invoice or LDK (see [`NO_INCOMING_CIRCUIT`]).
    pub fn incoming_circuit(&self) -> Option<(u64, u64)> {
        let circuit = (self.incoming_chan_id, self.htlc_id);
        (circuit != NO_INCOMING_CIRCUIT).then_some(circuit)
    }

    /// Blocks left before the gateway can no longer settle this payment, for
    /// deciding whether a fresh LNv2 incoming contract may be funded from it.
    ///
    /// Honest LNv2 payments arrive through an LND HOLD invoice or an LDK
    /// claimable payment, never as an intercepted forward. A forward's expiry
    /// is chosen by the sender and does not account for LND failing it back
    /// early, so it counts as having no time left. Without the current height
    /// the deadline cannot be checked, which counts the same.
    pub fn lnv2_blocks_to_claim_deadline(&self, current_block_height: Option<u32>) -> u32 {
        if self.incoming_circuit().is_some() {
            return 0;
        }

        current_block_height.map_or(0, |height| self.expiry.saturating_sub(height))
    }
}
