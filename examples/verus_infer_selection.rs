// rust_verify/tests/example.rs expect-success --infer-goals
//! Borrowed Result queries infer complete success, error, and contents relations.
//! These methods stay within the crate so their inferred contracts can describe private fields.
use vstd::prelude::*;

#[verus_verify]
#[derive(Debug, Copy, Clone)]
pub enum SelectionError { TooFew, MissingOwn }

#[verus_verify]
pub struct Collector<T: Copy> {
    slots: Vec<Option<(T, u16)>>,
    direct_parents: usize,
    round: u32,
}

#[verus_verify]
impl<T: Copy> Collector<T> {
    #[verus_infer]
    pub(crate) fn parents(&self, creator: usize) -> Result<&Vec<Option<(T, u16)>>, SelectionError> {
        if self.direct_parents < 3 { return Err(SelectionError::TooFew); }
        if creator >= self.slots.len() { return Err(SelectionError::MissingOwn); }
        match self.slots[creator] {
            Some((_, round)) if u32::from(round) + 1 == self.round => Ok(&self.slots),
            _ => Err(SelectionError::MissingOwn),
        }
    }
}

verus! {
fn query_goal<T: Copy>(collector: &Collector<T>, creator: usize)
    -> (result: Result<&Vec<Option<(T, u16)>>, SelectionError>)
    ensures
        result.is_ok() == (collector.direct_parents >= 3
            && creator < collector.slots@.len()
            && collector.slots@[creator as int].is_some()
            && collector.slots@[creator as int].unwrap().1 as int + 1 == collector.round),
        result.is_ok() ==> result.unwrap()@ == collector.slots@,
        collector.direct_parents < 3 ==> (result matches Err(SelectionError::TooFew)),
{ collector.parents(creator) }
}

fn main() {
    let mut collector = Collector { slots: vec![Some((7u64, 1)), None], direct_parents: 0, round: 2 };
    assert!(matches!(collector.parents(0), Err(SelectionError::TooFew)));
    collector.direct_parents = 3;
    assert_eq!(collector.parents(0).unwrap(), &collector.slots);
    assert!(matches!(collector.parents(1), Err(SelectionError::MissingOwn)));
    assert!(matches!(collector.parents(usize::MAX), Err(SelectionError::MissingOwn)));
}
