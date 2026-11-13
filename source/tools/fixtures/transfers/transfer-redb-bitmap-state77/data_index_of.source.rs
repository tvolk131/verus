    fn data_index_of(bit: u32) -> (usize, usize) {
        ((bit as usize) / 64, (bit as usize) % 64)
    }
