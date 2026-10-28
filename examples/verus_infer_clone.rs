use vstd::prelude::*;
use vstd::std_specs::clone::{CloneIdentity, lemma_cloned_identity};

// Keep Rust's native derive and prove its relation through a supplied View.
#[verus_verify(clone_via_view)]
#[derive(Clone)]
pub struct Slots<T>(Vec<Option<T>>);

verus! {
impl<T> View for Slots<T> {
    type V = Seq<Option<T>>;
    closed spec fn view(&self) -> Self::V { self.0@ }
}
broadcast use lemma_cloned_identity;
}

// Copy alone does not supply this law. The vstd implementations for primitives,
// Option, arrays and bounded tuples prove CloneIdentity against their Clone specs.
#[verus_infer]
pub fn duplicate<T: Copy + CloneIdentity>(values: &Slots<T>) -> Slots<T> {
    values.clone()
}

verus! {
fn check<T: Copy + CloneIdentity>(values: &Slots<T>) -> (result: Slots<T>)
    ensures result@ == values@,
{ duplicate(values) }
}

fn main() {
    let original = Slots(vec![Some((42u64, 3u16)), None]);
    let copied = duplicate(&original);
    assert_eq!(copied.0, original.0);
}
