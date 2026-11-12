#!/usr/bin/env python3
"""Finish the same bounded preparation: allow existing rustc's expansion printer."""
import json, os, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[4]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require
from infer_eval_support import ArtifactGuard,digest
BASE=Path('/tmp/fedimint-bootstrap-worker-3ad8038e');OUT=BASE/'expansion-command'
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
def main():
    OUT.mkdir(exist_ok=False);r=dict(kind='SAME_PREPARATION_EXPANSION_PRINTER_SETUP_CORRECTION',commands=[],native_context_only=True,crypto_proved=False)
    with replay_receipt(OUT,r):
        f=json.loads((HERE/'EXPANSION_RETRY_FREEZE.json').read_text())
        for n,h in f['inputs'].items():require(digest(Path(n))==h,'expansion input '+n)
        guard=ArtifactGuard([Path(n) for n in f['inputs']]);run=LoggedCommands(OUT,BASE/'expansion/project',r['commands'],timeout=180,guard=guard)
        env=dict(os.environ,PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN=str(TC),LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(BASE/'expansion/cargo-home'),CARGO_TARGET_DIR=str(BASE/'expansion/target'),RUSTC_BOOTSTRAP='1')
        r['RUSTC_BOOTSTRAP']='1 (native expansion printing only; no Verus checks suppressed)'
        code=run('expand',[str(TC/'bin/cargo'),'rustc','--offline','--lib','--','-Zunpretty=expanded'],env)
        require(code==0,'stop at failed bounded native expansion')
        r['expanded_sha256']=digest(OUT/'expand.stdout');r['inputs']=f['inputs']
    print(OUT/'receipt.json')
if __name__=='__main__':main()
