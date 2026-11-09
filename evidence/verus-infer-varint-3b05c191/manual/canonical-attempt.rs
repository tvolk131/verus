#![feature(proc_macro_hygiene)]
use vstd::prelude::*;

verus! {

pub open spec fn radix_power(exponent: nat) -> nat
    decreases exponent,
{
    if exponent == 0 { 1 } else { 128 * radix_power((exponent - 1) as nat) }
}

pub open spec fn canonical(number: u64, encoded: Seq<u8>) -> bool {
    1 <= encoded.len() <= 10
    && (number as nat) < radix_power(encoded.len())
    && (encoded.len() > 1 ==> (number as nat) >= radix_power((encoded.len() - 1) as nat))
    && forall|index: int| 0 <= index < encoded.len() ==>
        #[trigger] encoded[index] as nat ==
            (number as nat / radix_power(index as nat)) % 128
            + if index + 1 < encoded.len() { 128nat } else { 0nat }
}
}

verus! {
// Authored mechanical lemma; no assume/admit, ordinary checked bit-vector query.
pub proof fn shift_step(number: u64, n: u64, index: u64)
    requires index < 10, n == number >> (7 * index),
    ensures
        index == 9 ==> n >> 7 == 0,
        index < 9 ==> n >> 7 == number >> (7 * (index + 1)),
{
    assert(index == 9 ==> n >> 7 == 0) by (bit_vector)
        requires index < 10, n == number >> (7 * index);
    assert(index < 9 ==> n >> 7 == number >> (7 * (index + 1))) by (bit_vector)
        requires index < 10, n == number >> (7 * index);
}
}

macro_rules! encode {
    ($number:expr, $buf:expr) => { verus_exec_expr! {{
        let mut n = $number;
        let mut i = 0;
        proof { assert($number >> 0 == $number) by (bit_vector); }
        for b in it: $buf.iter_mut()
            invariant_except_break
                i == it.index(),
                n == $number >> (7 * (i as u64)),
            invariant
                i <= 9,
                it.seq().len() == 10,
            ensures
                n == 0,
        {
            proof { shift_step($number, n, i as u64); }
            *b = n as u8 | 0x80;
            n >>= 7;
            if n == 0 {
                *b &= 0x7f;
                break
            }
            i += 1
        }
        debug_assert_eq!(n, 0);
        &$buf[0..=i]
    }} }
}

#[verus_spec(encoded => ensures canonical(number, encoded@))]
pub fn u64(number: u64, buf: &mut [u8; U64_LEN]) -> &[u8] {
    encode!(number, buf)
}

verus! {
const U64_LEN: usize = 10;
}
