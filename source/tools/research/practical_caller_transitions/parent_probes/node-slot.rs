use vstd::prelude::*;
verus! {
// Original carriers; unused derives/codec impls are omitted from this probe.
pub struct NodeIndex(pub usize);
pub struct NodeMap<T>(Vec<Option<T>>);
impl<T> NodeMap<T> {
    pub closed spec fn slots(self) -> Seq<Option<T>> { self.0@ }
    pub fn insert(&mut self, node_id: NodeIndex, value: T)
        requires node_id.0 < old(self).slots().len(),
        ensures final(self).slots() == old(self).slots().update(node_id.0 as int, Some(value)),
            final(self).slots().len() == old(self).slots().len(),
    {
        self.0[node_id.0] = Some(value)
    }
}
}
