use vstd::prelude::*;
verus! {
// Original carriers; unused derives/codec impls are omitted from this probe.
pub struct NodeIndex(pub usize);
pub struct NodeMap<T>(Vec<Option<T>>);
impl<T> NodeMap<T> {
    pub fn insert(&mut self, node_id: NodeIndex, value: T)
        requires node_id.0 < old(self).0@.len(),
        ensures final(self).0@ == old(self).0@.update(node_id.0 as int, Some(value)),
            final(self).0@.len() == old(self).0@.len(),
    {
        self.0[node_id.0] = Some(value)
    }
}
}
