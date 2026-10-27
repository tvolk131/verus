use vstd::prelude::*;

#[verus_infer]
pub fn pair(value: u8) -> [u8; 2] { [value, value + 1] }

#[verus_infer]
pub fn optional_pair(input: Option<u8>) -> [u8; 2] {
    let Some(value) = input else { return [0, 0]; };
    match value {
        0 | 255 => [value, 0],
        _ => pair(value),
    }
}

#[verus_infer]
pub fn normalize(input: Result<u8, u8>) -> u8 {
    match input {
        Ok(value) if value < 255 => value + 1,
        Ok(value) | Err(value) => value,
    }
}

#[verus_infer]
pub fn widened(input: Result<u8, u8>) -> Result<u8, u16> { Ok(input?) }

#[verus_infer]
pub fn at(values: &[u8], index: usize) -> u8 { values[index] }

#[verus_infer]
pub fn window_at(values: &[u8], start: usize, end: usize, index: usize) -> u8 {
    let window = &values[start..end];
    at(window, index)
}

#[verus_infer]
pub fn selected(values: &[u8], input: Result<u8, u8>) -> Result<u8, u16> {
    let index = widened(input)?;
    Ok(window_at(values, index as usize, values.len(), 0))
}

#[verus_infer]
pub fn window_pair(values: &[u8], start: usize, end: usize) -> [u8; 2] {
    let window = &values[start..end];
    [at(window, 0), at(window, 1)]
}

#[verus_infer]
pub fn prefix_length(values: &[u8], start: usize, count: usize) -> usize {
    let suffix = &values[start..];
    let prefix = &suffix[..count];
    let all = &prefix[..];
    all.len()
}

verus! {
    pub fn caller_properties() {
        let p = optional_pair(Some(254));
        assert(p[0] == 254 && p[1] == 255);
        let p = window_pair(&[1, 3, 7, 255], 2, 4);
        assert(p[0] == 7 && p[1] == 255);
        let r = selected(&[], Err(255));
        assert(r == Err(255u16));
        let r = selected(&[1, 3, 7, 255], Ok(3));
        assert(r == Ok(255));
        let n = prefix_length(&[], 0, 0);
        assert(n == 0);
    }
}

fn main() {
    for value in 0..=255u8 {
        let expected = match value { 0 | 255 => [value, 0], _ => [value, value + 1] };
        assert_eq!(optional_pair(Some(value)), expected);
        assert_eq!(normalize(Ok(value)), value.saturating_add(1));
        assert_eq!(normalize(Err(value)), value);
        assert_eq!(widened(Err(value)), Err(u16::from(value)));
    }
    assert_eq!(optional_pair(None), [0, 0]);
    assert_eq!(window_at(&[1, 3, 7, 255], 1, 4, 2), 255);
    assert_eq!(window_pair(&[1, 3, 7, 255], 2, 4), [7, 255]);
    assert_eq!(selected(&[], Err(255)), Err(255));
    assert_eq!(prefix_length(&[1, 3, 7], 3, 0), 0);
    caller_properties();
    println!("Verus Infer Rust coverage examples passed");
}
