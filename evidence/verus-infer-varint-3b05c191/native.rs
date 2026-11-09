#![feature(proc_macro_hygiene)]
// Finite native supporting oracle, never a replacement for the frozen proof.
#[path = "original-selected.rs"]
mod selected;
#[path = "upstream/src/encode.rs"]
mod upstream;
#[path = "fixtures/adapted-ordinary.rs"]
mod adapted;
#[path = "manual/length-attempt.rs"]
mod authored_length;

fn check(number: u64, initial: [u8; 10]) {
    let mut a = initial;
    let mut b = initial;
    let mut c = initial;
    let mut d = initial;
    let pointer = c.as_ptr();
    let x = upstream::u64(number, &mut a).to_vec();
    let y = selected::u64(number, &mut b).to_vec();
    let z = adapted::u64(number, &mut c);
    assert_eq!(z.as_ptr(), pointer);
    assert_eq!(z, x);
    let authored_pointer = d.as_ptr();
    let authored = authored_length::u64(number, &mut d);
    assert_eq!(authored.as_ptr(), authored_pointer);
    assert_eq!(authored, x);
    assert_eq!(x, y);
    let mut n = number as u128;
    let mut expected = Vec::new();
    loop {
        let payload = (n % 128) as u8;
        n /= 128;
        expected.push(payload + if n == 0 { 0 } else { 128 });
        if n == 0 { break; }
    }
    assert_eq!(x, expected);
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert_eq!(a, d);
    assert_eq!(&c[x.len()..], &initial[x.len()..]);
    assert!((1..=10).contains(&x.len()));
    assert!((number as u128) < 128u128.pow(x.len() as u32));
    assert!(x.len() == 1 || number as u128 >= 128u128.pow(x.len() as u32 - 1));
    for (i, byte) in x.iter().enumerate() {
        assert_eq!(*byte as u128 % 128, number as u128 / 128u128.pow(i as u32) % 128);
        assert_eq!(*byte >= 128, i + 1 < x.len());
    }
}

#[test]
fn low_16_bits() {
    for n in 0..=u16::MAX { check(n as u64, [0xa5; 10]); }
}

#[test]
fn radix_boundaries_with_all_uniform_byte_patterns() {
    let mut values = vec![0, 1, u64::MAX - 1, u64::MAX];
    for k in 1..=9 {
        let boundary = 1u64 << (7 * k);
        values.extend([boundary - 1, boundary, boundary + 1]);
    }
    for n in values {
        for byte in 0..=255 { check(n, [byte; 10]); }
        check(n, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
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
