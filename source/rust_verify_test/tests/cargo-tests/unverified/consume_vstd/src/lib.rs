pub fn foo(x: u64, y: u64, z: u64) {
    vstd_macro_usage::do_something(x, y, z);
}

/// Exercise executable finite iterators with all proof-only interfaces erased.
pub fn sparse_entries() -> Vec<(usize, u64)> {
    let slots = [Some(10), None, Some(30)];
    vstd::iter::SparseIter::new(&slots).map(|(i, value)| (i, *value)).collect()
}
