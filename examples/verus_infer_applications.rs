// rust_verify/tests/example.rs expect-warnings
//! Small application kernels within Infer's current pure Rust subset.
use vstd::prelude::*;

#[verus_infer]
pub fn read_be16(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 2 { None }
    else { Some((bytes[0] as u16) * 256 + bytes[1] as u16) }
}

#[verus_infer(precision = "sound")]
pub fn message_kind(flags: u8) -> u8 { flags & 15 }

#[verus_infer(precision = "sound")]
pub fn priority(flags: u8) -> u8 { (flags & 240) >> 4 }

#[verus_infer]
pub fn field_at(bytes: &[u8], start: usize, end: usize, index: usize) -> Option<u8> {
    let field = &bytes[start..end];
    if index < field.len() { Some(field[index]) } else { None }
}

#[verus_infer]
pub fn record_end(length: usize, start: usize, width: usize) -> Option<usize> {
    let end = start.checked_add(width)?;
    if end <= length { Some(end) } else { None }
}

#[verus_infer]
pub fn subtotal(price: u32, quantity: u32) -> Option<u32> { price.checked_mul(quantity) }

#[verus_infer]
pub fn purchase(balance: u32, price: u32, quantity: u32) -> Option<u32> {
    let total = subtotal(price, quantity)?;
    balance.checked_sub(total)
}

#[verus_infer(precision = "sound")]
pub fn discount_percent(requested: u8) -> u8 {
    let copied = requested;
    let below_cap = copied < 100;
    if below_cap { requested } else { 100 }
}

#[verus_infer(precision = "sound")]
pub fn capped_shipping(tier: u8) -> u8 {
    if tier < 10 { tier.saturating_mul(3) } else { 30 }
}

#[verus_infer(precision = "sound")]
pub fn rollover_tick(tick: u8) -> u8 {
    if tick > 250 { tick.wrapping_add(10) } else { 0 }
}

#[verus_infer(precision = "sound")]
pub fn signed_wire_code(code: i16) -> i8 {
    if code >= 128 && code <= 130 { code as i8 } else { -128 }
}

#[verus_infer]
pub fn next_price_pair(price: u16) -> [u16; 2] { [price, price + 1] }

#[verus_infer]
pub fn read_error(input: Result<u8, u8>) -> Result<u8, u16> { Ok(input?) }

#[verus_infer]
pub fn increment_cost(value: u8) -> u8 { (value + 1) - value }

#[verus_infer]
pub fn first_remaining(bytes: &[u8], start: usize) -> u8 {
    let remaining = &bytes[start..];
    remaining[0]
}

verus! {
    pub fn caller_properties(flags: u8, requested: u8, tick: u8) {
        let kind = message_kind(flags); assert(kind <= 15);
        let p = priority(flags); assert(p <= 15);
        let discount = discount_percent(requested); assert(discount <= 100);
        let shipping = capped_shipping(requested); assert(shipping <= 30);
        let next = rollover_tick(tick); assert(next <= 9);
        let code = signed_wire_code(129); assert(-128 <= code && code <= -126);
        let word = read_be16(&[18, 52]); assert(word == Some(4660));
        let word = read_be16(&[255]); assert(word == None);
        let field = field_at(&[4, 5, 6], 1, 3, 1); assert(field == Some(6));
        let field = field_at(&[4, 5, 6], 1, 3, 2); assert(field == None);
        let end = record_end(10, 8, 2); assert(end == Some(10));
        let end = record_end(10, usize::MAX, 1); assert(end == None);
        let paid = purchase(100, 7, 3); assert(paid == Some(79));
        let paid = purchase(100, u32::MAX, 2); assert(paid == None);
        let paid = purchase(20, 7, 3); assert(paid == None);
        let prices = next_price_pair(65534); assert(prices[1] == 65535);
        let err = read_error(Err(255)); assert(err == Err(255u16));
        let cost = increment_cost(254); assert(cost == 1);
        let first = first_remaining(&[4, 5, 6], 2); assert(first == 6);
    }
}

fn main() {
    for x in 0..=255u8 {
        assert_eq!(message_kind(x), x & 15);
        assert_eq!(priority(x), x >> 4);
        assert_eq!(discount_percent(x), x.min(100));
        assert_eq!(capped_shipping(x), x.min(10) * 3);
        assert_eq!(rollover_tick(x), if x > 250 { x.wrapping_add(10) } else { 0 });
        caller_properties(x, x, x);
    }
    assert_eq!(read_be16(&[255, 255]), Some(65535));
    assert_eq!(record_end(10, usize::MAX, 1), None);
    assert_eq!(purchase(100, u32::MAX, 2), None);
    assert_eq!(signed_wire_code(130), -126);
    println!("Verus Infer application kernels passed");
}
