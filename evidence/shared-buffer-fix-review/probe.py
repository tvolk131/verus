"""Frozen, normally checked fix-review probes; run with python3 -B -O."""
import sys, json, os, argparse, shutil
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'source/tools'))
from infer_experiment import require, validate_goal_run, validate_run, require_function_coverage, require_exact, LoggedCommands
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
p=argparse.ArgumentParser();p.add_argument('--raw',type=Path,required=True);p.add_argument('--freeze-only',action='store_true');a=p.parse_args()
out=a.raw.resolve()/'probes';out.mkdir(exist_ok=True)
repros=Path(__file__).parent/'repros'
if a.freeze_only:
    cases=[]
    def add(label,text,execs,selection):
        path=repros/(label+'.rs');path.write_text(text)
        private=out/'sources'/(label+'.rs');private.parent.mkdir(exist_ok=True);shutil.copy2(path,private)
        require(not private.is_symlink() and private.stat().st_ino!=path.stat().st_ino and digest(private)==digest(path),'independent source copy')
        cases.append(dict(label=label,committed_source=str(path.resolve()),source=str(private.resolve()),source_sha256=digest(private),execs=execs,proofs=[],on_selection=selection,expected='success off/on; target mutation must skip observer'))
    def simple(body,ret=''):
        return 'use vstd::prelude::*;\nverus! { pub fn client(buf: &mut [u8;10])'+ret+' {\n'+body+'\n} }\n'
    loop='for b in buf.iter_mut() { *b = 1; }'
    for n in ['before-write','after-write']:
        path=ROOT/'evidence/shared-buffer-review/repros'/(n+'.rs')
        add(n,path.read_text(),[n.replace('-','_')],[])
    add('simple',simple(loop),['client'],['client'])
    helper='use vstd::prelude::*; verus! { fn write(buf:&mut[u8;10]) {buf[0]=42;} pub fn client(buf:&mut[u8;10]) { BODY } }\n'
    add('helper-before',helper.replace('BODY','write(buf); '+loop),['write','client'],[])
    add('alias-after',helper.replace('BODY',loop+' let alias = &mut *buf; write(alias);'),['write','client'],[])
    add('escape',simple(loop+' buf',' -> &mut [u8;10]'),['client'],[])
    add('move-let-after',simple(loop+' let alias = buf; alias[0]=42;'),['client'],[])
    add('pattern-ref-before',simple('let ref mut alias = *buf; alias[0]=42; '+loop),['client'],[])
    add('pattern-ref-after',simple(loop+' let ref mut alias = *buf; alias[0]=42;'),['client'],[])
    add('match-move-after',simple(loop+' match buf { alias => { alias[0]=42; } }'),['client'],[])
    add('nested-before',simple('if true { match 1u8 { 1 => {buf[0]=42;}, _ => {} } } '+loop),['client'],[])
    add('projection-helper', 'use vstd::prelude::*; verus! { fn write(b:&mut u8){*b=42;} fn client(buf:&mut[u8;10]){'+loop+' write(&mut buf[0]);} }\n',['write','client'],[])
    add('returned-helper', 'use vstd::prelude::*; verus! { fn id(b:&mut[u8;10])->&mut[u8;10]{b} fn client(buf:&mut[u8;10]){'+loop+' id(buf)[0]=42;} }\n',['id','client'],[])
    add('ghost-read',simple('let ghost snap = buf@; '+loop),['client'],None)
    add('skipped-safety',simple('buf[0]=42; for b in buf.iter_mut(){*b+=1;}'),['client'],[]);cases[-1]['expected_both']='safety'
    add('skipped-lifetime',simple(loop+' let a=&mut *buf; let b=&mut *buf; a[0]=1; b[0]=2;'),['client'],[]);cases[-1]['expected_both']='rust_frontend'
    prefix=(ROOT/'source/tools/research/buffer_observation_goal/prefix-zip.rs').read_text().replace('fn main() {}','')
    unrelated=prefix.replace('key: &[u8])','key: &[u8], other: &mut [u8;10])').replace('    for (b_i, b_h)','    other[0] = 42;\n    for (b_i, b_h)').replace('        *b_i ^= *b_h;\n    }','        *b_i ^= *b_h;\n    }\n    other[0] = 43;')
    add('unrelated-prefix',unrelated,['xor_key'],['xor_key']);cases[-1]['expected_off']='postcondition'
    numeric=(ROOT/'source/tools/research/varint_canonical_goal/candidate-008.rs').read_text().replace('fn main() {}','')
    add('numeric-observer-composition',numeric+'\n'+prefix.replace('use vstd::prelude::*;',''),['u64','xor_key'],['u64','xor_key']);cases[-1]['expected_off']='diagnostic'
    (out/'freeze.json').write_text(json.dumps(cases,indent=2)+'\n');print('frozen',len(cases));sys.exit()
cases=json.loads((out/'freeze.json').read_text());bundle=a.raw.resolve()/'bundle'
guard=ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=[Path(c['source']) for c in cases]+[out/'freeze.json',Path(__file__)]))
receipt=dict(artifacts=guard.initial,commands=[],runs=[],findings=[])
(out/'results.json').write_text(json.dumps(receipt,indent=2)+'\n')
call=LoggedCommands(out,out,receipt['commands'],timeout=240,guard=guard,status_key='returncode')
env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),RUST_MIN_STACK='33554432')
for c in cases:
    require(digest(c['source'])==c['source_sha256'],'pretrial source drift')
    for on in [False,True]:
        label=c['label']+('-on' if on else '-off');cmd=[str(bundle/'verus'),c['source'],'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']+(['--infer-goals'] if on else [])
        rc=call(label,cmd,env);run=dict(returncode=rc,stdout=(out/(label+'.stdout')).read_text(),stderr=(out/(label+'.stderr')).read_text())
        data=json.loads(run['stdout']);v=data['verification-results'];expected_off=c.get('expected_off');failure=None
        if c.get('expected_both'):
            validate_run(run,False,allow_frontend=c['expected_both']=='rust_frontend');kind=c['expected_both']
            require_function_coverage(data['verus-infer']['functions'],set());require_exact(data['verus-infer']['functions'])
            if kind=='safety':
                require('possible arithmetic underflow/overflow' in run['stderr'] and 'invariant not satisfied' not in run['stderr'] and 'error[E' not in run['stderr'],'safety diagnostic')
                require(v.get('is-verifying-entire-crate') is True,'whole safety check')
            else:require('error[E0499]' in run['stderr'],'missing lifetime rejection')
        elif not on and expected_off=='postcondition':
            validate_goal_run(run,False,expected_functions=set(),exec_bodies=set(c['execs']),proof_bodies=set(c['proofs']),failure_kind='postcondition not satisfied');kind='postcondition'
        elif not on and expected_off=='diagnostic':
            validate_run(run,False);require_function_coverage(data['verus-infer']['functions'],set());require_exact(data['verus-infer']['functions']);kind='diagnostic-off-composition'
        elif v.get('success') is True:
            validate_goal_run(run,True,expected_functions=set(),exec_bodies=set(c['execs']),proof_bodies=set(c['proofs']));kind='success'
        else:
            validate_run(run,False,allow_frontend=True);kind='frontend' if v.get('encountered-vir-error') or 'error[E' in run['stderr'] else 'invariant' if 'invariant not satisfied' in run['stderr'] else 'other-failure'
            failure=label+': unexpected '+kind
        actual=[e['function'].rsplit('::',1)[-1] for e in data.get('verus-infer',{}).get('buffer_observation_assistance',[])]
        if on and c['on_selection'] is not None and sorted(actual)!=sorted(c['on_selection']):failure=(failure or label)+': observer selection '+str(actual)+' expected '+str(c['on_selection'])
        if not on:require(not actual,'disabled observer')
        record=dict(label=label,classification=kind,verification=v,selection=actual,source_sha256=c['source_sha256'],stdout_sha256=digest(out/(label+'.stdout')),stderr_sha256=digest(out/(label+'.stderr')),expected_execs=c['execs'],expected_proofs=c['proofs'],scope=data.get('verus-infer',{}).get('verification_scope'),functions=data.get('verus-infer',{}).get('functions'))
        receipt['runs'].append(record)
        if failure:receipt['findings'].append(failure)
        (out/'results.json').write_text(json.dumps(receipt,indent=2)+'\n');print(label,kind,v.get('verified'),v.get('errors'),actual,flush=True)
guard.check()
