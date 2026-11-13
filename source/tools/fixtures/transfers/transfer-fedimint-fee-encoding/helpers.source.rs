pub fn fee_encoded_expiration(fee_msats: u64) -> u64 {
    u64::MAX - fee_msats
}
pub fn fee_from_expiration(expiration: u64) -> u64 {
    u64::MAX - expiration
}
