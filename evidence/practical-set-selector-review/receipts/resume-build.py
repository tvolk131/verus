import os,json,hashlib,shutil,sys
from pathlib import Path
ROOT=Path("/home/dev/.t3/worktrees/verus/t3-37b5504b");OUT=Path("/tmp/practical-set-review-902041b3");home=OUT/"cargo-home";bundle=OUT/"bundle"
sys.path.insert(0,str(ROOT/"source/tools"));from infer_experiment import LoggedCommands,replay_receipt
H=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
results={"production_source":json.loads((OUT/"setup-failure.json").read_text())["source"]}

T=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
env=dict(os.environ,CARGO_HOME=str(home),CARGO_TARGET_DIR=str(OUT/'target'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTC=str(T/'bin/rustc'),RUSTDOC=str(T/'bin/rustdoc'),RUSTC_BOOTSTRAP='1',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',VERUS_IN_VARGO='1',VERUS_Z3_PATH=str(bundle/'z3'))
env['PATH']='/home/dev/.cargo/bin:'+str(T/'bin')+':'+env['PATH'];env['LD_LIBRARY_PATH']=str(T/'lib')+':'+env.get('LD_LIBRARY_PATH','')
(OUT/'env.json').write_text(json.dumps({k:v for k,v in env.items() if os.environ.get(k)!=v},indent=2))
receipt={'commands':[],'source':results['production_source'],'copied_pin_manifest':H(OUT/'copied-pins.json')}
with replay_receipt(OUT,receipt):
 run=LoggedCommands(OUT,ROOT,receipt['commands'],timeout=7200)
 cmd=[str(T/'bin/cargo'),'build','--release','--offline','--manifest-path','source/Cargo.toml','-p','rust_verify','-p','verus','-p','verus_builtin','-p','verus_builtin_macros','-p','verus_state_machines_macros','-p','vstd_build']
 assert run('build-release',cmd,env)==0
 for n in ['verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so']:shutil.copy2(OUT/'target/release'/n,bundle/n)
 assert run('units',[str(T/'bin/cargo'),'test','--release','--offline','--manifest-path','source/Cargo.toml','-p','vir','-p','air','--lib'],env)==0
 run=LoggedCommands(OUT,ROOT/'source',receipt['commands'],timeout=7200)
 assert run('vstd',[str(OUT/'target/release/vstd_build'),str(bundle),'--release','--verbose'],env)==0
 receipt['tools']={n:H(bundle/n) for n in ['verus','rust_verify','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib','z3']}
 (bundle/'receipt.json').write_text(json.dumps({'source':receipt['source'],'tools':receipt['tools']},indent=2)+'\n')
print('FRESH BUILD/UNITS/VSTD COMPLETE',flush=True)
