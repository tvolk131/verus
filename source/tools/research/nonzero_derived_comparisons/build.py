"""Offline own compiler build; all writable Cargo state is private."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
from trial import ROOT, BUNDLE, RUSTC, authenticate
sys.path.insert(0,str(HERE.parents[1]))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--cargo-home',type=Path);ap.add_argument('--target',type=Path);a=ap.parse_args()
    out=Path(tempfile.mkdtemp(prefix='nonzero-derive-build-',dir='/tmp'));out.chmod(0o700)
    print(out,flush=True)
    rec=dict(schema=1,commands=[],publication=False,goal_complete=False)
    with replay_receipt(out,rec):
        authenticate()
        # Copy already present package inputs, never install or update dependencies.
        home=a.cargo_home or out/'cargo-home'
        if a.cargo_home is None:
            home.mkdir()
            subprocess.run(['cp','-a','--reflink=auto','/home/dev/.cargo/registry',str(home/'registry')],check=True)
        if not (home/'git/db').exists():
            (home/'git').mkdir(exist_ok=True)
            subprocess.run(['cp','-a','--reflink=auto','/home/dev/.cargo/git/db',str(home/'git/db')],check=True)
        names=subprocess.check_output(['git','ls-files','source','dependencies','rust-toolchain.toml'],cwd=ROOT,text=True).splitlines()
        inputs=[ROOT/n for n in names if (ROOT/n).is_file()]
        inputs += [Path(__file__), HERE/'trial.py', RUSTC, RUSTC.with_name('cargo'), BUNDLE/'z3']
        # Pin all copied registry packages, including their build scripts.
        inputs += [p for p in (home/'registry'/'src').rglob('*') if p.is_file()]
        inputs += [p for p in (home/'git/db').rglob('*') if p.is_file()]
        inputs += [p for p in (ROOT/'dependencies').rglob('*') if p.is_file() and '.git' not in p.parts]
        guard=ArtifactGuard(inputs)
        target=a.target or out/'target'
        rec['target']=str(target)
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',
                 CARGO_HOME=str(home),CARGO_TARGET_DIR=str(target),CARGO_BUILD_BUILD_DIR=str(target),
                 CARGO_BUILD_JOBS='2',TMPDIR=str(out),XDG_CACHE_HOME=str(out/'cache'),
                 VERUS_INFER_CACHE=str(out/'infer-cache'),VERUS_Z3_PATH=str(BUNDLE/'z3'))
        argv=['/home/dev/.cargo/bin/cargo','build','--offline','--locked','--release',
              '-p','rust_verify','-p','verus','-p','verus_builtin_macros','-p','verus_state_machines_macros',
              '-p','verus_builtin','-p','vstd_build']
        (out/'input-freeze.json').write_text(json.dumps(dict(inputs=guard.initial,argv=argv,environment={k:env[k] for k in ['CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_BUILD_DIR','CARGO_BUILD_JOBS','TMPDIR','VERUS_INFER_CACHE','VERUS_Z3_PATH']},before_first_trial=True),indent=2)+'\n')
        fg=ArtifactGuard([out/'input-freeze.json'])
        rec['freeze_sha256']=digest(out/'input-freeze.json')
        run=LoggedCommands(out,ROOT/'source',rec['commands'],timeout=1800,guard=guard)
        try:
            rc=run('compiler',argv,env);rec['terminal_exit']=rc
            print('compiler exit',rc,flush=True)
            require(rc==0,'own compiler build failed')
            rec['products']={str(p):digest(p) for p in (target/'release').iterdir() if p.is_file()}
        finally:
            guard.check();fg.check();rec['final_guards_passed']=True
            rec['logs']={str(p.name):digest(p) for p in [out/'compiler.stdout',out/'compiler.stderr'] if p.exists()}
if __name__=='__main__':main()
