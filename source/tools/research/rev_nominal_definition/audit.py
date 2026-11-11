"""Read-only diagnostic acceptance; explicit exceptions also under python -O.
This authenticates evidence, not the missing native semantic definition.
"""
import hashlib,json,pathlib,re,subprocess,sys
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[3]
def need(b,m):
    if not b:raise RuntimeError(m)
def sha(p):return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(pathlib.Path(p).read_text())
def bind(x):need(sha(x['path'])==x['sha256'],'binding '+x['path'])
def main():
    need(sys.flags.dont_write_bytecode,'use -B')
    f=load(HERE/'FREEZE.json');r=load(HERE/'RESULTS.json');d=load(HERE/'DECISION.json')
    root=pathlib.Path(r['private_root']); run=load(root/'freeze.json')
    need(sha(HERE/'INTERFACE.json')==f['interface_sha256'],'interface freeze')
    for p,h in f['inputs'].items():need(sha(ROOT/p)==h,'preregistered input '+p)
    for p,h in f['toolchain'].items():need(sha(p)==h,'pinned compiler/core '+p)
    need(r==load(root/'RESULTS.json'),'private result agreement')
    need(r['status']=='DIAGNOSTIC_COMPATIBILITY_ONLY_NATIVE_COHERENCE_UNDISCHARGED','bounded status')
    need(r['final_guards'] is True,'run final guards')
    need(run['tools']==r['tools'] and run['toolchain']==r['toolchain'],'tool manifests')
    need(len(r['tools'])==8,'eight tools')
    for n,h in r['tools'].items():need(sha(root/'bundle'/n)==h,'private tool '+n)
    need((root/'bundle/z3').stat().st_mode & 0o111,'solver executable mode')
    need(run['receipt_sha256']=='5e7e79cf6b6cef0509226cf6134c8fce0a8c47ed608fc4c5a7520b5023aa4c45','receipt')
    need(run['environment']['RUSTUP_HOME']=='/home/dev/.rustup','rustup home')
    need(run['environment']['CARGO_NET_OFFLINE']=='true' and run['environment']['CARGO_BUILD_JOBS']=='2','offline jobs2')
    for p,h in run['controllers'].items():need(sha(ROOT/p)==h,'controller '+p)
    for p,h in run['production'].items():need(sha(ROOT/p)==h,'production '+p)
    for p,h in d['additional_inspected_inputs'].items():need(sha(ROOT/p)==h,'inspected source '+p)
    for p,h in r['products'].items():need(sha(p)==h,'product '+p)
    for name,t in r['trials'].items():
        bind(t['stdout']);bind(t['stderr'])
        need(load(pathlib.Path(t['stdout']['path']).parent/'receipt.json')==t,'immutable receipt '+name)
    ids=load(HERE/'IDENTITIES.json');inv=load(HERE/'INVENTORY.json')
    need(len(ids['instances'])==4 and ids['diagnostic_only'],'four diagnostic instances')
    bind(ids['transcript']);text=pathlib.Path(ids['transcript']['path']).read_text()
    chunks=text.split('\nINSTANCE ')[1:]
    for chunk,x in zip(chunks,ids['instances']):
        need(x['instance']==chunk.splitlines()[0],'instance identity')
        for k,v in x['metadata'].items():need(re.search('^'+k+' (.*)$',chunk,re.M).group(1)==v,'instance '+k)
        need(re.findall('^BLOCK (.*)$',chunk,re.M)==x['blocks'],'instance blocks')
        need(list(dict.fromkeys(re.findall('^FIELD (.*)$',chunk,re.M)))==x['fields'],'instance fields')
        generic=chunk.split('GENERIC_BODY ',1)[1].split('\nBODY ',1)[0]
        body=chunk.split('\nBODY ',1)[1].split('\nLOCAL ',1)[0]
        need(hashlib.sha256(generic.encode()).hexdigest()==x['generic_mir_text_sha256'],'generic MIR')
        need(hashlib.sha256(body.encode()).hexdigest()==x['instantiated_mir_text_sha256'],'instantiated MIR')
        need('Item>>' in x['instance'] and 'u64>>' not in x['instance'],'concrete actual Item')
    need(ids['local_adts']==re.findall('^LOCAL_ADT (.*)$',text,re.M),'local ADT identity extraction')
    need(ids['local_fields']==re.findall('^LOCAL_FIELD (.*)$',text,re.M),'local fields')
    need(any('::Item)' in s for s in ids['local_adts']),'genuine local Item DefId')
    need(any('::Item::0)' in s and 'u64' in s for s in ids['local_fields']),'Item field')
    witness=(root/'genuine_item.rs').read_bytes(); fixture=pathlib.Path(f['allowed_item']['path']).read_bytes()
    need(sha(f['allowed_item']['path'])==f['allowed_item']['sha256'],'Item source pin')
    need(witness.startswith(fixture),'byte preserved complete Item source')
    need(b'for (_,_) in borrows.iter().enumerate().rev() {}' in witness,'original chain')
    need(b'into_inner' not in witness,'no into_inner application')
    if 'reused_inspector' in r:
        reuse=r['reused_inspector'];bind(reuse['results']);bind(reuse['binary'])
        old=load(reuse['results']['path']);of=load(pathlib.Path(old['private_root'])/'freeze.json')
        need(of['controllers'][str((HERE/'inspect.rs').relative_to(ROOT))]==sha(HERE/'inspect.rs') and of['toolchain']==r['toolchain'],'same inspector source/toolchain')
        need(old['trials']['driver-build']['exit']==0 and sha(root/'inspect')==reuse['binary']['sha256'],'completed exact diagnostic build')
    else:need(r['trials']['driver-build']['exit']==0,'fresh diagnostic build')
    for name in ['item-native-compile','item-mir','rustc-version']:need(r['trials'][name]['exit']==0,'actual MIR setup '+name)
    for name,x in inv['trials'].items():
        t=r['trials'][name];raw=load(t['stdout']['path']);v=raw['verification-results'];a=raw['verus-infer']
        need(v==x['verification'],'verification extraction '+name)
        need(a['verification_scope']==x['verification_scope'],'body extraction '+name)
        need(a['requested_functions']==x['helpers_requested']==0 and a['generated_functions']==x['helpers_generated']==0,'zero helpers '+name)
        need(v['is-verifying-entire-crate'] and not v['encountered-vir-error'],'whole crate/healthy frontend '+name)
        need('--no-cheating' in t['argv'] and '--compile' in t['argv'],'normal source checks '+name)
        need(not any(s.startswith(('--no-lifetime','--no-erasure','--verify-','--no-verify','--rlimit','--time-limit')) for s in t['argv']),'no bypass '+name)
        expected=(10,0) if name=='compatibility' else (10,1) if name in ['false-inner','false-law','false-index','missing-frame'] else {'available':(1,0),'lawful':(0,1),'inner':(0,1),'loop':(1,1)}[name.rsplit('-',1)[0]]
        need((v['verified'],v['errors'])==expected,'exact outcome '+name)
        if name in ['false-inner','false-law','false-index','missing-frame']:
            stderr=pathlib.Path(t['stderr']['path']).read_text()
            need(stderr.count('error: assertion failed')==1,'one intended assertion '+name)
        if name=='compatibility':
            need(x['verification_scope']['checked_proof_bodies']==10 and x['verification_scope']['checked_exec_bodies']==0,'model proof inventory')
            need(x['verification_scope']['complete_selected_scope_verified'],'model complete checking')
    for name,code in [('rust-private','E0616'),('rust-move','E0382'),('rust-borrow','E0502')]:
        t=r['trials'][name];need(t['exit']!=0 and code in pathlib.Path(t['stderr']['path']).read_text(),'Rust source control '+name)
    need(d['production_changes']==[] and d['new_assumptions']==0 and d['definition_installed'] is False,'no production fact')
    for a in load(HERE/'ATTEMPTS.json'):
        for p,h in a['evidence'].items():need(sha(p)==h,'retained attempt '+p)
    for o in [r,d,ids,inv]:
        need(o['publication'] is False and o['goal_complete'] is False and o['native_correspondences']==0,'bounded nonclaims')
    changes=subprocess.check_output(['git','diff','--name-only',f['parent_head']],cwd=ROOT,text=True).splitlines()
    status=subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=ROOT,text=True).splitlines()
    need(all(p.startswith('source/tools/research/rev_nominal_definition/') for p in changes),'committed scope')
    need(all(p[3:].startswith('source/tools/research/rev_nominal_definition/') for p in status),'working scope')
    print(json.dumps({'accepted_diagnostic_evidence':True,'native_definition_installed':False,'ordinary_baselines':8,'model_proofs':10,'model_assertion_controls':3,'missing_frame_control':1,'actual_Item_core_instances':4,'source_Rust_controls':3,'helpers':0,'native_correspondences':0,'publication':False,'goal_complete':False},sort_keys=True))
if __name__=='__main__':main()
