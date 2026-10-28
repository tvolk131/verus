// Shared executable corpus: adapted proof target and untouched native methods.
fn validation_code(result: Result<(), Error<Hasher64>>) -> (u8, usize, u16) {
    match result {
        Ok(()) => (0, 0, 0),
        Err(Error::NotDescendantOfPreviousUnit(index)) => (1, index.0, 0),
        Err(Error::DescendantOfPreviousUnitHasWrongRound(round)) => (2, 0, round),
        Err(Error::NotEnoughParentsForRound(round)) => (3, 0, round),
        Err(Error::ParentsHigherThanRound(round)) => (4, 0, round),
        _ => panic!("unexpected initial-round error"),
    }
}

fn validation_corpus() {
    let mut cases = 0;
    for size in 1..7 {
        for mask in 0..(1usize << size) {
            for pattern in 0..3 {
                let mut parents = NodeMap::with_size(NodeCount(size));
                for index in 0..size {
                    if mask & (1 << index) != 0 {
                        let round = match pattern {
                            0 => 2,
                            1 => (index % 5) as u16,
                            _ => u16::MAX - 1,
                        };
                        parents.insert(NodeIndex(index), round);
                    }
                }
                let control = ControlHash::<Hasher64> { parents, combined_hash: [0; 8] };
                for round in [1, 3, 5, u16::MAX] {
                    let previous = round - 1;
                    let mut count = 0;
                    let mut future = false;
                    for index in 0..size {
                        if let Some(value) = control.parents.get(NodeIndex(index)) {
                            count += usize::from(*value == previous);
                            future |= *value > previous;
                        }
                    }
                    for creator in 0..size {
                        let expected = match control.parents.get(NodeIndex(creator)) {
                            None => (1, creator, 0),
                            Some(value) if *value != previous => (2, 0, *value),
                            _ if count < 2 * size / 3 + 1 => (3, 0, previous),
                            _ if future => (4, 0, previous),
                            _ => (0, 0, 0),
                        };
                        let result = control.validate_non_initial_round(
                            UnitCoord::new(round, NodeIndex(creator)));
                        assert_eq!(validation_code(result), expected);
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 7704);
    // A runtime assertion is part of the original method's failure behavior.
    let empty = ControlHash::<Hasher64> {
        parents: NodeMap::with_size(NodeCount(0)), combined_hash: [0; 8],
    };
    assert!(std::panic::catch_unwind(||
        empty.validate_non_initial_round(UnitCoord::new(0, NodeIndex(0)))).is_err());
    println!("INFER_VALIDATION_CASES={cases};ZERO_ROUND_PANIC=1");
}
