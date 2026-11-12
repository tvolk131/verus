use vstd::prelude::*;
use f3_manual::{NodeIndex, NodeMap};
use f2::{NumPeers, quorum};
verus! {
pub fn retransmission(map: NodeMap<[u8;64]>, i: usize, j: usize,
    first: [u8;64], second: [u8;64], other: [u8;64], n: usize)
    requires i < map.slots().len(), j < map.slots().len(), i != j, n > 0,
{
    let ghost before = map.slots();
    let ghost signers_before = map.signers();
    let q = NumPeers::from(n).threshold();
    let map = map.add_signature(&first, NodeIndex(i));
    let ghost after_first = map.signers();
    let map = map.add_signature(&second, NodeIndex(i));
    assert(map.signers() == after_first);
    assert(map.signers().len() == after_first.len()); // DUPLICATE_CONTROL
    assert(map.slots()[i as int] == Some(second));
    let map = map.add_signature(&other, NodeIndex(j));
    assert(map.signers() == signers_before.insert(i).insert(j));
    assert(map.slots()[i as int] == Some(second));
    assert(map.slots()[j as int] == Some(other));
    assert forall|k: int| 0 <= k < before.len() && k != i && k != j implies
        #[trigger] map.slots()[k] == before[k] by { }
    assert(q == quorum(n as int));
    assert(signers_before.len() + 2 < q ==> map.signers().len() < q);
}
}
