use vstd::prelude::*;
use r2_api::CheckedBitmap;
verus! {
// Contract compatibility at the concrete set/clear call boundaries used by
// record_alloc_inner / free_inner / alloc. Their complete bodies are not checked.
fn record_alloc_membership(tree: &mut CheckedBitmap, page_number: u32)
    requires old(tree).valid(), page_number < old(tree).logical_len(),
        !old(tree).members().contains(page_number as int),
    ensures final(tree).valid(), final(tree).members().contains(page_number as int),
        final(tree).allocated_count() == old(tree).allocated_count() + 1,
{
    tree.mark_allocated(page_number);
}
fn free_membership(tree: &mut CheckedBitmap, page_number: u32)
    requires old(tree).valid(), page_number < old(tree).logical_len(),
        old(tree).members().contains(page_number as int),
    ensures final(tree).valid(), !final(tree).members().contains(page_number as int),
        final(tree).allocated_count() as int == old(tree).allocated_count() as int - 1,
{
    tree.mark_free(page_number);
}
fn repeat_alloc_membership(tree: &mut CheckedBitmap, page_number: u32)
    requires old(tree).valid(), page_number < old(tree).logical_len(),
        old(tree).members().contains(page_number as int),
    ensures final(tree).valid(), final(tree).members() == old(tree).members(),
        final(tree).allocated_count() == old(tree).allocated_count(),
{
    tree.mark_allocated(page_number);
}
}
