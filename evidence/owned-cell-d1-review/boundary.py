#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'))
import check
from infer_eval_support import ArtifactGuard
OUT=Path('/tmp/owned-cell-d1-review-973ca52d')
DEPS=OUT/'build/debug/deps'
inputs=[Path(__file__),Path(__file__).with_suffix('.rs'),ROOT/'source/vir/src/internal_repr.rs']
libs=[]
artifacts=[]
for lib in ['vir','serde_json','air']:
    matches=list(DEPS.glob('lib'+lib+'-*.rlib'));assert len(matches)==1,matches
    libs+=['--extern',lib+'='+str(matches[0])];artifacts+=matches
guard=ArtifactGuard(artifacts)
argv=[check.TC/'bin/rustc','--edition=2024','--test',Path(__file__).with_suffix('.rs'),'-L','dependency='+str(DEPS),*libs,'-o',OUT/'boundary-tests']
code=check.run(OUT,'boundary-compile-v4',argv,extra_inputs=inputs)
if code==0:code=check.run(OUT,'boundary-run-v4',[OUT/'boundary-tests','--nocapture','--test-threads=1'],extra_inputs=inputs)
guard.check()
sys.exit(code)
