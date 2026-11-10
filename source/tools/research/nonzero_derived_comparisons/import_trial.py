"""Ordinary checked import of the frozen comparison component; no metadata edits."""
import json
import os
from pathlib import Path
import sys
import tempfile
sys.dont_write_bytecode=True
from trial import ROOT,RUSTC,authenticate,BUNDLE
sys.path.insert(0,str(Path(__file__).resolve().parents[2]))
from infer_eval_support import ArtifactGuard,digest
from infer_experiment import LoggedCommands,replay_receipt,require

bundle=Path(sys.argv[1]);provider=Path(sys.argv[2]);authenticate()
out=Path(tempfile.mkdtemp(prefix='nonzero-derive-import-',dir='/tmp'));out.chmod(0o700);print(out,flush=True)
r=dict(schema=1,commands=[],publication=False,goal_complete=False)
with replay_receipt(out,r):
    pr=json.loads((provider/'receipt.json').read_bytes());require(pr['success'] is True and pr['source_acceptance'] is True,'provider not checked')
    br=json.loads((bundle/'receipt.json').read_bytes());require(br['success'] is True,'bundle failure')
    for n,h in br['tools'].items():require(digest(bundle/n)==h,'tool drift')
    src=out/'client.rs';src.write_text('''use vstd::prelude::*;
use component::BorTag;
#[verus_infer] fn equal(a:&BorTag,b:&BorTag)->bool { <BorTag as PartialEq>::eq(a,b) }
verus! {
fn goal(a:&BorTag,b:&BorTag) {
    let r=equal(a,b);
    assert(r == (a.__verus_infer_nonzero_u64_view() == b.__verus_infer_nonzero_u64_view()));
}
}
''')
    inputs=[bundle/n for n in br['tools']]+[bundle/'receipt.json',provider/'receipt.json',provider/'component.vir',provider/'libcomponent.rlib',src,Path(__file__),RUSTC]
    inputs += [ROOT/n for n in ['source/vir/src/infer.rs','source/vir/src/infer/newtypes.rs','source/vir/src/infer/wrapper_compare.rs','source/vir/src/infer/registry.rs']]
    guard=ArtifactGuard(inputs)
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',
             VERUS_Z3_PATH=str(bundle/'z3'),VERUS_INFER_CACHE=str(out/'cache'),TMPDIR=str(out),XDG_CACHE_HOME=str(out/'cache'))
    argv=[str(bundle/'verus'),str(src),'--crate-type=lib','--crate-name=client','--no-cheating','--output-json','--infer-goals',
          '--extern','component='+str(provider/'libcomponent.rlib'),'--import','component='+str(provider/'component.vir'),'-L',str(provider)]
    (out/'input-freeze.json').write_text(json.dumps(dict(inputs=guard.initial,argv=argv,before_first_trial=True),indent=2)+'\n');fg=ArtifactGuard([out/'input-freeze.json'])
    r['freeze_sha256']=digest(out/'input-freeze.json')
    run=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
    try:
        rc=run('check',argv,env);r['terminal_exit']=rc
        data=json.loads((out/'check.stdout').read_text());r['verification']=data['verification-results'];r['inference']=data['verus-infer']
        print(json.dumps(dict(exit=rc,verification=r['verification'])),flush=True)
    finally:
        guard.check();fg.check();r['final_guards_passed']=True
        r['products']={str(p.relative_to(out)):digest(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='receipt.json'}
