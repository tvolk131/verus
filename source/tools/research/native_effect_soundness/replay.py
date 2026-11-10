#!/usr/bin/env python3
"""Ordinary conditional resource proofs. No native theorem publication."""
import argparse
import json
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, tool_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require, validate_goal_run

p=argparse.ArgumentParser()
p.add_argument('--bundle',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--candidate',default='v1')
a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=False)
receipt=dict(schema=1,native_bridge_proved=False,commands=[],cases=[])
with replay_receipt(a.output,receipt):
    candidate=HERE/'candidates'/a.candidate
    frozen=json.loads((candidate/'freeze.json').read_text())
    require(frozen['before_first_trial'] is True,'pretrial freeze')
    require(frozen['expected_helpers']==[],'zero helper inventory')
    require(frozen['required_helper_checks']==['soundness','maximality','exactness'],'mandatory infer checks')
    for name,checksum in frozen['files'].items():
        require(digest(HERE/name)==checksum,'frozen source mismatch: '+name)
    verus=a.bundle.resolve()/'verus'
    solver=a.bundle.resolve()/'z3'
    inputs=[HERE/'replay.py',candidate/'freeze.json',*[HERE/n for n in frozen['files']]]
    guard=ArtifactGuard(tool_artifacts(verus,solver,inputs))
    receipt['artifacts']=guard.initial
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',VERUS_Z3_PATH=str(solver),VERUS_INFER_CACHE=str(a.output.resolve()/'cache'))
    run=LoggedCommands(a.output,a.output.resolve(),receipt['commands'],timeout=180,guard=guard,status_key='returncode')
    require(run('version',[str(verus),'--version'],env)==0,'version failed')
    require('61f2182' in (a.output/'version.stdout').read_text(),'runtime mismatch')
    cases=frozen.get('cases',[dict(id='laws',file=f'candidates/{a.candidate}/laws.rs',positive=True,exec_bodies=frozen['exec_bodies'],proof_bodies=frozen['proof_bodies'])])
    for case in cases:
        label=case['id']
        code=run(label,[str(verus),str(HERE/case['file']),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','8'],env)
        result=dict(returncode=code,stdout=(a.output/(label+'.stdout')).read_text(),stderr=(a.output/(label+'.stderr')).read_text())
        receipt['cases'].append(dict(id=label,returncode=code,classification=case.get('classification','conditional-resource-law'),stdout_sha256=digest(a.output/(label+'.stdout')),stderr_sha256=digest(a.output/(label+'.stderr'))))
        data=validate_goal_run(result,case['positive'],expected_functions=set(),exec_bodies=case['exec_bodies'],proof_bodies=case['proof_bodies'],failure_kind=case.get('failure_kind'))
        receipt['cases'][-1]['verification']=data['verification-results']
        receipt['cases'][-1]['scope']=data['verus-infer']['verification_scope']
        print(json.dumps(dict(case=label,verification=data['verification-results'])))
    guard.check()
