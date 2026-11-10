"""Focused repository compiler tests, with private target layout and solver."""
import json
import os
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
from trial import ROOT,RUSTC
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require

bundle=Path(sys.argv[1]);target=Path(sys.argv[2]);home=Path(sys.argv[3])
out=Path(tempfile.mkdtemp(prefix='nonzero-derive-tests-',dir='/tmp'));out.chmod(0o700);print(out,flush=True)
r=dict(schema=1,commands=[],publication=False,goal_complete=False)
with replay_receipt(out,r):
    br=json.loads((bundle/'receipt.json').read_bytes());require(br['success'] is True,'bundle failure')
    for n,h in br['tools'].items():require(digest(bundle/n)==h,'frozen tool drift')
    layout=target.parent/'target-verus';layout.mkdir(exist_ok=True)
    link=layout/'release'
    if link.exists():require(link.resolve()==bundle.resolve(),'test tool layout differs')
    else:link.symlink_to(bundle,target_is_directory=True)
    inputs=[bundle/n for n in br['tools']]+[bundle/'receipt.json',RUSTC,RUSTC.with_name('cargo'),Path(__file__)]
    inputs += list((ROOT/'source/rust_verify_test/tests/common').rglob('*.rs'))
    inputs += [ROOT/'source/rust_verify_test/tests'/n for n in ['infer_nonzero_derive.rs','infer_compare_derive.rs']]
    guard=ArtifactGuard(inputs)
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',
             LD_LIBRARY_PATH=str(RUSTC.parent.parent/'lib'),CARGO_HOME=str(home),
             CARGO_TARGET_DIR=str(target),CARGO_BUILD_BUILD_DIR=str(target),CARGO_BUILD_JOBS='2',
             VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),TMPDIR=str(out),XDG_CACHE_HOME=str(out/'cache'))
    argv=['/home/dev/.cargo/bin/cargo','test','--offline','--locked','--release','-p','rust_verify_test',
          '--test','infer_nonzero_derive','--test','infer_compare_derive']
    (out/'input-freeze.json').write_text(json.dumps(dict(inputs=guard.initial,argv=argv,environment={k:env[k] for k in ['CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_BUILD_DIR','CARGO_BUILD_JOBS','VERUS_Z3_PATH','VERUS_INFER_CACHE','TMPDIR']},before_first_trial=True),indent=2)+'\n');fg=ArtifactGuard([out/'input-freeze.json'])
    r['freeze_sha256']=digest(out/'input-freeze.json')
    run=LoggedCommands(out,ROOT/'source',r['commands'],timeout=1200,guard=guard)
    try:
        rc=run('tests',argv,env);r['terminal_exit']=rc;require(rc==0,'focused compiler tests failed')
    finally:
        guard.check();fg.check();r['final_guards_passed']=True
        r['products']={str(p.relative_to(out)):digest(p) for p in out.rglob('*') if p.is_file() and p.name!='receipt.json'}
        r['test_products']={str(p):digest(p) for d in (target/'release/test_inputs').glob('infer_*derive-*') for p in d.rglob('*') if p.is_file()}
        print('test exit',r.get('terminal_exit'),flush=True)
