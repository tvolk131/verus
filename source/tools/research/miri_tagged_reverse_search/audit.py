"""Read-only evidence acceptance (also under -O); seal is a separate one-shot step."""
import hashlib,json,re,sys
from pathlib import Path
import build as b
import source as s
R=b.root();H=b.HERE;B=R/'bundle'
_MANIFEST=b.manifest();_HASHES={}
PROBES={'forward_loop':(2,0),'slice_reverse_loop':(2,0),'forward_laws':(1,0),
        'reverse_law':(0,1),'reverse_decrease':(1,0),'reverse_remaining':(1,0),
        'reverse_inner':(0,1),'reverse_loop':(1,1)}
COMPONENTS={'dependencies':(31,0),'fallback_plain':(32,1),'fallback':(31,2),
            'full_original':None,'wrong_write_property':(30,1),'mutated_grants':(30,1)}
def read(p):return json.loads(Path(p).read_text())
def needed_hash(p,h):
    key=str(p)
    if key not in _HASHES:_HASHES[key]=b.sha(p)
    b.need(_HASHES[key]==h,'hash mismatch '+key)
def final_guard():
    b.need(b.manifest()==_MANIFEST,'source changed during audit')
    for p,h in _HASHES.items():b.need(b.sha(p)==h,'input changed during audit '+p)
def expected(name):
    if name in PROBES:return s.probe(name)
    text=s.component(name if name in ['dependencies','fallback_plain','fallback','full_original'] else 'dependencies')
    if name=='wrong_write_property':text=text.replace('ensures r==(p==Permission::Unique || p==Permission::SharedReadWrite)','ensures r!=(p==Permission::Unique || p==Permission::SharedReadWrite)')
    if name=='mutated_grants':text=text.replace('self != Permission::Disabled\n','self == Permission::Disabled\n')
    return text

def inventory(data):
    i=data.get('verus-infer',{});sc=i.get('verification_scope',{});groups={}
    for row in sc.get('native_functions',[]):groups.setdefault(row['body_status'],[]).append(row['function'])
    for k in groups:groups[k]=sorted(groups[k])
    fs=i.get('functions',[])
    return dict(reported_checked_exec_count=sc.get('checked_exec_bodies',0),reported_checked_proof_count=sc.get('checked_proof_bodies',0),
        complete_selected_scope_verified=sc.get('complete_selected_scope_verified',False),body_sets_by_reported_status=groups,
        inferred_functions=[dict(function=f['function'],proofs=f['proofs'],required_checks_passed=f['required_checks_passed'],exported=f['exported']) for f in fs],
        inferred_function_count=len(fs),synthetic_constructor_queries=sorted(k for k in data.get('func-details',{}) if k.startswith('tagged::') and 'ctor%' in k),
        all_local_query_items=sorted(k for k in data.get('func-details',{}) if k.startswith('tagged::')),
        failed_functions=i.get('failed_functions',[]),goal_assistance=i.get('goal_assistance',[]))

def final_record(label,name,counts):
    d=R/label;f=read(d/'freeze.json');j=read(d/'receipt.json');data=read(d/'stdout');err=(d/'stderr').read_text();v=data['verification-results'];i=data['verus-infer']
    b.need(j['guards'],'failed input guards '+label);needed_hash(d/'freeze.json',j['freeze_sha256']);needed_hash(d/'stdout',j['stdout_sha256']);needed_hash(d/'stderr',j['stderr_sha256'])
    b.need(f['source']==_MANIFEST,'production source drift')
    for n,h in f['tools'].items():needed_hash(B/n,h)
    for n,h in f['toolchain']['files'].items():needed_hash(n,h)
    for n,h in f['controllers'].items():needed_hash(R/'controllers'/(h+'.py'),h)
    b.need(len(f['fixture'])==1,'one immutable fixture');p,h=next(iter(f['fixture'].items()));needed_hash(p,h);text=Path(p).read_text();b.need(text==expected(name),'exact source transformation '+label)
    b.need('--no-cheating' in f['argv'] and '--compile' in f['argv'],'ordinary checking');b.need(not any(x in f['argv'] for x in ['--no-lifetime','--no-erasure-check','--no-verify','--verify-function','--verify-module']),'no filters/weakened checks')
    b.need(not any(x in err.lower() for x in ['resource limit','timed out','panicked at','internal compiler error']),'excluded resource/compiler failure '+label)
    b.need(not re.search(r'\b(?:assume|admit|axiom|external_body|assume_specification|unsafe)\b',text),'forbidden source escape')
    if counts is None:
        b.need(j['exit']!=0 and v['encountered-vir-error'] and 'swap` is not supported' in err,'literal full helper barrier')
        classification='unsupported_frontend_not_semantic_control'
    else:
        b.need((v['verified'],v['errors'])==counts and not v['encountered-vir-error'] and v['is-verifying-entire-crate'],'exact ordinary outcome '+label)
        b.need((j['exit']==0)==(counts[1]==0),'exit/result agreement')
        failed=[] if counts[1]==0 else ['tagged::'+({'fallback_plain':'Stack::tagged_fallback','fallback':'Stack::tagged_fallback','wrong_write_property':'write_precision','mutated_grants':'Permission::grants'}.get(name,name))]
        b.need(i['failed_functions']==failed,'exact intended function '+label)
        classification='positive' if not failed else ('strict_dependency_control' if name in ['wrong_write_property','mutated_grants'] else 'open_ordinary_support_or_application_obligation')
        for fn in i['functions']:b.need(fn['required_checks_passed'] and all(x['status']=='proved' for x in fn['proofs'].values()),'inferred helper health')
        if name in PROBES:b.need(i['functions']==[] and i['goal_assistance']==[],'explicit zero helper probes')
        else:
            b.need([fn['function'] for fn in i['functions']]==['tagged::BorTag::eq'],'exact inferred equality set')
            b.need('use std::ops::Range;' in text and 'pub struct BorTag(NonZero<u64>);' in text and 'pub struct Item(u64);' in text,'actual carriers')
            if name=='mutated_grants':
                try:s.fidelity(text,'dependencies')
                except RuntimeError as e:b.need(str(e)=='Permission.grants tokens','mutation fails only intended executable fidelity')
                else:raise RuntimeError('runtime mutation undetected')
            else:s.fidelity(text,'dependencies' if name=='wrong_write_property' else name)
        if name in ['wrong_write_property','mutated_grants']:
            b.need('postcondition not satisfied' in err,'semantic failure expected')
            b.need(not any(x in err for x in ['precondition not satisfied','assertion failed','decreases not satisfied','loop invariant','not supported']),'control precision/safety health')
    return dict(classification=classification,source_sha256=h,receipt_sha256=b.sha(d/'receipt.json'),freeze_sha256=j['freeze_sha256'],verification=v,failed_functions=i['failed_functions']),inventory(data)

def collect():
    b.need(read(R/'build-final/receipt.json')['guards'] and read(R/'build-final/receipt.json')['exit']==0,'stable matching compiler build')
    b.need(read(R/'vstd/receipt.json')['guards'] and read(R/'vstd/receipt.json')['exit']==0,'full vstd')
    b.need('2250 verified, 0 errors' in (R/'vstd/stdout').read_text(),'normal full vstd 2250/0')
    initial=read(R/'application-freeze-v2.json');b.need(initial['source']==_MANIFEST,'frozen production source')
    for n,h in initial['tools'].items():needed_hash(B/n,h)
    b.need(len(initial['tools'])==8,'eight frozen tools')
    s.primary();rows={};scopes={}
    for group,prefix in [(PROBES,'v2-'),(COMPONENTS,'v4-')]:
        for n,c in group.items():
            for g in ['plain-','goals-']:
                label=prefix+g+n;rows[label],scopes[label]=final_record(label,n,c)
                if g=='goals-':
                    off=rows[prefix+'plain-'+n];b.need(rows[label]['source_sha256']==off['source_sha256'],'causal source pairing');b.need(rows[label]['verification']==off['verification'],'exact measured off/on outcome agreement')
    attempts=[]
    for p in sorted(R.glob('*/freeze.json')):
        d=p.parent;j=read(d/'receipt.json') if (d/'receipt.json').exists() else None
        attempts.append(dict(label=d.name,freeze_sha256=b.sha(p),receipt_sha256=b.sha(d/'receipt.json') if j else None,exit=j.get('exit') if j else None,guards=j.get('guards') if j else None,
            status='final' if d.name in rows else ('excluded_incomplete_controller_interrupted_130' if not j else 'retained_setup_build_or_expanded_dependency_diagnostic'),
            verification=j.get('verification') if j else None,raw_directory=str(d)))
    # Retain the meaningful inherited ordering ablation, without attributing it to this lane.
    expanded={}
    for label,count in [('v3-plain-dependencies',(32,1)),('v3-goals-dependencies',(105,0))]:
        d=R/label;j=read(d/'receipt.json');f=read(d/'freeze.json');data=read(d/'stdout');b.need(j['guards'],'expanded diagnostic guard');needed_hash(d/'stdout',j['stdout_sha256']);needed_hash(d/'stderr',j['stderr_sha256']);needed_hash(d/'freeze.json',j['freeze_sha256']);
        p,h=next(iter(f['fixture'].items()));needed_hash(p,h);b.need(Path(p).read_text()==s.component('dependencies').replace('infer_derive(PartialEq)','infer_derive(PartialEq,PartialOrd,Ord)'),'expanded original dependency source')
        for n,h in f['controllers'].items():needed_hash(R/'controllers'/(h+'.py'),h)
        b.need((data['verification-results']['verified'],data['verification-results']['errors'])==count,'expanded ordering result')
        expanded[label]=dict(receipt_sha256=b.sha(d/'receipt.json'),source_sha256=j['source_sha256'],verification=data['verification-results'],scope=inventory(data))
    b.need(expanded['v3-plain-dependencies']['source_sha256']==expanded['v3-goals-dependencies']['source_sha256'],'expanded causal source identity')
    result=dict(schema=1,private_root=str(R),base='38b09d18',early_commit='dc65ef77',runtime_tools=initial['tools'],vstd='2250/0',final_trials=rows,inherited_ordering_diagnostic=expanded,
        conclusions=dict(fallback_internal_theorem='UNPROVED',whole_tagged_method='UNPROVED',native_correspondences=0,canonical='6/0 unchanged',HMAC='1/1 UNPROVED',writer_effects='44+3/101 OPEN',publication=False,goal_complete=False),
        cache_import='no new assistance or reusable iterator lemma; no cache/import theorem claimed; illegal ordering exports not attempted')
    final_guard()
    return {'RESULTS.json':result,'INVENTORY.json':scopes,'ATTEMPTS.json':attempts}

def seal():
    for n,v in collect().items():b.need(not (H/n).exists(),'immutable output '+n);b.write(H/n,v)
    targets=[p for p in H.iterdir() if p.is_file() and p.name not in ['PRIVATE_ROOT','AUDIT_FREEZE.json','VALIDATION.json']]
    b.write(H/'AUDIT_FREEZE.json',dict(files={p.name:b.sha(p) for p in sorted(targets)},source=_MANIFEST,tools={n:b.sha(B/n) for n in b.TOOLS}))

def check():
    frozen=read(H/'AUDIT_FREEZE.json');b.need(frozen['source']==_MANIFEST,'final source guard')
    for n,h in frozen['files'].items():needed_hash(H/n,h)
    for n,h in frozen['tools'].items():needed_hash(B/n,h)
    for n,v in collect().items():b.need(read(H/n)==v,'read-only audit agreement '+n)
    print(json.dumps(dict(audit='passed',result_sha256=b.sha(H/'RESULTS.json'),final_trials=28,fallback='UNPROVED',native=0,publication=False,goal_complete=False),sort_keys=True))
if __name__=='__main__':
    b.need(sys.flags.dont_write_bytecode,'python -B required')
    {'seal':seal,'check':check}[sys.argv[1]]()
