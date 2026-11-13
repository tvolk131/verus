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
