use vstd::prelude::*;

verus! {

pub struct StorageImage {
    pub durable: Seq<u8>,
    pub visible: Seq<u8>,
    pub permitted: Map<int, Set<u8>>,
}

pub open spec fn valid_storage(state: StorageImage) -> bool {
    &&& state.durable.len() == state.visible.len()
    &&& forall|offset: int| #[trigger] state.permitted.dom().contains(offset) <==>
        0 <= offset < state.visible.len()
    &&& forall|offset: int| 0 <= offset < state.visible.len() ==>
        #[trigger] state.permitted[offset].contains(state.durable[offset])
            && state.permitted[offset].contains(state.visible[offset])
}

pub open spec fn overwrite(before: Seq<u8>, offset: int, bytes: Seq<u8>) -> Seq<u8> {
    Seq::new(before.len(), |index: int|
        if offset <= index < offset + bytes.len() { bytes[index - offset] } else { before[index] })
}

pub open spec fn clean_storage(bytes: Seq<u8>) -> StorageImage {
    StorageImage { durable: bytes, visible: bytes,
        permitted: Map::new(Set::range(0, bytes.len() as int), |index: int| Set::empty().insert(bytes[index])) }
}

pub open spec fn issue_write(before: StorageImage, after: StorageImage, offset: int, bytes: Seq<u8>) -> bool {
    &&& 0 <= offset <= before.visible.len()
    &&& offset + bytes.len() <= before.visible.len()
    &&& after.durable == before.durable
    &&& after.visible == overwrite(before.visible, offset, bytes)
    &&& after.permitted == Map::new(before.permitted.dom(), |index: int|
        if offset <= index < offset + bytes.len() {
            before.permitted[index].insert(bytes[index - offset])
        } else { before.permitted[index] })
}

pub open spec fn written(before: StorageImage, offset: int, bytes: Seq<u8>) -> StorageImage {
    StorageImage { durable: before.durable, visible: overwrite(before.visible, offset, bytes),
        permitted: Map::new(before.permitted.dom(), |index: int|
            if offset <= index < offset + bytes.len() {
                before.permitted[index].insert(bytes[index - offset])
            } else { before.permitted[index] }) }
}

pub open spec fn persist_some(before: StorageImage, after: StorageImage) -> bool {
    &&& after.visible == before.visible
    &&& after.permitted == before.permitted
    &&& after.durable.len() == before.durable.len()
    &&& forall|offset: int| 0 <= offset < after.durable.len() ==>
        #[trigger] before.permitted[offset].contains(after.durable[offset])
}

pub open spec fn sync_success(before: StorageImage, after: StorageImage) -> bool {
    &&& after.durable == before.visible
    &&& after.visible == before.visible
    &&& after.permitted == Map::new(before.permitted.dom(), |index: int| Set::empty().insert(before.visible[index]))
}

pub open spec fn power_restart_from(before: StorageImage, after: StorageImage) -> bool {
    &&& after.durable == before.durable
    &&& after.visible == before.durable
    &&& after.permitted == Map::new(before.permitted.dom(), |index: int| Set::empty().insert(before.durable[index]))
}

pub open spec fn process_restart_from(before: StorageImage, after: StorageImage) -> bool {
    &&& after.durable == before.durable
    &&& after.visible == before.visible
    &&& after.permitted == before.permitted
}

pub open spec fn protected_bytes(state: StorageImage, original: Seq<u8>, protected: Set<int>) -> bool {
    &&& original.len() == state.visible.len()
    &&& forall|index: int| #[trigger] protected.contains(index) ==> 0 <= index < original.len()
    &&& forall|index: int| protected.contains(index) ==>
        #[trigger] state.permitted[index] == Set::empty().insert(original[index])
}

pub proof fn clean_storage_is_valid(bytes: Seq<u8>)
    ensures valid_storage(clean_storage(bytes)),
{
    vstd::set_lib::lemma_int_range(0, bytes.len() as int);
    assert forall|index: int| #[trigger] clean_storage(bytes).permitted.dom().contains(index)
        <==> 0 <= index < bytes.len() by {}
    assert forall|index: int| 0 <= index < bytes.len() implies
        #[trigger] clean_storage(bytes).permitted[index].contains(bytes[index]) by {}
}

pub proof fn write_transition_exists(before: StorageImage, offset: int, bytes: Seq<u8>)
    requires valid_storage(before), 0 <= offset <= before.visible.len(),
        offset + bytes.len() <= before.visible.len(),
    ensures issue_write(before, written(before, offset, bytes), offset, bytes),
        valid_storage(written(before, offset, bytes)),
{
    write_preserves_storage(before, written(before, offset, bytes), offset, bytes);
}

pub proof fn sync_transition_exists(before: StorageImage)
    requires valid_storage(before),
    ensures sync_success(before, clean_storage(before.visible)), valid_storage(clean_storage(before.visible)),
{
    vstd::set_lib::lemma_int_range(0, before.visible.len() as int);
    assert(before.permitted.dom() =~= Set::range(0, before.visible.len() as int));
    clean_storage_is_valid(before.visible);
}

pub proof fn successful_sync_has_clean_image(before: StorageImage, after: StorageImage)
    requires valid_storage(before), sync_success(before, after),
    ensures after == clean_storage(before.visible),
{
    vstd::set_lib::lemma_int_range(0, before.visible.len() as int);
    assert(before.permitted.dom() =~= Set::range(0, before.visible.len() as int));
}

pub proof fn clean_image_protects_every_selected_byte(bytes: Seq<u8>, selected: Set<int>)
    requires forall|index: int| #[trigger] selected.contains(index) ==> 0 <= index < bytes.len(),
    ensures protected_bytes(clean_storage(bytes), bytes, selected),
{
    clean_storage_is_valid(bytes);
    assert forall|index: int| selected.contains(index) implies
        #[trigger] clean_storage(bytes).permitted[index] == Set::empty().insert(bytes[index]) by {}
}

pub proof fn write_preserves_storage(before: StorageImage, after: StorageImage, offset: int, bytes: Seq<u8>)
    requires valid_storage(before), issue_write(before, after, offset, bytes),
    ensures valid_storage(after),
{
    assert forall|index: int| 0 <= index < after.visible.len() implies
        #[trigger] after.permitted[index].contains(after.durable[index])
            && after.permitted[index].contains(after.visible[index]) by {
        assert(before.permitted[index].contains(before.durable[index]));
        assert(before.permitted[index].contains(before.visible[index]));
    }
}

pub proof fn persistence_preserves_storage(before: StorageImage, after: StorageImage)
    requires valid_storage(before), persist_some(before, after),
    ensures valid_storage(after),
{
    assert forall|index: int| 0 <= index < after.visible.len() implies
        #[trigger] after.permitted[index].contains(after.durable[index])
            && after.permitted[index].contains(after.visible[index]) by {
        assert(before.permitted[index].contains(before.durable[index]));
        assert(before.permitted[index].contains(after.durable[index]));
        assert(before.permitted[index].contains(before.visible[index]));
    }
}

pub proof fn sync_preserves_storage(before: StorageImage, after: StorageImage)
    requires valid_storage(before), sync_success(before, after),
    ensures valid_storage(after), after.durable == before.visible,
{
    assert forall|index: int| 0 <= index < after.visible.len() implies
        #[trigger] after.permitted[index].contains(after.durable[index])
            && after.permitted[index].contains(after.visible[index]) by {}
}

pub proof fn power_restart_preserves_storage(before: StorageImage, after: StorageImage)
    requires valid_storage(before), power_restart_from(before, after),
    ensures valid_storage(after), after.visible == before.durable,
{
    assert forall|index: int| 0 <= index < after.visible.len() implies
        #[trigger] after.permitted[index].contains(after.durable[index])
            && after.permitted[index].contains(after.visible[index]) by {}
}

pub proof fn process_restart_preserves_storage(before: StorageImage, after: StorageImage)
    requires valid_storage(before), process_restart_from(before, after),
    ensures valid_storage(after), after.visible == before.visible, after.durable == before.durable,
{}

pub proof fn protected_bytes_are_stable(state: StorageImage, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(state), protected_bytes(state, original, protected),
    ensures
        forall|index: int| protected.contains(index) ==> #[trigger] state.durable[index] == original[index],
        forall|index: int| protected.contains(index) ==> #[trigger] state.visible[index] == original[index],
{
    assert forall|index: int| protected.contains(index) implies
        #[trigger] state.durable[index] == original[index] by {
        assert(state.permitted[index].contains(state.durable[index]));
    }
    assert forall|index: int| protected.contains(index) implies
        #[trigger] state.visible[index] == original[index] by {
        assert(state.permitted[index].contains(state.durable[index]));
        assert(state.permitted[index].contains(state.visible[index]));
    }
}

pub proof fn disjoint_write_preserves_protection(
    before: StorageImage, after: StorageImage, offset: int, bytes: Seq<u8>, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected),
        issue_write(before, after, offset, bytes),
        forall|index: int| protected.contains(index) ==> !(offset <= index < offset + bytes.len()),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    write_preserves_storage(before, after, offset, bytes);
    assert forall|index: int| protected.contains(index) implies
        #[trigger] after.permitted[index] == Set::empty().insert(original[index]) by {
        assert(!(offset <= index < offset + bytes.len()));
        assert(before.permitted[index] == Set::empty().insert(original[index]));
    }
}

pub open spec fn write_compatible_with_protection(
    before: StorageImage, offset: int, bytes: Seq<u8>, protected: Set<int>) -> bool {
    forall|index: int| protected.contains(index) && offset <= index < offset + bytes.len() ==>
        #[trigger] bytes[index - offset] == before.visible[index]
}

pub proof fn compatible_write_preserves_protection(
    before: StorageImage, after: StorageImage, offset: int, bytes: Seq<u8>, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected),
        issue_write(before, after, offset, bytes), write_compatible_with_protection(before, offset, bytes, protected),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    protected_bytes_are_stable(before, original, protected);
    write_preserves_storage(before, after, offset, bytes);
    assert forall|index: int| protected.contains(index) implies
        #[trigger] after.permitted[index] == Set::empty().insert(original[index]) by {
        assert(before.permitted[index] == Set::empty().insert(original[index]));
        if offset <= index < offset + bytes.len() {
            assert(bytes[index - offset] == before.visible[index]);
            assert(before.visible[index] == original[index]);
        }
    }
}

pub proof fn persistence_preserves_protection(
    before: StorageImage, after: StorageImage, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected), persist_some(before, after),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    persistence_preserves_storage(before, after);
}

pub proof fn sync_preserves_protection(
    before: StorageImage, after: StorageImage, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected), sync_success(before, after),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    protected_bytes_are_stable(before, original, protected);
    sync_preserves_storage(before, after);
    assert forall|index: int| protected.contains(index) implies
        #[trigger] after.permitted[index] == Set::empty().insert(original[index]) by {
        assert(before.visible[index] == original[index]);
    }
}

pub proof fn power_restart_preserves_protection(
    before: StorageImage, after: StorageImage, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected), power_restart_from(before, after),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    protected_bytes_are_stable(before, original, protected);
    power_restart_preserves_storage(before, after);
    assert forall|index: int| protected.contains(index) implies
        #[trigger] after.permitted[index] == Set::empty().insert(original[index]) by {
        assert(before.durable[index] == original[index]);
    }
}

pub proof fn process_restart_preserves_protection(
    before: StorageImage, after: StorageImage, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected), process_restart_from(before, after),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    process_restart_preserves_storage(before, after);
}

pub enum StorageStep {
    Write { offset: int, bytes: Seq<u8> },
    Persist,
    Sync,
    PowerRestart,
    ProcessRestart,
}

pub open spec fn allowed_step(before: StorageImage, after: StorageImage, step: StorageStep, protected: Set<int>) -> bool {
    match step {
        StorageStep::Write { offset, bytes } => issue_write(before, after, offset, bytes)
            && write_compatible_with_protection(before, offset, bytes, protected),
        StorageStep::Persist => persist_some(before, after),
        StorageStep::Sync => sync_success(before, after),
        StorageStep::PowerRestart => power_restart_from(before, after),
        StorageStep::ProcessRestart => process_restart_from(before, after),
    }
}

pub proof fn step_preserves_protection(
    before: StorageImage, after: StorageImage, step: StorageStep, original: Seq<u8>, protected: Set<int>)
    requires valid_storage(before), protected_bytes(before, original, protected),
        allowed_step(before, after, step, protected),
    ensures valid_storage(after), protected_bytes(after, original, protected),
{
    match step {
        StorageStep::Write { offset, bytes } => compatible_write_preserves_protection(before, after, offset, bytes, original, protected),
        StorageStep::Persist => persistence_preserves_protection(before, after, original, protected),
        StorageStep::Sync => sync_preserves_protection(before, after, original, protected),
        StorageStep::PowerRestart => power_restart_preserves_protection(before, after, original, protected),
        StorageStep::ProcessRestart => process_restart_preserves_protection(before, after, original, protected),
    }
}

pub open spec fn valid_trace(states: Seq<StorageImage>, steps: Seq<StorageStep>, protected: Set<int>) -> bool {
    &&& states.len() == steps.len() + 1
    &&& forall|index: int| 0 <= index < steps.len() ==>
        #[trigger] allowed_step(states[index], states[index + 1], steps[index], protected)
}

pub proof fn trace_preserves_protection(
    states: Seq<StorageImage>, steps: Seq<StorageStep>, original: Seq<u8>, protected: Set<int>)
    requires valid_trace(states, steps, protected), valid_storage(states[0]),
        protected_bytes(states[0], original, protected),
    ensures valid_storage(states.last()), protected_bytes(states.last(), original, protected),
        forall|index: int| protected.contains(index) ==> #[trigger] states.last().durable[index] == original[index],
    decreases steps.len(),
{
    if steps.len() > 0 {
        let prefix_states = states.drop_last();
        let prefix_steps = steps.drop_last();
        assert forall|index: int| 0 <= index < prefix_steps.len() implies
            #[trigger] allowed_step(prefix_states[index], prefix_states[index + 1], prefix_steps[index], protected) by {
            assert(allowed_step(states[index], states[index + 1], steps[index], protected));
        }
        trace_preserves_protection(prefix_states, prefix_steps, original, protected);
        assert(allowed_step(states[steps.len() - 1], states[steps.len() as int], steps.last(), protected));
        step_preserves_protection(prefix_states.last(), states.last(), steps.last(), original, protected);
    }
    protected_bytes_are_stable(states.last(), original, protected);
}

pub proof fn single_byte_update_cannot_mix_selector_bits(
    initial: Seq<u8>, offset: int, value: u8, after_write: StorageImage, after_persist: StorageImage)
    requires 0 <= offset < initial.len(),
        issue_write(clean_storage(initial), after_write, offset, seq![value]),
        persist_some(after_write, after_persist),
    ensures after_persist.durable[offset] == initial[offset] || after_persist.durable[offset] == value,
        forall|index: int| 0 <= index < initial.len() && index != offset ==>
            #[trigger] after_persist.durable[index] == initial[index],
{
    clean_storage_is_valid(initial);
    write_preserves_storage(clean_storage(initial), after_write, offset, seq![value]);
    persistence_preserves_storage(after_write, after_persist);
    assert(after_write.permitted[offset] == Set::empty().insert(initial[offset]).insert(value));
    assert(after_write.permitted[offset].contains(after_persist.durable[offset]));
    assert forall|index: int| 0 <= index < initial.len() && index != offset implies
        #[trigger] after_persist.durable[index] == initial[index] by {
        assert(after_write.permitted[index] == Set::empty().insert(initial[index]));
        assert(after_write.permitted[index].contains(after_persist.durable[index]));
    }
}

}
