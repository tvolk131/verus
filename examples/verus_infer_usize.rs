use vstd::prelude::*;

// Overflow bounds use Verus's symbolic usize::MAX (32 or 64 bits).
#[verus_infer]
pub fn advance(index: usize, count: usize) -> usize { index + count }

#[verus_infer]
pub fn next(values: &[usize], index: usize) -> Option<usize> {
    if index < usize::MAX && index + 1 < values.len() {
        Some(values[index + 1])
    } else {
        None
    }
}

#[verus_infer]
pub fn truncate(value: u64) -> usize { value as usize }

#[verus_infer]
pub fn counted_add(x: usize, n: usize) -> usize {
    let mut result = x;
    let mut i = 0usize;
    while i < n {
        result += 1;
        i += 1;
    }
    result
}

#[verus_infer]
pub fn twice(x: usize, n: usize) -> usize { counted_add(counted_add(x, n), n) }

verus! {
    pub fn application_property(x: usize, n: usize) -> (result: usize)
        requires x as int + 2 * n as int <= usize::MAX,
        ensures result as int == x as int + 2 * n as int,
    { twice(x, n) }
}

fn main() {
    assert_eq!(advance(usize::MAX - 1, 1), usize::MAX);
    assert_eq!(next(&[2, 9], 0), Some(9));
    assert_eq!(next(&[], usize::MAX), None);
    assert_eq!(truncate(u64::MAX), usize::MAX);
    assert_eq!(counted_add(usize::MAX, 0), usize::MAX);
    assert_eq!(application_property(3, 2), 7);
    println!("Verus Infer usize examples passed");
}
