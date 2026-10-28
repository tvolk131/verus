// Checked bounded collection adapters for the ControlHash query experiment.
// The sparse iterator/consumers replace the native enumerate/filter_map/count
// pipelines explicitly. These are ordinary proofs, not inferred abstractions.
use vstd::iter::{SparseIter, sparse_entries};
use vstd::std_specs::iter::IteratorSpec;
verus! {

pub trait ParentQueries {
    spec fn count_range_value(&self, lo: u16, hi: u16) -> usize;

    spec fn after_value(&self, round: u16) -> bool;

    fn count_range(&self, lo: u16, hi: u16) -> (result: usize)
        ensures
            result == self.count_range_value(lo, hi),
    ;

    fn has_after(&self, round: u16) -> (result: bool)
        ensures
            result == self.after_value(round),
    ;
}

impl ParentQueries for NodeMap<u16> {
    closed spec fn count_range_value(&self, lo: u16, hi: u16) -> usize {
        let entries = sparse_entries(self@, 0);
        entries.count_prefix(|entry: (usize, &u16)| lo <= *entry.1 <= hi, entries.len()) as usize
    }

    closed spec fn after_value(&self, round: u16) -> bool {
        let entries = sparse_entries(self@, 0);
        exists|i: int| 0 <= i < entries.len() && *entries[i].1 > round
    }

    fn count_range(&self, lo: u16, hi: u16) -> (result: usize) {
        vstd::iter::count_where(
            SparseIter::new(&self.0),
            |entry: &(usize, &u16)| -> (keep: bool)
                ensures
                    keep == (lo <= *entry.1 <= hi),
                { lo <= *entry.1 && *entry.1 <= hi },
            Ghost(|entry: (usize, &u16)| lo <= *entry.1 <= hi),
        )
    }

    fn has_after(&self, round: u16) -> (result: bool) {
        let found = vstd::iter::find(
            SparseIter::new(&self.0),
            |entry: &(usize, &u16)| -> (keep: bool)
                ensures
                    keep == (*entry.1 > round),
                { *entry.1 > round },
            Ghost(|entry: (usize, &u16)| *entry.1 > round),
        );
        found.is_some()
    }
}

pub open spec fn round_projection<T>(parents: Seq<Option<(T, u16)>>) -> Seq<Option<u16>> {
    Seq::new(
        parents.len(),
        |i: int|
            match parents[i] {
                None => None,
                Some((_, round)) => Some(round),
            },
    )
}

// Checked counterpart of the index-preserving loop in ControlHash::new. No
// hash or encoding relation is inferred or assumed by this collection operation.
pub fn project_parent_rounds<T>(parents: &NodeMap<(T, u16)>) -> (result: NodeMap<u16>)
    ensures
        result@ == round_projection(parents@),
{
    let values = vstd::iter::map_sparse(
        &parents.0,
        |entry: &(T, u16)| -> (round: u16)
            ensures
                round == entry.1,
            { entry.1 },
    );
    proof {
        assert(values@ =~= round_projection(parents@));
    }
    NodeMap(values)
}

} // verus!
