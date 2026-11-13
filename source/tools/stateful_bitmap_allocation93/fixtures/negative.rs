extern crate initialized93;
use initialized93::*;
use vstd::prelude::*;
verus! {
pub fn negative_0() {
let result = new_initialized_allocations(2, 2);
assert(result.first == Some(1u32));
}
pub fn negative_1() {
let result = new_initialized_allocations(2, 2);
assert(result.first == result.second);
}
pub fn negative_2() {
let result = new_initialized_allocations(1, 130);
assert(result.second == Some(1u32));
}
pub fn negative_3() {
let mut tree = BtreeBitmap::new(2, 2);
let first = tree.alloc();
assert(first == Some(0u32));
}
pub fn negative_4() {
let result = new_initialized_allocations(65, 200);
assert(!bit(result.tree.levels().last().words(), 65));
}
pub fn negative_5() {
let result = new_initialized_allocations(65, 200);
assert(result.tree.levels().last().words()[3] != u64::MAX);
}
pub fn negative_6() {
let mut tree = BtreeBitmap::new(2, 2);
tree.clear(2);
}
pub fn negative_7() {
let result = new_initialized_allocations(1, 130);
assert(result.tree.levels() != result.after_first@);
}
pub fn excluded_raw_domain(pages:u32, capacity:u32)
    requires !constructor_domain(pages,capacity),
{
    let tree = BtreeBitmap::new(pages,capacity);
    assert(tree.invariant());
}
}
