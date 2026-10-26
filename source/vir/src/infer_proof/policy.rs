//! Small acceptance kernel, independent of compiler IR, serialization and I/O.
//! This production file can also be compiled directly with `rustc --test`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Obligation {
    Soundness,
    Maximality,
    Exactness,
}

impl Obligation {
    fn index(self) -> usize {
        match self {
            Self::Soundness => 0,
            Self::Maximality => 1,
            Self::Exactness => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Valid,
    Invalid,
    Incomplete,
    Skipped,
}

/// No required subquery may be skipped or repaired by a later successful one.
#[derive(Clone, Debug, Default)]
pub struct Aggregate {
    checked: bool,
    rejected: bool,
    incomplete: bool,
}

impl Aggregate {
    pub fn observe_query(&mut self, checked: bool, unproved: bool, timed_out: bool) {
        self.observe(if timed_out {
            Outcome::Incomplete
        } else if unproved {
            Outcome::Invalid
        } else if checked {
            Outcome::Valid
        } else {
            Outcome::Skipped
        });
    }

    pub fn observe(&mut self, outcome: Outcome) {
        self.checked = true;
        self.rejected |= outcome != Outcome::Valid;
        self.incomplete |= outcome == Outcome::Incomplete;
    }

    pub fn outcome(&self) -> Outcome {
        if !self.checked {
            Outcome::Skipped
        } else if self.rejected {
            if self.incomplete { Outcome::Incomplete } else { Outcome::Invalid }
        } else {
            Outcome::Valid
        }
    }
}

pub fn accepts(allow_approx: bool, passed: [bool; 3]) -> bool {
    passed[0] && (allow_approx || (passed[1] && passed[2]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    WrongCandidate,
    Duplicate,
}

/// A rejected record permanently invalidates this evidence, even if ignored by
/// an adapter. Evidence cannot be deserialized or constructed with passed flags.
#[derive(Clone, Debug)]
pub struct Evidence<Id> {
    id: Id,
    outcomes: [Option<Outcome>; 3],
    poisoned: bool,
}

impl<Id: PartialEq> Evidence<Id> {
    pub fn new(id: Id) -> Self {
        Self { id, outcomes: [None; 3], poisoned: false }
    }

    pub fn record(
        &mut self,
        id: &Id,
        obligation: Obligation,
        outcome: Outcome,
    ) -> Result<(), RecordError> {
        if id != &self.id {
            self.poisoned = true;
            return Err(RecordError::WrongCandidate);
        }
        let slot = &mut self.outcomes[obligation.index()];
        if slot.is_some() {
            self.poisoned = true;
            return Err(RecordError::Duplicate);
        }
        *slot = Some(outcome);
        Ok(())
    }

    pub fn passed(&self) -> [bool; 3] {
        self.outcomes.map(|o| !self.poisoned && o == Some(Outcome::Valid))
    }

    /// Read-only observations; poisoned evidence cannot report successful checks.
    pub fn outcomes(&self) -> Option<[Option<Outcome>; 3]> {
        (!self.poisoned).then_some(self.outcomes)
    }

    pub fn accepts(&self, allow_approx: bool) -> bool {
        accepts(allow_approx, self.passed())
    }

    pub fn is_empty(&self) -> bool {
        self.outcomes == [None; 3] && !self.poisoned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [Obligation; 3] =
        [Obligation::Soundness, Obligation::Maximality, Obligation::Exactness];
    const OUTCOMES: [Outcome; 4] =
        [Outcome::Valid, Outcome::Invalid, Outcome::Incomplete, Outcome::Skipped];

    #[test]
    fn every_mode_outcome_and_order_obeys_the_protocol() {
        // Independent oracle: acceptance requires one successful ordinary body
        // check and, in exact mode, both successful precision checks.
        for a in 0..5 {
            for b in 0..5 {
                for c in 0..5 {
                    for order in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]]
                    {
                        let outcomes = [a, b, c];
                        let mut e = Evidence::new("candidate");
                        for i in order {
                            if outcomes[i] < 4 {
                                e.record(&"candidate", KINDS[i], OUTCOMES[outcomes[i]]).unwrap();
                            }
                        }
                        assert_eq!(e.accepts(true), a == 0);
                        assert_eq!(e.accepts(false), a == 0 && b == 0 && c == 0);
                    }
                }
            }
        }
    }

    #[test]
    fn duplicate_and_mismatched_records_poison_even_previously_accepted_evidence() {
        for kind in KINDS {
            for outcome in OUTCOMES {
                for wrong_candidate in [false, true] {
                    let mut e = Evidence::new(("function", "body-and-contract", "dependencies"));
                    for kind in KINDS {
                        e.record(&e.id.clone(), kind, Outcome::Valid).unwrap();
                    }
                    assert!(e.accepts(false));
                    let id = if wrong_candidate {
                        ("other-function", "other-contract", "other-dependencies")
                    } else {
                        e.id
                    };
                    assert!(e.record(&id, kind, outcome).is_err());
                    assert!(!e.accepts(false));
                    assert!(!e.accepts(true));
                }
            }
        }
    }

    #[test]
    fn mismatched_identity_cannot_supply_a_missing_receipt() {
        let expected = ("function", "body-and-contract", "dependencies");
        for wrong in [
            ("other-function", "body-and-contract", "dependencies"),
            ("function", "other-contract", "dependencies"),
            ("function", "body-and-contract", "other-dependencies"),
        ] {
            let mut e = Evidence::new(expected);
            assert_eq!(
                e.record(&wrong, Obligation::Soundness, Outcome::Valid),
                Err(RecordError::WrongCandidate)
            );
            for kind in KINDS {
                e.record(&expected, kind, Outcome::Valid).unwrap();
            }
            assert!(!e.accepts(true));
            assert!(!e.accepts(false));
        }
    }

    #[test]
    fn solver_result_flags_cannot_hide_a_failure_timeout_or_skip() {
        for checked in [false, true] {
            for unproved in [false, true] {
                for timed_out in [false, true] {
                    for before in [false, true] {
                        let mut a = Aggregate::default();
                        if before {
                            a.observe(Outcome::Valid);
                        }
                        a.observe_query(checked, unproved, timed_out);
                        a.observe(Outcome::Valid);
                        assert_eq!(
                            a.outcome() == Outcome::Valid,
                            matches!((checked, unproved, timed_out), (true, false, false))
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn no_required_subquery_can_be_hidden_by_success_or_order() {
        assert_ne!(Aggregate::default().outcome(), Outcome::Valid);
        for length in 1..=5u32 {
            for mut pattern in 0..4usize.pow(length) {
                let mut a = Aggregate::default();
                let mut all_valid = true;
                for _ in 0..length {
                    let outcome = OUTCOMES[pattern % 4];
                    pattern /= 4;
                    all_valid &= outcome == Outcome::Valid;
                    a.observe(outcome);
                }
                assert_eq!(a.outcome() == Outcome::Valid, all_valid);
            }
        }
    }

    #[test]
    fn resource_limit_remains_distinguishable_without_becoming_success() {
        for before in OUTCOMES {
            for after in OUTCOMES {
                let mut aggregate = Aggregate::default();
                aggregate.observe(before);
                aggregate.observe(Outcome::Incomplete);
                aggregate.observe(after);
                assert_eq!(aggregate.outcome(), Outcome::Incomplete);
            }
        }
    }
}
