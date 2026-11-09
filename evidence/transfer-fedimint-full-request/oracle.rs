mod original;
#[path = "native_regression.rs"]
mod regression;
use bitcoin::hashes::{sha256, Hash};
fn main() {
    let mut checked = 0u64;
    let ids = [0, 1, 101, u64::MAX];
    let heights = [0, 1, 2, 3, 18, 19, 36, 144, 1000, u32::MAX - 1, u32::MAX];
    // Unread native fields vary across representative payloads in finite tests.
    for payload in 0..4 {
        for incoming_chan_id in ids {
            for htlc_id in ids {
                for expiry in heights {
                    for height in std::iter::once(None).chain(heights.into_iter().map(Some)) {
                        let payment_hash = sha256::Hash::from_byte_array([payload as u8; 32]);
                        let amount_msat = [0, 1, 101, u64::MAX][payload];
                        let incoming_amount_msat = [u64::MAX, 101, 1, 0][payload];
                        let short_channel_id = [None, Some(0), Some(101), Some(u64::MAX)][payload];
                        let original = original::InterceptPaymentRequest { payment_hash,
                            amount_msat, incoming_amount_msat, expiry, incoming_chan_id,
                            short_channel_id, htlc_id };
                        let erased = deadline_full::InterceptPaymentRequest { payment_hash,
                            amount_msat, incoming_amount_msat, expiry, incoming_chan_id,
                            short_channel_id, htlc_id };
                        let expected = if incoming_chan_id != 0 || htlc_id != 0 { 0 }
                            else { height.map_or(0, |h| (expiry as u64).checked_sub(h as u64).unwrap_or(0) as u32) };
                        assert_eq!(original.lnv2_blocks_to_claim_deadline(height), expected);
                        assert_eq!(erased.lnv2_blocks_to_claim_deadline(height), expected);
                        assert_eq!(erased.incoming_circuit(), original.incoming_circuit());
                        checked += 1;
                    }
                }
            }
        }
    }
    let mutant = regression::InterceptPaymentRequest {
        payment_hash: sha256::Hash::from_byte_array([0; 32]), amount_msat: 0,
        incoming_amount_msat: 0, expiry: 1000, incoming_chan_id: 101,
        short_channel_id: None, htlc_id: 0,
    };
    assert_eq!(mutant.lnv2_blocks_to_claim_deadline(Some(990)), 10);
    let fixed = deadline_full::InterceptPaymentRequest {
        payment_hash: mutant.payment_hash, amount_msat: mutant.amount_msat,
        incoming_amount_msat: mutant.incoming_amount_msat, expiry: mutant.expiry,
        incoming_chan_id: mutant.incoming_chan_id, short_channel_id: mutant.short_channel_id,
        htlc_id: mutant.htlc_id,
    };
    assert_eq!(fixed.lnv2_blocks_to_claim_deadline(Some(990)), 0);
    let hold = deadline_full::InterceptPaymentRequest {
        expiry: u32::MAX, incoming_chan_id: 0, htlc_id: 0, ..fixed
    };
    assert_eq!(hold.lnv2_blocks_to_claim_deadline(Some(u32::MAX - 2)), 2);
    assert_eq!(hold.lnv2_blocks_to_claim_deadline(Some(u32::MAX)), 0);
    println!("{{\"native_erasure_comparisons\":{checked},\"guard_deletion_budget\":10,\"fixed_budget\":0,\"max_height_coverage_witness\":2}}");
}
