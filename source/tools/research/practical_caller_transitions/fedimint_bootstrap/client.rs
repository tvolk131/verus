use vstd::prelude::*;
use f3c_bootstrap::{Keychain, aleph_bft::{NodeMap, NodeIndex, NodeCount}};
verus! {
pub fn initializer_observation(keychain: &Keychain, signature: &[u8;64], index: usize)
    requires index < keychain.configured_len(),
{
    let partial = keychain.bootstrap_multi(signature, NodeIndex(index));
    assert(partial.slots().len() == keychain.configured_len());
    assert(partial.slots()[index as int] == Some(*signature));
    assert forall|j: int| 0 <= j < partial.slots().len() && j != index implies
        #[trigger] partial.slots()[j] == None by { }
    assert(partial.signers() == Set::<usize>::empty().insert(index));
    assert(partial.signers().len() == 1);
}
pub fn zero_constructor_and_conversion_boundaries() {
    let empty = NodeMap::<[u8;64]>::with_size(NodeCount::from(0usize));
    assert(empty.slots().len() == 0);
    assert(empty.signers() == Set::<usize>::empty());
    let highest: usize = NodeCount::from(usize::MAX).into();
    assert(highest == usize::MAX);
}
}
