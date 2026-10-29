//! Small examples of deterministic contract inference, checked by Verus.
//! Run: verus verus_infer_quickstart.rs --no-cheating
use vstd::prelude::*;

// Inferred: x < 255; returns exactly x + 1.
#[verus_infer]
pub fn add_one(x: u8) -> u8 {
    x + 1
}

// Inferred: every u8 input is safe; None iff x == 255,
// otherwise Some(x + 1).
#[verus_infer]
pub fn checked_add_one(x: u8) -> Option<u8> {
    if x == u8::MAX { None } else { Some(x + 1) }
}

// Composes the checked helper contracts.
// Inferred: x <= 253; returns exactly x + 2.
#[verus_infer]
pub fn add_two(x: u8) -> u8 {
    add_one(add_one(x))
}

fn main() {}
