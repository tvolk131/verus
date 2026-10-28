//! Checked From/Into and AddAssign for transparent scalar tuple wrappers.
//! Impl attributes generate ordinary extension definitions; executable bodies
//! retain the independent soundness, domain-maximality and exactness checks.
use vstd::prelude::*;

#[verus_verify]
#[derive(Copy, Clone)]
pub struct Count(pub usize);

#[verus_infer]
impl From<usize> for Count {
    fn from(value: usize) -> Self {
        Count(value)
    }
}

#[verus_infer]
impl From<Count> for usize {
    fn from(value: Count) -> Self {
        value.0
    }
}

#[verus_infer]
impl core::ops::AddAssign for Count {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

#[verus_infer]
pub fn roundtrip(value: usize) -> usize {
    let wrapped: Count = value.into();
    wrapped.into()
}

#[verus_verify]
pub struct State {
    pub count: Count,
    pub epoch: u64,
}

#[verus_verify]
impl State {
    #[verus_infer]
    pub fn increment(&mut self) {
        self.count += Count(1);
    }
}

verus! {
pub fn conversion_goal(value: usize) -> (result: usize)
    ensures result == value,
{ roundtrip(value) }

pub fn update_goal(state: &mut State)
    requires old(state).count.0 < usize::MAX,
    ensures final(state).count.0 == old(state).count.0 + 1,
            final(state).epoch == old(state).epoch,
{ state.increment(); }
}

fn main() {
    let mut state = State {
        count: 3usize.into(),
        epoch: 7,
    };
    state.increment();
    assert_eq!(state.count.0, 4);
    assert_eq!(state.epoch, 7);
    assert_eq!(roundtrip(usize::MAX), usize::MAX);
}
