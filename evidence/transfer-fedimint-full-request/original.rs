use bitcoin::hashes::sha256;
pub const NO_INCOMING_CIRCUIT: (u64, u64) = (0, 0);
pub struct InterceptPaymentRequest {
    pub payment_hash: sha256::Hash,
    /// The amount the HTLC claims to deliver. On the LND forward-intercept path
    /// this is the sender-written onion `amt_to_forward`, so it must never be
    /// trusted for funding decisions. On the HOLD-invoice and LDK paths it is
    /// the real received amount.
    pub amount_msat: u64,
    /// The amount actually locked in the incoming HTLC -- the real value the
    /// gateway receives on settlement. Funding and fee checks must use this.
    pub incoming_amount_msat: u64,
    /// Block height at which the HTLC can no longer be claimed; `0` if unknown.
    /// On the LND forward-intercept path this is the HTLC's raw incoming
    /// expiry instead, which the sender chooses and which LND fails back
    /// `DefaultFinalCltvRejectDelta` blocks before (19 since LND v0.21, 13
    /// before).
    pub expiry: u32,
    pub incoming_chan_id: u64,
    pub short_channel_id: Option<u64>,
    pub htlc_id: u64,
}
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
