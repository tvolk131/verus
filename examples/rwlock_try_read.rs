use vstd::prelude::*;
use vstd::rwlock::{RwLock, RwLockPredicate};

verus! {
pub struct Allowed;
impl RwLockPredicate<Option<u64>> for Allowed {
    open spec fn inv(self, value: Option<u64>) -> bool {
        value.is_none() || value == Some(7)
    }
}

fn checked(lock: &RwLock<Option<u64>, Allowed>) {
    let value = lock.try_read_or(None);
    assert(value.is_none() || value == Some(7));
    match lock.try_acquire_read() {
        None => {},
        Some(handle) => {
            let value = *handle.borrow();
            assert(value.is_none() || value == Some(7));
            handle.release_read();
        },
    }
}
}

#[cfg(verus_keep_ghost)]
fn main() {}

// Runtime contention checks are separate from the predicate-only formal API.
#[cfg(not(verus_keep_ghost))]
fn main() {
    let lock = RwLock::<Option<u64>, Allowed>::new(Some(7));
    assert_eq!(lock.try_read_copy(), Some(Some(7)));
    let (value, writer) = lock.acquire_write();
    assert_eq!(value, Some(7));
    assert!(lock.try_acquire_read().is_none());
    assert_eq!(lock.try_read_copy(), None);
    assert_eq!(lock.try_read_or(None), None);
    writer.release_write(Some(7));
    assert_eq!(lock.try_read_or(None), Some(7));
    // A failed attempt must leave the reader count balanced.
    lock.replace(None);
    assert_eq!(lock.read_copy(), None);
}
