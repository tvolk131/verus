use vstd::prelude::*;
use std::collections::BTreeMap;
use secp256k1::{Keypair, PublicKey};
use secp256k1::hashes::sha256;

// Resolve exactly the actual derive_more reexports, without loading unused macros.
mod derive_more { pub use core; pub use core::convert::From; }

verus! {
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPublicKey(secp256k1::PublicKey);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExHash(secp256k1::hashes::sha256::Hash);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExKeypair(secp256k1::Keypair);
}

pub mod aleph_bft {
use vstd::prelude::*;
use crate::derive_more;
#[verus_verify]
#[derive(Copy, Clone)]
pub struct NodeIndex(pub usize);
#[verus_verify]
pub struct NodeCount(pub usize);

verus! {
impl vstd::std_specs::convert::FromSpecImpl<usize> for NodeCount {
    pub open spec fn obeys_from_spec() -> bool { true }
    pub closed spec fn from_spec(value: usize) -> Self { NodeCount(value) }
}
impl vstd::std_specs::convert::FromSpecImpl<NodeCount> for usize {
    pub open spec fn obeys_from_spec() -> bool { true }
    pub closed spec fn from_spec(value: NodeCount) -> Self { value.0 }
}
}
#[verus_verify]
impl derive_more::From<(usize)> for NodeCount {
    #[inline]
    #[verus_spec(r => ensures r.0 == value)]
    fn from(value: (usize)) -> Self { NodeCount(value) }
}
#[verus_verify]
impl derive_more::core::convert::From<NodeCount> for (usize) {
    #[inline]
    #[verus_spec(r => ensures r == value.0)]
    fn from(value: NodeCount) -> Self {
        (<usize as derive_more::core::convert::From<_>>::from(value.0))
    }
}
#[verus_verify]
pub struct NodeMap<T>(Vec<Option<T>>);
verus! {
impl<T> NodeMap<T> {
    pub closed spec fn slots(&self) -> Seq<Option<T>> { self.0@ }
    pub open spec fn signers(&self) -> Set<usize> {
        Set::from_finite_type(|i: usize| i < self.slots().len() && self.slots()[i as int].is_some())
    }
}
}
#[verus_verify]
impl<T> NodeMap<T> {
    #[verus_spec(r => ensures r.slots().len() == len.0,
        forall|i: int| 0 <= i < r.slots().len() ==> #[trigger] r.slots()[i] == None)]
    pub fn with_size(len: NodeCount) -> Self
    where
        T: Clone,
    {
        let v = vec![None; len.into()];
        NodeMap(v)
    }

    #[verus_spec(requires node_id.0 < old(self).slots().len(),
        ensures final(self).slots() == old(self).slots().update(node_id.0 as int, Some(value)),
            final(self).slots().len() == old(self).slots().len())]
    pub fn insert(&mut self, node_id: NodeIndex, value: T) {
        self.0[node_id.0] = Some(value)
    }
}
}

verus! {
pub struct PeerId(u16);
pub struct Keychain {
    identity: PeerId,
    pks: BTreeMap<PeerId, PublicKey>,
    message_tag: sha256::Hash,
    keypair: Keypair,
}
impl Keychain {
    // Exact existing runtime observation; NO map-view/cardinality/key-order law.
    pub closed spec fn configured_len(&self) -> usize {
        vstd::std_specs::btree::spec_btree_map_len(&self.pks)
    }
}
}
#[verus_verify]
impl Keychain {
    #[verus_spec(r => requires index.0 < self.configured_len(),
        ensures r.slots().len() == self.configured_len(),
            r.slots()[index.0 as int] == Some(*signature),
            forall|j: int| 0 <= j < r.slots().len() && j != index.0 ==> #[trigger] r.slots()[j] == None,
            r.signers() == Set::<usize>::empty().insert(index.0),
            r.signers().len() == 1)]
    pub fn bootstrap_multi(
        &self,
        signature: &[u8; 64],
        index: aleph_bft::NodeIndex,
    ) -> aleph_bft::NodeMap<[u8; 64]> {
        let mut partial = aleph_bft::NodeMap::with_size(self.pks.len().into());

        partial.insert(index, *signature);

        partial
    }
}
