use vstd::prelude::*;
use std::sync::Mutex;
verus! {
pub fn native_handoff(initial: u64, next: u64) -> (result: u64)
    ensures result == next,
{
    let lock = Mutex::new(initial);
    {
        let mut guard = lock.lock().unwrap();
        *guard = next;
    }
    let guard = lock.lock().unwrap();
    *guard
}
}
