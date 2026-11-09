use vstd::prelude::*;
use persistence94::*;

verus! {

pub struct FileImage {
    pub bytes: StorageImage,
    pub durable_len: int,
    pub visible_len: int,
    pub permitted_lengths: Set<int>,
}

pub open spec fn valid_file(state: FileImage) -> bool {
    &&& valid_storage(state.bytes)
    &&& state.permitted_lengths.contains(state.durable_len)
    &&& state.permitted_lengths.contains(state.visible_len)
    &&& forall|length: int| #[trigger] state.permitted_lengths.contains(length) ==>
        0 <= length <= state.bytes.visible.len()
}

pub open spec fn clean_file(bytes: Seq<u8>, length: int) -> FileImage {
    FileImage { bytes: clean_storage(bytes), durable_len: length, visible_len: length,
        permitted_lengths: Set::empty().insert(length) }
}

pub open spec fn intermediate_lengths(old_length: int, new_length: int) -> Set<int> {
    Set::range(if old_length <= new_length { old_length } else { new_length },
        if old_length <= new_length { new_length + 1 } else { old_length + 1 })
}

pub proof fn intermediate_length_membership(old_length: int, new_length: int)
    ensures forall|length: int| #[trigger] intermediate_lengths(old_length, new_length).contains(length)
        <==> old_length <= length <= new_length || new_length <= length <= old_length,
{
    vstd::set_lib::lemma_int_range(
        if old_length <= new_length { old_length } else { new_length },
        if old_length <= new_length { new_length + 1 } else { old_length + 1 });
}

pub open spec fn resize_issued(before: FileImage, after: FileImage, length: int) -> bool {
    &&& 0 <= length <= before.bytes.visible.len()
    &&& after.durable_len == before.durable_len
    &&& after.visible_len == length
    &&& after.permitted_lengths == before.permitted_lengths.union(
        intermediate_lengths(before.visible_len, length))
    &&& if before.visible_len < length {
        issue_write(before.bytes, after.bytes, before.visible_len,
            Seq::new((length - before.visible_len) as nat, |index: int| 0u8))
    } else {
        after.bytes == before.bytes
    }
}

pub open spec fn in_bounds_write(before: FileImage, after: FileImage, offset: int, bytes: Seq<u8>) -> bool {
    &&& 0 <= offset <= before.visible_len
    &&& offset + bytes.len() <= before.visible_len
    &&& issue_write(before.bytes, after.bytes, offset, bytes)
    &&& after.durable_len == before.durable_len
    &&& after.visible_len == before.visible_len
    &&& after.permitted_lengths == before.permitted_lengths
}

pub open spec fn file_persistence(before: FileImage, after: FileImage) -> bool {
    &&& persist_some(before.bytes, after.bytes)
    &&& before.permitted_lengths.contains(after.durable_len)
    &&& after.visible_len == before.visible_len
    &&& after.permitted_lengths == before.permitted_lengths
}

pub open spec fn file_sync(before: FileImage, after: FileImage) -> bool {
    &&& sync_success(before.bytes, after.bytes)
    &&& after.durable_len == before.visible_len
    &&& after.visible_len == before.visible_len
    &&& after.permitted_lengths == Set::empty().insert(before.visible_len)
}

pub open spec fn file_power_restart(before: FileImage, after: FileImage) -> bool {
    &&& power_restart_from(before.bytes, after.bytes)
    &&& after.durable_len == before.durable_len
    &&& after.visible_len == before.durable_len
    &&& after.permitted_lengths == Set::empty().insert(before.durable_len)
}

pub open spec fn live_protection(state: FileImage, original: Seq<u8>, selected: Set<int>) -> bool {
    &&& protected_bytes(state.bytes, original, selected)
    &&& forall|length: int, index: int| state.permitted_lengths.contains(length)
        && selected.contains(index) ==> 0 <= index < length
}

pub proof fn clean_file_is_valid(bytes: Seq<u8>, length: int)
    requires 0 <= length <= bytes.len(),
    ensures valid_file(clean_file(bytes, length)),
{
    clean_storage_is_valid(bytes);
}

pub proof fn resize_preserves_file(before: FileImage, after: FileImage, length: int)
    requires valid_file(before), resize_issued(before, after, length),
    ensures valid_file(after),
{
    intermediate_length_membership(before.visible_len, length);
    if before.visible_len < length {
        write_preserves_storage(before.bytes, after.bytes, before.visible_len,
            Seq::new((length - before.visible_len) as nat, |index: int| 0u8));
    }
}

pub proof fn in_bounds_write_preserves_file(before: FileImage, after: FileImage, offset: int, bytes: Seq<u8>)
    requires valid_file(before), in_bounds_write(before, after, offset, bytes),
    ensures valid_file(after),
{
    write_preserves_storage(before.bytes, after.bytes, offset, bytes);
}

pub proof fn persistence_preserves_file(before: FileImage, after: FileImage)
    requires valid_file(before), file_persistence(before, after),
    ensures valid_file(after),
{
    persistence_preserves_storage(before.bytes, after.bytes);
}

pub proof fn sync_preserves_file(before: FileImage, after: FileImage)
    requires valid_file(before), file_sync(before, after),
    ensures valid_file(after),
{
    sync_preserves_storage(before.bytes, after.bytes);
}

pub proof fn restart_preserves_file(before: FileImage, after: FileImage)
    requires valid_file(before), file_power_restart(before, after),
    ensures valid_file(after),
{
    power_restart_preserves_storage(before.bytes, after.bytes);
}

pub proof fn resize_existence(before: FileImage, length: int)
    requires valid_file(before), 0 <= length <= before.bytes.visible.len(),
    ensures exists|after: FileImage| resize_issued(before, after, length) && valid_file(after),
{
    intermediate_length_membership(before.visible_len, length);
    let next_bytes = if before.visible_len < length {
        let zeros = Seq::new((length - before.visible_len) as nat, |index: int| 0u8);
        write_transition_exists(before.bytes, before.visible_len, zeros);
        written(before.bytes, before.visible_len, zeros)
    } else { before.bytes };
    let after = FileImage { bytes: next_bytes, durable_len: before.durable_len, visible_len: length,
        permitted_lengths: before.permitted_lengths.union(intermediate_lengths(before.visible_len, length)) };
    resize_preserves_file(before, after, length);
    assert(resize_issued(before, after, length) && valid_file(after));
}

pub proof fn sync_existence(before: FileImage)
    requires valid_file(before),
    ensures file_sync(before, clean_file(before.bytes.visible, before.visible_len)),
        valid_file(clean_file(before.bytes.visible, before.visible_len)),
{
    sync_transition_exists(before.bytes);
    clean_file_is_valid(before.bytes.visible, before.visible_len);
}

pub proof fn successful_sync_publishes_bytes_and_length(before: FileImage, after: FileImage)
    requires valid_file(before), file_sync(before, after),
    ensures after == clean_file(before.bytes.visible, before.visible_len),
        after.bytes.durable.subrange(0, after.durable_len)
            == before.bytes.visible.subrange(0, before.visible_len),
{
    successful_sync_has_clean_image(before.bytes, after.bytes);
}

pub proof fn resize_growth_zeroes_visible_extension(before: FileImage, after: FileImage, length: int)
    requires valid_file(before), resize_issued(before, after, length), before.visible_len < length,
    ensures forall|index: int| before.visible_len <= index < length ==>
        #[trigger] after.bytes.visible[index] == 0u8,
{
}

pub proof fn live_selected_bytes_survive(state: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(state), live_protection(state, original, selected),
    ensures forall|index: int| selected.contains(index) ==> 0 <= index < state.durable_len
        && index < state.visible_len && #[trigger] state.bytes.durable[index] == original[index]
        && state.bytes.visible[index] == original[index],
{
    protected_bytes_are_stable(state.bytes, original, selected);
    assert forall|index: int| selected.contains(index) implies 0 <= index < state.durable_len
        && index < state.visible_len && #[trigger] state.bytes.durable[index] == original[index]
        && state.bytes.visible[index] == original[index] by {
        assert(state.permitted_lengths.contains(state.durable_len));
        assert(state.permitted_lengths.contains(state.visible_len));
    }
}

pub proof fn safe_resize_preserves_live_protection(
    before: FileImage, after: FileImage, length: int, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected),
        resize_issued(before, after, length),
        forall|index: int| selected.contains(index) ==> index < length,
    ensures valid_file(after), live_protection(after, original, selected),
{
    intermediate_length_membership(before.visible_len, length);
    resize_preserves_file(before, after, length);
    live_selected_bytes_survive(before, original, selected);
    if before.visible_len < length {
        disjoint_write_preserves_protection(before.bytes, after.bytes, before.visible_len,
            Seq::new((length - before.visible_len) as nat, |index: int| 0u8), original, selected);
    }
}

pub proof fn compatible_write_preserves_live_protection(
    before: FileImage, after: FileImage, offset: int, bytes: Seq<u8>, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected),
        in_bounds_write(before, after, offset, bytes),
        write_compatible_with_protection(before.bytes, offset, bytes, selected),
    ensures valid_file(after), live_protection(after, original, selected),
{
    in_bounds_write_preserves_file(before, after, offset, bytes);
    compatible_write_preserves_protection(before.bytes, after.bytes, offset, bytes, original, selected);
}

pub proof fn persistence_preserves_live_protection(
    before: FileImage, after: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected), file_persistence(before, after),
    ensures valid_file(after), live_protection(after, original, selected),
{
    persistence_preserves_file(before, after);
    persistence_preserves_protection(before.bytes, after.bytes, original, selected);
}

pub proof fn sync_preserves_live_protection(
    before: FileImage, after: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected), file_sync(before, after),
    ensures valid_file(after), live_protection(after, original, selected),
{
    sync_preserves_file(before, after);
    sync_preserves_protection(before.bytes, after.bytes, original, selected);
    assert(before.permitted_lengths.contains(before.visible_len));
}

pub proof fn restart_preserves_live_protection(
    before: FileImage, after: FileImage, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected), file_power_restart(before, after),
    ensures valid_file(after), live_protection(after, original, selected),
{
    restart_preserves_file(before, after);
    power_restart_preserves_protection(before.bytes, after.bytes, original, selected);
    assert(before.permitted_lengths.contains(before.durable_len));
}

pub open spec fn partially_issued_bytes(
    before: FileImage, offset: int, requested: Seq<u8>, touched: Set<int>) -> Seq<u8> {
    Seq::new(requested.len(), |index: int|
        if touched.contains(index) { requested[index] } else { before.bytes.visible[offset + index] })
}

pub enum FileStep {
    Write { offset: int, bytes: Seq<u8> },
    InterruptedWrite { offset: int, requested: Seq<u8>, touched: Set<int> },
    Resize { length: int },
    Persist,
    Sync,
    FailedSync,
    PowerRestart,
    ProcessRestart,
}

pub open spec fn allowed_file_step(
    before: FileImage, after: FileImage, step: FileStep, selected: Set<int>) -> bool {
    match step {
        FileStep::Write { offset, bytes } => in_bounds_write(before, after, offset, bytes)
            && write_compatible_with_protection(before.bytes, offset, bytes, selected),
        FileStep::InterruptedWrite { offset, requested, touched } =>
            in_bounds_write(before, after, offset, partially_issued_bytes(before, offset, requested, touched))
            && write_compatible_with_protection(before.bytes, offset, requested, selected),
        FileStep::Resize { length } => resize_issued(before, after, length)
            && forall|index: int| selected.contains(index) ==> index < length,
        FileStep::Persist => file_persistence(before, after),
        FileStep::Sync => file_sync(before, after),
        FileStep::FailedSync => file_persistence(before, after),
        FileStep::PowerRestart => file_power_restart(before, after),
        FileStep::ProcessRestart => after == before,
    }
}

pub proof fn step_preserves_live_protection(
    before: FileImage, after: FileImage, step: FileStep, original: Seq<u8>, selected: Set<int>)
    requires valid_file(before), live_protection(before, original, selected),
        allowed_file_step(before, after, step, selected),
    ensures valid_file(after), live_protection(after, original, selected),
{
    match step {
        FileStep::Write { offset, bytes } => {
            compatible_write_preserves_live_protection(before, after, offset, bytes, original, selected);
        },
        FileStep::InterruptedWrite { offset, requested, touched } => {
            let partial = partially_issued_bytes(before, offset, requested, touched);
            assert forall|index: int| selected.contains(index) && offset <= index < offset + partial.len()
                implies #[trigger] partial[index - offset] == before.bytes.visible[index] by {
                if touched.contains(index - offset) {
                    assert(requested[index - offset] == before.bytes.visible[index]);
                }
            }
            compatible_write_preserves_live_protection(before, after, offset, partial, original, selected);
        },
        FileStep::Resize { length } => {
            safe_resize_preserves_live_protection(before, after, length, original, selected);
        },
        FileStep::Persist | FileStep::FailedSync => {
            persistence_preserves_live_protection(before, after, original, selected);
        },
        FileStep::Sync => {
            sync_preserves_live_protection(before, after, original, selected);
        },
        FileStep::PowerRestart => {
            restart_preserves_live_protection(before, after, original, selected);
        },
        FileStep::ProcessRestart => {},
    }
}

pub open spec fn allowed_file_trace(states: Seq<FileImage>, steps: Seq<FileStep>, selected: Set<int>) -> bool {
    &&& states.len() == steps.len() + 1
    &&& forall|position: int| 0 <= position < steps.len() ==>
        #[trigger] allowed_file_step(states[position], states[position + 1], steps[position], selected)
}

pub proof fn trace_preserves_live_protection(
    states: Seq<FileImage>, steps: Seq<FileStep>, original: Seq<u8>, selected: Set<int>, count: nat)
    requires allowed_file_trace(states, steps, selected), count <= steps.len(),
        valid_file(states[0]), live_protection(states[0], original, selected),
    ensures valid_file(states[count as int]), live_protection(states[count as int], original, selected),
    decreases count,
{
    if count > 0 {
        trace_preserves_live_protection(states, steps, original, selected, (count - 1) as nat);
        let position = count as int - 1;
        assert(allowed_file_step(states[position], states[position + 1], steps[position], selected));
        step_preserves_live_protection(states[position], states[position + 1], steps[position], original, selected);
    }
}

}
