"""Own checkout build and full, unfiltered Infer target suites. Raw command binding."""
import os,sys,subprocess,json,time,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];RAW=Path('/tmp/stable-key-review-e84eed6c');SRC=ROOT/'source'
env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+str(SRC/'target-verus/release')+':'+os.environ['PATH'],CARGO_BUILD_JOBS='2',VERUS_Z3_PATH=str(SRC/'z3'),VERUS_IN_VARGO='1',RUST_MIN_STACK='33554432',VERUS_INFER_CACHE=str(RAW/'compiler-cache'),VERUS_TEST_LOG_DIR=str(RAW/'compiler-inputs'))
records=json.loads((RAW/'validation.json').read_text()) if (RAW/'validation.json').exists() else []
def run(label,cmd):
    start=time.time();p=RAW/(label+'.log')
    with p.open('w') as f:r=subprocess.run(cmd,cwd=SRC,env=env,stdout=f,stderr=subprocess.STDOUT)
    records.append(dict(label=label,argv=cmd,cwd=str(SRC),environment={k:env[k] for k in ['PATH','CARGO_BUILD_JOBS','VERUS_Z3_PATH','VERUS_IN_VARGO','RUST_MIN_STACK','VERUS_INFER_CACHE','VERUS_TEST_LOG_DIR']},returncode=r.returncode,elapsed=time.time()-start,log=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
    (RAW/'validation.json').write_text(json.dumps(records,indent=2)+'\n');print(label,r.returncode,flush=True)
    return r.returncode
if '--build-vstd' in sys.argv:
    if run('vstd-lock',['cargo','generate-lockfile','--offline','--manifest-path','vstd/Cargo.toml']):sys.exit(1)
    sys.exit(run('vstd',[str(SRC/'target-verus/release/cargo-verus'),'verus','build','--release','--locked','--offline','--manifest-path','vstd/Cargo.toml','--','--num-threads','2']))
if '--build' in sys.argv:
    sys.exit(run('compiler-build',['cargo','build','--release','--locked','-p','rust_verify','-p','verus','-p','cargo-verus','-p','vstd_build']))
run('vir',['cargo','test','--release','--locked','-p','vir','--lib'])
for t in ['infer_buffer_observation_goals','infer_mutable_buffer_goals','infer']:
    if run(t,['cargo','test','--release','--locked','-p','rust_verify_test','--test',t,'--','--test-threads=2']):sys.exit(1)
