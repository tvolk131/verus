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
pub proof fn one_byte_semantics(number: u64)
    requires number < 128,
    ensures canonical(number, seq![number as u8]),
{
    reveal_with_fuel(radix_power, 3);
    let s = seq![number as u8];
    vstd::arithmetic::div_mod::lemma_small_mod(number as nat, 128);
    assert(radix_power(0) == 1) by (compute);
    assert(radix_power(1) == 128) by (compute);
    assert forall|index: int| 0 <= index < s.len() implies
        #[trigger] s[index] as nat ==
            (number as nat / radix_power(index as nat)) % 128
            + if index + 1 < s.len() { 128nat } else { 0nat } by {
        assert(index == 0);
    }
    assert(canonical(number, s));
}
pub proof fn two_byte_boundary_semantics()
    ensures canonical(128, seq![128u8, 1u8]),
{
    reveal_with_fuel(radix_power, 4);
    let s = seq![128u8, 1u8];
    assert(radix_power(0) == 1) by (compute);
    assert(radix_power(1) == 128) by (compute);
    assert(radix_power(2) == 16384) by (compute);
    assert forall|index: int| 0 <= index < s.len() implies
        #[trigger] s[index] as nat ==
            (128nat / radix_power(index as nat)) % 128
            + if index + 1 < s.len() { 128nat } else { 0nat } by {
        assert(index == 0 || index == 1);
        if index == 0 {
            assert(s[index] == 128);
        } else {
            assert(s[index] == 1);
        }
    }
    assert(canonical(128, s));
}
}
