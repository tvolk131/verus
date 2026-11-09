// Native execution is separate from verifier 32/64-bit usize model checks.
fn data_index_of(bit: u32) -> (usize, usize) {
    ((bit as usize) / 64, (bit as usize) % 64)
}
fn main() {
    for bit in (0..=65535).chain([u32::MAX - 1, u32::MAX]) {
        let (index, offset) = data_index_of(bit);
        assert_eq!((index as u64, offset as u64), (u64::from(bit) / 64, u64::from(bit) % 64));
        assert_eq!(index as u64 * 64 + offset as u64, u64::from(bit));
    }
    println!("native usize={} bits; 65538 cases passed", usize::BITS);
}
