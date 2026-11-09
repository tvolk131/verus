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
pub proof fn wrong_encoding()
    ensures canonical(0, seq![128u8, 0u8]),
{
    reveal_with_fuel(radix_power, 13);
}
}
