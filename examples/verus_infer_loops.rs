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
    println!("Verus Infer counted-loop examples passed");
}
