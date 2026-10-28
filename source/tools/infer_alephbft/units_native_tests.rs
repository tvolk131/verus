
    // Additional runtime checks of the real parking_lot implementation.
    #[test]
    fn infer_cache_cold_warm_clone_and_contended_read() {
        for unit in random_full_parent_units_up_to(3, NodeCount(4), 43).into_iter().flatten() {
            let expected = unit.using_encoded(Hasher64::hash);
            let cold = unit.clone();
            assert_eq!(cold.encode(), unit.encode());
            assert_eq!(cold.hash(), expected);
            assert_eq!(unit.hash(), expected);
            assert_eq!(*unit.hash.read(), Some(expected));
            let warm = unit.clone();
            assert_eq!(*warm.hash.read(), Some(expected));
            assert_eq!(warm.encode(), unit.encode());
            // Native try_read must fail promptly while a writer holds the lock.
            // The formal verified-lock adaptation uses a blocking read instead.
            let write_guard = unit.hash.write();
            let contended = unit.clone();
            assert!(contended.hash.read().is_none());
            drop(write_guard);
            assert_eq!(contended.encode(), unit.encode());
            assert_eq!(contended.hash(), expected);
        }
    }

    #[test]
    fn infer_hasher64_uses_exactly_the_supplied_bytes() {
        use std::{collections::hash_map::DefaultHasher, hash::Hasher as StdHasher};
        for bytes in [&[][..], &[0][..], &[1,2,3,4,5][..], &[255;64][..]] {
            let mut reference = DefaultHasher::new();
            reference.write(bytes);
            assert_eq!(Hasher64::hash(bytes), reference.finish().to_ne_bytes());
        }
    }
