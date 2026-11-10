#!/usr/bin/env python3
"""Exercise dependency-readiness refusal. These are setup controls, not semantics."""
import argparse,json,os,sys
from pathlib import Path
sys.dont_write_bytecode=True
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,digest,support_artifacts
from infer_experiment import LoggedCommands,require,replay_receipt
p=argparse.ArgumentParser();p.add_argument('--freeze',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
r=dict(schema=1,commands=[],cases=[],publication=False,semantic_negative_count=0)
with replay_receipt(a.output,r):
 f=json.loads(a.freeze.read_text());require(f['before_first_trial'] is True,'pretrial freeze')
 for n,h in f['files'].items():require(digest(ROOT/n)==h,'frozen input changed '+n)
 guard=ArtifactGuard([a.freeze,*[ROOT/n for n in f['files']],*support_artifacts()])
 run=LoggedCommands(a.output,ROOT,r['commands'],guard=guard,timeout=60)
 failed=a.output/'failed-baseline-receipt.json';failed.write_text(json.dumps(dict(success=False)))
 for name,content,marker in [
  ('missing-ready',None,'No such file'),
  ('unchecked-baseline',dict(source_commit='115a618dd35ff8337112cf2081a7f5391d3973d5',baseline_kernel_checked=False),'unready/wrong foundation'),
  ('failed-final-receipt',dict(source_commit='115a618dd35ff8337112cf2081a7f5391d3973d5',baseline_kernel_checked=True,receipt=str(failed)),'baseline final receipt did not succeed')]:
  ready=a.output/(name+'.json')
  if content is not None:ready.write_text(json.dumps(content))
  out=a.output/(name+'-trial')
  code=run(name,[sys.executable,'-B','-O',str(HERE/'check.py'),'--ready',str(ready),'--freeze',str(a.freeze),'--output',str(out)],os.environ)
  result=json.loads((out/'receipt.json').read_text())
  require(code!=0 and marker in result['failure']['message'],'unexpected dependency gate failure')
  require(result['commands']==[] and result['success'] is False and result['publication'] is False
   and result['model_proofs_kernel_checked'] is False,'failed setup acquired a proof status')
  r['cases'].append(dict(name=name,classification='dependency_setup_rejection',receipt=str(out/'receipt.json'),sha256=digest(out/'receipt.json')))
 require(len(r['cases'])==3,'gate control inventory');r['artifacts']=guard.initial;guard.check()
