use vstd::prelude::*;
use std::collections::BTreeMap;
use secp256k1::{Keypair, PublicKey};
use secp256k1::hashes::sha256;

verus! {
// Original carrier/fields; unused derives are not part of this diagnostic.
pub struct PeerId(u16);
pub struct Keychain {
    identity: PeerId,
    pks: BTreeMap<PeerId, PublicKey>,
    message_tag: sha256::Hash,
    keypair: Keypair,
}
impl Keychain {
    // Synthetic eligibility method for the exact caller subexpression only.
    fn configured_length(&self) -> (n: usize)
        ensures n == vstd::std_specs::btree::spec_btree_map_len(&self.pks),
    {
        self.pks.len()
    }
}
}
