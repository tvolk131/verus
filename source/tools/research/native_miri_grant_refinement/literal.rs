use vstd::prelude::*;
macro_rules! bug { ($($arg:tt)*) => { panic!($($arg)*) }; }
verus! {
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Item(u64);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Permission { Unique, SharedReadWrite, SharedReadOnly, Disabled }
const PERM_MASK: u64 = 0x3 << 61;
const PROTECTED_MASK: u64 = 0x1 << 63;
const PERM_SHIFT: u64 = 61;
// Explicit projection adapter: only borrows is observed by this component.
pub struct Stack { borrows: Vec<Item> }
impl Permission {
    const UNIQUE: u64 = 0;
    const SHARED_READ_WRITE: u64 = 1;
    const SHARED_READ_ONLY: u64 = 2;
    const DISABLED: u64 = 3;
fn to_bits(self) -> u64 {
        match self {
            Permission::Unique => Self::UNIQUE,
            Permission::SharedReadWrite => Self::SHARED_READ_WRITE,
            Permission::SharedReadOnly => Self::SHARED_READ_ONLY,
            Permission::Disabled => Self::DISABLED,
        }
    }
fn from_bits(perm: u64) -> Self {
        match perm {
            Self::UNIQUE => Permission::Unique,
            Self::SHARED_READ_WRITE => Permission::SharedReadWrite,
            Self::SHARED_READ_ONLY => Permission::SharedReadOnly,
            Self::DISABLED => Permission::Disabled,
            _ => unreachable!(),
        }
    }
}
impl Item {
pub fn perm(self) -> Permission {
        Permission::from_bits((self.0 & PERM_MASK) >> PERM_SHIFT)
    }
pub fn protected(self) -> bool {
        self.0 & PROTECTED_MASK > 0
    }
pub fn set_permission(&mut self, perm: Permission) {
        // Clear the current set permission
        self.0 &= !PERM_MASK;
        // Write Permission::Disabled to the Permission bits
        self.0 |= perm.to_bits() << PERM_SHIFT;
    }
}
impl Stack {
pub fn get(&self, idx: usize) -> Option<Item> {
        self.borrows.get(idx).cloned()
    }
fn find_first_write_incompatible(&self, granting: usize) -> usize {
        let perm = self.get(granting).unwrap().perm();
        match perm {
            Permission::SharedReadOnly => bug!("Cannot use SharedReadOnly for writing"),
            Permission::Disabled => bug!("Cannot use Disabled for anything"),
            Permission::Unique => {
                // On a write, everything above us is incompatible.
                granting + 1
            }
            Permission::SharedReadWrite => {
                // The SharedReadWrite *just* above us are compatible, to skip those.
                let mut idx = granting + 1;
                while let Some(item) = self.get(idx) {
                    if item.perm() == Permission::SharedReadWrite {
                        // Go on.
                        idx += 1;
                    } else {
                        // Found first incompatible!
                        break;
                    }
                }
                idx
            }
        }
    }
}
}
