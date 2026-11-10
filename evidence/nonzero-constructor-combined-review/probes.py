"""Combined-scope application controls; no historical controller execution."""
import json, re, sys, shutil
from pathlib import Path as P
import review as e
HERE=e.HERE
W=P(e.read(HERE/'work.json')['work'])
B=W/'final-bundle'
DECL='#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct BorTag(NonZero<u64>);'
BASE='use vstd::prelude::*;\nuse std::num::NonZero;\n#[verus_verify(infer_derive(PartialEq))]\n'+DECL+'\n'
GOAL='''verus! {
spec fn tag_value(t:&BorTag)->u64 {t.0@}
fn compare_equal(a:&BorTag,b:&BorTag)->(r:bool)
    ensures r == (tag_value(a) == tag_value(b)),
{ a == b }
}
'''
TYPED='''verus! {
impl BorTag {
    pub open spec fn __verus_infer_derived_eq(&self)->bool {true}
    pub closed spec fn __verus_infer_nonzero_u64_view(&self)->u64 {self.0@}
}
impl vstd::std_specs::cmp::PartialEqSpecImpl for BorTag {
    open spec fn obeys_eq_spec()->bool {true}
    closed spec fn eq_spec(&self,rhs:&Self)->bool {self.0@ == rhs.0@}
}
}
'''
COMPOSE='''verus! {
impl BorTag {
    pub closed spec fn value(&self)->u64 {self.0@}
    pub fn new(i:u64)->(r:Option<Self>)
        ensures r.is_some() == (i!=0), r.is_some() ==> r.unwrap().value()==i,
    { NonZero::new(i).map(BorTag) }
    pub fn get(&self)->(r:u64) ensures r==self.value(),
    { self.0.get() }
}
fn all_u64(i:u64)->(r:u64) ensures r==i,
{ match BorTag::new(i) {Some(tag)=>tag.get(),None=>0} }
fn compare_created(i:u64,j:u64) {
    match (BorTag::new(i),BorTag::new(j)) {
        (Some(a),Some(b))=> { let r=a==b; assert(r==(i==j)); }
        _=>{}
    }
}
fn compare_width(a:&BorTag,b:&BorTag) {
    let r=a==b;
    assert(r==(a.value()==b.value()));
    let av=a.get();let bv=b.get();
    if av==u64::MAX && bv==u64::MAX {assert(r);}
    if av==9223372036854775808u64 && bv==1 {assert(!r);}
}
}
'''
CLIENT='''use vstd::prelude::*;
use component::BorTag;
#[verus_infer] fn equal(a:&BorTag,b:&BorTag)->bool {<BorTag as PartialEq>::eq(a,b)}
verus! {
fn goal(a:&BorTag,b:&BorTag) {
    let r=equal(a,b);
    assert(r==(a.__verus_infer_nonzero_u64_view()==b.__verus_infer_nonzero_u64_view()));
}
}
'''
def src(case):
    if case=='import' or case=='old-policy':return CLIENT
    if case=='compose':return BASE+COMPOSE
    if case=='weak':return BASE.replace('infer_derive(PartialEq)','')+TYPED.replace('{self.0@}','{0}')+GOAL
    if case=='reversed':return BASE.replace('infer_derive(PartialEq)','')+TYPED.replace('self.0@ == rhs.0@','self.0@ != rhs.0@')+GOAL
    s=BASE+GOAL
    if case=='false':s=s.replace('ensures r ==','ensures r !=')
    if case=='width':s=s.replace('{ a == b }','''{ let r=a==b;
        proof {
        if tag_value(a)==u64::MAX && tag_value(b)==u64::MAX {assert(r);}
        if tag_value(a)==9223372036854775808u64 && tag_value(b)==1 {assert(!r);}
        }
        r }''')
    if case=='u32':s=s.replace('NonZero<u64>','NonZero<u32>')
    if case=='ordering':s=s.replace('infer_derive(PartialEq)','infer_derive(PartialEq,PartialOrd,Ord)')
    return s

def bodies(i):
    return [f for f in i['verification_scope']['native_functions'] if f['selected'] and f['body_status'] not in ['specification_definition_or_declaration','selected_item_without_translated_body']]
def exact(i,helper='component::BorTag::eq'):
    e.need([f['function'] for f in i['functions']]==[helper],'helper set differs')
    for f in i['functions']:
        e.need(f['mode']=='exact' and f['required_checks_passed'] is True,'not exact')
        for k in ['soundness','maximality','exactness']:e.need(f['proofs'][k]['required'] is True and f['proofs'][k]['status']=='proved','obligation '+k)
        e.need(f['contract']['requires']=='true','domain is not full inherited domain')

def proof(case):
    d=W/'probes-v3'/case;d.mkdir(parents=True,exist_ok=True)
    source=d/('client.rs' if case in ['import','old-policy'] else 'component.rs');source.write_text(src(case))
    provider=W/'probes-v3/macro'
    extra=[]
    if case=='old-policy':provider=P('/tmp/nonzero-owned-macro-4okwhwsz')
    if case in ['import','old-policy']:extra=[provider/'component.vir',provider/'libcomponent.rlib',provider/'receipt.json'] if case=='old-policy' else [provider/'component.vir',provider/'libcomponent.rlib']
    plan=dict(case=case,expected_helper='client::equal' if case=='import' else 'component::BorTag::eq',original_eq_body='{ self.0 == other.0 }',body_sha256='1774afdac1844226361985602334fc3b076cb8194e4f4efcc8c0b18fd2eb229b',policy=69,cache=72,full_domain=True,application_no_cheating=True,expected='shape_rejection' if case in ['u32','ordering','old-policy'] else 'original_eq_obligation_failure' if case in ['weak','reversed'] else 'healthy_eq_false_goal' if case=='false' else 'full_positive',expected_bodies=['<BorTag as std::clone::Clone>::clone','<BorTag as std::cmp::PartialEq>::eq','BorTag::ctor%','compare_equal'],constructor_body_included=True)
    if case=='compose':plan['expected_bodies']=['<BorTag as std::clone::Clone>::clone','<BorTag as std::cmp::PartialEq>::eq','BorTag::ctor%','BorTag::new','BorTag::get','all_u64','compare_created','compare_width']
    if case in ['import','old-policy']:plan['expected_bodies']=['equal','goal']
    crate='client' if case in ['import','old-policy'] else 'component'
    execs=['equal','goal'] if crate=='client' else ['BorTag::clone','BorTag::eq','BorTag::ctor%']+(['BorTag::new','BorTag::get','all_u64','compare_created','compare_width'] if case=='compose' else ['compare_equal'])
    specs=[] if crate=='client' else ['BorTag::__verus_infer_derived_eq','BorTag::__verus_infer_nonzero_u64_view','BorTag::obeys_eq_spec','BorTag::eq_spec']+(['BorTag::value'] if case=='compose' else ['tag_value'])
    raw={'BorTag::clone':'impl&%2::clone','BorTag::eq':'impl&%5::eq','BorTag::new':'impl&%12::new','BorTag::get':'impl&%12::get'}
    plan['expected_original_exec_paths']=[crate+'::'+raw.get(x,x) for x in execs]
    plan['expected_queries']=3 if case in ['weak','reversed'] else len(execs)+2
    plan['ordinary_body_phase_blocked_by_bad_eq']=case in ['weak','reversed']
    plan['expected_all_local_detail_paths']=[crate+'::'+x for x in execs+specs]
    plan['generated_exec_paths']=[] if crate=='client' else ['component::BorTag::ctor%']
    e.write(d/'plan.json',plan)
    argv=[B/'verus',source,'--crate-type=lib','--crate-name='+('client' if case in ['import','old-policy'] else 'component'),'--no-cheating','--output-json','--multiple-errors','20','--log-all','--log-dir',d/'vir']
    if case!='off':argv+=['--infer-goals']
    # Request export for all component controls, including failed obligations/goals.
    if case not in ['import','old-policy']:argv+=['--export',d/'component.vir','--compile','-o',d/'libcomponent.rlib']
    else:argv+=['--extern','component='+str(provider/'libcomponent.rlib'),'--import','component='+str(provider/'component.vir'),'-L',provider]
    if case=='old-policy':
        old_bundle=P('/tmp/nonzero-derive-frozen-tools-8hnei2sw');argv+=['-L',old_bundle];extra += [old_bundle/n for n in e.NAMES]+[old_bundle/'receipt.json']
    tools=[B/n for n in e.NAMES]+[B/'receipt.json']
    results=[]
    for phase in (['cold','warm'] if case=='macro' else ['check']):
        out,r=e.epoch(W,case+'-'+phase,argv,extra=tools+[source,d/'plan.json']+extra,envextra=dict(VERUS_INFER_CACHE=str(d/'cache')),cwd=d,timeout=180,products=[d/'component.vir',d/'libcomponent.rlib'])
        retained=out/'retained';retained.mkdir()
        for name in ['component.vir','libcomponent.rlib',source.name,'plan.json']:
            if (d/name).is_file():shutil.copyfile(d/name,retained/name)
        if (d/'vir').is_dir():shutil.copytree(d/'vir',retained/'vir')
        err=(out/'stderr').read_text();data=e.read(out/'stdout');v=data['verification-results'];i=data['verus-infer'];rc=r['exit']
        summary=dict(retained_products={str(x.relative_to(out)):e.sha(x) for x in retained.rglob('*') if x.is_file()},path=str(out),case=case,phase=phase,verification=v,inference=i,source_sha256=e.sha(source),plan_sha256=e.sha(d/'plan.json'),classification='unclassified')
        try:
            if case in ['u32','ordering','old-policy']:
                e.need(rc!=0 and (v.get('encountered-error') or v.get('encountered-vir-error')),'not frontend/policy rejection')
                e.need(not i['functions'],'unexpected inference')
                text='infer_derive' if case=='u32' else 'support only PartialEq' if case=='ordering' else 'incompatible inference schema, safety policy, or producer'
                e.need(text in err,'missing intended diagnostic');summary['classification']='policy_rejection' if case=='old-policy' else 'unsupported_shape'
            else:
                e.need(v.get('is-verifying-entire-crate') is True and isinstance(v.get('success'),bool) and not v.get('encountered-vir-error'),'not full ordinary checking')
                actual=bodies(i);hir_expected=set(plan['expected_bodies'])-{'BorTag::ctor%'}
                e.need(len(actual)==len(hir_expected) and {b['rust_function'] for b in actual}==hir_expected,'HIR-native body inventory differs')
                local_details={k:v for k,v in data['func-details'].items() if k.startswith(crate+'::')}
                e.need(set(local_details)==set(plan['expected_all_local_detail_paths']),'all ordinary local function details differ')
                e.need(all(not v['failed_proof_notes'] for k,v in local_details.items() if k in plan['generated_exec_paths']),'generated constructor failure')
                import identity
                text=(d/'vir/crate.vir').read_text();original_execs={}
                for m in re.finditer(r'\(Function\s',text):
                    f=text[m.start():identity.end(text,m.start())];name=identity.field(f,'name')
                    path=name[len('(Fun :path '):-1]
                    if path.startswith(crate+'::') and identity.field(f,'mode')=='Exec' and identity.field(f,'body')!='None':
                        original_execs[path]=__import__('hashlib').sha256(identity.field(f,'body').encode()).hexdigest()
                e.need(set(original_execs)==set(plan['expected_original_exec_paths']),'original executable body inventory differs: '+str(list(original_execs)))
                e.need(v['verified']+v['errors']==plan['expected_queries'],'query inventory differs')
                summary.update(all_local_function_details=local_details,original_exec_body_hashes=original_execs,generated_exec_paths=plan['generated_exec_paths'])
                if case in ['weak','reversed']:
                    f=i['functions'][0];e.need([f['function'] for f in i['functions']]==['component::BorTag::eq'],'wrong helper')
                    e.need(rc!=0 and v['errors']==2 and i['failed_functions']==['component::BorTag::eq'],'wrong original eq failure')
                    for k in ['soundness','exactness']:e.need(f['proofs'][k]['status']=='not_proved' and f['proofs'][k]['required'] is True,'bad model accepted')
                    e.need(f['proofs']['maximality']['status']=='proved','domain obligation failed');summary['classification']='original_eq_obligation_failure'
                elif case=='false':
                    exact(i);e.need(rc!=0 and v['errors']==1 and i['failed_functions']==['component::compare_equal'] and 'postcondition not satisfied' in err,'not healthy helper semantic rejection');summary['classification']='healthy_eq_false_goal'
                else:
                    exact(i,'client::equal' if case=='import' else 'component::BorTag::eq')
                    e.need(rc==0 and v['success'] is True and v['errors']==0 and i['verification_scope']['complete_selected_scope_verified'] is True,'ordinary proof failed')
                    e.need(all(b['body_status']=='checked_exec_body' for b in actual),'unverified selected body')
                    if case=='macro':e.need(i['functions'][0]['metrics']['cache']==('miss' if phase=='cold' else 'hit'),'not cold/warm')
                    if case=='import':e.need(bool(i['imported_inference_dependencies']),'missing import dependency')
                    summary['classification']='full_positive'
            exported=case not in ['import','old-policy','u32','ordering','weak','reversed','false']
            e.need(i['interface_written'] is exported,'publication status differs')
            e.need((d/'component.vir').exists() is exported,'artifact status differs')
            summary['acceptance_checks_passed']=True
        finally:
            e.write(out/'validation.json',summary)
            with (W/'validations.jsonl').open('a') as f:f.write(json.dumps(dict(path=str(out),validation_sha256=e.sha(out/'validation.json')))+'\n')
        results.append(summary)
        print(case,phase,summary['classification'],v,flush=True)
    return results

def tests():
    layout=W/'target-verus';layout.mkdir(exist_ok=True);
    if not (layout/'release').exists():(layout/'release').symlink_to(B,target_is_directory=True)
    argv=[e.TC/'bin/cargo','test','--offline','--locked','--release','-p','rust_verify_test','--test','infer_nonzero_derive','--test','infer_compare_derive','--test','constructor_function_values','--','--test-threads=2']
    out,r=e.epoch(W,'tests',argv,extra=[B/n for n in e.NAMES]+[B/'receipt.json'],envextra={'VERUS_EXTRA_ARGS':'','VERUS_KEEP_TEST_DIR':'1','VERUS_TEST_LOG_DIR':str(W/'test-logs')},timeout=1800)
    e.need(r['exit']==0,'focused suites failed')
    log=(out/'stdout').read_text()
    for n in [3,7,13]:e.need(f'test result: ok. {n} passed' in log,'missing test result')
    print(log[-4000:],flush=True)
if __name__=='__main__':
    e.need(DECL in e.UPSTREAM.read_text(),'native declaration drift')
    if sys.argv[1]=='tests':tests()
    else:proof(sys.argv[1])
