#!/usr/bin/env python3
"""Build/query the local writer with ordinary Rust analysis; fail closed."""
import argparse, json, os, sys
from pathlib import Path
sys.dont_write_bytecode = True
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
sys.path.insert(0,str(ROOT/'source/tools'))
from infer_eval_support import ArtifactGuard, digest, support_artifacts
from infer_experiment import LoggedCommands, replay_receipt, require
from entry_check import inspect, controls
p=argparse.ArgumentParser()
p.add_argument('--output',type=Path,required=True)
p.add_argument('--freeze',type=Path,required=True)
p.add_argument('--sysroot',type=Path,default=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu'))
a=p.parse_args();a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
receipt=dict(schema=1,commands=[],native_bridge_proved=False,publication=False)
with replay_receipt(a.output,receipt):
 f=json.loads(a.freeze.read_text());require(f['before_first_trial'] is True,'missing pretrial freeze')
 paths=[]
 for rel,sha in f['files'].items():
  path=ROOT/rel;require(digest(path)==sha,'frozen source changed: '+rel);paths.append(path)
 tools=[a.sysroot/'bin/rustc',a.sysroot/'bin/cargo',*a.sysroot.glob('lib/librustc_driver-*.so'),*a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/libcore-*.rlib'),*a.sysroot.glob('lib/rustlib/x86_64-unknown-linux-gnu/lib/librustc_*.rmeta')]
 guard=ArtifactGuard([*paths,*tools,*support_artifacts(),a.freeze]);receipt['artifacts']=guard.initial
 env=dict(os.environ,PATH='/home/dev/.cargo/bin:'+os.environ['PATH'],RUSTUP_TOOLCHAIN='1.99.1',RUSTC_BOOTSTRAP='1',CARGO_BUILD_JOBS='2',CARGO_TARGET_DIR=str(a.output/'target'),LD_LIBRARY_PATH=str(a.sysroot/'lib'))
 run=LoggedCommands(a.output,ROOT/'source',receipt['commands'],timeout=240,guard=guard)
 require(run('build',[str(a.sysroot/'bin/cargo'),'build','--offline','--locked','--manifest-path',str(HERE/'Cargo.toml'),'--config',f'build.build-dir="{a.output/"build"}"'],env)==0,'build failed; excluded compiler/API failure')
 binary=a.output/'target/debug/native-borrow-entry-extract';bg=ArtifactGuard([binary]);receipt['extractor']=bg.initial
 selected=HERE.parent/'buffer_transfer_goal/original-selected.rs'
 original=HERE/'fixtures/original.rs'
 transfer=HERE.parent/'buffer_transfer_goal/freeze.json'
 target=HERE.parent/'mutable_buffer_goal/transfer-freeze.json'
 require(digest(transfer)==json.loads(target.read_text())['pretrial_manifest_sha256'],'original pretrial manifest changed')
 require(digest(selected)==json.loads(transfer.read_text())['files']['original-selected.rs'],'upstream selected source changed')
 require('\n'.join(l.strip() for l in selected.read_text().splitlines()) in '\n'.join(l.strip() for l in original.read_text().splitlines()),'frozen original loop mismatch')
 first=None
 for case in f['cases']:
  name=case['name'];src=HERE/'fixtures'/(name+'.rs');out=a.output/(name+'.json');bg.check()
  rc=run(name,[str(binary),str(out),str(src),'--crate-name','native_writer','--edition','2024','--sysroot',str(a.sysroot),'-Zmir-opt-level=0','-Coverflow-checks=on'],env);bg.check()
  stderr=(a.output/(name+'.stderr')).read_text()
  if case['classification']=='lifetime-rejection':
   require(rc!=0 and not out.exists() and case['expected_message'] in stderr,'expected overlapping borrow rejection')
   require('panicked at' not in stderr and 'resource' not in stderr,'not lifetime evidence')
   receipt.setdefault('cases',[]).append(dict(case=name,source_sha256=digest(src),classification='lifetime-rejection',native_bridge_proved=False))
   continue
  require(rc==0,'extraction failed; no native proof')
  data=json.loads(out.read_text());require(data['source_sha256']==digest(src),'source evidence mismatch')
  require(data['normal_analysis_completed'] is True and data['publication'] is False and data['native_bridge_proved'] is False,'invalid status')
  try:result=inspect(data,digest(src))
  except ValueError as e:
   require(case['classification']=='eligibility-rejection' and str(e)==case['reason'],'wrong source control rejection: '+str(e))
   result=dict(rejected=True,reason=str(e),semantic_negative=False)
  else: require(case['classification']=='entry-eligible','source mutation accepted')
  receipt.setdefault('cases',[]).append(dict(case=name,source_sha256=digest(src),evidence_sha256=digest(out),classification=case['classification'],loans=len(data['loans']),inspection=result,native_bridge_proved=False))
  if name=='original':
   first=data;receipt['certificate_controls']=controls(data)
   # Determinism is checked on actual typed output, not an expected JSON fixture.
   repeat=a.output/'original-repeat.json'
   require(run('original-repeat',[str(binary),str(repeat),str(src),'--crate-name','native_writer','--edition','2024','--sysroot',str(a.sysroot),'-Zmir-opt-level=0','-Coverflow-checks=on'],env)==0,'repeat failed')
   require(json.loads(repeat.read_text())==data,'nondeterministic local borrow evidence')
 require(first is not None,'original writer evidence required')
 guard.check();bg.check()
 print(json.dumps(dict(entry_eligible=True,native_bridge_proved=False,publication=False,cases=len(receipt['cases']),controls=len(receipt['certificate_controls']))))
