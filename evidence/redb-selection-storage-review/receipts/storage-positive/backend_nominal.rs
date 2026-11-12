use vstd::prelude::*;
verus! {

#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExIoError(std::io::Error);
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExPanicLocation<'a>(core::panic::Location<'a>);

use std::io; use core::panic;
use core::ops::Bound; use core::fmt::Debug;
use core::sync::atomic::{AtomicBool, Ordering};
pub enum BackendError {
    /// The underlying storage operation failed.
    Io(io::Error),
    /// The backend does not support the requested operation.
    Unsupported,
}
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
pub type Result<T = (), E = StorageError> = core::result::Result<T, E>;
pub trait StorageBackend: 'static + Debug + Send + Sync {
    /// Gets the current length of the storage.
    fn len(&self) -> core::result::Result<u64, io::Error>;

    /// Reads the specified array of bytes from the storage.
    ///
    /// If `out.len()` + `offset` exceeds the length of the storage an appropriate `Error` must be returned.
    fn read(&self, offset: u64, out: &mut [u8]) -> core::result::Result<(), io::Error>;

    /// Sets the length of the storage.
    ///
    /// New positions in the storage must be initialized to zero.
    fn set_len(&self, len: u64) -> core::result::Result<(), io::Error>;

    /// Syncs all buffered data with the persistent storage.
    fn sync_data(&self) -> core::result::Result<(), io::Error>;

    /// Writes the specified array to the storage.
    fn write(&self, offset: u64, data: &[u8]) -> core::result::Result<(), io::Error>;

    /// Release any resources held by the backend
    ///
    /// Must release all locks acquired by one of the lock methods.
    ///
    /// Note: redb will not access the backend after calling this method and will call it exactly
    /// once: when the [`Database`] is dropped, or, if a [`WriteTransaction`] was live at that
    /// point, when that transaction completes, or if opening the database fails
    fn close(&self) -> core::result::Result<(), io::Error> {
        Ok(())
    }

    /// Attempts to acquire an exclusive byte-range lock.
    ///
    /// Returns `Ok(true)` on acquisition, `Ok(false)` on a conflicting lock, or an error.
    /// Defaults to [`BackendError::Unsupported`].
    fn try_lock_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<bool, BackendError> {
        Err(BackendError::Unsupported)
    }

    /// Attempts to acquire a shared byte-range lock.
    ///
    /// Returns `Ok(true)` on acquisition, `Ok(false)` on a conflicting lock, or an error.
    /// Defaults to [`BackendError::Unsupported`].
    fn try_lock_shared_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<bool, BackendError> {
        Err(BackendError::Unsupported)
    }

    /// Acquires an exclusive byte-range lock, waiting for conflicting locks to be released.
    ///
    /// Defaults to [`BackendError::Unsupported`].
    fn lock_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<(), BackendError> {
        Err(BackendError::Unsupported)
    }

    /// Acquires a shared byte-range lock, waiting for conflicting locks to be released.
    ///
    /// Defaults to [`BackendError::Unsupported`].
    fn lock_shared_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<(), BackendError> {
        Err(BackendError::Unsupported)
    }

    /// Releases the lock acquired over exactly this range.
    ///
    /// The range passed will exactly match a range successfully locked by one of the lock methods.
    ///
    /// Defaults to [`BackendError::Unsupported`].
    fn unlock_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<(), BackendError> {
        Err(BackendError::Unsupported)
    }

    /// Reports whether an exclusive lock over the range would conflict with a lock held elsewhere.
    ///
    /// The queried range will not overlap any lock currently held by this backend.
    /// Defaults to [`BackendError::Unsupported`].
    fn query_lock_range(
        &self,
        _start: Bound<u64>,
        _end: Bound<u64>,
    ) -> core::result::Result<bool, BackendError> {
        Err(BackendError::Unsupported)
    }
}
struct CheckedBackend {
    file: Box<dyn StorageBackend>,
    io_failed: AtomicBool,
    closed: AtomicBool,
}
impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> StorageError {
        StorageError::Io(err)
    }
}
impl CheckedBackend {
    fn check_failure(&self) -> Result<()> {
        if self.io_failed.load(Ordering::Acquire) {
            if self.closed.load(Ordering::Acquire) {
                Err(StorageError::DatabaseClosed)
            } else {
                Err(StorageError::PreviousIo)
            }
        } else {
            Ok(())
        }
    }
    fn sync_data(&self) -> Result<()> {
        self.check_failure()?;
        let result = self.file.sync_data();
        if result.is_err() {
            self.io_failed.store(true, Ordering::Release);
        }
        result.map_err(StorageError::from)
    }
}

impl vstd::std_specs::convert::FromSpecImpl<std::io::Error> for StorageError {
    open spec fn obeys_from_spec() -> bool { true }
    closed spec fn from_spec(error: std::io::Error) -> Self { StorageError::Io(error) }
}

}
