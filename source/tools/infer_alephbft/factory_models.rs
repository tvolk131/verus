// Supplied collection observations. The factory annotation requests an ordinary
// checked finite-view proof; callback equations are proposed by --infer-goals.
// Both executable bodies below are extracted unchanged from the pinned source.
use vstd::std_specs::iter::IteratorSpec;
use vstd::std_specs::iter_adapters::*;
verus! {

impl<T> NodeMap<T> {
    pub open spec fn entries_value<'a>(&'a self) -> Seq<(NodeIndex, &'a T)> {
        vstd::seq_update::sparse_entries(
            self@,
            |i: int| (NodeIndex(i as usize), &self@[i].unwrap()),
        )
    }
}

impl<H: Hasher> ControlHash<H> {
    pub closed spec fn parents_value(&self) -> Seq<UnitCoord> {
        self.parents.entries_value().map_values(
            |entry: (NodeIndex, &Round)| UnitCoord { round: *entry.1, creator: entry.0 },
        )
    }
}

} // verus!
impl<T> NodeMap<T>{
 #[verus_verify(iterator_via_view(self.entries_value()))]
 // ORIGINAL_NODE_FACTORY
}
impl<H:Hasher> ControlHash<H>{
 #[verus_verify(iterator_via_view(self.parents_value()))]
 // ORIGINAL_PARENT_FACTORY
}
verus! {

impl<H: Hasher> ControlHash<H> {
    pub fn check_empty_future_at_zero(&self)
        requires
            self.parents_value().len() == 0,
    {
        let result = self.check_if_parents_greater_than_previous_round(0);
        assert(result.is_ok());
    }
}

} // verus!
