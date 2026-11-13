use vstd::prelude::*;
use core::num::NonZero;
verus! {
struct BorTag(NonZero<u64>);
impl BorTag {
    fn new(i: u64) -> (r: Option<Self>)
        ensures
            r.is_some() == (i != 0),
            r.is_some() ==> r.unwrap().0@ == i,
    {
        NonZero::new(i).map(BorTag)
    }
}
}
