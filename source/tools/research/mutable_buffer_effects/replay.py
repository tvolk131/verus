#!/usr/bin/env python3
"""Replay frozen mutable-effects boundaries with full ordinary checking."""
import argparse,json,os,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,tool_artifacts,digest
from infer_experiment import (LoggedCommands,replay_receipt,require,validate_run,
    require_function_coverage,require_exact,require_failure_kind)
# Count obligations, including known zero-inference cases, before inspecting output.
# Model gaps and rejected proposals are not false-property semantic controls.
CASES=[
 ('baseline-prefix-v1',2,0,None,'positive'),
 ('baseline-unused-frame-v1',0,1,'postcondition not satisfied','model-gap'),
 ('baseline-xor-v1',4,0,None,'positive'),
 ('baseline-return-v1',2,0,None,'positive'),
 ('iterator-resolution-v1',2,0,None,'positive'),
 ('resolved-frame-v1',0,1,'postcondition not satisfied','missing-law'),
 ('zip-resolution-v1',0,1,'postcondition not satisfied','missing-law'),
 ('concrete-model-v3',11,0,None,'checked-concrete-model-law'),
 ('direct-frame-v1',2,0,None,'positive'),
 ('empty-zip-v1',1,1,'postcondition not satisfied','model-gap'),
 ('wrong-index-v1',2,1,'assertion failed','semantic-negative'),
 ('wrong-referent-v1',2,1,'assertion failed','semantic-negative'),
 ('wrong-untouched-v1',0,1,'postcondition not satisfied','semantic-negative'),
 ('zip-prefix-v1',1,1,'postcondition not satisfied','failed-prefix-bridge-proposal'),
 ('zip-prefix-v2',2,0,None,'positive-with-mechanical-inputs'),
]

def main():
 p=argparse.ArgumentParser()
 p.add_argument('--verus',type=Path,required=True)
 p.add_argument('--solver',type=Path,required=True)
 p.add_argument('--output',type=Path,required=True)
 a=p.parse_args()
 a.output.mkdir(parents=True)
 receipt=dict(commands=[],runs=[],checking='--no-cheating; no disabled lifetime/trait/body checks',expected_inferred_helpers=[])
 with replay_receipt(a.output,receipt):
  sources=[]
  for name,*_ in CASES:
   directory=HERE/'candidates'/name
   manifest=json.loads((directory/'freeze.json').read_text())
   require(digest(directory/'probe.rs')==manifest['source_sha256'],'candidate changed: '+name)
   sources.extend([directory/'probe.rs',directory/'freeze.json'])
  concrete=(HERE/'candidates/concrete-model-v3/probe.rs').read_bytes()
  original=(ROOT/'examples/iterators/slice_iter_mut.rs').read_bytes()
  require(concrete.startswith(original),'concrete implementation modified')
  receipt['concrete_model_source_sha256']=digest(ROOT/'examples/iterators/slice_iter_mut.rs')
  guard=ArtifactGuard(tool_artifacts(a.verus.resolve(),a.solver.resolve(),
    [*sources,Path(__file__),ROOT/'examples/iterators/slice_iter_mut.rs']))
  receipt['artifacts']=guard.initial
  env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(a.solver.resolve()),VERUS_INFER_CACHE=str(a.output.resolve()/'cache'))
  invoke=LoggedCommands(a.output,ROOT,receipt['commands'],timeout=180,guard=guard,status_key='returncode')
  for name,verified,errors,failure,classification in CASES:
   for goals in [False,True]:
    label=name+('-goals' if goals else '-plain')
    source=HERE/'candidates'/name/'probe.rs'
    command=[str(a.verus.resolve()),str(source),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','8']
    if goals: command+=['--infer-goals']
    rc=invoke(label,command,env)
    result=dict(returncode=rc,stdout=(a.output/(label+'.stdout')).read_text(),stderr=(a.output/(label+'.stderr')).read_text())
    data=json.loads(result['stdout'])
    row=dict(label=label,classification=classification,verification=data.get('verification-results'),runtime=data.get('verus'),stdout_sha256=digest(a.output/(label+'.stdout')),stderr_sha256=digest(a.output/(label+'.stderr')))
    receipt['runs'].append(row)
    validate_run(result,errors==0)
    require(data['verification-results'].get('is-verifying-entire-crate') is True,'incomplete scope')
    require(data['verification-results'].get('verified')==verified and data['verification-results'].get('errors')==errors,'unexpected proof coverage: '+label)
    functions=data.get('verus-infer',{}).get('functions',[])
    require_function_coverage(functions,set())
    require_exact(functions)
    if failure: require_failure_kind(result['stderr'],failure)
    if not errors:
     require(data['verus-infer']['verification_scope']['complete_selected_scope_verified'] is True,'incomplete full checking')
    print(label,verified,errors,flush=True)
  guard.check()
 print(a.output/'receipt.json')

if __name__=='__main__': main()
