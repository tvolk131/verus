// rust_verify/tests/example.rs expect-success --infer-goals
//! Supplied direct sequence views preserve collection privacy during inference.
//! Run with --infer-goals. In native Rust, use #[verus_infer] on the methods.
use vstd::prelude::*;

verus! {
pub mod storage {
    use vstd::prelude::*;

    pub struct Slots<T>(Vec<Option<T>>);

    impl<T> View for Slots<T> {
        type V = Seq<Option<T>>;
        closed spec fn view(&self) -> Self::V { self.0@ }
    }

    impl<T> Slots<T> {
        #[verifier::infer]
        pub fn len(&self) -> usize { self.0.len() }

        #[verifier::infer]
        pub fn get(&self, i: usize) -> Option<&T> { self.0[i].as_ref() }

        #[verifier::infer]
        pub fn put(&mut self, i: usize, value: T) { self.0[i] = Some(value); }
    }
}

pub struct Registry<T> {
    pub entries: storage::Slots<T>,
    pub occupied: usize,
    pub epoch: u64,
}

impl<T: Copy> Registry<T> {
    pub open spec fn valid(&self) -> bool {
        self.occupied == self.entries@.count_prefix(
            |slot: Option<T>| slot.is_some(), self.entries@.len())
    }

    #[verifier::infer]
    pub fn insert_if_empty(&mut self, i: usize, value: T) {
        if i >= self.entries.len() { return; }
        let empty = match self.entries.get(i) { None => true, Some(_) => false };
        if empty {
            self.entries.put(i, value);
            self.occupied += 1;
        }
    }
}

pub fn check<T: Copy>(r: &mut Registry<T>, i: usize, value: T)
    requires old(r).valid(),
    ensures final(r).valid(), final(r).epoch == old(r).epoch,
{
    // The checked query exposes the length bound hidden by the closed view.
    let _ = r.entries.len();
    r.insert_if_empty(i, value);
}
}

fn main() {}
