// Runs only in untouched native Aleph: this exercises its actual SCALE codec.
fn initial_validation_corpus() {
    let mut cases = 0;
    for size in 0..7 {
        for mask in 0..(1usize << size) {
            for corrupt in [false, true] {
                let mut input = NodeMap::with_size(NodeCount(size));
                for index in 0..size {
                    if mask & (1 << index) != 0 {
                        input.insert(NodeIndex(index), ([index as u8; 8], u16::MAX));
                    }
                }
                let mut control = ControlHash::<Hasher64>::new(&input);
                if corrupt { control.combined_hash[0] ^= 1; }
                let expected_hash = ControlHash::<Hasher64>::create_control_hash(
                    &NodeMap::with_size(NodeCount(size)));
                // Initial validation does not index by the creator.
                match (mask == 0, corrupt, control.validate(UnitCoord::new(0, NodeIndex(usize::MAX)))) {
                    (false, _, Err(Error::RoundZeroWithSomeParents(count))) =>
                        assert_eq!(count.0, mask.count_ones() as usize),
                    (true, true, Err(Error::RoundZeroBadControlHash(actual, expected))) => {
                        assert_eq!(actual, control.combined_hash);
                        assert_eq!(expected, expected_hash);
                    }
                    (true, false, Ok(())) => {},
                    _ => panic!("incorrect initial validation result or error priority"),
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 254);
    println!("NATIVE_INITIAL_VALIDATION_CASES=254");
}
