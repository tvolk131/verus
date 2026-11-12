#!/usr/bin/env python3
"""Bounded frozen review; existing execution/guard helpers, private raw outputs."""
import sys
sys.dont_write_bytecode = True
import json, os, shutil, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
WORK=Path('/tmp/owned-cell-f1-fix-review-ad51b8c2')
SNAP=Path('/tmp/owned-cell-F1-skip-fix-9cc0f65c-parent-snapshot')
sys.path.insert(0,str(ROOT/'source/tools'))
sys.path.insert(0,str(ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation'))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require
import check

def authenticate():
    require(not WORK.exists(),'private output already exists')
    require(digest(SNAP/'receipt.json')=='9b414f0493385c649d856fd181625ec74778173e10623ee6ba3c3fda959b945f','receipt')
    r=json.loads((SNAP/'receipt.json').read_text())
    require(r['source_commit']=='9cc0f65c5e8ebb75f6858475277991e3bfd940d7','source identity')
    for n,h in r['tools_sha256'].items():require(digest(SNAP/n)==h,'snapshot '+n)
    frozen=json.loads((ROOT/'source/tools/research/redb_crash_protocol/native_state_link/implementation/F1_FIX_RESULTS.json').read_text())
    require(r['tools_sha256']==frozen['tools'],'complete tool map')
    for p,h in frozen['sources'].items():require(digest(ROOT/p)==h,'corrected source '+p)
    old=json.loads((ROOT/'evidence/owned-cell-f1-review/AUTHENTICATION.json').read_text())
    require(all(old['tools'][n]==h for n,h in r['tools_sha256'].items() if n!='rust_verify'),'unchanged seven')
    WORK.mkdir(); b=WORK/'bundle';b.mkdir()
    for n,h in r['tools_sha256'].items():
        shutil.copy2(SNAP/n,b/n);require(digest(b/n)==h,'private '+n)
    (b/'verus-root').touch()
    ids={s:subprocess.check_output(['git','rev-parse',s],cwd=ROOT,text=True).strip() for s in ['HEAD','9cc0f65c','6f0b3b4c']}
    a=dict(identities=ids,snapshot=str(SNAP),receipt_sha256=digest(SNAP/'receipt.json'),tools=r['tools_sha256'],sources=frozen['sources'],private=str(WORK),authenticated_before_copy=True,unchanged_seven=True)
    (HERE/'AUTHENTICATION.json').write_text(json.dumps(a,indent=2)+'\n')
    print('authenticated eight tools and corrected sources')


def setup_harness():
    check.setup(WORK/'environment')
    h=WORK/'harness';(h/'tests/common').mkdir(parents=True)
    (h/'Cargo.toml').write_text('[package]\nname="f1-fix-review-harness"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[dev-dependencies]\nserde_json="=1.0.151"\ntempfile="=3.27.0"\n')
    launch=WORK/'launchers';launch.mkdir()
    (h/'tests/common/mod.rs').write_text('pub mod infer;\n')
    (h/'tests/common/infer.rs').write_text('pub struct InferCrates {pub directory:tempfile::TempDir,pub profile:std::path::PathBuf}\nimpl InferCrates {pub fn new()->Self {Self {directory:tempfile::tempdir().unwrap(),profile:std::path::PathBuf::from('+json.dumps(str(launch))+')}} pub fn vir(&self,name:&str)->std::path::PathBuf {self.directory.path().join(format!("{name}.vir"))}}\n')
    frozen=ROOT/'source/tools/research/redb_crash_protocol/native_state_link/owned_goals.rs'
    for n in ['local_checked_owned_cell','local_checked_owned_cell_admission']:
        src=(ROOT/f'source/rust_verify_test/tests/{n}.rs').read_text()
        src=src.replace('#![feature(rustc_private)]','').replace('"../../tools/research/redb_crash_protocol/native_state_link/owned_goals.rs"',json.dumps(str(frozen)))
        (h/f'tests/{n}.rs').write_text(src)
    wrapper = """#!/usr/bin/env python3
import sys
sys.dont_write_bytecode=True
import os,tempfile,shutil
from pathlib import Path
ROOT=Path(ROOT_LITERAL)
WORK=Path(WORK_LITERAL)
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands,replay_receipt
logs=WORK/'runs';logs.mkdir(exist_ok=True)
out=Path(tempfile.mkdtemp(prefix='invocation-',dir=logs))
argv=[str(WORK/'bundle'/Path(sys.argv[0]).name),*sys.argv[1:]]
for i,arg in enumerate(sys.argv[1:]):
    p=Path(arg)
    if p.suffix=='.rs' and p.is_file():shutil.copy2(p,out/f'input-{i}.rs')
r=dict(commands=[])
with replay_receipt(out,r):
    code=LoggedCommands(out,Path.cwd(),r['commands'],timeout=60)('compile',argv,dict(os.environ))
    r['exit']=code
sys.stdout.write((out/'compile.stdout').read_text());sys.stderr.write((out/'compile.stderr').read_text());sys.exit(code)
""".replace('ROOT_LITERAL',repr(str(ROOT))).replace('WORK_LITERAL',repr(str(WORK)))
    for n in ['verus','rust_verify']:
        (launch/n).write_text(wrapper);(launch/n).chmod(0o755)
    print('private focused harness ready')

def environment():
    e=check.env(WORK/'environment')
    e.update(VERUS_ROOT=str(WORK/'bundle'),VERUS_Z3_PATH=str(WORK/'bundle/z3'),PYTHONDONTWRITEBYTECODE='1')
    return e

def focused():
    out=WORK/'focused';out.mkdir()
    auth=json.loads((HERE/'AUTHENTICATION.json').read_text())
    paths=[WORK/'bundle'/n for n in auth['tools']]
    paths += [Path(__file__),ROOT/'source/tools/infer_experiment.py',ROOT/'source/tools/infer_eval_support.py',*sorted((WORK/'harness/tests').rglob('*.rs'))]
    guard=ArtifactGuard(paths);r=dict(commands=[],inputs={str(p):digest(p) for p in paths})
    args=[str(check.TC/'bin/cargo'),'test','--offline','--manifest-path',str(WORK/'harness/Cargo.toml'),'--','--test-threads=1']
    with replay_receipt(out,r):
        code=LoggedCommands(out,ROOT,r['commands'],timeout=600,guard=guard)('focused',args,environment())
        r['exit']=code;require(code==0,'focused tests failed')
        guard.check();r['guard_passed']=True
    print('PASS focused')

if __name__=='__main__':
    {'authenticate':authenticate,'setup':setup_harness,'focused':focused}[sys.argv[1]]()
