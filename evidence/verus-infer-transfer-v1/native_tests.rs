extern crate unsigned_varint;

#[path = "original-selected.rs"]
mod selected;

fn check(number: u64, initial: [u8; 10]) {
    let mut upstream_buffer = initial;
    let mut selected_buffer = initial;
    let actual = unsigned_varint::encode::u64(number, &mut upstream_buffer).to_vec();
    let selected_bytes = selected::u64(number, &mut selected_buffer).to_vec();
    let mut quotient = number as u128;
    let mut expected = Vec::new();
    loop {
        let payload = (quotient % 128) as u8;
        quotient /= 128;
        expected.push(payload + if quotient == 0 { 0 } else { 128 });
        if quotient == 0 {
            break;
        }
    }
    assert_eq!(actual, expected, "number={number}, initial={initial:?}");
    assert_eq!(selected_bytes, actual);
    assert_eq!(selected_buffer, upstream_buffer);
    assert_eq!(&upstream_buffer[actual.len()..], &initial[actual.len()..]);
    assert!((1..=10).contains(&actual.len()));
    let mut reconstructed = 0u128;
    let mut place = 1u128;
    for (index, byte) in actual.iter().enumerate() {
        reconstructed += (byte % 128) as u128 * place;
        assert_eq!(byte & 128 != 0, index + 1 < actual.len());
        place *= 128;
    }
    assert_eq!(reconstructed, number as u128);
    assert!(actual.len() == 1 || actual[actual.len() - 1] != 0);
}

#[test]
fn exhaustive_low_16_bits_with_nonzero_buffers() {
    for number in 0..=u16::MAX {
        check(number as u64, [0xa5; 10]);
    }
}

#[test]
fn all_radix_boundaries_and_buffer_byte_patterns() {
    let mut values = vec![0, 1, u64::MAX - 1, u64::MAX];
    for exponent in 1..=9 {
        let boundary = 1u64 << (7 * exponent);
        values.extend([boundary - 1, boundary, boundary + 1]);
    }
    for number in values {
        for byte in 0..=255 {
            check(number, [byte; 10]);
        }
        check(number, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}

#[test]
fn deterministic_full_width_samples_and_arbitrary_buffers() {
    let mut state = 0x4d595df4d0f33173u64;
    for _ in 0..100_000 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let number = state;
        let mut initial = [0; 10];
        for byte in &mut initial {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (state >> 32) as u8;
        }
        check(number, initial);
    }
}
