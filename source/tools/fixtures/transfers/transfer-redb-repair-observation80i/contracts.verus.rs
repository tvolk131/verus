pub open spec fn normalized(observation: Result<bool>, result: Result<bool>) -> bool {
    match observation {
        Ok(verified) => result == Ok(verified),
        Err(StorageError::Corrupted(_)) => result == Ok(false),
        Err(error) => result == Err(error),
    }
}

pub open spec fn noncorruption_result(result: Result<bool>) -> bool {
    match result {
        Err(StorageError::Corrupted(_)) => false,
        _ => true,
    }
}

pub fn normalize_observation(observation: Result<bool>) -> (result: Result<bool>)
    ensures normalized(observation, result), noncorruption_result(result),
{
    match observation {
        Ok(verified) => Ok(verified),
        Err(StorageError::Corrupted(_)) => Ok(false),
        Err(err) => Err(err),
    }
}

pub fn true_observation() -> (result: Result<bool>)
    ensures result == Ok(true),
{
    normalize_observation(Ok(true))
}

pub fn false_observation() -> (result: Result<bool>)
    ensures result == Ok(false),
{
    normalize_observation(Ok(false))
}

pub fn corrupted_observation(message: String) -> (result: Result<bool>)
    ensures result == Ok(false),
{
    normalize_observation(Err(StorageError::Corrupted(message)))
}

pub fn noncorrupt_error(error: StorageError) -> (result: Result<bool>)
    requires !matches!(error, StorageError::Corrupted(_)),
    ensures result == Err(error),
{
    normalize_observation(Err(error))
}

pub fn io_observation(error: io::Error) -> (result: Result<bool>)
    ensures result == Err(StorageError::Io(error)),
{
    normalize_observation(Err(StorageError::Io(error)))
}

pub fn poisoned_observation(location: &'static panic::Location<'static>) -> (result: Result<bool>)
    ensures result == Err(StorageError::LockPoisoned(location)),
{
    normalize_observation(Err(StorageError::LockPoisoned(location)))
}

pub proof fn true_is_not_false()
    ensures !normalized(Ok(true), Ok(false)),
{}

pub proof fn corruption_is_not_success(message: String)
    ensures !normalized(Err(StorageError::Corrupted(message)), Ok(true)),
{}

pub proof fn io_is_not_boolean(error: io::Error, value: bool)
    ensures !normalized(Err(StorageError::Io(error)), Ok(value)),
{}

pub open spec fn gate_post(before: UnrepairedDatabaseHeader, result: Result<DatabaseHeader>) -> bool {
    match result {
        Ok(header) => !before.primary_corrupted && header == before.inner,
        Err(StorageError::Corrupted(message)) => before.primary_corrupted
            && message@ == "Primary commit slot is corrupted"@,
        Err(_) => false,
    }
}

pub fn accepted_gate(header: UnrepairedDatabaseHeader) -> (result: Result<DatabaseHeader>)
    requires !header.primary_corrupted,
    ensures result == Ok(header.inner),
{
    header.finalize_transaction_slots()
}

pub fn rejected_gate(header: UnrepairedDatabaseHeader) -> (result: Result<DatabaseHeader>)
    requires header.primary_corrupted,
    ensures match result {
        Err(StorageError::Corrupted(message)) => message@ == "Primary commit slot is corrupted"@,
        _ => false,
    },
{
    header.finalize_transaction_slots()
}

pub fn invalid_index_is_not_validated(header: UnrepairedDatabaseHeader) -> (result: Result<DatabaseHeader>)
    requires !header.primary_corrupted, header.inner.primary_slot == usize::MAX,
    ensures result == Ok(header.inner),
        match result { Ok(value) => value.primary_slot == usize::MAX, _ => false },
{
    header.finalize_transaction_slots()
}
