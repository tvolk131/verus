use vstd::prelude::*;
macro_rules! bug { ($($arg:tt)*) => { panic!($($arg)*) }; }
verus! {
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Item(pub u64);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]
pub enum Permission { Unique, SharedReadWrite, SharedReadOnly, Disabled }
pub const PERM_MASK: u64 = 0x3 << 61;
pub const PROTECTED_MASK: u64 = 0x1 << 63;
pub const PERM_SHIFT: u64 = 61;
// Explicit projection adapter: only borrows is observed by this component.
pub struct Stack { pub borrows: Vec<Item> }
pub open spec fn decode(p: u64) -> Permission {
    if p == 0 { Permission::Unique } else if p == 1 { Permission::SharedReadWrite }
    else if p == 2 { Permission::SharedReadOnly } else { Permission::Disabled }
}
pub open spec fn bits(p: Permission) -> u64 {
    match p { Permission::Unique => 0, Permission::SharedReadWrite => 1,
        Permission::SharedReadOnly => 2, Permission::Disabled => 3 }
}
impl Permission {
    const UNIQUE: u64 = 0;
    const SHARED_READ_WRITE: u64 = 1;
    const SHARED_READ_ONLY: u64 = 2;
    const DISABLED: u64 = 3;
fn to_bits(self) -> (r: u64)
    ensures r == bits(self), r <= 3,
{
        match self {
            Permission::Unique => Self::UNIQUE,
            Permission::SharedReadWrite => Self::SHARED_READ_WRITE,
            Permission::SharedReadOnly => Self::SHARED_READ_ONLY,
            Permission::Disabled => Self::DISABLED,
        }
    }
fn from_bits(perm: u64) -> (r: Self)
    requires perm <= 3,
    ensures r == decode(perm),
{
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
    pub open spec fn permission(self) -> Permission {
        decode((self.0 & PERM_MASK) >> PERM_SHIFT)
    }

pub fn perm(self) -> (r: Permission)
    ensures r == self.permission(),
{
    proof {
        assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
        let w = self.0;
        assert((w & 0x6000000000000000u64) >> 61u64 <= 3u64) by(bit_vector);
    }
        Permission::from_bits((self.0 & PERM_MASK) >> PERM_SHIFT)
    }
pub fn protected(self) -> (r: bool)
    ensures r == (self.0 & PROTECTED_MASK > 0),
{
        self.0 & PROTECTED_MASK > 0
    }
pub fn set_permission(&mut self, perm: Permission)
    ensures final(self).permission() == perm,
        final(self).0 & !PERM_MASK == old(self).0 & !PERM_MASK,
{
        // Clear the current set permission
        self.0 &= !PERM_MASK;
        // Write Permission::Disabled to the Permission bits
        self.0 |= perm.to_bits() << PERM_SHIFT;
    proof {
        assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
        let b = bits(perm);
        let w = old(self).0;
        assert(b <= 3);
        assert((((w & !0x6000000000000000u64) | (b << 61u64)) & 0x6000000000000000u64) >> 61u64 == b) by(bit_vector)
            requires b <= 3;
        assert(((w & !0x6000000000000000u64) | (b << 61u64)) & !0x6000000000000000u64 == w & !0x6000000000000000u64) by(bit_vector)
            requires b <= 3;
    }
    }
}
impl Stack {
pub fn get(&self, idx: usize) -> (r: Option<Item>)
    ensures r == (if idx < self.borrows@.len() { Some(self.borrows@[idx as int]) } else { None }),
{
        self.borrows.get(idx).cloned()
    }
fn find_first_write_incompatible(&self, granting: usize) -> (r: usize)
    requires granting < self.borrows@.len(),
        self.borrows@[granting as int].permission() == Permission::Unique
            || self.borrows@[granting as int].permission() == Permission::SharedReadWrite,
    ensures granting + 1 <= r <= self.borrows@.len(),
        self.borrows@[granting as int].permission() == Permission::Unique ==> r == granting + 1,
        self.borrows@[granting as int].permission() == Permission::SharedReadWrite ==> (
            (forall|j: int| granting < j < r ==> self.borrows@[j].permission() == Permission::SharedReadWrite)
            && (r == self.borrows@.len() || self.borrows@[r as int].permission() != Permission::SharedReadWrite)),
{
        proof {
            vstd::std_specs::vec::axiom_spec_len(&self.borrows);
        }
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
                while let Some(item) = self.get(idx)
                    invariant
                        granting < self.borrows@.len() <= usize::MAX,
                        self.borrows@[granting as int].permission() == Permission::SharedReadWrite,
                        granting + 1 <= idx <= self.borrows@.len(),
                        forall|j: int| granting < j < idx ==> self.borrows@[j].permission() == Permission::SharedReadWrite,
                    ensures
                        granting + 1 <= idx <= self.borrows@.len(),
                        forall|j: int| granting < j < idx ==> self.borrows@[j].permission() == Permission::SharedReadWrite,
                        idx == self.borrows@.len() || self.borrows@[idx as int].permission() != Permission::SharedReadWrite,
                    decreases self.borrows@.len() - idx,
                {
                    proof {
                        assert(idx < self.borrows@.len());
                        assert(item == self.borrows@[idx as int]);
                    }
                    if item.perm() == Permission::SharedReadWrite {
                        // Go on.
                        proof {
                            assert(self.borrows@[idx as int].permission() == Permission::SharedReadWrite);
                            assert forall|j: int| granting < j < idx + 1 implies self.borrows@[j].permission() == Permission::SharedReadWrite by {
                                if j < idx { } else { assert(j == idx); }
                            }
                        }
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
