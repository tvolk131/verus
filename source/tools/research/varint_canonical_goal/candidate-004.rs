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
// Diagnostic mechanical laws, not application semantic inputs.
pub proof fn radix_positive(k: nat)
    ensures radix_power(k) > 0,
    decreases k,
{
    if k > 0 { radix_positive((k - 1) as nat); }
}

pub proof fn quotient_step(number: u64, n: u64, i: usize)
    requires i < 10, n as nat == number as nat / radix_power(i as nat),
    ensures
        (n >> 7) as nat == number as nat / radix_power((i + 1) as nat),
        n >> 7 == 0 ==> (number as nat) < radix_power((i + 1) as nat),
        n > 0 ==> (number as nat) >= radix_power(i as nat),
        i == 9 ==> n >> 7 == 0,
{
    radix_positive(i as nat);
    vstd::bits::lemma_u64_shr_is_div(n, 7);
    reveal(vstd::arithmetic::power::pow);
    assert(vstd::arithmetic::power2::pow2(7) == 128) by (compute);
    vstd::arithmetic::div_mod::lemma_div_denominator(number as int, radix_power(i as nat) as int, 128);
    vstd::arithmetic::div_mod::lemma_fundamental_div_mod(number as int, radix_power(i as nat) as int);
    vstd::arithmetic::div_mod::lemma_fundamental_div_mod(number as int, radix_power((i + 1) as nat) as int);
    radix_positive((i + 1) as nat);
    vstd::arithmetic::div_mod::lemma_small_div_converse(number as int, radix_power((i + 1) as nat) as int);
    if (number as nat) < radix_power(i as nat) {
        vstd::arithmetic::div_mod::lemma_basic_div(number as int, radix_power(i as nat) as int);
    }
    if i == 9 {
        assert(radix_power(10) == 1180591620717411303424nat) by (compute);
        vstd::arithmetic::div_mod::lemma_basic_div(number as int, radix_power(10) as int);
    }
}

pub proof fn byte_step(n: u64)
    ensures
        (n as u8 | 0x80u8) as nat == n as nat % 128 + 128,
        ((n as u8 | 0x80u8) & 0x7fu8) as nat == n as nat % 128,
{
    assert((n as u8 | 0x80u8) as nat == n as nat % 128 + 128) by (bit_vector);
    assert(((n as u8 | 0x80u8) & 0x7fu8) as nat == n as nat % 128) by (bit_vector);
}
}

macro_rules! encode {
    ($number:expr, $buf:expr) => { verus_exec_expr! {{
        let mut n = $number;
        let mut i = 0;
        proof { vstd::arithmetic::div_mod::lemma_div_basics($number as int); }
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
                // Mechanical iterator alignment.
                i == it.index(),
            invariant
                i <= 9,
                it.seq().len() == 10,
            ensures
                n == 0,
                ($number as nat) < radix_power((i + 1) as nat),
                i > 0 ==> ($number as nat) >= radix_power(i as nat),
                forall|j: int| #![auto] 0 <= j <= i ==>
                    *final(it.seq()[j]) as nat ==
                        ($number as nat / radix_power(j as nat)) % 128
                        + if j < i { 128nat } else { 0nat },
        {
            proof { quotient_step($number, n, i); byte_step(n); }
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
