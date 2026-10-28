// Explicit checked collection boundary while native iterator composition grows.
// Query meanings and the state constructor are supplied ordinary contracts.
verus! {
impl NodeMap<u16> {
    pub closed spec fn rounds_in_range(&self, lo: u16, hi: u16) -> usize {
        self.count_range_value(lo, hi)
    }
    pub fn rounds_in_range_checked(&self, lo: u16, hi: u16) -> (result: usize)
        ensures result == self.rounds_in_range(lo, hi),
    { self.count_range(lo, hi) }

    pub closed spec fn contains_after(&self, round: u16) -> bool {
        self.after_value(round)
    }
    pub fn contains_after_checked(&self, round: u16) -> (result: bool)
        ensures result == self.contains_after(round),
    { self.has_after(round) }
}

// NATIVE_ERROR_ENUM

impl<H: Hasher> ControlHash<H> {
    // NATIVE_VALIDATION_METHODS

    pub fn supplied_state(parents: NodeMap<Round>, combined_hash: H::Hash) -> (result: Self)
        ensures result@ == (parents@, combined_hash),
    { Self { parents, combined_hash } }

    pub closed spec fn eligible(&self, coord: UnitCoord) -> bool {
        &&& coord.parts().1 > 0
        &&& coord.parts().0.0 < self@.0.len()
        &&& self@.0[coord.parts().0.0 as int] == Some((coord.parts().1 - 1) as u16)
        &&& self.parents.rounds_in_range((coord.parts().1 - 1) as u16, (coord.parts().1 - 1) as u16)
            >= (2 * self@.0.len()) / 3 + 1
        &&& !self.parents.contains_after((coord.parts().1 - 1) as u16)
    }

    pub closed spec fn expected_non_initial(&self, coord: UnitCoord) -> Result<(), Error<H>> {
            if self@.0[coord.parts().0.0 as int].is_none() {
                Err::<(), Error<H>>(Error::NotDescendantOfPreviousUnit(coord.parts().0))
            } else if self@.0[coord.parts().0.0 as int].unwrap() != coord.parts().1 as int - 1 {
                Err::<(), Error<H>>(Error::DescendantOfPreviousUnitHasWrongRound(
                    self@.0[coord.parts().0.0 as int].unwrap()))
            } else if self.parents.rounds_in_range((coord.parts().1 - 1) as u16, (coord.parts().1 - 1) as u16)
                < (2 * self@.0.len()) / 3 + 1 {
                Err::<(), Error<H>>(Error::NotEnoughParentsForRound((coord.parts().1 - 1) as u16))
            } else if self.parents.contains_after((coord.parts().1 - 1) as u16) {
                Err::<(), Error<H>>(Error::ParentsHigherThanRound((coord.parts().1 - 1) as u16))
            } else { Ok(()) }
    }

    pub fn check_non_initial(&self, coord: UnitCoord) -> (result: Result<(), Error<H>>)
        requires coord.parts().1 > 0, coord.parts().0.0 < self@.0.len(),
        ensures
            result.is_ok() == self.eligible(coord),
            result == self.expected_non_initial(coord),
            self@.0[coord.parts().0.0 as int].is_none() ==>
                result == Err::<(), Error<H>>(Error::NotDescendantOfPreviousUnit(coord.parts().0)),
            self@.0[coord.parts().0.0 as int].is_some()
                && self@.0[coord.parts().0.0 as int].unwrap() != coord.parts().1 as int - 1 ==>
                result == Err::<(), Error<H>>(Error::DescendantOfPreviousUnitHasWrongRound(
                    self@.0[coord.parts().0.0 as int].unwrap())),
    { self.validate_non_initial_round(coord) }

    pub fn check_query_errors(&self, round: Round)
        requires round > 0,
    {
        let enough = self.previous_round_have_enough_parents(round);
        assert(enough == if self.parents.rounds_in_range((round - 1) as u16, (round - 1) as u16)
            < (2 * self@.0.len()) / 3 + 1 {
                Err::<(), Error<H>>(Error::NotEnoughParentsForRound((round - 1) as u16))
            } else { Ok(()) });
        let future = self.check_if_parents_greater_than_previous_round(round);
        assert(future == if self.parents.contains_after((round - 1) as u16) {
            Err::<(), Error<H>>(Error::ParentsHigherThanRound((round - 1) as u16))
        } else { Ok(()) });
    }
}
}
