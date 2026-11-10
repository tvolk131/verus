"""Source overlay only; upstream bodies are emitted privately, never vendored here."""
import hashlib, json, re
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
PIN=HERE.parent/'native_miri_policy_pin/pins.json'

def need(b,m):
    if not b: raise RuntimeError(m)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def block(t,anchor):
    start=t.index(anchor); a=t.index('{',start); depth=1;i=a+1
    while depth:
        if t[i]=='{':depth+=1
        if t[i]=='}':depth-=1
        i+=1
    return t[start:i]
def overlay(b,signature): return signature+' '+b[b.index('{'):]
def load():
    pins=json.loads(PIN.read_text()); sources={}
    for name in ['src/tools/miri/src/borrow_tracker/mod.rs','src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs']:
        p=pins['sources'][name];need(sha(p['path'])==p['sha256'],'upstream hash');sources[name]=Path(p['path']).read_text()
    return list(sources.values())
def generate(kind):
    bor,it=load(); full=kind.startswith('full')
    bd=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord\)\]\npub struct BorTag\(NonZero<u64>\);',bor).group()
    idc=re.search(r'#\[derive\(Copy, Clone, Hash, PartialEq, Eq\)\]\npub struct Item\(u64\);',it).group()
    out='use vstd::prelude::*;\nuse std::num::NonZero;\n#[verus_verify(infer_derive(PartialEq))]\n'+bd+'\nverus! {\n'+idc+'\n'
    out+='\n'.join(re.findall(r'^const \w+: u64 = .*;',it,re.M))+'\n'
    out+='pub open spec fn item_word_wf(w:u64)->bool { (w & (u64::MAX >> 3)) != 0 }\n'
    out+='impl BorTag {\npub closed spec fn value(&self)->u64 { self.0@ }\n'
    out+=overlay(block(bor,'    pub fn new(i:'),'pub fn new(i:u64)->(r:Option<Self>) ensures r.is_some()==(i!=0), r.is_some() ==> r.unwrap().value()==i,')+'\n'
    out+=overlay(block(bor,'    pub fn get('),'pub fn get(&self)->(r:u64) ensures r==self.value(), r!=0,')+'\n}\n'
    out+='impl Item {\npub closed spec fn word(self)->u64 { self.0 }\n'
    out+=overlay(block(it,'    pub fn tag('),'pub fn tag(self)->(r:BorTag) requires item_word_wf(self.word()), ensures r.value()==(self.word() & (u64::MAX >> 3)),')+'\n'
    if full:
        out+=overlay(block(it,'    pub fn new(tag:'),'pub fn new(tag:BorTag,perm:Permission,protected:bool)->(r:Self) requires tag.value() <= (u64::MAX >> 3), ensures item_word_wf(r.word()), r.word() & (u64::MAX >> 3)==tag.value(),')+'\n'
        out+='pub closed spec fn permission(self)->Permission { decode((self.0 & PERM_MASK) >> PERM_SHIFT) }\n'
        out+=overlay(block(it,'    pub fn perm('),'pub fn perm(self)->(r:Permission) ensures r==self.permission(),')+'\n'
        out+=overlay(block(it,'    pub fn protected('),'pub fn protected(self)->(r:bool) ensures r==(self.word() & 0x8000000000000000u64 > 0),')+'\n'
    out+='}\n'
    if full:
        enum=block(it,'pub enum Permission');out+='#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]\n'+enum+'\n'
        out+='pub closed spec fn bits(p:Permission)->u64 { match p { Permission::Unique=>0,Permission::SharedReadWrite=>1,Permission::SharedReadOnly=>2,Permission::Disabled=>3 } }\n'
        out+='pub closed spec fn decode(p:u64)->Permission { if p==0 {Permission::Unique} else if p==1 {Permission::SharedReadWrite} else if p==2 {Permission::SharedReadOnly} else {Permission::Disabled} }\n'
        out+='impl Permission {\n'+'\n'.join(re.findall(r'^    const \w+: u64 = .*;',it,re.M))+'\n'
        out+=overlay(block(it,'    fn to_bits('),'fn to_bits(self)->(r:u64) ensures r==bits(self), r<=3,')+'\n'
        out+=overlay(block(it,'    fn from_bits('),'fn from_bits(perm:u64)->(r:Self) requires perm<=3, ensures r==decode(perm),')+'\n}\n'
    out+='''pub fn roundtrip(i:u64)->(r:u64) ensures r==i,
{ match BorTag::new(i) {Some(t)=>t.get(),None=>0} }
pub fn compare_equal(a:&BorTag,b:&BorTag)->(r:bool)
ensures r==(a.value()==b.value()), { a==b }
pub fn lookup(w:u64)->(r:u64) requires item_word_wf(w),
ensures r==(w & (u64::MAX >> 3)), { Item(w).tag().get() }
pub fn boundaries(w:u64)->(r:u64) requires item_word_wf(w), (w & (u64::MAX >> 3)) == (u64::MAX >> 3), ensures r==(u64::MAX >> 3),
{ Item(w).tag().get() }
'''
    if kind=='false':out+='pub fn false_relation(w:u64)->(r:u64) requires item_word_wf(w), ensures r!=(w & (u64::MAX >> 3)), { Item(w).tag().get() }\n'
    if kind=='invalid':out+='pub fn invalid_zero_low()->BorTag { Item(0xe000000000000000u64).tag() }\n'
    out+='}\n'
    if kind in ['full-help','full-helpers']:
        out=out.replace('        debug_assert!(new.tag() == tag);', """        proof {
            let t=tag.value(); let p=bits(perm);
            assert forall|raw:u64| #[trigger] ((t | (p << 61) | (raw << 63)) & TAG_MASK) == t
                && ((t | (p << 61) | (raw << 63)) & PERM_MASK) >> 61 == p
                && item_word_wf(t | (p << 61) | (raw << 63)) by { packed_fields_raw(t,p,raw); }
        }
        debug_assert!(new.tag() == tag);""")
        out=out.replace('pub fn perm(self)->(r:Permission) ensures r==self.permission(), {',"""pub fn perm(self)->(r:Permission) ensures r==self.permission(), {
        proof {
            assert(PERM_MASK == 0x6000000000000000u64) by(bit_vector);
            let w=self.0;
            assert((w & 0x6000000000000000u64) >> 61u64 <= 3u64) by(bit_vector);
        }""")
        out=out.replace('        self.0 & PROTECTED_MASK > 0','        proof { assert(PROTECTED_MASK == 0x8000000000000000u64) by(bit_vector); }\n        self.0 & PROTECTED_MASK > 0')
        helper=block((HERE.parent/'native_miri_tag_representation/bitfields.rs').read_text(),'pub proof fn packed_fields_raw(')
        helper=helper.replace('pub proof fn','proof fn')
        out=out[:-2]+helper+'\n}\n'
    if kind=='full-helpers':out=out.replace(block(out,'pub fn new(tag:'),'',1)
    return out

def fidelity(text):
    bor,it=load(); rows={}
    selected=[('BorTag.new',bor,'    pub fn new(i:','pub fn new(i:'),('BorTag.get',bor,'    pub fn get(','pub fn get('),('Item.tag',it,'    pub fn tag(','pub fn tag(')]
    if 'pub fn new(tag:' in text:selected += [('Item.new',it,'    pub fn new(tag:','pub fn new(tag:')]
    if 'pub fn perm(' in text:
        selected += [('Item.perm',it,'    pub fn perm(','pub fn perm('),('Item.protected',it,'    pub fn protected(','pub fn protected('),('Permission.to_bits',it,'    fn to_bits(','fn to_bits('),('Permission.from_bits',it,'    fn from_bits(','fn from_bits(')]
    def toks(s):return re.findall(r'\w+|[^\s]',re.sub(r'//[^\n]*','',s))
    for name,orig,a,b in selected:
        ob=block(orig,a);nb=block(text,b);ob=ob[ob.index('{'):];nb=nb[nb.index('{'):]
        while 'proof {' in nb:
            pb=block(nb,'proof {');nb=nb.replace(pb,'',1)
        need(toks(ob)==toks(nb),'runtime fidelity '+name)
        rows[name]=dict(original_body_sha256=hashlib.sha256(ob.encode()).hexdigest(),adapted_body_sha256=hashlib.sha256(nb.encode()).hexdigest(),tokens_equal=True)
    need('pub struct BorTag(NonZero<u64>);' in text and 'pub struct Item(u64);' in text,'private carriers')
    return rows
