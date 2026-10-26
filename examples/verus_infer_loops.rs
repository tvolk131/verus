use vstd::prelude::*;

// The loop, prefix invariants and decreasing measure are verified together.
#[verus_infer]
pub fn add_n(x: u8, n: u8) -> u8 {
    let mut result = x;
    let mut i = 0u8;
    while i < n {
        result += 1;
        i += 1;
    }
    result
}

#[verus_infer]
pub fn add_n64(x: u64, n: u64) -> u64 {
    let mut total = x;
    let mut counter = 0u64;
    while counter < n {
        total = total + 1;
        counter = counter + 1;
    }
    total
}

// The step is immutable; multiplication in the contract is mathematical.
#[verus_infer]
pub fn add_steps(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut i = 0u64;
    while i < n { result += step; i += 1; }
    result
}

#[verus_infer]
pub fn subtract_steps(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut i = 0u64;
    while i < n { result -= step; i += 1; }
    result
}

#[verus_infer]
pub fn countdown_add(x: u64, n: u64, step: u64) -> u64 {
    let mut remaining = n;
    let mut result = x;
    while remaining > 0 { result = result + step; remaining = remaining - 1; }
    result
}

#[verus_infer]
pub fn countdown_subtract(x: u64, n: u64, step: u64) -> u64 {
    let mut result = x;
    let mut remaining = n;
    while remaining > 0 { result -= step; remaining -= 1; }
    result
}

#[verus_infer]
pub fn subtract_twos(x: u8, n: u8) -> u8 {
    let mut result = x;
    let mut remaining = n;
    while remaining > 0 { result -= 2; remaining -= 1; }
    result
}

#[verus_infer]
pub fn zero_steps(x: usize, n: usize) -> usize {
    let mut result = x;
    let mut i = 0usize;
    while i < n { result += 0; i += 1; }
    result
}

#[verus_infer]
pub fn twice(x: u8, n: u8) -> u8 { add_n(add_n(x, n), n) }

verus! {
    pub fn application_property(x: u8, n: u8) -> (result: u8)
        requires x as int + 2 * n as int <= 255,
        ensures result as int == x as int + 2 * n as int,
    { twice(x, n) }
}

fn main() {
    assert_eq!(add_n(255, 0), 255);
    assert_eq!(add_n(0, 255), 255);
    assert_eq!(add_n(254, 1), 255);
    assert_eq!(add_n64(u64::MAX, 0), u64::MAX);
    assert_eq!(application_property(3, 126), 255);
    assert_eq!(add_steps(3, 126, 2), 255);
    assert_eq!(subtract_steps(255, 127, 2), 1);
    assert_eq!(countdown_add(0, 1, u64::MAX), u64::MAX);
    assert_eq!(countdown_subtract(u64::MAX, 1, u64::MAX), 0);
    assert_eq!(countdown_subtract(u64::MAX, 0, u64::MAX), u64::MAX);
    assert_eq!(subtract_twos(254, 127), 0);
    assert_eq!(zero_steps(usize::MAX, 255), usize::MAX);
    println!("Verus Infer counted-loop examples passed");
}
