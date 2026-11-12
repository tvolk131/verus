use vstd::prelude::*;
#[verus_verify]
#[derive(Copy, Clone)]
pub struct NodeIndex(pub usize);
#[verus_verify]
pub struct NodeMap<T>(Vec<Option<T>>);
#[verus_verify]
impl<T> NodeMap<T> {
    pub fn iter(&self) -> impl Iterator<Item = (NodeIndex, &T)> {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(idx, maybe_value)| Some((NodeIndex(idx), maybe_value.as_ref()?)))
    }

}

#[verus_verify]
impl<T> NodeMap<T> {
    pub fn item_count(&self) -> usize {
        self.iter().count()
    }
}
