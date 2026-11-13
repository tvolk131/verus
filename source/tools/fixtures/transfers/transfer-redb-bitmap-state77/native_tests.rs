#[test]
fn exact_methods_against_independent_division_oracle() {
    let mut observations = 0u64;
    for len in [1u32, 2, 63, 64, 65, 127, 128, 129] {
        let words = ((u64::from(len) + 63) / 64) as usize;
        for spare in [0usize, 2] {
            for pattern in [0u64, 1, u64::MAX, 0xaaaa_aaaa_aaaa_aaaa, 0x1234_5678_9abc_def0] {
                for changed in 0..len {
                    let original = vec![pattern; words + spare];
                    let mut bitmap = U64GroupedBitmap { len, data: original.clone() };
                    let full = bitmap.set(changed);
                    let index = (changed / 64) as usize;
                    let selected = 1u64 << (changed % 64);
                    assert_eq!(full, bitmap.data[index] == u64::MAX);
                    assert_eq!(bitmap.len, len);
                    assert_eq!(bitmap.data.len(), words + spare);
                    for stored in 0..bitmap.data.len() {
                        assert_eq!(bitmap.data[stored], if stored == index {
                            original[stored] | selected
                        } else { original[stored] });
                    }
                    for observed in 0..len {
                        let before = (original[(observed / 64) as usize] / (1u64 << (observed % 64))) % 2 == 1;
                        assert_eq!(bitmap.get(observed), observed == changed || before);
                        observations += 1;
                    }
                    bitmap.data = original.clone();
                    bitmap.clear(changed);
                    assert_eq!(bitmap.len, len);
                    assert_eq!(bitmap.data.len(), words + spare);
                    for stored in 0..bitmap.data.len() {
                        assert_eq!(bitmap.data[stored], if stored == index {
                            original[stored] & !selected
                        } else { original[stored] });
                    }
                    for observed in 0..len {
                        let before = (original[(observed / 64) as usize] / (1u64 << (observed % 64))) % 2 == 1;
                        assert_eq!(bitmap.get(observed), observed != changed && before);
                        observations += 1;
                    }
                }
            }
        }
    }
    assert_eq!(observations, 1_228_980);
    println!("independent live-bit observations={observations}");
}

#[test]
fn return_is_full_stored_word_not_logical_bitmap() {
    let mut bitmap = U64GroupedBitmap { len: 1, data: vec![0] };
    assert!(!bitmap.set(0));
    assert!(bitmap.get(0));
    let mut bitmap = U64GroupedBitmap { len: 65, data: vec![0, u64::MAX] };
    assert!(bitmap.set(64));
    assert!(!bitmap.get(0));
}

#[test]
fn assertions_and_short_storage_fail_in_both_modes() {
    std::panic::set_hook(Box::new(|_| {}));
    for operation in 0..3 {
        for (len, words, bit) in [(0u32, 0usize, 0u32), (1, 1, 1), (1, 1, 64), (65, 1, 64)] {
            let result = std::panic::catch_unwind(|| {
                let mut bitmap = U64GroupedBitmap { len, data: vec![0; words] };
                match operation {
                    0 => { bitmap.get(bit); }
                    1 => { bitmap.set(bit); }
                    _ => bitmap.clear(bit),
                }
            });
            assert!(result.is_err());
        }
    }
}

#[test]
fn shift_helpers_distinguish_native_release_wrapping() {
    for bit in 0u32..=u16::MAX as u32 {
        let (word, offset) = U64GroupedBitmap::data_index_of(bit);
        assert_eq!(word, (bit / 64) as usize);
        assert_eq!(offset, (bit % 64) as usize);
        assert_eq!(U64GroupedBitmap::select_mask(offset), 1u64 << offset);
    }
    let invalid = std::panic::catch_unwind(|| U64GroupedBitmap::select_mask(std::hint::black_box(64)));
    if cfg!(debug_assertions) {
        assert!(invalid.is_err());
    } else {
        assert_eq!(invalid.unwrap(), 1);
    }
}
