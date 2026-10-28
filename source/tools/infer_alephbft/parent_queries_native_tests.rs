
    #[test]
    fn infer_parent_query_corpus_matches_indexed_reference() {
        for size in 0..8 {
            for mask in 0..(1usize << size) {
                let mut parents = NodeMap::with_size(NodeCount(size));
                for index in 0..size {
                    if mask & (1 << index) != 0 {
                        parents.insert(NodeIndex(index), ([index as u8; 8], (index % 5) as u16));
                    }
                }
                let control = ControlHash::<Hasher64>::new(&parents);
                let mut occupied = 0;
                for index in 0..size {
                    if let Some((_, round)) = parents.get(NodeIndex(index)) {
                        occupied += 1;
                        assert_eq!(control.parents.get(NodeIndex(index)), Some(round));
                    } else {
                        assert_eq!(control.parents.get(NodeIndex(index)), None);
                    }
                }
                assert_eq!(parents.item_count(), occupied);
                assert_eq!(control.parents().count(), occupied);
                for round in 1..7 {
                    let previous = round - 1;
                    let mut count = 0;
                    let mut future = false;
                    for index in 0..size {
                        if let Some((_, value)) = parents.get(NodeIndex(index)) {
                            count += usize::from(*value == previous);
                            future |= *value > previous;
                        }
                    }
                    assert_eq!(control.previous_round_have_enough_parents(round).is_ok(),
                        count >= 2 * size / 3 + 1);
                    assert_eq!(control.check_if_parents_greater_than_previous_round(round).is_ok(), !future);
                }
            }
        }
    }
