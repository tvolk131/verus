// Shared by native and ghost builds. These wrappers isolate the bounded
// operations supported by inference; the collector's invariant stays explicit.
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_infer)]
pub fn length<T>(v: &Vec<T>) -> usize { v.len() }

#[cfg_attr(verus_keep_ghost, verus_infer)]
pub fn read<T: Copy>(v: &Vec<T>, i: usize) -> T { v[i] }

#[cfg_attr(verus_keep_ghost, verus_infer(precision = "sound"))]
pub fn replace<T: Copy>(v: &mut Vec<T>, i: usize, value: T) { v[i] = value; }
