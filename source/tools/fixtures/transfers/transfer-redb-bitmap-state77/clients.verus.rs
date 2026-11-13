fn observe_set(bitmap: &mut U64GroupedBitmap, bit: u32, observed: u32)
    -> (result: (bool, bool))
    requires old(bitmap).valid(), bit < old(bitmap).len, observed < old(bitmap).len,
    ensures
        final(bitmap).valid(), final(bitmap).len == old(bitmap).len,
        final(bitmap).data@.len() == old(bitmap).data@.len(),
        result.0 == (final(bitmap).data@[bit as int / 64] == u64::MAX),
        result.1 == (if observed == bit { true } else { old(bitmap).live(observed as int) }),
{
    let full = bitmap.set(bit);
    (full, bitmap.get(observed))
}

fn observe_clear(bitmap: &mut U64GroupedBitmap, bit: u32, observed: u32)
    -> (result: bool)
    requires old(bitmap).valid(), bit < old(bitmap).len, observed < old(bitmap).len,
    ensures
        final(bitmap).valid(), final(bitmap).len == old(bitmap).len,
        final(bitmap).data@.len() == old(bitmap).data@.len(),
        result == (if observed == bit { false } else { old(bitmap).live(observed as int) }),
{
    bitmap.clear(bit);
    bitmap.get(observed)
}

fn padding_flag() -> (result: bool)
    ensures !result,
{
    let mask = U64GroupedBitmap::select_mask(0);
    assert((0u64 | mask) != u64::MAX);
    let mut data = Vec::new();
    data.push(0u64);
    let mut bitmap = U64GroupedBitmap { len: 1, data };
    bitmap.set(0)
}

fn spare_words() -> (result: bool)
    ensures result,
{
    let mask = U64GroupedBitmap::select_mask(0);
    assert((u64::MAX | mask) == u64::MAX);
    let mut data = Vec::new();
    data.push(0u64);
    data.push(u64::MAX);
    data.push(17u64);
    let mut bitmap = U64GroupedBitmap { len: 65, data };
    let full = bitmap.set(64);
    bitmap.clear(0);
    full && bitmap.get(64) && !bitmap.get(0)
}
