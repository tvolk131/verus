// Supplied application goals. No constructor or validator implementation is replaced.
verus! {

impl<H: Hasher> ControlHash<H> {
    pub closed spec fn projected_rounds(parents: &NodeMap<(H::Hash, Round)>) -> Seq<Option<Round>> {
        Seq::new(
            parents@.len(),
            |i: int|
                match parents@[i] {
                    None => None,
                    Some(value) => Some(value.1),
                },
        )
    }

    pub closed spec fn empty_parent_hash(size: nat) -> H::Hash {
        H::digest(
            encoded_fields::<Seq<Option<(H::Hash, Round)>>>(
                Seq::new(size, |i: int| None::<(H::Hash, Round)>),
            ),
        )
    }

    pub closed spec fn expected_validation(&self, coord: UnitCoord) -> Result<(), Error<H>> {
        if coord.parts().1 == 0 {
            if self.parents_value().len() > 0 {
                Err(Error::RoundZeroWithSomeParents(NodeCount(self.parents_value().len() as usize)))
            } else if self@.1 != Self::empty_parent_hash(self@.0.len()) {
                Err(Error::RoundZeroBadControlHash(self@.1, Self::empty_parent_hash(self@.0.len())))
            } else {
                Ok(())
            }
        } else {
            self.expected_non_initial(coord)
        }
    }

    pub fn check_projected_construction(parents: &NodeMap<(H::Hash, Round)>) -> (result: Self)
        ensures
            result@.0.len() == parents@.len(),
            forall|i: int|
                0 <= i < parents@.len() ==> #[trigger] result@.0[i] == Self::projected_rounds(
                    parents,
                )[i],
            result@.1 == Self::construction_hash(parents),
    {
        Self::new(parents)
    }

    pub fn check_validation(&self, coord: UnitCoord) -> (result: Result<(), Error<H>>)
        requires
            coord.parts().1 == 0 || coord.parts().0.0 < self@.0.len(),
        ensures
            result == self.expected_validation(coord),
    {
        self.validate(coord)
    }

    pub fn check_initial_construction(
        parents: &NodeMap<(H::Hash, Round)>,
        creator: NodeIndex,
    ) -> (result: Result<(), Error<H>>)
        requires
            parents@ == Seq::new(parents@.len(), |i: int| None::<(H::Hash, Round)>),
        ensures
            result.is_ok(),
    {
        let control = Self::new(parents);
        control.validate(UnitCoord::new(0, creator))
    }

    pub fn build_and_validate(parents: &NodeMap<(H::Hash, Round)>, coord: UnitCoord) -> (pair: (
        Self,
        Result<(), Error<H>>,
    ))
        requires
            coord.parts().1 == 0 || coord.parts().0.0 < parents@.len(),
        ensures
            pair.0@.0.len() == parents@.len(),
            forall|i: int|
                0 <= i < parents@.len() ==> #[trigger] pair.0@.0[i] == Self::projected_rounds(
                    parents,
                )[i],
            pair.0@.1 == Self::construction_hash(parents),
            pair.1 == pair.0.expected_validation(coord),
    {
        let control = Self::new(parents);
        let result = control.validate(coord);
        (control, result)
    }
    // ORIGINAL_FULL_VALIDATION

}

} // verus!
