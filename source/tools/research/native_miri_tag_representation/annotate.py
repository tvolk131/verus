"""Annotated genuine-carrier SUBSET with an explicit closure adapter; not original acceptance.

succ is excluded, since its standard checked_add spec is absent. All retained
runtime bodies match the frozen selection except new's constructor-as-function
closure spelling. Exact Item bodies, including all debug_asserts, are retained.
"""
from pathlib import Path
import json
import re
import tempfile
from source_selection import selection, require, block

HERE=Path(__file__).resolve().parent

def once(text,old,new):
    require(text.count(old)==1,'overlay anchor differs: '+old)
    return text.replace(old,new)

def generate():
    meta,text=selection()
    require(meta==json.loads((HERE/'selection.json').read_bytes()),'primary freeze differs')
    text=once(text,'pub struct BorTag(NonZero<u64>);','pub struct BorTag(pub NonZero<u64>);')
    text=once(text,'pub struct Item(u64);','pub struct Item(pub u64);')
    for name in ['TAG_MASK','PERM_MASK','PROTECTED_MASK','PERM_SHIFT','PROTECTED_SHIFT']:
        text=once(text,'const '+name+':','pub const '+name+':')
    text=once(text,block(text,'    pub fn succ('),'')
    text=once(text,'.map(BorTag)', '.map(|n: NonZero<u64>| -> (r: BorTag) ensures r.0 == n, { BorTag(n) })')
    text=once(text,'pub fn new(i: u64) -> Option<Self> {', '''pub fn new(i: u64) -> (r: Option<Self>)
        ensures match r { Some(t) => t.0@ == i && i != 0, None => i == 0 },
    {''')
    text=once(text,'pub fn get(&self) -> u64 {', '''pub fn get(&self) -> (r: u64)
        ensures r == self.0@, r != 0,
    {''')
    text=once(text,'pub fn inner(&self) -> NonZero<u64> {', '''pub fn inner(&self) -> (r: NonZero<u64>)
        ensures r == self.0,
    {''')
    text=once(text,'pub fn one() -> Self {', '''pub fn one() -> (r: Self)
        ensures r.0@ == 1,
    {''')
    text=once(text,'fn default() -> Self {', '''fn default() -> (r: Self)
        ensures r.0@ == 1,
    {''')
    text=once(text,'#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]','#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]')
    text=once(text,'fn to_bits(self) -> u64 {', '''fn to_bits(self) -> (r: u64)
        ensures r == bits(self), r <= 3,
    {''')
    text=once(text,'fn from_bits(perm: u64) -> Self {', '''fn from_bits(perm: u64) -> (r: Self)
        requires perm <= 3,
        ensures r == decode(perm),
    {''')
    text=once(text,'pub fn new(tag: BorTag, perm: Permission, protected: bool) -> Self {', '''pub fn new(tag: BorTag, perm: Permission, protected: bool) -> (r: Self)
        requires tag.0@ <= TAG_MASK,
        ensures item_word_wf(r.0), r.0 & TAG_MASK == tag.0@,
            r.permission() == perm, (r.0 & PROTECTED_MASK > 0) == protected,
    {''')
    text=once(text,'        debug_assert!(new.tag() == tag);', '''        proof {
            let t = tag.0@;
            let p = bits(perm);
            assert forall|raw: u64| #[trigger] ((t | (p << 61) | (raw << 63)) & TAG_MASK) == t
                && ((t | (p << 61) | (raw << 63)) & PERM_MASK) >> 61 == p
                && item_word_wf(t | (p << 61) | (raw << 63)) by {
                packed_fields_raw(t, p, raw);
            }
            assert forall|a: BorTag, b: BorTag| a.0@ == b.0@ implies a == b by {
                tag_identity(a, b);
            }
        }
        debug_assert!(new.tag() == tag);''')
    text=once(text,'pub fn tag(self) -> BorTag {', '''pub fn tag(self) -> (r: BorTag)
        requires item_word_wf(self.0),
        ensures r.0@ == self.0 & TAG_MASK,
    {''')
    text=once(text,'pub fn perm(self) -> Permission {', '''pub fn perm(self) -> (r: Permission)
        ensures r == self.permission(),
    {
        proof {
            assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
            let w = self.0;
            assert((w & 0x6000000000000000u64) >> 61u64 <= 3u64) by(bit_vector);
        }''')
    text=once(text,'pub fn protected(self) -> bool {', '''pub fn protected(self) -> (r: bool)
        ensures r == (self.0 & PROTECTED_MASK > 0),
    {''')
    text=once(text,'pub fn set_permission(&mut self, perm: Permission) {', '''pub fn set_permission(&mut self, perm: Permission)
        ensures final(self).permission() == perm,
            final(self).0 & TAG_MASK == old(self).0 & TAG_MASK,
            final(self).0 & PROTECTED_MASK == old(self).0 & PROTECTED_MASK,
            item_word_wf(final(self).0) == item_word_wf(old(self).0),
    {''')
    text=once(text,'        self.0 |= perm.to_bits() << PERM_SHIFT;', '''        self.0 |= perm.to_bits() << PERM_SHIFT;
        proof { updated_fields(old(self).0, bits(perm)); }''')
    text=once(text,'impl Item {', '''impl Item {
    pub open spec fn permission(self) -> Permission { decode((self.0 & PERM_MASK) >> 61) }
''')
    helpers='''
pub open spec fn decode(p: u64) -> Permission {
    if p == 0 { Permission::Unique } else if p == 1 { Permission::SharedReadWrite }
    else if p == 2 { Permission::SharedReadOnly } else { Permission::Disabled }
}
pub open spec fn bits(p: Permission) -> u64 {
    match p { Permission::Unique => 0, Permission::SharedReadWrite => 1,
        Permission::SharedReadOnly => 2, Permission::Disabled => 3 }
}
pub proof fn tag_identity(a: BorTag, b: BorTag)
    ensures (a == b) == (a.0@ == b.0@),
{
    vstd::std_specs::nonzero::axiom_view_nonzero_from_primitive_eq(a.0);
    vstd::std_specs::nonzero::axiom_view_nonzero_from_primitive_eq(b.0);
}
'''
    bf=(HERE/'bitfields.rs').read_text().removeprefix('use vstd::prelude::*;\nverus! {\n')
    require(bf.endswith('}\n'),'bitfields wrapper differs')
    bf=bf[:-2]
    for c in ['pub const TAG_MASK: u64 = u64::MAX >> 3;\n','pub const PERM_MASK: u64 = 0x3 << 61;\n','pub const PROTECTED_MASK: u64 = 0x1 << 63;\n']:
        bf=once(bf,c,'')
    return 'use vstd::prelude::*;\nverus! {\n'+text+helpers+bf+'\n}\n'

if __name__=='__main__':
    out=Path(tempfile.mkdtemp(prefix='native-miri-tag-annotated-',dir='/tmp'));out.chmod(0o700)
    full=generate()
    (out/'component.rs').write_text(full)
    subset=once(full,block(full,'    pub fn new(tag:'),'')
    (out/'subset.rs').write_text(subset)
    exact=subset
    start=exact.index('    pub fn new(i:')
    end=exact.index('    pub fn get(',start)
    exact=exact[:start]+exact[end:]
    for marker in ['    pub fn one(', '    pub fn tag(self)']:
        exact=once(exact,block(exact,marker),'')
    exact=once(exact,block(exact,'impl std::default::Default for BorTag {'),'')
    exact=re.sub(r'(?m)^[ \t]*///[^\n]*\n', '', exact)
    (out/'exact-subset.rs').write_text(exact)
    probe=exact[:-2]+'''pub fn compare_equal(a: BorTag, b: BorTag) -> (r: bool)
        ensures r == (a.0@ == b.0@),
    { a == b }
    pub fn compare_less(a: BorTag, b: BorTag) -> (r: bool)
        ensures r == (a.0@ < b.0@),
    { a < b }
    }\n'''
    (out/'comparison-probe.rs').write_text(probe)
    print(out)
