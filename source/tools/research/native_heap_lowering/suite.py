#!/usr/bin/env python3
"""Replay final source extraction, regenerated candidate equality, and ordinary proofs."""
import argparse,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import LoggedCommands,require,replay_receipt
from lower import Lower
from witness import make
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--freeze',type=Path,required=True);p.add_argument('--bundle',type=Path,required=True);a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,commands=[],publication=False,native_bridge_proved=False,cases=[])
with replay_receipt(a.output,r):
 f=json.loads(a.freeze.read_text());require(f['before_first_trial'] is True,'pretrial freeze')
 for n,h in f['files'].items():require(digest(ROOT/n)==h,'source changed '+n)
 g=ArtifactGuard([*[ROOT/n for n in f['files']],a.freeze,*support_artifacts()]);r['artifacts']=g.initial
 run=LoggedCommands(a.output,ROOT,r['commands'],guard=g,timeout=300)
 env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],CARGO_BUILD_JOBS='2')
 def call(name,script,args):
  require(run(name,[sys.executable,'-B','-O',str(HERE/script),*map(str,args)],env)==0,'replay failed '+name)
 call('extract','extract.py',['--freeze',a.freeze,'--output',a.output/'source'])
 d=json.loads((a.output/'source/drop-native.json').read_text());b=json.loads((a.output/'source/drop-borrowck.json').read_text())
 plan=Lower(d,b,digest(HERE/'fixtures/drop.rs')).build()
 require(make(plan,'../kernel-v7/kernel.rs')==(HERE/'candidates/trace-v9/trace.rs').read_text(),'generated proof differs from frozen candidate')
 require(plan['steps']==json.loads((HERE/'candidates/trace-v9/plan.json').read_text())['steps'],'instruction coverage differs')
 r['regenerated_trace_sha256']=digest(HERE/'candidates/trace-v9/trace.rs')
 call('audit','audit.py',['--extraction',a.output/'source','--freeze',a.freeze,'--output',a.output/'audit'])
 cases=[HERE/'candidates/trace-v9/case.json',HERE/'candidates/drop-v1/case.json',HERE/'candidates/full-writer-case.json',*sorted((HERE/'candidates/semantic-controls-v1').glob('*.json')),*sorted((HERE/'candidates/semantic-controls-v2').glob('*.json'))]
 for n,case in enumerate(cases):
  output=a.output/('proof-'+str(n));call('proof-'+str(n),'replay.py',['--bundle',a.bundle,'--freeze',a.freeze,'--case',case,'--output',output])
  result=json.loads((output/'receipt.json').read_text());require(result['success'] is True,'incomplete receipt')
  r['cases'].append(dict(case=str(case.relative_to(HERE)),verification=result['verification'],receipt=str(output/'receipt.json'),receipt_sha256=digest(output/'receipt.json')))
 g.check();print(json.dumps(dict(cases=len(r['cases']),publication=False)))
