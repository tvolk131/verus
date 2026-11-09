fn main() {
    let mut cases = 0u64;
    let mut seed = 0x9e3779b97f4a7c15u64;
    let mut fingerprint = 0u64;
    for len in 1u32..=64 {
        let padding = if len == 64 { 0 } else { u64::MAX << len };
        for p in 0..72 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let word = match p {
                0 => padding, 1 => u64::MAX,
                2..=65 => padding | (1u64 << (p - 2)),
                _ => seed | padding,
            };
            for target in 0..len {
                let mut map = U64GroupedBitmap { len, data: vec![word] };
                let before = (0..len).map(|j| map.get(j)).collect::<Vec<_>>();
                let n = before.iter().filter(|x| **x).count();
                let full = map.set(target);
                let after = (0..len).map(|j| map.get(j)).collect::<Vec<_>>();
                assert_eq!(after.iter().filter(|x| **x).count(), n + usize::from(!before[target as usize]));
                assert!(after[target as usize]);
                for j in 0..len { if j != target { assert_eq!(after[j as usize], before[j as usize]); } }
                assert_eq!(full, after.iter().all(|x| *x));
                assert_eq!(map.len, len);
                assert_eq!(map.data.len(), 1);
                assert_eq!(map.data[0] & padding, padding);
                let saved = map.data[0];
                assert_eq!(map.set(target), full);
                assert_eq!(map.data[0], saved);
                fingerprint = fingerprint.rotate_left(7) ^ map.data[0] ^ u64::from(full);
                cases += 1;
            }
        }
    }
    println!("cases={cases} fingerprint={fingerprint:016x}");
}
