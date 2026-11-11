"""Exact private offline build; no shared Cargo/config/target writes."""
import os,json,hashlib,shutil,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]; OUT=Path('/tmp/practical-word-combined-review-5ecea5fe'); OUT.mkdir(exist_ok=False)
sys.path.insert(0,str(ROOT/'source/tools')); from infer_experiment import LoggedCommands,replay_receipt
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
base=Path('/tmp/certified-foreign-bool-46i7h2jy'); home=OUT/'cargo-home'
pins={}
for name in ['cache-inputs.json','index-git-authentication.json']:
 d=json.loads((base/name).read_text()); shutil.copy2(base/name,OUT/name)
 entries=d['archives'] if name=='cache-inputs.json' else d
 for rel,h in entries.items():
  src=base/rel if name=='cache-inputs.json' else base/'cargo-home'/rel
  assert H(src)==h,(rel,'hash'); dest=OUT/rel if name=='cache-inputs.json' else home/rel
  dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest);pins[rel]=h
refs=home/'git/db/getopts-a5b2265512d62342/refs/remotes/origin';refs.mkdir(parents=True);(refs/'parse-partial').write_text('1ba7b831cfa9ebf9dea584ba43c71ea49613267f\n')
(OUT/'copied-pins.json').write_text(json.dumps(pins,indent=2)+'\n')
results=json.loads((ROOT/'source/tools/research/practical_redb_fedimint/common_set/RESULTS_S1.json').read_text())
production={str(p.relative_to(ROOT)):H(p) for p in (ROOT/'source').rglob('*') if p.is_file() and ('target' not in p.parts) and p.suffix in ('.rs','.toml','.lock','.py')}
results['production_source']=production
sel=json.loads((ROOT/'source/tools/research/practical_redb_fedimint/SELECTION_R2_F2.json').read_text())
baseline=Path(sel['runtime']['path']); dst=OUT/'accepted75';dst.mkdir()
assert H(baseline/'receipt.json')==sel['runtime']['receipt_sha256']
for n,h in sel['runtime']['tools'].items():assert H(baseline/n)==h;shutil.copy2(baseline/n,dst/n)
shutil.copy2(baseline/'receipt.json',dst/'receipt.json');(dst/'verus-root').touch()
bundle=OUT/'bundle';bundle.mkdir();shutil.copy2(dst/'z3',bundle/'z3');(bundle/'verus-root').touch()
T=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
env=dict(os.environ,CARGO_HOME=str(home),CARGO_TARGET_DIR=str(OUT/'target'),CARGO_BUILD_JOBS='2',CARGO_BUILD_BUILD_DIR=str(OUT/'build'),CARGO_NET_OFFLINE='true',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTC=str(T/'bin/rustc'),RUSTDOC=str(T/'bin/rustdoc'),RUSTC_BOOTSTRAP='1',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',VERUS_IN_VARGO='1',VERUS_Z3_PATH=str(bundle/'z3'))
env['PATH']='/home/dev/.cargo/bin:'+str(T/'bin')+':'+env['PATH'];env['LD_LIBRARY_PATH']=str(T/'lib')+':'+env.get('LD_LIBRARY_PATH','')
(OUT/'env.json').write_text(json.dumps({k:v for k,v in env.items() if os.environ.get(k)!=v},indent=2))
receipt={'commands':[],'source':results['production_source'],'copied_pin_manifest':H(OUT/'copied-pins.json')}
with replay_receipt(OUT,receipt):
 run=LoggedCommands(OUT,ROOT,receipt['commands'],timeout=7200)
 cmd=[str(T/'bin/cargo'),'build','--release','--offline','--manifest-path','source/Cargo.toml','--config','build.build-dir="'+str(OUT/'build')+'"','--config','build.target-dir="'+str(OUT/'target')+'"','-p','rust_verify','-p','verus','-p','verus_builtin','-p','verus_builtin_macros','-p','verus_state_machines_macros','-p','vstd_build']
 assert run('build-release',cmd,env)==0
 for n in ['verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so']:shutil.copy2(OUT/'target/release'/n,bundle/n)
 assert run('units',[str(T/'bin/cargo'),'test','--release','--offline','--manifest-path','source/Cargo.toml','--config','build.build-dir="'+str(OUT/'build')+'"','--config','build.target-dir="'+str(OUT/'target')+'"','-p','vir','-p','air','--lib'],env)==0
 run=LoggedCommands(OUT,ROOT/'source',receipt['commands'],timeout=7200)
 assert run('vstd',[str(OUT/'target/release/vstd_build'),str(bundle),'--release','--verbose'],env)==0
 receipt['tools']={n:H(bundle/n) for n in ['verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib','z3']}
 (bundle/'receipt.json').write_text(json.dumps({'source':receipt['source'],'tools':receipt['tools']},indent=2)+'\n')
(OUT/'ready').touch()
print('FRESH BUILD/UNITS/VSTD COMPLETE',flush=True)
