use vstd::prelude::*;
use persistence94::*;

verus! {

pub proof fn reject_unflushed_write_as_durable() {
    let initial = seq![0u8];
    let before = clean_storage(initial);
    let after = written(before, 0, seq![1u8]);
    clean_storage_is_valid(initial);
    write_transition_exists(before, 0, seq![1u8]);
    assert(after.visible[0] == 1u8);
    assert(after.durable[0] == 0u8);
    assert(after.visible[0] == after.durable[0]);
}

pub proof fn reject_overwrite_of_protected_byte() {
    let initial = seq![0u8];
    let before = clean_storage(initial);
    let protected = set![0int];
    clean_storage_is_valid(initial);
    assert(protected_bytes(before, initial, protected));
    let after = written(before, 0, seq![1u8]);
    write_transition_exists(before, 0, seq![1u8]);
    assert(after.permitted[0].contains(1u8));
    assert(protected_bytes(after, initial, protected));
}

pub proof fn reject_process_restart_as_power_loss() {
    let before = clean_storage(seq![0u8]);
    let after = written(before, 0, seq![1u8]);
    clean_storage_is_valid(seq![0u8]);
    write_transition_exists(before, 0, seq![1u8]);
    assert(process_restart_from(after, after));
    assert(after.visible[0] == after.durable[0]);
}

}
