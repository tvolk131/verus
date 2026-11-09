extern crate initialized93;
use initialized93::*;
use vstd::prelude::*;

verus! {
pub fn imported_lifecycle(pages: u32, capacity: u32) -> (result: InitializedRun)
    requires constructor_domain(pages, capacity),
    ensures result.tree.invariant(),
        result.first == (if pages == 0 { None } else { Some(0u32) }),
        result.second == (if pages < 2 { None } else { Some(1u32) }),
        pages >= 2 ==> result.first.is_some() && result.second.is_some()
            && result.first.unwrap() != result.second.unwrap(),
        clear_trace_prefix(result.clear_trace@, pages as int),
        allocation_step(result.initialized@, result.after_first@, result.first),
        allocation_step(result.after_first@, result.tree.levels(), result.second),
        physical_frame(result.clear_trace@[0], result.tree.levels()),
{
    new_initialized_allocations(pages, capacity)
}

pub fn imported_clear_frames(tree: &mut BtreeBitmap, entry: u32, height: u32, word: usize, query: usize)
    requires old(tree).invariant(), entry < old(tree).levels().last().model_len(),
        height < old(tree).levels().len(), word < old(tree).levels()[height as int].words().len(),
        word as int != path(entry as int, old(tree).levels().len() as int, height as int) / 64,
        query < 64 * old(tree).levels()[height as int].words().len(),
        query as int != path(entry as int, old(tree).levels().len() as int, height as int),
    ensures final(tree).invariant(),
        edit_effect(old(tree).levels(), final(tree).levels(), entry as int, false),
        !bit(final(tree).levels().last().words(), entry as int),
        physical_frame(old(tree).levels(), final(tree).levels()),
        final(tree).levels()[height as int].words()[word as int] == old(tree).levels()[height as int].words()[word as int],
        bit(final(tree).levels()[height as int].words(), query as int) ==
            bit(old(tree).levels()[height as int].words(), query as int),
{
    let ghost before = tree.levels();
    tree.clear(entry);
    proof { effect_frame(before, tree.levels(), entry as int, false); }
}

pub fn imported_no_initialization(pages: u32, capacity: u32) -> (result: BtreeBitmap)
    requires constructor_domain(pages, capacity),
    ensures result.invariant(), result.levels().last().model_len() == pages,
        prefix_full(result.levels().last(), pages as int),
{
    let mut tree = BtreeBitmap::new(pages, capacity);
    let ghost before = tree;
    let first = tree.alloc();
    let second = tree.alloc();
    assert(first.is_none() && second.is_none());
    assert(tree == before);
    tree
}

pub fn imported_empty() -> (result: InitializedRun)
    ensures result.first == None::<u32>, result.second == None::<u32>,
        result.tree.invariant(), result.clear_trace@.len() == 1,
        result.tree.levels().last().words().len() == 0,
{
    new_initialized_allocations(0, 0)
}

pub fn imported_empty_spare() -> (result: InitializedRun)
    ensures result.first == None::<u32>, result.second == None::<u32>,
        result.tree.invariant(), result.tree.levels().last().words().len() == 3,
        result.tree.levels().last().words()[2] == u64::MAX,
{
    new_initialized_allocations(0, 130)
}

pub fn imported_singleton_spare() -> (result: InitializedRun)
    ensures result.first == Some(0u32), result.second == None::<u32>,
        result.tree.invariant(), result.tree.levels() == result.after_first@,
        result.tree.levels().last().words()[2] == u64::MAX,
        bit(result.tree.levels().last().words(), 1),
{
    let result = new_initialized_allocations(1, 130);
    proof { full_word(result.clear_trace@[0].last().words(), 1); }
    result
}

pub fn imported_two_pages() -> (result: InitializedRun)
    ensures result.first == Some(0u32), result.second == Some(1u32),
        result.first != result.second, result.tree.invariant(),
        prefix_full(result.initialized@.last(), result.first.unwrap() as int),
        prefix_full(result.after_first@.last(), result.second.unwrap() as int),
        set_effect(result.initialized@, result.after_first@, 0),
        set_effect(result.after_first@, result.tree.levels(), 1),
{
    new_initialized_allocations(2, 2)
}

pub fn imported_padding_spare() -> (result: InitializedRun)
    ensures result.first == Some(0u32), result.second == Some(1u32),
        result.tree.invariant(), result.tree.levels().last().words().len() == 4,
        bit(result.tree.levels().last().words(), 0), bit(result.tree.levels().last().words(), 1),
        !bit(result.tree.levels().last().words(), 2), !bit(result.tree.levels().last().words(), 64),
        bit(result.tree.levels().last().words(), 65), bit(result.tree.levels().last().words(), 127),
        result.tree.levels().last().words()[3] == u64::MAX,
{
    let result = new_initialized_allocations(65, 200);
    proof {
        full_word(result.clear_trace@[0].last().words(), 65);
        full_word(result.clear_trace@[0].last().words(), 127);
    }
    result
}

pub fn imported_rounded_capacity() -> (result: InitializedRun)
    ensures result.first == Some(0u32), result.second == Some(1u32),
        result.tree.invariant(), result.tree.levels().last().model_len() == 64,
        result.tree.levels().last().words().len() == 1,
{
    new_initialized_allocations(64, 1)
}

pub fn imported_named_maximum() -> (result: InitializedRun)
    ensures result.first == Some(0u32), result.second == Some(1u32),
        result.tree.invariant(), result.tree.levels().last().model_len() == MAX_PAGE_INDEX + 1,
        result.tree.levels().last().words().len() == 67108864,
{
    new_initialized_allocations(MAX_PAGE_INDEX + 1, u32::MAX)
}
}
