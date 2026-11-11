"""Independent private arithmetic audit runner. python3 -B; outputs never overwrite receipts."""
import hashlib, importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
OLD=Path('/tmp/item-packing-jmxyh6mz')
PREV=Path('/tmp/local-checked-foreign-review-l3x_pgnw')
PARENT=Path('/tmp/verus-infer-buffer-cycle-20261109/parent-combined70-bundle-35bc7b12')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(b,s):
 if not b: raise RuntimeError(s)
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def write(p,j):Path(p).write_text(json.dumps(j,indent=2,sort_keys=True)+'\n')
def module(n,p):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def root():return Path((HERE/'PRIVATE_ROOT').read_text().strip())
def manifest(base):
 return {p:sha(base/p) for p in subprocess.check_output(['git','ls-files','-z','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().split('\0') if p}
def env(r,b):return dict(HOME=str(r/'home'),PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTUP_HOME='/home/dev/.rustup',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(r/'cargo-home'),CARGO_TARGET_DIR=str(r/'target'),CARGO_BUILD_BUILD_DIR=str(r/'cargo-build'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',XDG_CACHE_HOME=str(r/'cache'),TMPDIR=str(r/'tmp'),VERUS_ROOT=str(b),VERUS_Z3_PATH=str(b/'z3'),VERUS_INFER_CACHE=str(r/'candidate-cache'),RUST_MIN_STACK='20971520',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(r/'test-logs'))
def run(label,argv,b=None,extra=None,outputs=()):
 r=root();b=b or r/'bundle';d=r/label;d.mkdir(parents=True,exist_ok=True);need(not (d/'receipt.json').exists(),'immutable label '+label)
 f=dict(argv=list(map(str,argv)),cwd=str(r/'src/source'),environment=env(r,b)|dict(extra or {}),source=manifest(r/'src'),tools={n:sha(b/n) for n in TOOLS if (b/n).exists() and n not in outputs},fixtures={str(p):sha(p) for p in argv if str(p).endswith('.rs') and Path(p).is_file()},runner_sha256=sha(__file__))
 write(d/'freeze.json',f);start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(f['argv'],cwd=f['cwd'],env=f['environment'],stdout=out,stderr=err)
 guards=f['source']==manifest(r/'src') and all(sha(p)==h for p,h in f['fixtures'].items()) and all(sha(b/n)==h for n,h in f['tools'].items()) and sha(__file__)==f['runner_sha256']
 j=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(b/n) for n in outputs if (b/n).exists()})
 try:j['verification']=json.loads((d/'stdout').read_text())['verification-results']
 except (ValueError,KeyError):pass
 write(d/'receipt.json',j);print(label,json.dumps(j),flush=True);need(guards,'input drift');return j
def setup():
 r=Path(tempfile.mkdtemp(prefix='item-packing-review-'));(HERE/'PRIVATE_ROOT').write_text(str(r)+'\n')
 for d in ['src','tmp','home','bundle','cargo-home/registry','cargo-home/git']:(r/d).mkdir(parents=True,exist_ok=True)
 need(sha(PARENT/'receipt.json')=='f24584b8ccfde3ac92e84c146d87317b04231b2b165ef759411a0c53ea3199e3','parent receipt')
 j=json.loads((PARENT/'receipt.json').read_text());need(j['success'] and j['matching_vstd_verified']==2250,'accepted bundle')
 for n,h in j['tools'].items():need(sha(PARENT/n)==h,'accepted tool '+n)
 write(r/'accepted-bundle-auth.json',j)
 # Copy only independently checksum-validated archives and recorded sparse-index/getopts Git inputs.
 inputs=json.loads((OLD/'index-git-input-identities.json').read_text());cache=json.loads((PREV/'cache-manifest.json').read_text())['validated']
 for rel,h in cache.items():
  p=PREV/rel;need(sha(p)==h,'archive '+rel);q=r/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);need(sha(q)==h,'copy')
 for part in ['cargo-home/registry/index','cargo-home/git/db']:
  for rel,h in inputs[part].items():
   p=PREV/part/rel;need(sha(p)==h,'index/git '+rel);q=r/part/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);need(sha(q)==h,'copy')
 (r/'cargo-home/git/db/getopts-a5b2265512d62342/refs').mkdir(exist_ok=True)
 need(sha(ROOT/'source/Cargo.lock')==inputs['lock_sha256'],'lock')
 write(r/'bootstrap-inputs.json',dict(archive=cache,index_git=inputs))
 for rel in manifest(ROOT):
  p=ROOT/rel;q=r/'src'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
 write(r/'shipping-source.json',manifest(ROOT));need(manifest(ROOT)==manifest(r/'src'),'private source identity')
 subprocess.run(['git','init','-q',str(r/'src')],check=True)
 subprocess.run(['git','add','source','dependencies','tools','rust-toolchain.toml'],cwd=r/'src',check=True)
 subprocess.run(['git','-c','user.name=Codex Review','-c','user.email=codex-review@local','commit','-q','-m','Private exact review source snapshot'],cwd=r/'src',check=True)
 shutil.copy2(PARENT/'z3',r/'bundle/z3')
 # ONLY fixture() is called; other historical methods are never executed.
 m=module('pure_fixture',ROOT/'source/tools/research/item_packing_goal/run.py');full,helped,fidelity=m.fixture()
 need(hashlib.sha256(full.encode()).hexdigest()=='1adb8b6c2ee64cc1c6930c9530920105ad26cee8abf3212caff53dda0c36518d','full freeze')
 (r/'full.rs').write_text(full);(r/'full-help.rs').write_text(helped);shutil.copy2(ROOT/'source/tools/research/item_packing_goal/reuse.rs',r/'packet.rs');write(r/'fidelity.json',fidelity)
 for n in ['rustc','cargo']:
  write(r/(n+'-identity.json'),dict(path=str(TC/'bin'/n),sha256=sha(TC/'bin'/n),version=subprocess.check_output([TC/'bin'/n,'--version','--verbose']).decode()))
 print(r,flush=True)
def build(label):
 r=root();j=run(label,[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
 need(j['exit']==0,'build failed')
 for n in TOOLS:
  if n not in ['z3','vstd.vir','libvstd.rlib']:shutil.copy2(r/'target/debug'/n,r/'bundle'/n)
 (r/'bundle/verus-root').touch()
 if not (r/'bundle/vstd').exists():shutil.copytree(r/'src/source/vstd',r/'bundle/vstd')
def vstd(label):
 r=root();j=run(label,[r/'target/debug/vstd_build',r/'bundle'],outputs=['vstd.vir','libvstd.rlib']);need(j['exit']==0,'vstd failed')
def trial(label,p,b=None,cache=None):
 r=root();b=b or r/'bundle';d=r/label
 return run(label,[b/'verus',p,'--crate-name=component','--crate-type=lib','--no-cheating','--compile','--infer-goals','--output-json','--multiple-errors','30','--log-all','--log-dir',d/'vir','-o',d/'libcomponent.rlib'],b,{'VERUS_INFER_CACHE':str(cache or d/'candidate-cache')})
def stable(name,src):
 r=root();base=r/name;base.mkdir();p=base/'component.rs';p.write_text(src)
 for phase in ['cold','warm']:
  j=trial(name+'/run',p,cache=base/'candidate-cache');shutil.move(base/'run',base/phase);need(j['exit']==0 and j['verification']['errors']==0,'stable failure')
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'python -B required')
 if sys.argv[1]=='setup':setup()
 elif sys.argv[1]=='build':build(sys.argv[2])
 elif sys.argv[1]=='vstd':vstd(sys.argv[2])
 elif sys.argv[1]=='stable':stable(sys.argv[2],(root()/sys.argv[3]).read_text())
