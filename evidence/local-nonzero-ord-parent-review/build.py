"""Independent combined runner. Invoke with python3 -B (also -O -B)."""
import hashlib, importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time, tomllib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
OLD=Path('/tmp/certified-foreign-bool-46i7h2jy')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(b,s):
 if not b: raise RuntimeError(s)
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def write(p,j):Path(p).write_text(json.dumps(j,indent=2,sort_keys=True)+'\n')
def root():return Path(os.environ.get('LOCAL_NONZERO_ORD_ROOT') or (HERE/'PRIVATE_ROOT').read_text().strip())
def manifest(base):
 return {p:sha(base/p) for p in subprocess.check_output(['git','ls-files','-z','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().split('\0') if p and not p.startswith('source/tools/research/local_nonzero_ord_factoring/')}
def bundle():return root()/os.environ.get("LOCAL_NONZERO_ORD_BUNDLE", "bundle")
def env():
 r=root();return dict(HOME=str(r/'home'),PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTUP_HOME='/home/dev/.rustup',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(r/'cargo-home'),CARGO_TARGET_DIR=str(r/'target'),CARGO_BUILD_BUILD_DIR=str(r/'build'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',XDG_CACHE_HOME=str(r/'cache'),TMPDIR=str(r/'tmp'),VERUS_ROOT=str(bundle()),VERUS_Z3_PATH=str(bundle()/'z3'),VERUS_INFER_CACHE=str(r/'candidate-cache'),RUST_MIN_STACK='20971520',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(r/'test-logs'))
def run(label,argv,extra=None,outputs=()):
 r=root();b=bundle();d=r/label;d.mkdir(parents=True,exist_ok=True);need(not (d/'receipt.json').exists(),'immutable label '+label)
 f=dict(argv=list(map(str,argv)),cwd=str(r/'src/source'),environment=env()|dict(extra or {}),source=manifest(r/'src'),tools={n:sha(b/n) for n in TOOLS if (b/n).exists() and n not in outputs},fixtures={str(p):sha(p) for p in argv if str(p).endswith('.rs') and Path(p).is_file()},runner_sha256=sha(__file__))
 write(d/'freeze.json',f);start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(f['argv'],cwd=f['cwd'],env=f['environment'],stdout=out,stderr=err)
 guards=f['source']==manifest(r/'src') and all(sha(p)==h for p,h in f['fixtures'].items()) and all(sha(b/n)==h for n,h in f['tools'].items()) and sha(__file__)==f['runner_sha256']
 j=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(b/n) for n in outputs if (b/n).exists()})
 try:j['verification']=json.loads((d/'stdout').read_text())['verification-results']
 except (ValueError,KeyError):pass
 write(d/'receipt.json',j);print(label,json.dumps(j),flush=True);need(guards,'input drift');return j

def setup():
 r=Path(tempfile.mkdtemp(prefix='local-nonzero-ord-'));r.chmod(0o700);(HERE/'PRIVATE_ROOT').write_text(str(r)+'\n')
 for d in ['src','tmp','home','bundle','cargo-home/registry','cargo-home/git']:(r/d).mkdir(parents=True,exist_ok=True)
 archives=json.loads((OLD/'cache-inputs.json').read_text())['archives'];index=json.loads((OLD/'index-git-authentication.json').read_text())
 checksums={p['name']+'-'+p['version']+'.crate':p['checksum'] for p in tomllib.loads((ROOT/'source/Cargo.lock').read_text())['package'] if 'checksum' in p}
 for rel,h in archives.items():
  p=OLD/rel;need(sha(p)==h,'completed archive '+rel)
  if p.name in checksums:need(h==checksums[p.name],'lock '+p.name)
  q=r/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
 for rel,h in index.items():
  p=OLD/'cargo-home'/rel;need(sha(p)==h,'completed index/git '+rel);q=r/'cargo-home'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
 (r/'cargo-home/git/db/getopts-a5b2265512d62342/refs').mkdir(exist_ok=True)
 write(r/'bootstrap.json',dict(archives=archives,index_git=index))
 for rel in manifest(ROOT):
  p=ROOT/rel;q=r/'src'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
 write(r/'shipping-source.json',manifest(ROOT));need(manifest(ROOT)==manifest(r/'src'),'private source identity')
 subprocess.run(['git','init','-q',str(r/'src')],check=True)
 subprocess.run(['git','add','source','dependencies','tools','rust-toolchain.toml'],cwd=r/'src',check=True)
 subprocess.run(['git','-c','user.name=Codex Review','-c','user.email=codex-review@local','commit','-q','-m','Private local NonZero Ord source snapshot'],cwd=r/'src',check=True)
 need(sha(OLD/'bundle/z3')=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','solver')
 shutil.copy2(OLD/'bundle/z3',bundle()/'z3')
 write(r/'toolchain.json',dict(rustc=subprocess.check_output([TC/'bin/rustc','-vV']).decode(),files={str(p):sha(p) for p in [TC/'bin/rustc',TC/'bin/cargo',*sorted((TC/'lib').glob('librustc_driver*'))]}))
 print(r,flush=True)
def build():
 r=root();j=run('build',[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
 need(j['exit']==0,'build failed')
 for n in TOOLS:
  if n not in ['z3','vstd.vir','libvstd.rlib']:shutil.copy2(r/'target/debug'/n,bundle()/n)
 (r/'bundle/verus-root').touch();shutil.copytree(r/'src/source/vstd',r/'bundle/vstd')
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'python -B required')
 if sys.argv[1]=='setup':setup()
 elif sys.argv[1]=='build':build()
 elif sys.argv[1]=='vstd':need(run('vstd',[root()/'target/debug/vstd_build',root()/'bundle'],outputs=['vstd.vir','libvstd.rlib'])['exit']==0,'vstd failure')
