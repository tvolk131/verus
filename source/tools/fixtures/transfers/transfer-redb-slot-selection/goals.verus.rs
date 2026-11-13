pub fn two_phase_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), old(header).inner.two_phase_commit,
    ensures final(header).inner.primary_slot == old(header).inner.primary_slot,
        result.is_ok() == !old(header).primary_corrupted,
        result.is_ok() ==> result->Ok_0,
{
    header.select_primary_slot()
}

pub fn both_corrupt_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        old(header).primary_corrupted, old(header).secondary_corrupted,
    ensures result.is_err(), *final(header) == *old(header),
{
    header.select_primary_slot()
}

pub fn fallback_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        old(header).primary_corrupted, !old(header).secondary_corrupted,
    ensures result.is_ok(), !result->Ok_0,
        final(header).inner.primary_slot == (old(header).inner.primary_slot ^ 1),
{
    header.select_primary_slot()
}

pub fn newer_secondary_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, !old(header).secondary_corrupted,
        old(header).secondary_newer(),
    ensures result.is_ok(), !result->Ok_0,
        final(header).inner.primary_slot == (old(header).inner.primary_slot ^ 1),
{
    header.select_primary_slot()
}

pub fn equal_or_stale_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, !old(header).secondary_newer(),
    ensures result.is_ok(), result->Ok_0,
        final(header).inner.primary_slot == old(header).inner.primary_slot,
{
    header.select_primary_slot()
}

pub fn corrupt_secondary_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(), !old(header).inner.two_phase_commit,
        !old(header).primary_corrupted, old(header).secondary_corrupted,
    ensures result.is_ok(), result->Ok_0,
        final(header).inner.primary_slot == old(header).inner.primary_slot,
{
    header.select_primary_slot()
}

pub fn selected_observation_goal(header: &mut UnrepairedDatabaseHeader) -> (result: Result<bool>)
    requires old(header).representation_valid(),
    ensures result.is_ok() == old(header).selection_success(),
        result.is_ok() ==> old(header).slot_admissible(final(header).inner.primary_slot),
        result.is_ok() ==> (result->Ok_0
            == (final(header).inner.primary_slot == old(header).inner.primary_slot)),
        DatabaseHeader::frame(old(header).inner, final(header).inner),
        final(header).primary_corrupted == old(header).primary_corrupted,
        final(header).secondary_corrupted == old(header).secondary_corrupted,
{
    header.select_primary_slot()
}
