#!/usr/bin/env python3
"""Ordinary proof checking. Successful fragment proofs cannot publish native facts."""
import argparse,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,tool_artifacts
from infer_experiment import LoggedCommands,replay_receipt,require,validate_goal_run
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--freeze',type=Path,required=True);p.add_argument('--case',type=Path,required=True);p.add_argument('--bundle',type=Path,default=Path('/tmp/native-heap-lowering-c3bf5d36/bundle'))
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,commands=[],publication=False,native_bridge_proved=False)
with replay_receipt(a.output,r):
 f=json.loads(a.freeze.read_text());require(f['before_first_trial'] is True,'pretrial freeze missing')
 for n,s in f['files'].items(): require(digest(ROOT/n)==s,'frozen source changed '+n)
 case=json.loads(a.case.read_text());source=(HERE/case['file']).resolve(); require(str(source.relative_to(ROOT)) in f['files'],'unfrozen proof input')
 bundle=a.bundle.resolve();ident=json.loads((HERE/'bundle-identity.json').read_text())
 originals=[]
 for n,x in ident.items():
  q=bundle/n;p=Path(x['source']);require(digest(q)==digest(p)==x['sha256'] and not q.is_symlink() and q.stat().st_ino!=p.stat().st_ino,'runtime identity '+n); originals.append(p)
 g=ArtifactGuard(tool_artifacts(bundle/'verus',bundle/'z3',[*[ROOT/n for n in f['files']],a.freeze,a.case,*originals,HERE/'bundle-identity.json']));r['artifacts']=g.initial
 env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(a.output/'cache'))
 run=LoggedCommands(a.output,a.output,r['commands'],guard=g,timeout=240,status_key='returncode')
 code=run('proof',[str(bundle/'verus'),str(source),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','8'],env)
 result=dict(returncode=code,stdout=(a.output/'proof.stdout').read_text(),stderr=(a.output/'proof.stderr').read_text())
 data=validate_goal_run(result,case['positive'],expected_functions=set(),exec_bodies=case.get('exec_bodies',[]),proof_bodies=case['proof_bodies'],failure_kind=case.get('failure_kind'))
 if 'errors' in case: require(data['verification-results']['errors']==case['errors'],'unexpected error count')
 if 'verified' in case: require(data['verification-results']['verified']==case['verified'],'unexpected query count')
 r['verification']=data['verification-results'];r['scope']=data['verus-infer']['verification_scope'];r['case']=case;r['source_sha256']=digest(source)
 g.check();print(json.dumps(r['verification']))
