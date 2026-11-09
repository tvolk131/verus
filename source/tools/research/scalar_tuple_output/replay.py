#!/usr/bin/env python3
"""Fresh exact pair checks. All generated artifacts are private to --output."""
import argparse, hashlib, json, os, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, tool_artifacts
from infer_experiment import (LoggedCommands, replay_receipt, require,
                              require_exact, require_publication, validate_run)
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def fidelity(stateful=None):
    frozen=json.loads((HERE/'freeze.json').read_text())
    for p,h in frozen['inputs'].items(): require(sha(ROOT/p)==h, f'frozen input changed: {p}')
    old=(HERE.parent/'word_goal_automation/automatic_inferred_mask.rs').read_text()
    old=old.replace('    fn data_index_of(bit: u32) -> (r: (usize, usize))\n        ensures r.0 == (bit as usize) / 64, r.1 == (bit as usize) % 64,','    #[verifier::infer]\n    fn data_index_of(bit: u32) -> (r: (usize, usize))')
    require(old==(stateful or HERE/'automatic_inferred_helpers.rs').read_text(), 'stateful helper adaptation changed')

def check_run(result, success, error=None):
    semantic = error in ('postcondition not satisfied', 'precondition not satisfied', 'assertion failed')
    data = validate_run(result, success, allow_frontend=bool(error) and not semantic)
    if error:
        require(error in result['stderr'], 'expected diagnostic missing')
    if success or semantic:
        require(data['verification-results'].get('is-verifying-entire-crate') is True,
                'incomplete verifier scope')
    if success:
        info = data['verus-infer']
        require(info['verification_scope']['complete_selected_scope_verified'] is True,
                'incomplete verification scope')
        require_exact(info['functions'])
    return data

def main():
    p=argparse.ArgumentParser();p.add_argument('--verus',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    fidelity()
    a.output.mkdir(parents=True, exist_ok=True)
    out=Path(tempfile.mkdtemp(prefix='scalar-pairs-',dir=a.output.resolve()))
    os.chmod(out, 0o700)
    receipt={'runs':[], 'commands':[], 'sources':{str(f.relative_to(ROOT)):sha(f) for f in HERE.glob('*') if f.is_file()}}
    with replay_receipt(out, receipt):
        replay(a, out, receipt)
    print(out/'receipt.json')

def replay(a, out, receipt):
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(ROOT/'source/z3'),VERUS_INFER_CACHE=str(out/'cache'))
    guard=ArtifactGuard(tool_artifacts(a.verus.resolve(), ROOT/'source/z3',
        extra=[*(ROOT/p for p in receipt['sources']), HERE/'freeze.json']))
    receipt['tools']=guard.initial
    invoke=LoggedCommands(out,out,receipt['commands'],timeout=240,guard=guard,status_key='returncode')
    def run(label,s,success=True,opts=(),error=None,name=None):
        path=out/((name or label)+'.rs');path.write_text(s)
        cmd=[str(a.verus.resolve()),str(path),'--no-cheating','--output-json','--multiple-errors','20',*map(str,opts)]
        rc=invoke(label,cmd,env)
        result=dict(returncode=rc,stdout=(out/(label+'.stdout')).read_text(),stderr=(out/(label+'.stderr')).read_text())
        d=json.loads(result['stdout'])
        receipt['runs'].append(dict(label=label,command=cmd,returncode=rc,stdout_sha256=sha(out/(label+'.stdout')),stderr_sha256=sha(out/(label+'.stderr')),result=d))
        (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        check_run(result,success,error)
        print(label,rc,d.get('verification-results',{}),flush=True)
        return d
    pairs=(HERE/'pairs.rs').read_text()
    run('pairs_portable',pairs)
    # Select each SMT architecture in ordinary caller preconditions; helper
    # inference remains unconditional and all normal/erasure checks stay enabled.
    for bits in [32,64]:
        model=pairs.replace('fn check(bit: u32) {',f'fn check(bit: u32)\n    requires arch_word_bits() == {bits},\n    {{\n    assert(usize::MAX as int == {2**bits-1});')
        run('model_'+str(bits),model)
    state=(HERE/'automatic_inferred_helpers.rs').read_text()
    run('stateful',state,opts=['--infer-goals'])
    run('stateful_xor',state.replace('group |=','group ^='),False,['--infer-goals'],error='postcondition not satisfied')
    run('stateful_disabled',state,False,error='postcondition not satisfied')
    run('historical_baseline',(HERE.parent/'redb_single_word_set/baseline.rs').read_text(),False,['--infer-goals'],error='postcondition not satisfied')
    run('forwarding_wrapper',(HERE.parent/'word_goal_automation/automatic.rs').read_text(),False,['--infer-goals'],error='postcondition not satisfied')
    run('representation_equivalence',(HERE.parent/'word_goal_automation/representation_equivalence.rs').read_text(),opts=['--infer-goals'])
    for case in ['arithmetic(255, 1)','arithmetic(1, 0)','ignored_left(1, 0)','ignored_right(1, 0)','ignored_left(255, 1)','ignored_right(255, 1)','branch(0, false)','branch(255, true)','signed(i8::MIN, -1)','signed(1, 0)','slice_pair(&[1u8])']:
        run('unsafe_'+str(len(receipt['runs'])),pairs.replace('fn check(bit: u32) {',f'fn unsafe_caller() {{ let _ = {case}; }}\nfn check(bit: u32) {{'),False,error='precondition not satisfied')
    for label,sig,body in [('arity','(u8,u8,u8)','(x,x,x)'),('nested','((u8,u8),u8)','((x,x),x)'),('tuple_input','u8','x.0')]:
        param='x:(u8,u8)' if label=='tuple_input' else 'x:u8'
        run('reject_'+label,f'use vstd::prelude::*; verus! {{ #[verifier::infer] fn f({param})->{sig}{{{body}}} }} fn main(){{}}',False,error='Verus Infer unsupported')
    run('reject_approx','use vstd::prelude::*; verus! { #[verifier::infer(sound)] fn f(x:u8)->(u8,u8){(x,x)} } fn main(){}',False,error='sound interval inference requires scalar signatures')
    provider='use vstd::prelude::*;verus!{#[verifier::infer] pub fn split(bit:u32)->(usize,usize){((bit as usize)/64,(bit as usize)%64)}}'
    popts=['--crate-type=lib','--crate-name=provider','--compile','--export',out/'provider.vir','-o',out/'libprovider.rlib']
    for label in ['cold','warm']:
        d=run('provider_'+label,provider,opts=popts,name='provider')
        require(d['verus-infer']['functions'][0]['metrics']['cache']==('hit' if label=='warm' else 'miss'), 'incorrect cache provenance')
        require_publication(d, True, out/'provider.vir')
    consumer='extern crate provider; use vstd::prelude::*;verus!{#[verifier::infer]fn composed(x:u32)->usize{let p=provider::split(x);p.1} fn check(x:u32){let r=composed(x);assert(r==(x as usize)%64);}}fn main(){}'
    copts=['-L','dependency='+str(out),'--extern','provider='+str(out/'libprovider.rlib'),'--import','provider='+str(out/'provider.vir')]
    d=run('consumer',consumer,opts=copts);require(d['verus-infer']['imported_inference_dependencies'], 'missing checked import dependency')
    original=(out/'provider.vir').read_bytes();start=original.rindex(b'{"schema":');meta=json.loads(original[start:-8])
    for key,value in [('policy','candidate-bound-evidence-v66'),('schema',37),('soundness',False),('maximality',False),('exactness',False),('producer','poison'),('summaries','0'*64),('dependencies',[])]:
        # Empty dependencies might be the genuine value; mutate an existing fingerprint instead.
        m=json.loads(json.dumps(meta))
        if key=='dependencies':
            if not m[key]:continue
            m[key][0]['fingerprint']='0'*64
        else:m[key]=value
        (out/'provider.vir').write_bytes(original[:start]+json.dumps(m,separators=(',',':')).encode()+b'VRINFER1')
        run('metadata_'+key,consumer,False,copts,error='rebuild the library')
    (out/'provider.vir').write_bytes(original)
    bridgeopts=[*copts,'--crate-type=lib','--crate-name=bridge','--compile','--export',out/'bridge.vir','-o',out/'libbridge.rlib']
    run('bridge_provider',consumer.replace('fn composed','pub fn composed'),opts=bridgeopts,name='bridge')
    leaf='extern crate bridge;use vstd::prelude::*;verus!{fn check(x:u32){let r=bridge::composed(x);assert(r==(x as usize)%64);}}fn main(){}'
    leafopts=[*copts,'--extern','bridge='+str(out/'libbridge.rlib'),'--import','bridge='+str(out/'bridge.vir')]
    run('transitive_consumer',leaf,opts=leafopts)
    bridge=(out/'bridge.vir').read_bytes();pos=bridge.rindex(b'{"schema":');bm=json.loads(bridge[pos:-8]);require(bm['dependencies'], 'missing transitive dependency')
    bm['dependencies'][0]['fingerprint']='0'*64
    (out/'bridge.vir').write_bytes(bridge[:pos]+json.dumps(bm,separators=(',',':')).encode()+b'VRINFER1')
    run('transitive_dependency_poison',leaf,False,leafopts,error='rebuild the library')
    (out/'bridge.vir').write_bytes(bridge)
    # Poison the exact current cache key, retaining source/dependency identity.
    files=list((out/'cache').glob('*.json'))
    selected=[f for f in files if json.loads(f.read_text()).get('post',{}).get('Binary')]
    # Find the provider output's pair rather than arbitrary other fixture entries.
    selected=[f for f in selected if 'Pair' in json.dumps(json.loads(f.read_text())['post']) and 'USize' in json.dumps(json.loads(f.read_text())['post'])]
    require(selected, 'provider pair cache entry missing')
    for kind in ['weak','swapped','malformed','wrong_type','wrong_key','wrong_dependencies']:
        saved={f:f.read_bytes() for f in selected}
        for f in selected:
            c=json.loads(f.read_text())
            if kind=='weak':c['post']={'Bool':True}
            elif kind=='wrong_key':c['key']='0'*64
            elif kind=='wrong_dependencies':c['dependencies']=['0'*64]
            else:
                def poison(x):
                    if isinstance(x,dict):
                        if 'Pair' in x:
                            if kind=='swapped':x['Pair'][1].reverse()
                            elif kind=='wrong_type':x['Pair'][0][0]='Bool'
                            else:x['Pair'][0]=x['Pair'][0][:1]
                        else:
                            for v in x.values():poison(v)
                    elif isinstance(x,list):
                        for v in x:poison(v)
                poison(c['post'])
            f.write_text(json.dumps(c))
        # Weak/malformed candidates fail the import-shape gate and regenerate.
        run('cache_'+kind,provider,kind!='swapped',popts,error='postcondition not satisfied' if kind=='swapped' else None,name='provider')
        for f,b in saved.items():f.write_bytes(b)
    d=run('changed_body',provider.replace('%64','%32'),opts=popts,name='provider')
    require(d['verus-infer']['functions'][0]['metrics']['cache']=='miss', 'changed body used stale cache')
    d=run('changed_type',provider.replace('usize','u64'),opts=popts,name='provider')
    require(d['verus-infer']['functions'][0]['metrics']['cache']=='miss', 'changed type used stale cache')
    run('restore_provider',provider,opts=popts,name='provider')
    failed=out/'failed.vir';failed.write_bytes(original)
    d=run('failed_export',provider+'verus!{fn bad(){assert(false);}}',False,['--crate-type=lib','--crate-name=failed','--export',failed],error='assertion failed')
    require_publication(d, False, failed)
    failed.write_bytes(original)
    d=run('no_verify_export',provider,False,['--no-verify','--crate-type=lib','--crate-name=failed','--export',failed],error='Verus Infer')
    require_publication(d, False, failed)
    guard.check()
    receipt['artifacts_after']=guard.snapshot()
if __name__=='__main__':main()
