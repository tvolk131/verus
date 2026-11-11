"""Read-only acceptance audit; --write emits only compact lane-owned manifests.
All checks use explicit exceptions and remain active with Python -B -O.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import trials as t

R=Path(sys.argv[1]); B=R/'bundle'

def load(p):return json.loads(Path(p).read_text())
def digest(x):return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()
def checked(d,tools=None):
    d=Path(d);x=load(d/'receipt.json');f=load(d/'freeze.json')
    t.need(x['guards'] is True,'failed receipt guards '+str(d))
    for n in ['freeze','stdout','stderr']:
        t.need(t.sha(d/(n+'.json' if n=='freeze' else n))==x[n+'_sha256'],'receipt bytes '+str(d)+' '+n)
    for p,h in f.get('fixtures',{}).items():t.need(t.sha(p)==h,'fixture drift '+p)
    if tools:
        t.need(f['tools']==tools,'proof tool identity drift '+str(d))
    return dict(path=str(d),receipt_sha256=t.sha(d/'receipt.json'),exit=x['exit'],
                verification=x.get('verification'),failed_functions=x.get('failed_functions',[])),f

def query(d,name):
    txt=(d/'vir/root-final.air').read_text();a=txt.index(';; Function-Def '+name+'\n')
    end=txt.find('\n;; Function-',a+1)
    return txt[a:end if end>=0 else len(txt)]

def main():
    t.need(sys.flags.dont_write_bytecode,'use Python -B')
    tools={n:t.sha(B/n) for n in t.TOOLS}
    build,_=checked(R/'final-build')
    vstd,vf=checked(R/'accepted-vstd')
    t.need(vstd['exit']==0 and '2250 verified, 0 errors' in (R/'accepted-vstd/stdout').read_text(),'matching vstd')
    t.need(load(R/'accepted-vstd/receipt.json')['outputs']=={n:tools[n] for n in ['vstd.vir','libvstd.rlib']},'vstd output identities')
    t.need(vf['tools']=={n:tools[n] for n in tools if n not in ['vstd.vir','libvstd.rlib']},'vstd compiler/macro identities')
    expected={'final-cold':(2,0),'final-warm':(2,0),'final-false':(1,1),'final-off':(0,1),
              'final-full':(52,0),'final-full-warm':(52,0),'final-support':(36,0)}
    proofs=[]
    for name,(v,e) in expected.items():
        row,f=checked(R/name,tools);out=load(R/name/'stdout');result=out['verification-results']
        t.need((result['verified'],result['errors'])==(v,e),'proof counts '+name)
        t.need(result['is-verifying-entire-crate'] is True,'complete scope '+name)
        t.need(out['verus-infer']['interface_written'] is False,'foreign publication '+name)
        proofs.append(row)
    t.need(proofs[2]['failed_functions']==['component::all_bool'],'healthy false caller')
    t.need(t.sha(R/'source-actual.rs')==t.sha(R/'source-off.rs'),'same source goals off')
    # Pure source generation and fidelity only; no old epoch/audit drivers.
    origin=t.HERE.parent/'item_tag_constructor_composition';sys.path.insert(0,str(origin))
    from source import generate,fidelity
    from inventory import inventory,signature
    text=generate('full-help');t.need(text==(R/'source-full-help.rs').read_text(),'full original source')
    t.need(t.sha(R/'source-full-help.rs')==t.sha('/tmp/checked-bool-full-help-tjcf3qnk/component.rs'),'previous frozen full bytes')
    identities={}
    for name in ['final-cold','final-full','final-support']:
        inv=inventory(R/name);identities[name]=inv
        if name=='final-full':
            for helper in ['component::checked_foreign_bool_to_u64%','component::BorTag::ctor%','component::Item::ctor%']:
                t.need(inv['ordinary_original_functions'][helper]['body_present'],'missing ordinary body '+helper)
            scope=inv['native_scope'];t.need(scope['checked_exec_bodies']==16 and scope['checked_proof_bodies']==1,'full HIR scope')
            row=load(R/name/'stdout')['verus-infer']['functions']
            t.need(len(row)==1 and row[0]['function']=='component::BorTag::eq','exact Eq scope')
            t.need(row[0]['contract']['requires']=='true','original Eq full domain')
            for p in row[0]['proofs'].values():t.need(p['required'] and p['status']=='proved','mandatory Eq proof')
    stable={}
    helper='component::checked_foreign_bool_to_u64%'
    for kind in ['actual','full-help']:
        base=R/('stable-'+kind);a,af=checked(base/'cold',tools);b,bf=checked(base/'warm',tools)
        t.need(af==bf,'exact frozen invocation changed '+kind)
        t.need(signature(base/'cold')==signature(base/'warm'),'typed body/owner identity changed '+kind)
        cold=query(base/'cold',helper);warm=query(base/'warm',helper)
        t.need(cold==warm,'ordinary helper proof query identity changed '+kind)
        stable[kind]=dict(cold=a,warm=b,frozen_inputs_sha256=digest(af),ordinary_helper_query_sha256=hashlib.sha256(cold.encode()).hexdigest())
        if kind=='full-help':
            ca=load(base/'cold/stdout')['verus-infer']['functions'][0]
            wa=load(base/'warm/stdout')['verus-infer']['functions'][0]
            t.need(wa['metrics']['cache']=='hit','warm candidate-cache reuse')
            stable[kind]['Eq_cold_candidate_id']=ca['candidate_id']
            stable[kind]['Eq_warm_candidate_id']=wa['candidate_id']
            stable[kind]['Eq_cold_warm_candidate_equal']=ca['candidate_id']==wa['candidate_id']
            stable[kind]['parent_cache73_reconciliation_required']=ca['candidate_id']!=wa['candidate_id']
            stable[kind]['Eq_warm_proofs']=wa['proofs']
            repeat,rf=checked(base/'warm-repeat',tools)
            ra=load(base/'warm-repeat/stdout')['verus-infer']['functions'][0]
            t.need(rf==bf and ra['candidate_id']==wa['candidate_id'],'warm repeat identity changed')
            for proof in ra['proofs'].values():t.need(proof['required'] and proof['status']=='proved','repeat precision proof')
            stable[kind]['warm_repeat']=repeat
    exports=load(R/'export-controls.json');t.need(len(exports['records'])==24,'all explicit/cargo export controls')
    for row in exports['records']:
        t.need(not Path(row['selected']).exists(),'stale foreign export '+row['label'])
        checked(R/row['label'],tools)
    shapes=load(R/'call-shape-controls.json');t.need(len(shapes)==6,'nearby shape coverage')
    for row in shapes:t.need(row['off']==row['on'],'nearby source behavior changed')
    faults=[]
    for name in ['generated-body','generated-type','false-proposal','mir-local-type','mir-extra-block','packing-proposal']:
        row,_=checked(R/('fault-'+name)/'proof');t.need(row['exit']!=0,'fault accepted '+name);faults.append(dict(control=name,**row))
    compiler,_=checked(R/'final-all-compiler-tests',tools)
    t.need(compiler['exit']==0 and '5 passed; 0 failed' in (R/'final-all-compiler-tests/stdout').read_text(),'final compiler tests')
    runtime,_=checked(R/'runtime-execution',tools);t.need(runtime['exit']==0,'concrete runtime')
    mir=R/'runtime/component.mir';t.need(mir.exists(),'runtime MIR')
    t.need('checked_foreign_bool_to_u64' not in mir.read_text(),'synthetic runtime replacement')
    unchanged={}
    for p in ['source/vir','source/vstd']:
        before=subprocess.check_output(['git','rev-parse','15045979:'+p],cwd=t.ROOT).decode().strip()
        after=subprocess.check_output(['git','rev-parse','HEAD:'+p],cwd=t.ROOT).decode().strip()
        t.need(before==after,'parent-owned tree changed '+p);unchanged[p]=after
    result=dict(status='implemented_local_checked_path_pending_independent_parent_review',
                full_identity_acceptance=False,parent_cache73_reconciliation_required=True,
                engine_source_commit='3e4c8d94',tools=tools,private_root=str(R),
                baseline_receipt_sha256='2e3fcc58f6fca9364061329104710937ec2aac6aab95a5c48d068c1ca6ee9bb7',
                build=build,vstd=vstd,proofs=proofs,stable=stable,fidelity=fidelity(text),
                full_source_sha256=t.sha(R/'source-full-help.rs'),ordinary_identities=identities,
                export_controls=exports,call_shapes=shapes,fault_controls=faults,
                compiler_tests=compiler,semantic_compiler_tests='5 passed, 0 failed in final-all-compiler-tests',
                runtime=runtime,runtime_mir_sha256=t.sha(mir),parent_owned_unchanged=unchanged,
                build_products_manifest_sha256=t.sha(R/'final-build-products.json'),
                native_correspondences=0,publication=False,goal_complete=False,
                inherited=dict(canonical='parent 6/0, not rerun',HMAC='1/1 UNPROVED, not rerun',open_obligations='44 minimal +3 phase /101 writer effects OPEN'))
    if '--write' in sys.argv:t.write(t.HERE/'RESULTS.json',result)
    print(json.dumps(dict(local_evidence_validated=True,full_identity_acceptance=False,parent_review_pending=True,publication=False,native=0,goal_complete=False,tools_sha256=digest(tools)),sort_keys=True))

if __name__=='__main__':main()
