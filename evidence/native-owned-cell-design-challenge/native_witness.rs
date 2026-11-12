//! Ordinary Rust scope witnesses; no verifier result is claimed.
use std::cell::UnsafeCell;
#[path = "../../source/tools/research/redb_crash_protocol/native_state_link/owned_native.rs"]
mod frozen;

trait NativeHook { fn run(&self, cell: &UnsafeCell<u64>, next: u64); }
struct Write;
impl NativeHook for Write {
    fn run(&self, cell: &UnsafeCell<u64>, next: u64) {
        // No inner references or competing accesses are live.
        unsafe { *cell.get() = next; }
    }
}
fn abstract_client(hook: &impl NativeHook, initial: u64, next: u64) -> u64 {
    let cell = UnsafeCell::new(initial);
    hook.run(&cell, next);
    cell.into_inner()
}
struct Outer { cell: UnsafeCell<u64> }
fn outer_hook(outer: &Outer, next: u64) {
    unsafe { *outer.cell.get() = next; }
}
fn return_only(_cell: &mut UnsafeCell<u64>, next: u64) -> u64 { next }

fn main() {
    assert_eq!(abstract_client(&Write, 3, 7), 7);
    let outer = Outer { cell: UnsafeCell::new(3) };
    outer_hook(&outer, 7);
    assert_eq!(outer.cell.into_inner(), 7);
    println!("shared and shared-outer callbacks: 3 -> 7 (outside the proposed grammar)");

    for initial in [0, 1, 3, u64::MAX] {
        assert_eq!(frozen::owned_roundtrip(initial), initial);
        for next in [0, 7, u64::MAX] {
            assert_eq!(frozen::exclusive_update(initial, next, initial), (next, initial));
            let mut cell = UnsafeCell::new(99);
            Write.run(&cell, initial); // Prior lawful mutation, ended before unique borrow.
            assert_eq!(frozen::exclusive_borrow(&mut cell, next), next);
            assert_eq!(cell.into_inner(), next); // Actual native input after return.
        }
    }
    println!("unchanged frozen functions: 4 initial x 3 next values; native final input read succeeds");

    let mut cell = UnsafeCell::new(3);
    assert_eq!(return_only(&mut cell, 7), 7);
    assert_eq!(cell.into_inner(), 3);
    println!("return-only control: result=7 but final input=3; returned scalar cannot certify final state");
}
