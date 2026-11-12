"""Existing actual Rev source probes, ordinary/Infer, on authenticated D2 tools."""
import importlib.util
import json
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest
from infer_experiment import LoggedCommands, replay_receipt, require
S=Path(sys.argv[1]); O=Path(sys.argv[2]); O.mkdir(exist_ok=False)
binding=json.loads(Path(__file__).with_name('BINDING.json').read_text())
fixture=ROOT/'source/tools/research/checked_rev_bridge_design/probes.py'
spec=importlib.util.spec_from_file_location('frozen_rev_probes',fixture)
probes=importlib.util.module_from_spec(spec);spec.loader.exec_module(probes)
receipt=dict(commands=[], results={}, builds=0, provenance='Existing actual core Rev source probes; ordinary and --infer-goals opt-in. No installed Internal Rev representation/source authority is claimed.')
with replay_receipt(O,receipt):
    for n,h in binding['tools'].items(): require(digest(S/n)==h,'D2 tool '+n)
    for d in ['tmp','cache']: (O/d).mkdir()
    TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
    env=dict(os.environ,PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',VERUS_IN_VARGO='1',VERUS_ROOT=str(S),VERUS_Z3_PATH=str(S/'z3'),RUST_MIN_STACK='20971520',LD_LIBRARY_PATH=str(TC/'lib')+':'+str(TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib'),TMPDIR=str(O/'tmp'),XDG_CACHE_HOME=str(O/'cache'),VERUS_INFER_CACHE=str(O/'infer-cache'))
    sources=[]
    for name in probes.BODIES:
        p=O/(name+'.rs');p.write_text(probes.source(name));sources.append(p)
    guard=ArtifactGuard([*[S/n for n in binding['tools']],fixture,Path(__file__),*sources,*[ROOT/p for p in binding['source']]])
    run=LoggedCommands(O,ROOT/'source',receipt['commands'],timeout=180,guard=guard)
    expected={'available':(1,0),'lawful':(0,1),'inner':(0,1),'loop':(1,1)}
    for p in sources:
        for infer in [False,True]:
            label=p.stem+('-infer' if infer else '-ordinary')
            code=run(label,[str(S/'verus'),str(p),'--crate-name','rev_probe','--crate-type=lib','--no-cheating','--compile','--output-json','--multiple-errors','10','-o',str(O/(label+'.rlib')),*(['--infer-goals'] if infer else [])],env)
            v=json.loads((O/(label+'.stdout')).read_text())['verification-results']
            stderr=(O/(label+'.stderr')).read_text().lower()
            require(not v['encountered-vir-error'],'frontend '+label)
            require(v['is-verifying-entire-crate'],'full crate '+label)
            require((v['verified'],v['errors'])==expected[p.stem], 'existing behavior '+label)
            require((code==0)==(p.stem=='available'),'exit '+label)
            require(not any(s in stderr for s in ['panicked at','resource limit','timed out']),'resource '+label)
            receipt['results'][label]=v
    guard.check(); receipt.update(guard_passed=True,inputs={p:d['sha256'] for p,d in guard.initial.items()},logs={p.name:digest(p) for p in O.glob('*.std*')})
print(json.dumps(dict(output=str(O),success=receipt['success'],receipt_sha256=digest(O/'receipt.json'))))
