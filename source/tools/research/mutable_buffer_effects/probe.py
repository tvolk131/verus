#!/usr/bin/env python3
"""Freeze an ordinary proof candidate before any trial; emit guarded receipts."""
import argparse,json,os,sys,shutil
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard,tool_artifacts,digest
from infer_experiment import (LoggedCommands,replay_receipt,require,validate_run,
 require_function_coverage,require_exact,require_failure_kind)
p=argparse.ArgumentParser()
p.add_argument('--source',type=Path,required=True)
p.add_argument('--id',required=True)
p.add_argument('--verus',type=Path,required=True)
p.add_argument('--solver',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--failure',choices=['postcondition not satisfied','assertion failed','precondition not satisfied'])
p.add_argument('--extra-input',type=Path,action='append',default=[])
a=p.parse_args()
require(a.id.replace('-','').replace('_','').isalnum(),'invalid candidate ID')
candidate=HERE/'candidates'/a.id
candidate.mkdir()
shutil.copy2(a.source,candidate/'probe.rs')
inputs={str(f.resolve()):digest(f) for f in a.extra_input}
freeze=dict(schema=1,id=a.id,source_sha256=digest(candidate/'probe.rs'),extra_inputs=inputs,
 expected_helpers=[],required_helper_checks=['soundness','maximality','exactness'],
 expected_success=not a.failure,expected_failure=a.failure,checking='full ordinary, --no-cheating; lifetime and trait checks enabled')
(candidate/'freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
a.output.mkdir(parents=True)
receipt=dict(candidate=a.id,freeze=freeze,commands=[])
with replay_receipt(a.output,receipt):
 guard=ArtifactGuard(tool_artifacts(a.verus.resolve(),a.solver.resolve(),[candidate/'probe.rs',candidate/'freeze.json',*a.extra_input]))
 receipt['artifacts']=guard.initial
 env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(a.solver.resolve()),VERUS_INFER_CACHE=str(a.output.resolve()/'cache'))
 run=LoggedCommands(a.output,ROOT,receipt['commands'],timeout=180,guard=guard,status_key='returncode')
 command=[str(a.verus.resolve()),str((candidate/'probe.rs').resolve()),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','8']
 rc=run(a.id,command,env)
 result=dict(returncode=rc,stdout=(a.output/(a.id+'.stdout')).read_text(),stderr=(a.output/(a.id+'.stderr')).read_text())
 receipt['result']=json.loads(result['stdout'])
 receipt['stdout_sha256']=digest(a.output/(a.id+'.stdout'))
 receipt['stderr_sha256']=digest(a.output/(a.id+'.stderr'))
 data=validate_run(result,not a.failure)
 require(data['verification-results'].get('is-verifying-entire-crate') is True,'incomplete scope')
 functions=data.get('verus-infer',{}).get('functions',[])
 require_function_coverage(functions,set())
 require_exact(functions)
 if a.failure: require_failure_kind(result['stderr'],a.failure)
 guard.check()
print(json.dumps(receipt.get('result',{}).get('verification-results',{}),indent=2))
