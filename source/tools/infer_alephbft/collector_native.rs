// Appended inside the actual pinned production collector module by the runner.
// The deterministic Unit adapter supplies exactly the fields read by add_unit.
#[cfg(test)]
mod infer_parent_probe {
    use super::*;
    use crate::units::{ControlHash, UnitCoord};
    use aleph_bft_mock::Hasher64;
    use std::collections::{HashSet, VecDeque};

    #[derive(Clone, Debug)]
    struct Input { creator: usize, round: u16, variant: u8 }
    impl Unit for Input {
        type Hasher = Hasher64;
        fn hash(&self) -> [u8; 8] {
            (((self.creator as u64) << 32) | ((self.round as u64) << 8) | self.variant as u64).to_le_bytes()
        }
        fn coord(&self) -> UnitCoord { UnitCoord::new(self.round, NodeIndex(self.creator)) }
        fn session_id(&self) -> crate::SessionId { 0 }
        fn control_hash(&self) -> &ControlHash<Hasher64> {
            panic!("collector must not consult control_hash")
        }
    }

    fn check(state: &UnitsCollector<Hasher64>, n: usize, trace: &[String]) {
        let slots: Vec<_> = state.candidates.iter().collect();
        let direct = slots.iter().filter(|(_, (_, r))| u32::from(*r) + 1 == u32::from(state.for_round)).count();
        assert_eq!(state.direct_parents.0, direct, "accounting: {trace:?}");
        assert!(slots.iter().all(|(_, (_, r))| *r < state.for_round), "future slot: {trace:?}");
        let threshold = ((n as u128 * 2) / 3 + 1) as usize;
        for creator in 0..n {
            let own = state.candidates.get(NodeIndex(creator))
                .map_or(false, |(_, r)| u32::from(*r) + 1 == u32::from(state.for_round));
            let selected = state.prospective_parents(NodeIndex(creator));
            assert_eq!(selected.is_ok(), direct >= threshold && own, "selection creator={creator}: {trace:?}");
            if let Ok(parents) = selected {
                assert_eq!(parents, &state.candidates, "selection contents: {trace:?}");
            }
        }
    }

    #[test]
    fn bounded_actual_api_sequences() {
        let mut transitions = 0usize;
        let mut states = 0usize;
        for n in 1..=4 {
            let initial = UnitsCollector::<Hasher64>::new_initial(NodeCount(n));
            let mut queue = VecDeque::from([(initial, Vec::<String>::new())]);
            let mut seen = HashSet::new();
            while let Some((state, trace)) = queue.pop_front() {
                let key = format!("{}:{:?}:{:?}", state.for_round, state.direct_parents, state.candidates);
                if !seen.insert(key) { continue; }
                states += 1;
                check(&state, n, &trace);
                if trace.len() == 4 { continue; }
                for creator in 0..n {
                    for round in 0..=3 {
                        for variant in 0..=1 {
                            let input = Input { creator, round, variant };
                            let mut next = state.clone();
                            let mut next_trace = trace.clone(); next_trace.push(format!("add({input:?})"));
                            next.add_unit(&input);
                            // Independent frame/maximal-slot check: every other slot
                            // is identical; eligible input keeps the first maximum.
                            for slot in 0..n {
                                let before = state.candidates.get(NodeIndex(slot));
                                let after = next.candidates.get(NodeIndex(slot));
                                let eligible = slot == creator && round < state.for_round;
                                let replace = eligible && before.map_or(true, |(_, old)| round > *old);
                                let expected = (input.hash(), round);
                                assert_eq!(after, if replace { Some(&expected) } else { before }, "slot={slot}: {next_trace:?}");
                            }
                            transitions += 1;
                            check(&next, n, &next_trace);
                            queue.push_back((next, next_trace));
                        }
                    }
                }
                if state.for_round < 4 {
                    let next = UnitsCollector::from_previous(&state);
                    let mut next_trace = trace.clone(); next_trace.push("advance".into());
                    assert_eq!(next.for_round, state.for_round + 1);
                    assert_eq!(next.candidates, state.candidates);
                    transitions += 1;
                    check(&next, n, &next_trace);
                    queue.push_back((next, next_trace));
                }
            }
        }
        assert!(states > 100 && transitions > 1000);
        println!("INFER_PARENT_EXPLORE states={states} transitions={transitions} depth=4 committees=1..4 rounds=0..3 variants=2");
    }

    #[test]
    fn original_threshold_has_a_concrete_overflow() {
        let n = usize::MAX / 2 + 1;
        let observed = std::panic::catch_unwind(|| NodeCount(n).consensus_threshold());
        assert!(observed.is_err(), "original n*2 must overflow with debug arithmetic");
        println!("INFER_PARENT_COUNTEREXAMPLE threshold n={n}");
    }

    #[test]
    fn original_round_advance_has_a_concrete_overflow() {
        let mut state = UnitsCollector::<Hasher64>::new_initial(NodeCount(1));
        for _ in 1..u16::MAX { state = UnitsCollector::from_previous(&state); }
        assert_eq!(state.for_round, u16::MAX);
        let observed = std::panic::catch_unwind(|| UnitsCollector::from_previous(&state));
        assert!(observed.is_err(), "round advancement must overflow at u16::MAX");
        println!("INFER_PARENT_COUNTEREXAMPLE advance round={} reachable_advances={}", state.for_round, u16::MAX - 1);
    }

    #[test]
    fn corrected_threshold_matches_boundary_oracle() {
        for n in [0, 1, 2, 3, 4, 6, 7, 8, usize::MAX / 2, usize::MAX / 2 + 1, usize::MAX - 1, usize::MAX] {
            let expected = ((n as u128 * 2) / 3 + 1) as usize;
            assert_eq!(NodeCount(n).consensus_threshold().0, expected);
        }
    }
}
