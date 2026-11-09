use vstd::prelude::*;
verus! {
pub open spec fn pad_xor(before: Seq<u8>, key: Seq<u8>, after: Seq<u8>) -> bool {
    &&& before.len() == 128
    &&& after.len() == 128
    &&& forall|i: int| 0 <= i < 128 ==> #[trigger] after[i] ==
        if i < key.len() { before[i] ^ key[i] } else { before[i] }
}
pub fn xor_key(ipad: &mut [u8; 128], key: &[u8])
    ensures pad_xor(old(ipad)@, key@, final(ipad)@),
{
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}
}
fn main() {}
