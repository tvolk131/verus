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


macro_rules! encode {
    ($number:expr, $buf:expr) => { verus_exec_expr! {{
        let mut n = $number;
        let mut i = 0;
        for b in it: $buf.iter_mut()
            invariant_except_break
                // Semantic: quotient of the unencoded suffix.
                n as nat == $number as nat / radix_power(i as nat),
                // Semantic: previous digits require the current radix place.
                i > 0 ==> n > 0,
                // Semantic: each previous digit has its payload and continuation bit.
                forall|j: int| #![auto] 0 <= j < i ==>
                    *final(it.seq()[j]) as nat ==
                        ($number as nat / radix_power(j as nat)) % 128 + 128,
            ensures
                n == 0,
                ($number as nat) < radix_power((i + 1) as nat),
                i > 0 ==> ($number as nat) >= radix_power(i as nat),
                forall|j: int| #![auto] 0 <= j <= i ==>
                    *final(it.seq()[j]) as nat ==
                        ($number as nat / radix_power(j as nat)) % 128
                        + if j < i { 128nat } else { 0nat },
        {
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


verus! {
pub open spec fn pad_xor(before: Seq<u8>, key: Seq<u8>, after: Seq<u8>) -> bool {
    &&& before.len() == 128
    &&& after.len() == 128
    &&& forall|i: int| 0 <= i < 128 ==> #[trigger] after[i] ==
        if i < key.len() { before[i] ^ key[i] } else { before[i] }
}
pub fn xor_key(ipad: &mut [u8; 128], key: &[u8])
    ensures forall|j: int| #![auto] 0 <= j < 128 && j < key@.len() ==>
        final(ipad)@[j] == (old(ipad)@[j] ^ key@[j]),
{
    for (b_i, b_h) in it: ipad.iter_mut().zip(key)
        invariant
            forall|j: int| #![auto] 0 <= j < it.index() ==>
                *final(it.seq()[j].0) == (*it.seq()[j].0 ^ *it.seq()[j].1),
    {
        *b_i ^= *b_h;
    }
}
}

