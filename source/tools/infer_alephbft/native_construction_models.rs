// The codec remains an explicit existing library boundary. This bridge changes
// no collection algorithm and supplies only the encoded-view equation.
verus! {

impl<T> NodeMap<T> {
    fn encode(&self) -> (bytes: Vec<u8>)
        ensures
            bytes@ == encoded_fields::<Seq<Option<T>>>(self@),
    {
        codec_model::encode_with_view(self)
    }

    pub fn using_encoded<R, F: Fn(&[u8]) -> R>(&self, f: F) -> (result: R)
        requires
            forall|bytes: &[u8]|
                bytes@ == encoded_fields::<Seq<Option<T>>>(self@) ==> f.requires((bytes,)),
        ensures
            exists|bytes: &[u8]|
                bytes@ == encoded_fields::<Seq<Option<T>>>(self@) && f.ensures((bytes,), result),
    {
        f(&self.encode())
    }
}

impl<H: Hasher> ControlHash<H> {
    pub closed spec fn construction_hash(parents: &NodeMap<(H::Hash, Round)>) -> H::Hash {
        H::digest(encoded_fields::<Seq<Option<(H::Hash, Round)>>>(parents@))
    }

    pub closed spec fn construction_rounds(parents: &NodeMap<(H::Hash, Round)>) -> Seq<
        Option<Round>,
    > {
        parents.entries_value().fold_left(
            Seq::new(parents@.len(), |i: int| None::<Round>),
            |acc: Seq<Option<Round>>, entry: (NodeIndex, &(H::Hash, Round))|
                acc.update(entry.0.0 as int, Some(entry.1.1)),
        )
    }

    pub fn check_construction(parents: &NodeMap<(H::Hash, Round)>) -> (result: Self)
        ensures
            result@ == (Self::construction_rounds(parents), Self::construction_hash(parents)),
    {
        Self::new(parents)
    }

    pub fn build_and_check(parents: &NodeMap<(H::Hash, Round)>, coord: UnitCoord) -> (pair: (
        Self,
        Result<(), Error<H>>,
    ))
        requires
            coord.parts().1 > 0,
            coord.parts().0.0 < Self::construction_rounds(parents).len(),
        ensures
            pair.0@ == (Self::construction_rounds(parents), Self::construction_hash(parents)),
            pair.1 == pair.0.expected_non_initial(coord),
            pair.1.is_ok() == pair.0.eligible(coord),
    {
        let control = Self::new(parents);
        let result = control.check_non_initial(coord);
        (control, result)
    }
    // ORIGINAL_HASH_HELPER
    // ORIGINAL_CONSTRUCTOR

}

} // verus!
