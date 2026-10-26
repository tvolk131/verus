use vstd::prelude::*;

#[verus_infer]
pub fn leaf(x: u16) -> u16 { x + 1 }
#[verus_infer]
pub fn middle(x: u16) -> u16 { leaf(x) / 2 }
#[verus_infer]
pub fn root(x: u16) -> u16 { middle(x) + 3 }
#[verus_infer]
pub fn independent(x: u16) -> u16 { x / 10 }

verus! {
    fn caller() {
        let r = root(10);
        assert(r == 8 || r == 9);
        let r = independent(99);
        assert(r == 9);
    }
}
