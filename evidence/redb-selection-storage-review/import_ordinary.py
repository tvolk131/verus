"""Two bounded supplemental ordinary callers: strict replay enables Infer on all clients."""
from pathlib import Path
import json, os, sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_experiment import LoggedCommands, replay_receipt, require, validate_goal_run
from infer_eval_support import ArtifactGuard, digest
provider=Path('/tmp/redb-new-review-03fd44ed-selection')
out=Path('/tmp/redb-new-review-03fd44ed-import-ordinary')
out.mkdir(exist_ok=False)
r=dict(commands=[],outcomes=[],source_coupling_proved=False)
previous=json.loads((provider/'receipt.json').read_text())
with replay_receipt(out,r):
    inputs=[*provider.glob('*.rs'),*provider.glob('*.vir'),*provider.glob('*.rlib'),*provider.joinpath('bundle').iterdir()]
    guard=ArtifactGuard(inputs)
    call=LoggedCommands(out,out,r['commands'],timeout=120,guard=guard)
    tc='/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'
    for mode in ['ordinary','infer']:
        original=next(c for c in previous['commands'] if c['label']==mode+'_client_healthy')
        argv=[s for s in original['argv'] if s!='--infer-goals']
        argv[argv.index('--export')+1]=str(out/(mode+'.vir'))
        env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+tc+'/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',
                 RUSTUP_TOOLCHAIN=tc,LD_LIBRARY_PATH=tc+'/lib',VERUS_Z3_PATH=str(provider/'bundle/z3'),
                 VERUS_INFER_CACHE=str(out/('cache_'+mode)))
        code=call(mode,argv,env)
        data=validate_goal_run(dict(returncode=code,stdout=(out/(mode+'.stdout')).read_text(),
              stderr=(out/(mode+'.stderr')).read_text()),True,expected_functions=set(),
              exec_bodies={'imported_selection'},proof_bodies=set())
        require(data['verification-results']['verified']==1,'1/0 caller')
        require((out/(mode+'.vir')).exists(),'ordinary export')
        r['outcomes'].append(dict(label=mode+'_provider_no_optional_infer_client',data=data))
        print(mode,'ordinary caller 1/0',flush=True)
    guard.check()
