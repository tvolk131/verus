// Supplied observations and checked native iterator factories. The executable
// chains retain slice::iter/enumerate/filter_map and map; named locals make the
// ghost proof of the lazy prefix and completion relation explicit.
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

    pub fn iter<'a>(&'a self) -> (result: impl Iterator<Item = (NodeIndex, &'a T)> + 'a)
        ensures
            finite_native_view(result, self.entries_value()),
            forall|i: int|
                0 <= i < self.entries_value().len() ==> #[trigger] self.entries_value()[i].0.0
                    < self@.len(),
    {
        let values = self.0.as_slice();
        let indexed = values.iter().enumerate();
        let callback = |(idx, maybe_value): (usize, &'a Option<T>)| -> (result: Option<
            (NodeIndex, &'a T),
        >)
            ensures
                result == match maybe_value {
                    None => None,
                    Some(value) => Some((NodeIndex(idx), value)),
                },
            { Some((NodeIndex(idx), maybe_value.as_ref()?)) };
        let result = indexed.filter_map(callback);
        proof {
            broadcast use {
                vstd::std_specs::iter::group_iter_axioms,
                group_native_adapter_axioms,
                vstd::seq::group_seq_lemmas,
            };

            filter_map_postcondition::<_, (NodeIndex, &'a T), _>(indexed, callback, result);
            let full = Seq::new(
                values@.len(),
                |i: int|
                    match values@[i] {
                        None => None,
                        Some(value) => Some((NodeIndex(i as usize), &value)),
                    },
            );
            vstd::seq_update::lemma_sparse_entries_eq(
                self@,
                |i: int| (NodeIndex(i as usize), &self@[i].unwrap()),
                full,
            );
            assert(self.entries_value() == flatten_options(full));
            assert(filter_map_outputs(result).is_prefix_of(full));
            assert forall|k: int|
                0 <= k < self.entries_value().len() implies #[trigger] self.entries_value()[k].0.0
                < self@.len() by {
                flatten_options_witness(full, k);
                let i = choose|i: int|
                    0 <= i < full.len() && full[i] == Some(flatten_options(full)[k]);
                assert(full[i].unwrap().0.0 == i);
            }

            flatten_options_prefix(filter_map_outputs(result), full);
            assert(result.will_return_none() ==> filter_map_outputs(result) =~= full);
        }
        result
    }
}

impl<H: Hasher> ControlHash<H> {
    pub closed spec fn parents_value(&self) -> Seq<UnitCoord> {
        self.parents.entries_value().map_values(
            |entry: (NodeIndex, &Round)| UnitCoord { round: *entry.1, creator: entry.0 },
        )
    }

    pub fn parents(&self) -> (result: impl Iterator<Item = UnitCoord> + '_)
        ensures
            finite_native_view(result, self.parents_value()),
    {
        let entries = self.parents.iter();
        let callback = |(node_index, &round): (NodeIndex, &Round)| -> (result: UnitCoord)
            ensures
                result.parts() == (node_index, round),
            { UnitCoord::new(round, node_index) };
        let result = entries.map(callback);
        proof {
            broadcast use {
                vstd::std_specs::iter::group_iter_axioms,
                group_native_adapter_axioms,
                vstd::seq::group_seq_lemmas,
            };

            vstd::std_specs::iter::map_postcondition(entries, callback, result);
            assert(vstd::std_specs::iter::map_iter(result) == entries);
            let full = self.parents_value();
            assert(native_count_supported(result));
            assert(result.obeys_prophetic_iter_laws());
            assert(result.decrease().is_some());
            assert(result.decrease().unwrap() <= usize::MAX as nat);
            assert(full.len() <= usize::MAX as nat);
            assert(result.remaining().len() <= full.len());
            assert forall|i: int|
                0 <= i < result.remaining().len() implies #[trigger] result.remaining()[i]
                == full[i] by {
                assert(callback.ensures((entries.remaining()[i],), result.remaining()[i]));
            }
            assert(result.remaining().is_prefix_of(full));
            assert(result.will_return_none() ==> result.remaining() =~= full);
            assert forall|count: usize| #[trigger] native_count_post(result, count) implies count
                == full.len() by {
                assert(native_count_post(entries, count));
            }
        }
        result
    }
}

} // verus!
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
