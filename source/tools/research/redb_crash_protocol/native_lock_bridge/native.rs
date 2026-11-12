use std::sync::Mutex;

pub fn native_handoff(initial: u64, next: u64) -> u64 {
    let lock = Mutex::new(initial);
    {
        let mut guard = lock.lock().unwrap();
        *guard = next;
    }
    let guard = lock.lock().unwrap();
    *guard
}

pub fn native_cell_pointer(cell: &std::cell::UnsafeCell<u64>) -> *mut u64 {
    cell.get()
}
