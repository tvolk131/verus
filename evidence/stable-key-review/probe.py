"""Replay the pretrial frozen inputs; explicit named scope and classification gates."""
import sys,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import require,validate_goal_run,validate_run,require_function_coverage,require_exact,LoggedCommands
from infer_eval_support import ArtifactGuard,digest,tool_artifacts
RAW=Path('/tmp/stable-key-review-e84eed6c');out=RAW/'probes';bundle=RAW/'bundle'
cases=json.loads((out/'freeze.json').read_text())
guard=ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',extra=[Path(c['source']) for c in cases]+[out/'freeze.json',Path(__file__)]))
receipt=dict(artifacts=guard.initial,commands=[],runs=[],objections=[])
call=LoggedCommands(out,out,receipt['commands'],timeout=240,guard=guard,status_key='returncode')
env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),RUST_MIN_STACK='33554432')
for c in cases:
    require(digest(c['source'])==c['sha256']==digest(ROOT/c['committed_source']),'frozen source drift')
    for on in (False,True):
        label=c['label']+('-on' if on else '-off');cmd=[str(bundle/'verus'),c['source'],'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20']+(['--infer-goals'] if on else [])
        rc=call(label,cmd,env);run=dict(returncode=rc,stdout=(out/(label+'.stdout')).read_text(),stderr=(out/(label+'.stderr')).read_text())
        data=json.loads(run['stdout']);v=data['verification-results'];info=data.get('verus-infer',{});err=run['stderr']
        require_function_coverage(info.get('functions',[]),set(c['helpers']));require_exact(info.get('functions',[]))
        kind='success' if v.get('success') else 'frontend' if v.get('encountered-vir-error') or 'error[E' in err or 'error: lifetime' in err else 'invariant' if 'invariant not satisfied' in err else 'postcondition' if 'postcondition not satisfied' in err else 'safety' if 'possible arithmetic underflow/overflow' in err else 'assertion' if 'assertion failed' in err else 'resource' if any(x in err.lower() for x in ('timed out','resource limit','out of memory')) else 'other'
        # Frontend/resource outcomes never substitute for a semantic negative.
        if kind=='success':validate_goal_run(run,True,expected_functions=set(c['helpers']),exec_bodies=set(c['execs']),proof_bodies=set(c['proofs']))
        elif kind=='postcondition' and c['on' if on else 'off']!='diagnostic':validate_goal_run(run,False,expected_functions=set(c['helpers']),exec_bodies=set(c['execs']),proof_bodies=set(c['proofs']),failure_kind='postcondition not satisfied')
        else:
            validate_run(run,False,allow_frontend=kind=='frontend')
            if kind!='frontend':
                require(v.get('is-verifying-entire-crate') is True and not v.get('encountered-vir-error') and 'error[E' not in err,'incomplete semantic diagnostic')
                selected=[b for b in info['verification_scope']['native_functions'] if b['body_status']=='selected_body_not_certified_by_successful_full_run']
                require_function_coverage(selected,set(c['execs'])|set(c['proofs']))
        selection=[x['function'].rsplit('::',1)[-1] for x in info.get('buffer_observation_assistance',[])]
        expected=c['on' if on else 'off']
        if kind!=expected and not (expected=='diagnostic' and kind in ('postcondition','invariant','other')):receipt['objections'].append(label+': expected '+expected+', actual '+kind)
        if on and c['selection'] is not None and sorted(selection)!=sorted(c['selection']):receipt['objections'].append(label+': selection '+str(selection)+' expected '+str(c['selection']))
        if not on:require(not selection,'disabled observer selection')
        receipt['runs'].append(dict(label=label,classification=kind,selection=selection,verification=v,expected_execs=c['execs'],expected_proofs=c['proofs'],scope=info.get('verification_scope'),functions=info.get('functions'),source_sha256=c['sha256'],stdout_sha256=digest(out/(label+'.stdout')),stderr_sha256=digest(out/(label+'.stderr'))))
        (out/'results.json').write_text(json.dumps(receipt,indent=2)+'\n');print(label,kind,v.get('verified'),v.get('errors'),selection,flush=True)
require(not receipt['objections'],'probe objections: '+str(receipt['objections']));guard.check();receipt['artifacts_after']=guard.snapshot();receipt['success']=True
(out/'results.json').write_text(json.dumps(receipt,indent=2)+'\n')
