pub mod word_facts {
    use vstd::prelude::*;

    pub broadcast proof fn set_bit_frame(word: u64, changed: u64, observed: u64)
        requires changed < 64, observed < 64,
        ensures
            (#[trigger] ((word | (1u64 << changed)) & (1u64 << observed)) != 0)
                == (changed == observed || (word & (1u64 << observed)) != 0),
    {
        assert(((word | (1u64 << changed)) & (1u64 << observed) != 0)
            == (changed == observed || (word & (1u64 << observed)) != 0)) by(bit_vector)
            requires changed < 64, observed < 64;
    }

    pub broadcast proof fn clear_bit_frame(word: u64, changed: u64, observed: u64)
        requires changed < 64, observed < 64,
        ensures
            (#[trigger] ((word & !(1u64 << changed)) & (1u64 << observed)) != 0)
                == (changed != observed && (word & (1u64 << observed)) != 0),
    {
        assert(((word & !(1u64 << changed)) & (1u64 << observed) != 0)
            == (changed != observed && (word & (1u64 << observed)) != 0)) by(bit_vector)
            requires changed < 64, observed < 64;
    }
}

broadcast use {word_facts::set_bit_frame, word_facts::clear_bit_frame};
