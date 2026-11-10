"""Deterministic proof-only annotation of the preselected literal component."""
from pathlib import Path
H=Path(__file__).resolve().parent
s=(H/'literal.rs').read_text()
s=s.replace('pub struct Item(u64);', 'pub struct Item(pub u64);')
s=s.replace('borrows: Vec<Item>', 'pub borrows: Vec<Item>')
s=s.replace('Debug, Hash, PartialEq, Eq)', 'Debug, Hash, PartialEq, Eq, Structural)')
s=s.replace('const PERM_', 'pub const PERM_').replace('const PROTECTED_MASK', 'pub const PROTECTED_MASK')
s=s.replace('impl Permission {', '''pub open spec fn decode(p: u64) -> Permission {
    if p == 0 { Permission::Unique } else if p == 1 { Permission::SharedReadWrite }
    else if p == 2 { Permission::SharedReadOnly } else { Permission::Disabled }
}
pub open spec fn bits(p: Permission) -> u64 {
    match p { Permission::Unique => 0, Permission::SharedReadWrite => 1,
        Permission::SharedReadOnly => 2, Permission::Disabled => 3 }
}
impl Permission {''')
s=s.replace('fn to_bits(self) -> u64 {', '''fn to_bits(self) -> (r: u64)
    ensures r == bits(self), r <= 3,
{''')
s=s.replace('fn from_bits(perm: u64) -> Self {', '''fn from_bits(perm: u64) -> (r: Self)
    requires perm <= 3,
    ensures r == decode(perm),
{''')
s=s.replace('impl Item {', '''impl Item {
    pub open spec fn permission(self) -> Permission {
        decode((self.0 & PERM_MASK) >> PERM_SHIFT)
    }
''')
s=s.replace('pub fn perm(self) -> Permission {', '''pub fn perm(self) -> (r: Permission)
    ensures r == self.permission(),
{
    proof {
        assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
        let w = self.0;
        assert((w & 0x6000000000000000u64) >> 61u64 <= 3u64) by(bit_vector);
    }''')
s=s.replace('pub fn protected(self) -> bool {', '''pub fn protected(self) -> (r: bool)
    ensures r == (self.0 & PROTECTED_MASK > 0),
{''')
s=s.replace('pub fn set_permission(&mut self, perm: Permission) {', '''pub fn set_permission(&mut self, perm: Permission)
    ensures final(self).permission() == perm,
        final(self).0 & !PERM_MASK == old(self).0 & !PERM_MASK,
{''')
s=s.replace('self.0 |= perm.to_bits() << PERM_SHIFT;', '''self.0 |= perm.to_bits() << PERM_SHIFT;
    proof {
        assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
        let b = bits(perm);
        let w = old(self).0;
        assert(b <= 3);
        assert((((w & !0x6000000000000000u64) | (b << 61u64)) & 0x6000000000000000u64) >> 61u64 == b) by(bit_vector)
            requires b <= 3;
        assert(((w & !0x6000000000000000u64) | (b << 61u64)) & !0x6000000000000000u64 == w & !0x6000000000000000u64) by(bit_vector)
            requires b <= 3;
    }''')
s=s.replace('pub fn get(&self, idx: usize) -> Option<Item> {', '''pub fn get(&self, idx: usize) -> (r: Option<Item>)
    ensures r == (if idx < self.borrows@.len() { Some(self.borrows@[idx as int]) } else { None }),
{''')
s=s.replace('impl Stack {', 'impl Stack {')
s=s.replace('fn find_first_write_incompatible(&self, granting: usize) -> usize {', '''fn find_first_write_incompatible(&self, granting: usize) -> (r: usize)
    requires granting < self.borrows@.len(),
        self.borrows@[granting as int].permission() == Permission::Unique
            || self.borrows@[granting as int].permission() == Permission::SharedReadWrite,
    ensures granting + 1 <= r <= self.borrows@.len(),
        self.borrows@[granting as int].permission() == Permission::Unique ==> r == granting + 1,
        self.borrows@[granting as int].permission() == Permission::SharedReadWrite ==> (
            (forall|j: int| granting < j < r ==> self.borrows@[j].permission() == Permission::SharedReadWrite)
            && (r == self.borrows@.len() || self.borrows@[r as int].permission() != Permission::SharedReadWrite)),
{''')
s=s.replace('let perm = self.get(granting).unwrap().perm();', '''proof {
            vstd::std_specs::vec::axiom_spec_len(&self.borrows);
        }
        let perm = self.get(granting).unwrap().perm();''')
s=s.replace('while let Some(item) = self.get(idx) {', '''while let Some(item) = self.get(idx)
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
                {''')
s=s.replace('if item.perm() == Permission::SharedReadWrite {', '''proof {
                        assert(idx < self.borrows@.len());
                        assert(item == self.borrows@[idx as int]);
                    }
                    if item.perm() == Permission::SharedReadWrite {''')
s=s.replace('idx += 1;', '''proof {
                            assert(self.borrows@[idx as int].permission() == Permission::SharedReadWrite);
                            assert forall|j: int| granting < j < idx + 1 implies self.borrows@[j].permission() == Permission::SharedReadWrite by {
                                if j < idx { } else { assert(j == idx); }
                            }
                        }
                        idx += 1;''')
(H/'component.rs').write_text(s)
