use vstd::prelude::*;
verus! {
pub fn exact(numerator: u32, divisor: u32) -> (result: u32)
    requires divisor != 0,
    ensures result as int == (numerator as int + divisor as int - 1) / divisor as int,
{
    numerator.div_ceil(divisor)
}

pub fn qualified(numerator: u32, divisor: u32) -> (result: u32)
    requires divisor != 0,
    ensures result as int == (numerator as int + divisor as int - 1) / divisor as int,
{
    u32::div_ceil(numerator, divisor)
}

pub fn extremes() {
    let zero = 0u32.div_ceil(u32::MAX);
    let max = u32::MAX.div_ceil(1);
    let overflow_sensitive = u32::MAX.div_ceil(2);
    let equal = u32::MAX.div_ceil(u32::MAX);
    let near = (u32::MAX - 1).div_ceil(u32::MAX);
    let words = u32::MAX.div_ceil(64);
    assert(zero == 0);
    assert(max == u32::MAX);
    assert(overflow_sensitive == 2147483648);
    assert(equal == 1);
    assert(near == 1);
    assert(words == 67108864);
}
}
