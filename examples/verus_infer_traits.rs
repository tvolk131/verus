//! Bounded exact trait getters and opaque associated types in record inference.
//! Run with --infer-goals. The logical getter contracts remain supplied.
use vstd::prelude::*;

verus! {
pub trait PayloadFamily {
    type Payload: Copy;
}

pub trait Event {
    type Family: PayloadFamily;
    spec fn slot_value(&self) -> usize;
    spec fn payload_value(&self) -> <Self::Family as PayloadFamily>::Payload;

    fn slot(&self) -> (result: usize)
        ensures result == self.slot_value();

    fn payload(&self) -> (result: <Self::Family as PayloadFamily>::Payload)
        ensures result == self.payload_value();
}

pub struct Registry<F: PayloadFamily> {
    pub slots: Vec<Option<F::Payload>>,
    pub occupied: usize,
    pub epoch: u64,
}

impl<F: PayloadFamily> Registry<F> {
    pub open spec fn valid(&self) -> bool {
        self.occupied == self.slots@.count_prefix(
            |slot: Option<F::Payload>| slot.is_some(), self.slots@.len())
    }

    #[verifier::infer]
    pub fn insert<E: Event<Family = F>>(&mut self, event: &E) {
        let i = event.slot();
        if i >= self.slots.len() { return; }
        let empty = match self.slots[i] { None => true, Some(_) => false };
        if empty {
            self.slots[i] = Some(event.payload());
            self.occupied += 1;
        }
    }
}

pub fn check<F: PayloadFamily, E: Event<Family = F>>(
    registry: &mut Registry<F>, event: &E,
)
    requires old(registry).valid(),
    ensures final(registry).valid(), final(registry).epoch == old(registry).epoch,
{
    registry.insert(event);
}
}

fn main() {}
