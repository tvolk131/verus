// rust_verify/tests/example.rs expect-success --infer-goals
// Run with --infer-goals to discharge the supplied counting invariant.
// Inside verus!, verifier::infer is the equivalent of verus_infer on ordinary Rust.
use vstd::prelude::*;

verus! {
#[derive(Copy, Clone)]
pub struct SlotIndex(pub usize);

pub struct SlotTable<T>(pub Vec<Option<T>>);

impl<T> SlotTable<T> {
    #[verifier::infer]
    pub fn len(&self) -> usize { self.0.len() }

    #[verifier::infer]
    pub fn get(&self, i: SlotIndex) -> Option<&T> { self.0[i.0].as_ref() }

    #[verifier::infer]
    pub fn insert(&mut self, i: SlotIndex, value: T) { self.0[i.0] = Some(value); }
}

pub struct Registry<T> {
    pub entries: SlotTable<T>,
    pub occupied: usize,
    pub epoch: u32,
}

impl<T: Copy> Registry<T> {
    pub closed spec fn valid(&self) -> bool {
        self.occupied == self.entries.0@.count_prefix(
            |slot: Option<T>| slot.is_some(), self.entries.0@.len())
    }

    #[verifier::infer]
    pub fn insert_if_empty(&mut self, i: SlotIndex, value: T) {
        if i.0 >= self.entries.len() { return; }
        let empty = match self.entries.get(i) { None => true, Some(_) => false };
        if empty {
            self.entries.insert(i, value);
            self.occupied += 1;
        }
    }

    // Intent stays explicit. This ordinary caller must cover every valid input
    // and prove preservation through independently checked helper contracts.
    pub fn check_insertion(&mut self, i: SlotIndex, value: T)
        requires old(self).valid(),
        ensures final(self).valid(), final(self).epoch == old(self).epoch,
    { self.insert_if_empty(i, value); }
}

fn main() {}
}
