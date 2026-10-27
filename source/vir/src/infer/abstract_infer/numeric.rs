//! Conservative numeric transfers for finite and mathematical scalar values.
use super::*;
use num_traits::ToPrimitive;

pub(in crate::infer) fn clip(a: BigInt, b: BigInt, min: BigInt, max: BigInt) -> Range {
    let modulus = &max - &min + 1;
    let cycle = |n: BigInt| {
        let q = &n / &modulus;
        if n % &modulus < BigInt::zero() { q - 1 } else { q }
    };
    let left = cycle(&a - &min);
    let right = cycle(&b - &min);
    if left == right {
        Range::Int(a - &left * &modulus, b - left * modulus)
    } else {
        Range::Int(min, max)
    }
}
pub(in crate::infer) fn bitwise(
    op: BitwiseOp,
    a: Range,
    b: Range,
    typ: &Typ,
    word_bits: u32,
) -> Range {
    let full = || Range::full(typ, word_bits).unwrap_or(Range::IntTop);
    let (Range::Int(a, b), Range::Int(c, d)) = (a, b) else { return full() };
    if matches!(op, BitwiseOp::BitAnd | BitwiseOp::BitOr | BitwiseOp::BitXor) {
        if a == b && c == d {
            let n = match op {
                BitwiseOp::BitAnd => a & c,
                BitwiseOp::BitOr => a | c,
                _ => a ^ c,
            };
            return Range::Int(n.clone(), n);
        }
        match op {
            BitwiseOp::BitAnd => {
                if a.is_zero() && b.is_zero() || c.is_zero() && d.is_zero() {
                    return Range::Int(0.into(), 0.into());
                }
                if a == b && a == -BigInt::one() {
                    return Range::Int(c, d);
                }
                if c == d && c == -BigInt::one() {
                    return Range::Int(a, b);
                }
                if a >= BigInt::zero() && c >= BigInt::zero() {
                    return Range::Int(0.into(), b.min(d));
                }
                if a >= BigInt::zero() {
                    return Range::Int(0.into(), b);
                }
                if c >= BigInt::zero() {
                    return Range::Int(0.into(), d);
                }
            }
            BitwiseOp::BitOr | BitwiseOp::BitXor => {
                if a.is_zero() && b.is_zero() {
                    return Range::Int(c, d);
                }
                if c.is_zero() && d.is_zero() {
                    return Range::Int(a, b);
                }
                if a >= BigInt::zero() && c >= BigInt::zero() {
                    let upper = (BigInt::one() << (b.bits().max(d.bits()) as usize)) - 1;
                    let lower = if op == BitwiseOp::BitOr { a.max(c) } else { 0.into() };
                    return Range::Int(lower, upper);
                }
            }
            _ => {}
        }
        return full();
    }
    let Some((width, _)) = bitwise::shape(typ) else { return full() };
    let bits = match width {
        IntegerTypeBitwidth::Width(n) => n,
        IntegerTypeBitwidth::ArchWordSize => word_bits,
    };
    let lo = c.max(0.into());
    let hi = d.min(BigInt::from(bits - 1));
    let (Some(lo), Some(hi)) = (lo.to_usize(), hi.to_usize()) else { return full() };
    if lo > hi {
        return full();
    }
    let shift = |x: &BigInt, n| if op == BitwiseOp::Shr { x >> n } else { x << n };
    let values = [shift(&a, lo), shift(&a, hi), shift(&b, lo), shift(&b, hi)];
    let min = values.iter().min().unwrap().clone();
    let max = values.iter().max().unwrap().clone();
    if op == BitwiseOp::Shr {
        return Range::Int(min, max);
    }
    let TypX::Int(range) = &**typ else { return full() };
    let Some((lower, upper)) = arithmetic::bounds_at(*range, word_bits) else { return full() };
    if min >= lower && max <= upper {
        Range::Int(min, max)
    } else if a == b && lo == hi {
        clip(min, max, lower, upper)
    } else {
        full()
    }
}
