use vstd::prelude::*;
use f2::{NumPeers, quorum};

#[verus_verify]
#[derive(Copy, Clone)]
pub struct NodeIndex(pub usize);
#[verus_verify]
pub struct NodeMap<T>(Vec<Option<T>>);

verus! {
impl<T> NodeMap<T> {
    // Direct observation inside the original defining type; no iterator model.
    pub closed spec fn slots(&self) -> Seq<Option<T>> { self.0@ }
    pub open spec fn signers(&self) -> Set<usize> { occupied(self.slots()) }
}
pub open spec fn occupied<T>(s: Seq<Option<T>>) -> Set<usize> {
    Set::from_finite_type(|i: usize| i < s.len() && s[i as int].is_some())
}
}
#[verus_verify]
impl<T> NodeMap<T> {
    #[verus_spec(r => requires node_id.0 < self.slots().len(),
        ensures match r { Some(v) => self.slots()[node_id.0 as int] == Some(*v),
                          None => self.slots()[node_id.0 as int] == None })]
    pub fn get(&self, node_id: NodeIndex) -> Option<&T> {
        self.0[node_id.0].as_ref()
    }

    #[verus_spec(requires node_id.0 < old(self).slots().len(),
        ensures final(self).slots() == old(self).slots().update(node_id.0 as int, Some(value)),
            final(self).slots().len() == old(self).slots().len())]
    pub fn insert(&mut self, node_id: NodeIndex, value: T) {
        self.0[node_id.0] = Some(value)
    }
}
verus! {
// Finite occupied-index set comes from the actual Vec observation, not a count premise.
pub proof fn occupied_finite<T>(s: Seq<Option<T>>)
    ensures occupied(s).finite(),
{
    // MANUAL_FINITE
}
pub proof fn occupied_update<T>(s: Seq<Option<T>>, i: usize, value: T)
    requires i < s.len(),
    ensures occupied(s.update(i as int, Some(value))) == occupied(s).insert(i),
        occupied(s.update(i as int, Some(value))).finite(),
        occupied(s.update(i as int, Some(value))).len() ==
            occupied(s).len() + if occupied(s).contains(i) { 0nat } else { 1nat },
{
    // MANUAL_UPDATE
}
// Executable client performs the ORIGINAL slot update and get, then observes bookkeeping.
// q comes from the ORIGINAL checked NumPeers::threshold; no count boolean mirror.
pub fn record_signature<T>(map: &mut NodeMap<T>, i: NodeIndex, signature: T, n: usize)
    requires n > 0, i.0 < old(map).slots().len(),
    ensures final(map).signers() == old(map).signers().insert(i.0),
        final(map).signers().finite(),
        final(map).signers().len() == old(map).signers().len() +
            if old(map).signers().contains(i.0) { 0nat } else { 1nat },
        forall|j: int| 0 <= j < old(map).slots().len() && j != i.0 ==>
            #[trigger] final(map).slots()[j] == old(map).slots()[j],
        old(map).signers().len() + 1 < quorum(n as int) ==> final(map).signers().len() < quorum(n as int),
        old(map).signers().contains(i.0) ==> (final(map).signers().len() >= quorum(n as int) <==>
            old(map).signers().len() >= quorum(n as int)),
        !old(map).signers().contains(i.0) && old(map).signers().len() + 1 == quorum(n as int)
            ==> final(map).signers().len() == quorum(n as int),
{
    let ghost before = map.slots();
    let peers = NumPeers::from(n);
    let q = peers.threshold();
    map.insert(i, signature);
    let stored = map.get(i);
    assert(stored == Some(&signature));
    proof {
        // MANUAL_CLIENT
    }
    assert(q == quorum(n as int));
}
}
