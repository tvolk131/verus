pub enum StorageError {
    /// The Database is corrupted
    Corrupted(String),
    /// The value being inserted exceeds the maximum of 3GiB
    ValueTooLarge(usize),
    Io(io::Error),
    /// The backend does not support a required storage operation.
    Unsupported,
    PreviousIo,
    DatabaseClosed,
    LockPoisoned(&'static panic::Location<'static>),
}
