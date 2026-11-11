"""Independent bounded review runner; private outputs, complete frozen receipts."""
import hashlib,json,os,shutil,subprocess,sys,time,tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E=ROOT/'evidence/local-checked-foreign-review'
W=Path('/tmp/checked-bool-build-klx7abqu')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
R=Path((E/'PRIVATE_ROOT').read_text().strip()) if (E/'PRIVATE_ROOT').exists() else None
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(x,m):
 if not x: raise RuntimeError(m)
def sha(p): return hashlib.file_digest(Path(p).open('rb'),'sha256').hexdigest()
def write(p,j): Path(p).write_text(json.dumps(j,indent=2,sort_keys=True)+'\n')
def manifest():
 files=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
 return {p:sha(ROOT/p) for p in files if p and not p.startswith('evidence/local-checked-foreign-review/')}
def env():
 return dict(HOME='/home/dev',PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',RUSTUP_HOME='/home/dev/.rustup',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(R/'cargo-home'),CARGO_TARGET_DIR=str(R/'target'),CARGO_BUILD_BUILD_DIR=str(R/'build'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',XDG_CACHE_HOME=str(R/'cache'),TMPDIR=str(R/'tmp'),VERUS_ROOT=str(R/'bundle'),VERUS_Z3_PATH=str(R/'bundle/z3'),VERUS_INFER_CACHE=str(R/'candidate-cache'),RUST_MIN_STACK=str(20*1024*1024),RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(R/'test-logs'))
def run(label,argv,extra=None,outputs=(),cwd=None):
 d=R/label;d.mkdir(parents=True,exist_ok=True)
 need(not (d/'receipt.json').exists(),'immutable receipt '+label)
 frozen=dict(argv=list(map(str,argv)),cwd=str(cwd or ROOT/'source'),environment=env()|dict(extra or {}),source=manifest(),tools={n:sha(R/'bundle'/n) for n in TOOLS if n not in outputs and (R/'bundle'/n).exists()},fixtures={str(a):sha(a) for a in argv if str(a).endswith('.rs') and Path(a).is_absolute() and Path(a).is_file()},runner_sha256=sha(__file__))
 write(d/'freeze.json',frozen);start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
  p=subprocess.run(frozen['argv'],cwd=frozen['cwd'],env=frozen['environment'],stdout=out,stderr=err)
 guards=frozen['source']==manifest() and all(sha(p)==h for p,h in frozen['fixtures'].items()) and all(sha(R/'bundle'/n)==h for n,h in frozen['tools'].items()) and sha(__file__)==frozen['runner_sha256']
 receipt=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(R/'bundle'/n) for n in outputs if (R/'bundle'/n).exists()})
 try:
  j=json.loads((d/'stdout').read_text());receipt['verification']=j.get('verification-results');receipt['failed_functions']=j.get('verus-infer',{}).get('failed_functions',[])
 except ValueError: pass
 write(d/'receipt.json',receipt);print(label,json.dumps(receipt),flush=True);need(guards,'source/tool drift');return receipt

def setup():
 import tempfile
 global R
 R=Path(tempfile.mkdtemp(prefix='local-checked-foreign-review-'));(E/'PRIVATE_ROOT').write_text(str(R)+'\n')
 for p in ['tmp','bundle','cargo-home/registry','cargo-home/git']: (R/p).mkdir(parents=True)
 # Only immutable crate archives and sparse-index entries; never compiled artifacts or extracted sources.
 for part in ['cache','index']:
  shutil.copytree(W/'cargo-home/registry'/part,R/'cargo-home/registry'/part)
 shutil.copytree(W/'cargo-home/git/db',R/'cargo-home/git/db')
 lock=tomllib.loads((ROOT/'source/Cargo.lock').read_text());archives={};missing=[]
 for pkg in lock['package']:
  if pkg.get('source','').startswith('registry+'):
   name=pkg['name']+'-'+pkg['version']+'.crate'
   matches=list((R/'cargo-home/registry/cache').glob('*/'+name))
   if not matches:missing.append(name)
   # Optional workspace packages need not be selected by this bounded build.
   for p in matches: need(sha(p)==pkg['checksum'],'checksum '+str(p));archives[str(p.relative_to(R))]=sha(p)
 write(R/'cache-manifest.json',{'validated':archives,'missing_workspace_optional':missing})
 shutil.copy2(W/'bundle/z3',R/'bundle/z3')
 write(R/'source-manifest.json',manifest())
 write(R/'toolchain.json',{'rustc':subprocess.check_output([TC/'bin/rustc','-vV']).decode(),'binaries':{str(p):sha(p) for p in [TC/'bin/rustc',TC/'bin/cargo',*sorted((TC/'lib').glob('librustc_driver*'))]},'solver':sha(R/'bundle/z3')})
 print(R)

def build():
 p=run('build',[TC/'bin/cargo','build','--offline','--locked','-j','2','-p','rust_verify','-p','verus','-p','verus_builtin','-p','verus_builtin_macros','-p','verus_state_machines_macros','-p','vstd_build'])
 need(p['exit']==0,'build failed')
 for n in TOOLS:
  if n in ['z3','libvstd.rlib','vstd.vir']:continue
  shutil.copy2(R/'target/debug'/n,R/'bundle'/n)
 (R/'bundle/verus-root').touch()
 shutil.copytree(ROOT/'source/vstd',R/'bundle/vstd')
 run('vstd',[R/'target/debug/vstd_build',R/'bundle'],outputs=('vstd.vir','libvstd.rlib'))
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 if sys.argv[1]=='setup':setup()
 elif sys.argv[1]=='build':build()
