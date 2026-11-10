use vstd::prelude::*;
#[path="../kernel-v7/kernel.rs"] mod kernel;
use kernel::*;
verus! {

pub proof fn rejected() ensures window_valid(Window{allocation_base:4096,allocation_size:128,array_base:4097}), {}
}
