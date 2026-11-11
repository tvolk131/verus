"""Full-domain source trials and bounded controls on the matching final epoch."""
from resume import *
BASE=fixtures.source('request')
CAND=BASE+'''\n#[verus_infer] pub fn inferred_less(a:&BorTag,b:&BorTag)->bool {a<b}
#[verus_infer] pub fn inferred_le(a:&BorTag,b:&BorTag)->bool {a<=b}
#[verus_infer] pub fn inferred_greater(a:&BorTag,b:&BorTag)->bool {a>b}
#[verus_infer] pub fn inferred_ge(a:&BorTag,b:&BorTag)->bool {a>=b}
verus! {pub fn observe_inferred(a:&BorTag,b:&BorTag){
 let lt=inferred_less(a,b);let le=inferred_le(a,b);let gt=inferred_greater(a,b);let ge=inferred_ge(a,b);
 assert(lt==(a.0@<b.0@));assert(le==(a.0@<=b.0@));
 assert(gt==(a.0@>b.0@));assert(ge==(a.0@>=b.0@));
}}\n'''
BOOL='use vstd::prelude::*;verus! {pub fn old_api(b:bool)->(r:u64) ensures r==if b {1u64}else{0u64}, {u64::from(b)}}'
# Import only own committed pure baseline generators; no historical main runs.
import importlib.util
spec=importlib.util.spec_from_file_location('own_source',ROOT/'source/tools/research/nonzero_ordering_goal/run.py');old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
def expect(label,src,ok=True,n=None,errors=0,**kw):
    j=app(label,src,**kw);v=j.get('verification',{});need((j['exit']==0)==ok,'unexpected exit '+label+': '+(R/label/'stderr').read_text()[-4000:])
    if n is not None:need((v.get('verified'),v.get('errors'))==(n,errors),'verification inventory '+label)
    return j

def semantics():
    for suffix in ['cold','warm']:
        label='final-exact-v3-'+suffix;expect(label,CAND,n=None,paths_label='final-identical-v3-invocation')
        shutil.copytree(R/'final-identical-v3-invocation/vir',R/label/'retained-vir')
    a=json.loads((R/'final-exact-v3-cold/freeze.json').read_text());c=json.loads((R/'final-exact-v3-warm/freeze.json').read_text());need(a==c,'identical-invocation freeze')
    for rel in ['Less','Equal','Greater','swapped']:
        condition='c==vstd::std_specs::cmp::OrdSpec::cmp_spec(&b.0@,&a.0@)' if rel=='swapped' else 'c==Ordering::'+rel
        j=expect('final-false-'+rel,CAND+'verus!{pub fn wrong(a:&BorTag,b:&BorTag){let c=a.cmp(b);assert('+condition+');}}',ok=False)
        need(j['inference']['failed_functions']==['wrapper::wrong'],'intended false client only')
        need(not j['verification']['encountered-vir-error'],'false client frontend failure')
    src=BASE+'verus!{\n'
    for i,(a,c) in enumerate([(1,1),(1,2**61),(2**61,2**63),(2**63,2**64-2),(2**64-2,2**64-1)]):
        for k,(x,y) in enumerate([(a,c),(c,a)]):
            src+=f'pub fn boundary_{i}_{k}(a:&BorTag,b:&BorTag)->(r:Ordering) requires value(a)=={x}u64,value(b)=={y}u64, ensures r==vstd::std_specs::cmp::OrdSpec::cmp_spec(&value(a),&value(b)),{{a.cmp(b)}}\n'
    expect('final-boundaries',src+'}\n')

def exports():
    # Each prior is genuinely produced by the SAME tools, not a dummy sentinel.
    provider=expect('final-bool-prior',BOOL,n=2,export=R/'final-bool-prior/wrapper.vir')
    raw=(R/'final-bool-prior/wrapper.vir').read_bytes();pos=raw.rfind(b'{"schema":');cert=json.loads(raw[pos:-8]);need(cert['schema']==39 and cert['policy']=='candidate-bound-evidence-v74','prior Bool metadata')
    rows={}
    for case,src in [('wrapper',CAND),('mixed',CAND+'verus!{pub fn boolean(b:bool)->u64{u64::from(b)}}'),('private',CAND.replace('pub fn total','fn total').replace('pub fn inferred_','fn inferred_')+'verus!{pub fn unrelated()->u64{9}}')]:
        for route in ['explicit','cargo','both','equal']:
            label='final-export-'+case+'-'+route;d=R/label;d.mkdir(exist_ok=True);cargo=d/'libwrapper.vir';explicit=cargo if route=='equal' else d/'selected.vir'
            selected=([explicit] if route=='explicit' else [cargo] if route=='cargo' else list(dict.fromkeys([explicit,cargo])))
            for p in selected:p.write_bytes(raw)
            j=expect(label,src,ok=False,cargo=route!='explicit',export=explicit if route!='cargo' else None)
            need('local-only' in (d/'stderr').read_text(),'local-only refusal '+label);need(all(not p.exists() for p in selected),'stale selected output '+label)
            rows[label]=dict(prior_sha256=hashlib.sha256(raw).hexdigest(),paths=list(map(str,selected)),removed=True)
    for flag in ['--no-verify','--no-lifetime','--no-trait-conflicts','--no-erasure-check']:
        for route in ['explicit','both']:
            label='final-disabled-'+flag[2:]+'-'+route;d=R/label;d.mkdir(exist_ok=True);a=d/'selected.vir';c=d/'libwrapper.vir';selected=[a,c] if route=='both' else [a]
            for p in selected:p.write_bytes(raw)
            expect(label,CAND,ok=False,cargo=route=='both',export=a,options=[flag]);need(all(not p.exists() for p in selected),'disabled selected output')
            rows[label]=dict(prior_sha256=hashlib.sha256(raw).hexdigest(),paths=list(map(str,selected)),removed=True)
    write(R/'final-export-invalidation.json',rows)
    deps=[('wrapper',R/'final-bool-prior')]
    for label,condition,ok,n,e in [('ordinary','r==if b {1u64}else{0u64}',True,1,0),('false','r==0u64',False,0,1)]:
        expect('final-bool-'+label,'use vstd::prelude::*;verus!{pub fn client(b:bool)->(r:u64) ensures '+condition+', {wrapper::old_api(b)}}',ok=ok,n=n,errors=e,goals=False,deps=deps,name='bool_'+label)

def compatibility():
    expect('final-stage1-direct',old.source('carrier_ord'),n=2)
    expect('final-stage1-off',old.source('carrier_ord'),ok=False,n=0,errors=1,goals=False)
    expect('final-stage1-partial',old.source('carrier_partial'),n=5)
    s=old.source('carrier_ord').replace('b:&NonZero<u64>','b:&NonZero<u64>').replace('(&a@,&b@)','(&a@,&a@)').replace('a.cmp(b)','a.cmp(a)');expect('final-stage1-alias',s,n=2)
    p=Path('/tmp/local-nonzero-ord-09t1tshb/source-1adb8b6c2ee64cc1.rs');need(sha(p)=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','authorized original Item hash')
    q=R/'original-proofless-item.rs';need(not q.exists(),'immutable Item copy');shutil.copyfile(p,q);expect('final-proofless-item',q.read_text(),n=132,export=R/'final-proofless-item/wrapper.vir')
    expect('final-packet',(ROOT/'source/tools/research/item_packing_goal/reuse.rs').read_text(),n=31,export=R/'final-packet/wrapper.vir')

def shapes():
    for name,s in [('generic',BASE.replace('BorTag(NonZero<u64>)','BorTag<T>(NonZero<T>)')),('u32',BASE.replace('NonZero<u64>','NonZero<u32>')),('multi',BASE.replace('BorTag(NonZero<u64>)','BorTag(NonZero<u64>,u64)')),('missing-ord',BASE.replace('PartialEq,PartialOrd,Ord))','PartialEq,PartialOrd))'))]:
        expect('final-shape-'+name,s,ok=False)
        for p in (R/('final-shape-'+name)/'vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'unintended helper '+name)
    look='''use vstd::prelude::*;
#[verus_verify] #[derive(Copy,Clone,Hash,PartialEq,Eq,PartialOrd,Ord)] pub struct NonZero<T>(T);
verus!{impl View for NonZero<u64>{type V=u64;open spec fn view(&self)->u64{self.0}}}
'''+BASE.replace('use vstd::prelude::*;','').replace('use std::num::NonZero;','')
    expect('final-shape-lookalike',look,ok=False)
    for p in (R/'final-shape-lookalike/vir').glob('*.vir'):need('checked_nonzero_u64_ord%' not in p.read_text(),'lookalike helper')

if __name__=='__main__':
    need(EPOCH=='proofs','final matching epoch required')
    {'semantics':semantics,'exports':exports,'compatibility':compatibility,'shapes':shapes}[sys.argv[1]]()
