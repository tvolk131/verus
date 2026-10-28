//! Owned collection construction and round advancement without method contracts.
//! Copying is an explicit executable loop; the supplied logical view is Vec's.
use vstd::prelude::*;

#[verus_verify]
pub struct Snapshot<T: Copy> {
    slots: Vec<Option<T>>,
    epoch: u16,
}

#[verus_verify]
impl<T: Copy> Snapshot<T> {
    #[verus_infer]
    pub fn new(size: usize) -> Self {
        Self { slots: vec![None; size], epoch: 0 }
    }

    #[verus_infer]
    pub fn next(&self) -> Option<Self> {
        if self.epoch == u16::MAX {
            return None;
        }
        let mut slots = Vec::with_capacity(self.slots.len());
        let mut index = 0usize;
        while index < self.slots.len() {
            slots.push(self.slots[index]);
            index += 1;
        }
        Some(Self { slots, epoch: self.epoch + 1 })
    }
}

#[verus_infer]
pub fn filled(size: usize, value: u32) -> Vec<u32> {
    let mut result = Vec::new();
    while result.len() < size {
        result.push(value);
    }
    result
}

verus! {
fn initial_goal<T: Copy>(size: usize) -> (s: Snapshot<T>)
    ensures s.epoch == 0, s.slots@.len() == size,
        forall|i: int| 0 <= i < size ==> s.slots@[i].is_none(),
{ Snapshot::new(size) }

fn next_goal<T: Copy>(s: &Snapshot<T>) -> (result: Option<Snapshot<T>>)
    ensures result.is_none() == (s.epoch == u16::MAX),
        result.is_some() ==> result.unwrap().epoch == s.epoch + 1,
        result.is_some() ==> result.unwrap().slots@ == s.slots@,
{ s.next() }
}

fn main() {
    let first = Snapshot::<u64>::new(4);
    let next = first.next().unwrap();
    assert_eq!(next.epoch, 1);
    assert_eq!(next.slots, vec![None; 4]);
    let terminal = Snapshot { slots: vec![Some(7u64)], epoch: u16::MAX };
    assert!(terminal.next().is_none());
    assert_eq!(filled(3, 9), vec![9, 9, 9]);
    assert!(filled(0, 9).is_empty());
}
