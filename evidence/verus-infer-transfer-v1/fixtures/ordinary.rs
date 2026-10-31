#![feature(proc_macro_hygiene)]
use vstd::prelude::*;

verus! {
global size_of usize == 8;

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

macro_rules! encode {
    ($number:expr, $buf:expr) => {{
        let mut n = $number;
        let mut i = 0;
        for b in $buf.iter_mut() {
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
    }}
}

#[verus_spec(encoded => ensures canonical(number, encoded@))]
pub fn u64(number: u64, buf: &mut [u8; U64_LEN]) -> &[u8] {
    encode!(number, buf)
}

const U64_LEN: usize = 10;
