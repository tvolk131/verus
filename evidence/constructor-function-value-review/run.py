"""Private build/proof driver; explicit guards, safe under -O."""
import json, pathlib, sys
import epoch as e
TARGET=e.WORK/'source/target-verus/release'
ENV={'PATH':str(e.TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin','CARGO_HOME':str(e.PRIVATE/'cargo-home'),
 'GIT_DIR':'/home/dev/projects/verus/.git/worktrees/t3-755cb0d1','GIT_WORK_TREE':str(e.ROOT),'VARGO_TOOLCHAIN':'1.99.1-x86_64-unknown-linux-gnu','CARGO_BUILD_JOBS':'2','RUSTUP_TOOLCHAIN':'1.99.1','RUSTUP_HOME':'/home/dev/.rustup',
 'LD_LIBRARY_PATH':str(e.TC/'lib'),'VERUS_Z3_PATH':str(e.BUNDLE/'z3'),
 'TMPDIR':str(e.PRIVATE/'tmp'),'VERUS_IN_VARGO':'1',
 'CARGO_TARGET_DIR':str(TARGET.parent),'CARGO_BUILD_BUILD_DIR':str(TARGET.parent)}
def tools():
 return [TARGET/n for n in ('rust_verify','verus','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd.vir','libvstd.rlib')]
def main():
 mode=sys.argv[1]
 if mode=='build':
  argv=[str(e.TC/'bin/cargo'),'build','--offline','--locked','--release']
  for p in ('rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build'): argv+=['-p',p]
  e.require(e.run('build-3',argv,env=ENV,products=[TARGET/x for x in ('rust_verify','verus','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','vstd_build')])['exit']==0,'build failed')
 elif mode=='vstd':
  e.require(e.run('vstd',[str(TARGET/'vstd_build'),str(TARGET),'--release','--verbose'],extra=[x for x in tools() if x.is_file()]+[TARGET/'vstd_build'],env={**ENV,'VERUS_INFER_CACHE':str(e.PRIVATE/'vstd-cache')},products=[TARGET/'vstd.vir',TARGET/'libvstd.rlib'])['exit']==0,'vstd failed')
 elif mode=='proof':
  label,src=sys.argv[2:4]; src=pathlib.Path(src).resolve(); flags=sys.argv[4:]
  e.run(label,[str(TARGET/'verus'),str(src),'--crate-type=lib','--no-cheating','--output-json','--multiple-errors','20',*flags],extra=tools()+[src],env={**ENV,'VERUS_ROOT':str(TARGET),'VERUS_INFER_CACHE':str(e.PRIVATE/(label+'-cache'))},cwd=e.PRIVATE)
 elif mode=='test-build':
  e.run('test-build',[str(e.TC/'bin/cargo'),'test','--offline','--locked','--release','-p','rust_verify_test','--no-run',*sys.argv[2:]],extra=tools(),env=ENV)
 elif mode=='test-run':
  label,binary=sys.argv[2:4]; binary=pathlib.Path(binary).resolve()
  e.run(label,[str(binary),*sys.argv[4:]],extra=tools()+[binary],env={**ENV,'VERUS_EXTRA_ARGS':'--no-cheating','VERUS_KEEP_TEST_DIR':'1','VERUS_TEST_LOG_DIR':str(e.PRIVATE/(label+'-logs')),'VERUS_INFER_CACHE':str(e.PRIVATE/(label+'-cache'))})
 elif mode=='tests':
  label=sys.argv[2]
  e.run(label,[str(e.TC/'bin/cargo'),'test','--offline','--locked','--release','-p','rust_verify_test',*sys.argv[3:]],extra=tools(),env={**ENV,'VERUS_EXTRA_ARGS':'--no-cheating','VERUS_KEEP_TEST_DIR':'1','VERUS_TEST_LOG_DIR':str(e.PRIVATE/(label+'-logs')),'VERUS_INFER_CACHE':str(e.PRIVATE/(label+'-cache'))})
 else: raise RuntimeError('unknown mode')
if __name__=='__main__': main()
