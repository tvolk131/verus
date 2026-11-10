#![crate_type = "lib"]

pub fn xor_key(ipad: &mut [u8; 128], key: &[u8]) {
    for (b_i, b_h) in ipad.iter_mut().zip(key) {
        *b_i ^= *b_h;
    }
}

// Both inputs are fields of the same allocation. Rust accepts disjoint borrows.
pub fn disjoint_same_allocation(storage: &mut ([u8;128],[u8;256])) {
    let (ipad,key) = storage;
    xor_key(ipad,key);
}
