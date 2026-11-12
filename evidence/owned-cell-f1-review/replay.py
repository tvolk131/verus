#!/usr/bin/env python3
"""Focused frozen test replay using existing receipt/guard helpers."""
import json,os,sys,shutil
sys.dont_write_bytecode=True
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'source/tools'))
sys.path.insert(0,str(ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
import check
PRIVATE=HERE/'private'

def main():
    action=sys.argv[1]
    if action=='setup':
        check.setup(PRIVATE/'environment')
        h=PRIVATE/'harness';(h/'tests/common').mkdir(parents=True)
        (h/'Cargo.toml').write_text('[package]\nname="f1-review-harness"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[dev-dependencies]\nserde_json="=1.0.151"\ntempfile="=3.27.0"\n')
        launch=PRIVATE/'launchers';launch.mkdir()
        (h/'tests/common/mod.rs').write_text('pub mod infer;\n')
        (h/'tests/common/infer.rs').write_text('pub struct InferCrates {pub directory:tempfile::TempDir,pub profile:std::path::PathBuf}\nimpl InferCrates {pub fn new()->Self {Self {directory:tempfile::tempdir().unwrap(),profile:std::path::PathBuf::from('+json.dumps(str(launch))+')}} pub fn vir(&self,name:&str)->std::path::PathBuf {self.directory.path().join(format!("{name}.vir"))}}\n')
        frozen=ROOT/'source/tools/research/redb_crash_protocol/native_state_link/owned_goals.rs'
        for n in ['local_checked_owned_cell','local_checked_owned_cell_admission']:
            src=(ROOT/f'source/rust_verify_test/tests/{n}.rs').read_text()
            src=src.replace('#![feature(rustc_private)]','').replace('"../../tools/research/redb_crash_protocol/native_state_link/owned_goals.rs"',json.dumps(str(frozen)))
            (h/f'tests/{n}.rs').write_text(src)
        wrapper='''#!/usr/bin/env python3
import sys,os,tempfile,json,shutil
sys.dont_write_bytecode=True
from pathlib import Path
HERE=Path(__file__).resolve().parents[2]
ROOT=HERE.parents[1]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands,replay_receipt
logs=HERE/'runs'; logs.mkdir(exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='invocation-',dir=logs))
argv=[str(HERE/'private/bundle'/Path(sys.argv[0]).name),*sys.argv[1:]]
for i,arg in enumerate(sys.argv[1:]):
    p=Path(arg)
    if p.suffix=='.rs' and p.is_file():shutil.copy2(p,out/f'input-{i}.rs')
r=dict(commands=[])
with replay_receipt(out,r):
    code=LoggedCommands(out,Path.cwd(),r['commands'],timeout=60)('compile',argv,dict(os.environ))
    r['exit']=code
sys.stdout.write((out/'compile.stdout').read_text());sys.stderr.write((out/'compile.stderr').read_text());sys.exit(code)
'''
        for n in ['verus','rust_verify']:
            (launch/n).write_text(wrapper);(launch/n).chmod(0o755)
        # Only private build/cache/output locations. All tools stay authenticated.
        (HERE/'HARNESS.md').write_text('Minimal std-only InferCrates shim replaces broad common module. Two original test files unchanged except removal of unused rustc_private feature and absolute frozen include path. Private launcher logs each compiler call using existing LoggedCommands/replay_receipt and copies source before TempDir removal. No rust_verify/vstd/compiler build. Cargo builds only serde_json/tempfile and focused harness.\n')
        return
    out=HERE/action;out.mkdir()
    paths=[PRIVATE/'bundle'/n for n in json.loads((HERE/'AUTHENTICATION.json').read_text())['tools']]
    paths += [Path(__file__),ROOT/'source/tools/infer_experiment.py',ROOT/'source/tools/infer_eval_support.py',*sorted((PRIVATE/'harness/tests').rglob('*.rs'))]
    guard=ArtifactGuard(paths)
    receipt=dict(commands=[],source={str(p):digest(p) for p in paths})
    env=check.env(PRIVATE/'environment');env['VERUS_ROOT']=str(PRIVATE/'bundle');env['VERUS_Z3_PATH']=str(PRIVATE/'bundle/z3')
    args=[str(check.TC/'bin/cargo'),'test','--offline','--manifest-path',str(PRIVATE/'harness/Cargo.toml'),'--','--test-threads=1']
    with replay_receipt(out,receipt):
        code=LoggedCommands(out,ROOT,receipt['commands'],timeout=600,guard=guard)(action,args,env)
        require(code==0,'focused harness failed; retained logs')
        guard.check();receipt['guard_passed']=True
    print('PASS',action)
if __name__=='__main__':main()
