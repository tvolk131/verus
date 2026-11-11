"""Exact source projection and minimal iterator probes; never a replacement loop."""
import importlib.util,json,re,hashlib
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
def need(b,m):
    if not b: raise RuntimeError(m)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load_module(path,name):
    sp=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m);return m
T=load_module(HERE.parent/'item_tag_constructor_composition/source.py','tag_source')
block=T.block
def primary():
    f=json.loads((HERE/'SOURCE_FREEZE.json').read_text());ts={}
    for n,p in f['primary'].items():
        need(sha(p['path'])==p['sha256'],'primary digest '+n);ts[n]=Path(p['path']).read_text()
    for p,h in f['dependencies'].items():need(sha(ROOT/p)==h,'committed dependency '+p)
    return ts
PRE='use vstd::prelude::*;\nuse vstd::std_specs::iter::IteratorSpec;\nverus! {\nbroadcast use {vstd::std_specs::iter::group_iter_axioms, vstd::std_specs::iter_adapters::group_native_adapter_axioms};\n'
def probe(kind):
    bodies={
        'forward_loop':'for (_, _) in v.iter().enumerate() { }',
        'slice_reverse_loop':'for _ in v.iter().rev() { }',
        'reverse_loop':'for (_, _) in v.iter().enumerate().rev() { }',
        'forward_laws':'''let e = v.iter().enumerate();
            assert(e.obeys_prophetic_iter_laws()); assert(e.decrease().is_some());''',
        'reverse_law':'let r = v.iter().enumerate().rev(); assert(r.obeys_prophetic_iter_laws());',
        'reverse_decrease':'let r = v.iter().enumerate().rev(); assert(r.decrease().is_some());',
        'reverse_remaining':'''let e = v.iter().enumerate(); let ghost before = e;
            let r = e.rev(); assert(r.remaining() == before.remaining().reverse());''',
        'reverse_inner':'''let e = v.iter().enumerate(); let ghost before = e;
            let r = e.rev(); assert(vstd::std_specs::iter::rev_iter(r) == before);''',
    }
    return PRE+'pub fn '+kind+'(v: &Vec<u64>) { '+bodies[kind]+' }\n}\n'
def component(kind='dependencies'):
    ts=primary();st=ts['src/tools/miri/src/borrow_tracker/stacked_borrows/stack.rs'];bor=ts['src/tools/miri/src/borrow_tracker/mod.rs'];sb=ts['src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs']
    # Selected tag subset only: original new/get/tag, no Item packing constructor.
    tag=T.generate('tag').replace('use std::num::NonZero;', 'use std::num::NonZero;\nuse std::ops::Range;');tag=tag[:tag.index('pub fn roundtrip(')]+'}\n'
    # Equality only is used by fallback. Original Ord/PartialOrd derives stay,
    # but their proof requests are excluded; no ordering import or export.
    cache=(HERE.parent/'native_miri_find_granting/cache_component.rs').read_text()
    sections=[block(cache,'pub struct Stack'),block(cache,'struct StackCache'),block(cache,'spec fn cache_consistent('),block(cache,'spec fn add_effect('),'impl StackCache {\n'+block(cache,'fn add(')+'\n}',block(cache,'proof fn add_effect_preserves_consistency(')]
    perm=block(cache,'spec fn decode(')+'\nimpl Permission {\n'+ '\n'.join(re.findall(r'^    const \w+: u64 = .*;',cache,re.M))+'\n'+block(cache,'fn from_bits(')+'\n}\nimpl Item {\n'+block(cache,'spec fn permission(')+'\n'+block(cache,'fn perm(')+'\n}\n'
    defs='#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Structural)]\n'+block(ts['src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs'],'pub enum Permission')+'\n'
    defs+='#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Structural)]\n'+block(bor,'pub enum AccessKind')+'\n'
    defs+='''spec fn granting(p:Permission,a:AccessKind)->bool {
        p != Permission::Disabled && (a == AccessKind::Read || p != Permission::SharedReadOnly)
    }
impl Permission {
'''+T.overlay(block(sb,'fn grants('),'fn grants(self,access:AccessKind)->(r:bool) ensures r==granting(self,access),')+'\n}\n'
    checks='''pub fn tag_equal(a: &BorTag,b: &BorTag)->(r:bool)
        ensures r==(a.value()==b.value()), { a==b }
    pub fn write_precision(p:Permission)->(r:bool)
        ensures r==(p==Permission::Unique || p==Permission::SharedReadWrite),
        {p.grants(AccessKind::Write)}
'''
    specs='''spec fn stack_wf(s:Stack)->bool {
        forall|i:int| 0 <= i < s.borrows@.len() ==> item_word_wf(s.borrows@[i].word())
    }
spec fn structural(c:StackCache,b:Seq<Item>)->bool {b.len()>32 ==> cache_consistent(c,b)}
spec fn matching(i:Item,q:BorTag,a:AccessKind)->bool {
    (i.word() & (u64::MAX >> 3)) == q.value() && granting(i.permission(),a)
}
spec fn result_ok(b:Seq<Item>,q:BorTag,a:AccessKind,r:Option<usize>)->bool {
    match r {
        Some(g) => g < b.len() && matching(b[g as int],q,a)
            && (forall|j:int| g < j < b.len() ==> !matching(b[j],q,a)),
        None => forall|j:int| 0 <= j < b.len() ==> !matching(b[j],q,a),
    }
}
'''
    extra=''
    if kind in ['fallback','fallback_plain']:
        full=block(st,'fn find_granting_tagged(');a=full.index('        for (stack_idx, item)');body=full[a:full.rindex('    }')]
        contract='' if kind=='fallback_plain' else '''requires stack_wf(*old(self)), structural(old(self).cache,old(self).borrows@),
            ensures final(self).borrows@==old(self).borrows@,
                final(self).unknown_bottom==old(self).unknown_bottom,
                final(self).unique_range==old(self).unique_range,
                structural(final(self).cache, final(self).borrows@),
                result_ok(old(self).borrows@,tag,access,r),'''
        extra='impl Stack {\nfn tagged_fallback(&mut self,access:AccessKind,tag:BorTag)->(r:Option<usize>)\n'+contract+'\n{\n'+body+'}\n}\n'
    if kind=='full_original':
        extra='impl Stack {\n'+block(st,'fn find_granting_tagged(')+'\n'+block(st,'fn find_granting_cache(')+'\n}\n'
    out=tag[:-2]+defs+perm+'const CACHE_LEN:usize=32;\n'+'\n'.join(sections)+'\n'+specs+checks+extra+'}\n'
    return out

def tokens(s):return re.findall(r'\w+|[^\s]',re.sub(r'//[^\n]*','',s))
def fidelity(s,kind):
    ts=primary();rows=T.fidelity(s)
    st=ts['src/tools/miri/src/borrow_tracker/stacked_borrows/stack.rs'];sb=ts['src/tools/miri/src/borrow_tracker/stacked_borrows/mod.rs']
    for label,orig,anchor in [('StackCache.add',st,'fn add('),('Permission.grants',sb,'fn grants('),('Item.perm',ts['src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs'],'fn perm('),('Permission.from_bits',ts['src/tools/miri/src/borrow_tracker/stacked_borrows/item.rs'],'fn from_bits(')]:
        a=block(orig,anchor);b=block(s,anchor);
        while 'proof {' in b:b=b.replace(block(b,'proof {'),'',1)
        need(tokens(a[a.index('{'):])==tokens(b[b.index('{'):]),label+' tokens');rows[label]={'tokens_equal':True}
    if kind.startswith('fallback'):
        f=block(st,'fn find_granting_tagged(');a=f[f.index('        for (stack_idx, item)'):f.rindex('    }')];b=block(s,'fn tagged_fallback(');b=b[b.index('{')+1:b.rindex('}')];need(tokens(a)==tokens(b),'fallback tokens');rows['fallback']={'tokens_equal':True,'cache_hit_prelude':'excluded/unproved','tokens':len(tokens(a))}
    if kind=='full_original':
        for anchor in ['fn find_granting_tagged(','fn find_granting_cache(']:need(tokens(block(st,anchor))==tokens(block(s,anchor)),'literal '+anchor)
    return rows
