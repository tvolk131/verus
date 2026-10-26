use vstd::prelude::*;

#[verus_infer]
pub fn offset(base: isize, delta: isize) -> isize { base + delta }

#[verus_infer]
pub fn checked_offset(base: isize, delta: isize) -> Option<isize> {
    base.checked_add(delta)
}

#[verus_infer]
pub fn checked_size(base: usize, count: usize) -> Option<usize> {
    base.checked_add(count)
}

#[verus_infer]
pub fn quotient(x: isize, y: isize) -> Option<isize> { x.checked_div(y) }

#[verus_infer]
pub fn reinterpret(value: usize) -> isize { value as isize }

// Separate ranges are inferred for the two verification architectures.
#[verus_infer(precision = "sound")]
pub fn coarse_size(value: usize) -> usize { value / 10 }

#[verus_infer(precision = "sound")]
pub fn coarse_offset(value: isize) -> isize { value / 10 }

#[verus_infer(precision = "sound")]
pub fn shifted_size(value: usize) -> usize { coarse_size(value) + 1 }

verus! {
    pub fn size_property(value: usize) -> (result: usize)
        ensures 1 <= result <= usize::MAX / 10 + 1,
    { shifted_size(value) }

    pub fn offset_property(value: isize) -> (result: isize)
        ensures isize::MIN / 10 <= result <= isize::MAX / 10,
    { coarse_offset(value) }
}

fn main() {
    assert_eq!(offset(isize::MIN + 1, -1), isize::MIN);
    assert_eq!(checked_offset(isize::MAX, 1), None);
    assert_eq!(checked_size(usize::MAX, 1), None);
    assert_eq!(quotient(isize::MIN, -1), None);
    assert_eq!(quotient(-10, 3), Some(-3));
    assert_eq!(reinterpret(usize::MAX), -1);
    assert_eq!(size_property(200), 21);
    assert_eq!(offset_property(-200), -20);
    println!("Verus Infer platform-integer examples passed");
}
