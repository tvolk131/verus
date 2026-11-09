use vstd::prelude::*;
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
    let ghost before = ipad@;
    let ghost future = final(ipad)@;
    for (b_i, b_h) in it: ipad.iter_mut().zip(key)
        invariant
            before.len() == 128, future.len() == 128,
            it.seq().len() == if key@.len() < 128 { key@.len() } else { 128 },
            forall|j: int| #![auto] 0 <= j < it.seq().len() ==>
                *it.seq()[j].0 == before[j]
                && *it.seq()[j].1 == key@[j]
                && *final(it.seq()[j].0) == future[j],
            forall|j: int| #![auto] 0 <= j < it.index() ==>
                *final(it.seq()[j].0) == (*it.seq()[j].0 ^ *it.seq()[j].1),
    {
        *b_i ^= *b_h;
    }
}
}
fn main() {}
