extern crate reference88;
use vstd::prelude::*;
use reference88::{live, resized, resize_read, set_get_clear, shrink_regrow, resize_update, word_count};

verus! {
pub fn imported_resize(data: Vec<u64>, old_len: u32, new_len: u32, full: bool, query: u32)
    -> (result: (Vec<u64>, bool, u32))
    requires old_len as int <= 64 * data@.len(), query < new_len,
    ensures resized(data@, old_len, result.0@, new_len, full), result.2 == new_len,
        result.1 == if old_len <= query { full } else { live(data@, query as int) },
{
    resize_read(data, old_len, new_len, full, query)
}

pub fn imported_access(data: Vec<u64>, length: u32, bit: u32) -> (result: (Vec<u64>, bool, bool))
    requires length as int <= 64 * data@.len(), bit < length,
    ensures result.1, !result.2, result.0@.len() == data@.len(),
        forall|query: int| 0 <= query < 64 * result.0@.len() ==>
            #[trigger] live(result.0@, query) == (query != bit && live(data@, query)),
{
    set_get_clear(data, length, bit)
}

pub fn imported_shrink_regrow(data: Vec<u64>, old_len: u32, small: u32, new_len: u32, full: bool, query: u32)
    -> (result: (Vec<u64>, bool, u32))
    requires old_len as int <= 64 * data@.len(), small <= old_len, small <= new_len, query < new_len,
    ensures resized(data@, small, result.0@, new_len, full), result.2 == new_len,
        result.1 == if small <= query { full } else { live(data@, query as int) },
{
    shrink_regrow(data, old_len, small, new_len, full, query)
}

pub fn imported_resize_update(data: Vec<u64>, old_len: u32, new_len: u32, full: bool, bit: u32)
    -> (result: (Vec<u64>, bool, bool, u32))
    requires old_len as int <= 64 * data@.len(), bit < new_len,
    ensures result.1, !result.2, result.3 == new_len,
        result.0@.len() == word_count(data@.len(), new_len),
        forall|query: int| 0 <= query < 64 * result.0@.len() ==>
            #[trigger] live(result.0@, query) ==
                (query != bit && (if old_len <= query < new_len { full }
                 else if query < 64 * data@.len() { live(data@, query) } else { full })),
{
    resize_update(data, old_len, new_len, full, bit)
}
}
