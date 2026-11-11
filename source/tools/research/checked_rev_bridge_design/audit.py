"""Read-only, explicit-exception acceptance. No assertions, subprocess probes or writes."""
import hashlib,json,pathlib,subprocess,sys,re
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[3]
def need(b,m):
    if not b: raise RuntimeError(m)
def sha(p):return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(pathlib.Path(p).read_text())
def binding(o):need(sha(o['path'])==o['sha256'],'binding '+o['path'])
def main():
    need(sys.flags.dont_write_bytecode,'run with -B')
    freeze=load(HERE/'FREEZE.json');result=load(HERE/'RESULTS.json');r=pathlib.Path(result['private_root'])
    for p,h in freeze['inputs'].items():need(sha(ROOT/p)==h,'early input '+p)
    f=load(r/'freeze.json')
    need(f['receipt_sha256']=='5e7e79cf6b6cef0509226cf6134c8fce0a8c47ed608fc4c5a7520b5023aa4c45','receipt binding')
    need(f['tools']==result['tools'] and f['toolchain']==result['toolchain'],'tool manifests')
    for n,h in f['tools'].items():need(sha(r/'bundle'/n)==h,'private tool '+n)
    need(len(f['tools'])==8,'eight tools')
    for p,h in f['toolchain'].items():need(sha(p)==h,'toolchain '+p)
    for p,h in f['controllers'].items():need(sha(p)==h,'controller '+p)
    for p,h in f['production'].items():need(sha(ROOT/p)==h,'production '+p)
    for p,h in result['products'].items():need(sha(p)==h,'product '+p)
    need(load(r/'RESULTS.json')==result,'private result agreement')
    for name,t in result['trials'].items():
        binding(t['stdout']);binding(t['stderr'])
        rec=load(pathlib.Path(t['stdout']['path']).parent/'receipt.json')
        need(all(t[k]==v for k,v in rec.items()),'trial receipt '+name)
    inv=load(HERE/'INVENTORY.json');ids=load(HERE/'IDENTITIES.json');decision=load(HERE/'DECISION.json')
    need(len(ids['instances'])==3 and ids['diagnostic_only'],'three diagnostic instances')
    binding(ids['transcript'])
    chunks=pathlib.Path(ids['transcript']['path']).read_text().split('\nINSTANCE ')[1:]
    need(len(chunks)==len(ids['instances']),'instance extraction')
    for chunk,ident in zip(chunks,ids['instances']):
        need(ident['instance']==chunk.splitlines()[0],'instance identity')
        generic=chunk.split('GENERIC_BODY ',1)[1].split('\nBODY ',1)[0]
        body=chunk.split('\nBODY ',1)[1].split('\nLOCAL ',1)[0]
        need(hashlib.sha256(generic.encode()).hexdigest()==ident['generic_mir_text_sha256'],'generic MIR digest')
        need(hashlib.sha256(body.encode()).hexdigest()==ident['instantiated_mir_text_sha256'],'instantiated MIR digest')
        need(re.findall(r'^BLOCK (.*)$',chunk,re.M)==ident['blocks'],'block extraction')
        need(list(dict.fromkeys(re.findall(r'^FIELD (.*)$',chunk,re.M)))==ident['fields'],'field extraction')
        for key,value in ident['metadata'].items():need(re.search(r'^'+key+r' (.*)$',chunk,re.M).group(1)==value,'signature/identity extraction')
    for x in ids['instances']:
        need(len(x['blocks'])==1 and 'CLEANUP false' in x['blocks'][0] and 'TERMINATOR return' in x['blocks'][0],'constructor/getter shape')
    for p,h in decision['additional_inspected_inputs'].items():need(sha(ROOT/p)==h,'additional inspected input')
    for name,t in inv['trials'].items():
        raw=load(pathlib.Path(result['trials'][name]['stdout']['path']))
        v=raw['verification-results'];need(v==t['verification'],'verification inventory')
        need(v['is-verifying-entire-crate'] and not v['encountered-vir-error'],'whole crate/frontend')
        kind,mode=name.rsplit('-',1);expected={'available':(1,0),'lawful':(0,1),'inner':(0,1),'loop':(1,1)}[kind]
        need((v['verified'],v['errors'])==expected,'outcome '+name)
        need(v['success']==(kind=='available'),'success '+name)
        scope=raw['verus-infer']['verification_scope']
        for key in ['checked_exec_bodies','checked_proof_bodies','native_functions','native_collection_completed','complete_selected_scope_verified']:
            need(t[key]==scope[key],'scope inventory '+key)
        need(t['helpers_requested']==0 and t['helpers_generated']==0,'zero helpers')
        need(t['checked_exec_bodies']==(1 if kind=='available' else 0) and t['checked_proof_bodies']==0,'certified bodies')
        need(t['native_collection_completed'],'body inventory complete')
        argv=result['trials'][name]['argv'];need('--no-cheating' in argv and '--compile' in argv,'ordinary flags')
        need(('--infer-goals' in argv)==(mode=='on'),'infer setting')
        need(not any(a.startswith(('--verify-','--no-lifetime','--no-erasure','--no-verify','--rlimit','--time-limit')) for a in argv),'no filters/disabled checks/resource exclusions')
        p=r/(kind+'.rs');need(sha(p)==t['source_sha256'],'probe digest')
        text=p.read_text();need('borrows: &Vec<u64>' in text,'borrowed arbitrary Vec')
        need('enumerate().rev()' in text or 'let r=e.rev()' in text,'unchanged iterator chain')
    for name in ['driver_build','mir','native_compile','rustc_version','observations']:need(result['trials'][name]['exit']==0,'diagnostic/setup '+name)
    obs=pathlib.Path(result['trials']['observations']['stdout']['path']).read_text().splitlines()
    need(obs[0]=='sat' and obs[-1]=='unsat','qualified observation probe')
    for record in [freeze,result,inv,ids,decision]:need(record['goal_complete'] is False and record['native_correspondences']==0,'no native/goal promotion')
    need(result['publication'] is False and decision['publication'] is False,'no publication')
    need(decision['feasibility']=='INFEASIBLE_under_unchanged_semantics_and_trust' and decision['implementation_ready_checker_interface'] is False and decision['new_assumptions']==0,'bounded decision')
    for attempt in load(HERE/'ATTEMPTS.json'):
        for p,h in attempt['evidence'].items():need(sha(p)==h,'attempt evidence')
    changes=subprocess.check_output(['git','diff','--name-only','1171bdf7544ec153491381fd3820e688c2bbcd1a'],cwd=ROOT,text=True).splitlines()
    need(all(p.startswith('source/tools/research/checked_rev_bridge_design/') for p in changes),'tracked scope')
    status=subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=ROOT,text=True).splitlines()
    need(all(line[3:].startswith('source/tools/research/checked_rev_bridge_design/') for line in status),'working scope')
    engine_changes=subprocess.check_output(['git','diff','--name-only',freeze['engine'],'--','source'],cwd=ROOT,text=True).splitlines()
    need(all(p.startswith(('source/tools/research/','source/docs/')) for p in engine_changes),'engine source match')
    print(json.dumps({'accepted':True,'ordinary_trials':8,'actual_core_instances_inspected':3,'helpers':0,'observation_fragment':['sat','unsat'],'feasibility':decision['feasibility'],'native_correspondences':0,'publication':False,'goal_complete':False},sort_keys=True))
if __name__=='__main__':main()
