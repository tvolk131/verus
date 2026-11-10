#![feature(freeze, unsafe_unpin)]
#![allow(dead_code)]

use std::marker::{Freeze, PhantomPinned, UnsafeUnpin};

fn require<T: ?Sized + Unpin + UnsafeUnpin + Freeze>() {}

// No where-clause supplies the result: rustc must establish it for arbitrary N.
pub fn all_array_traits<const N: usize>() {
    require::<[u8; N]>();
}

pub fn slice_traits() {
    require::<[u8]>();
}

pub fn probe_byte(_: u8) {}
pub fn probe_empty(_: &mut [u8; 0]) {}
pub fn probe_ten(_: &mut [u8; 10]) {}
pub fn probe_pad(_: &mut [u8; 128]) {}
pub fn probe_slice(_: &[u8]) {}
pub fn probe_pinned(_: &mut PhantomPinned) {}
pub fn probe_cell(_: &mut std::cell::UnsafeCell<u8>) {}

const _: () = {
    assert!(std::mem::size_of::<u8>() == 1);
    assert!(std::mem::align_of::<u8>() == 1);
    assert!(std::mem::size_of::<[u8; 10]>() == 10);
    assert!(std::mem::align_of::<[u8; 10]>() == 1);
    assert!(std::mem::size_of::<[u8; 128]>() == 128);
    assert!(std::mem::align_of::<[u8; 128]>() == 1);
};
