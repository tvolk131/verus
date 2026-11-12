verus! {
#[cfg(negative_leaf)]
fn wrong_set_membership(tree: &mut BtreeBitmap, i: u32)
    requires tree_wf(old(tree).heights@), i < old(tree).heights@.last().len,
    ensures !physical_bit(final(tree).heights@.last(),i as int),
{
    tree.set(i);
}
#[cfg(negative_count)]
fn wrong_repeat_count(tree: &mut BtreeBitmap, i: u32)
    requires tree_wf(old(tree).heights@), i < old(tree).heights@.last().len,
        physical_bit(old(tree).heights@.last(),i as int),
    ensures count(final(tree).heights@) == count(old(tree).heights@) + 1,
{
    tree.set(i);
}
#[cfg(negative_frame)]
fn wrong_clear_frame(tree: &mut BtreeBitmap, i: u32, j: u32)
    requires tree_wf(old(tree).heights@), i < old(tree).heights@.last().len,
        j < old(tree).heights@.last().len, j != i,
    ensures physical_bit(final(tree).heights@.last(),j as int) != physical_bit(old(tree).heights@.last(),j as int),
{
    tree.clear(i);
}
}
