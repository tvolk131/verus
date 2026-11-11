"""Owned, offline Stage A build/evidence. Immutable command epochs, private inputs."""
import hashlib,json,os,pathlib,shutil,subprocess,sys,tempfile,time,tomllib
HERE=pathlib.Path(__file__).resolve().parent
ROOT=HERE.parents[4]
TC=pathlib.Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
CACHE=pathlib.Path('/tmp/certified-foreign-bool-46i7h2jy')
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(b,m):
 if not b:raise RuntimeError(m)
def sha(p):
 with pathlib.Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def write(p,j):p.write_text(json.dumps(j,indent=2)+'\n')
def shipping():
 files=subprocess.check_output(['git','ls-files','-z','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().split('\0')
 return {p:sha(ROOT/p) for p in files if p and not p.startswith('source/tools/research/')}
def environment(r):
 return dict(HOME=str(r/'home'),PATH=str(TC/'bin')+':/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_HOME='/home/dev/.rustup',RUSTUP_TOOLCHAIN='1.99.1',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(r/'cargo-home'),CARGO_TARGET_DIR=str(r/'target'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',CARGO_PROFILE_DEV_DEBUG='0',XDG_CACHE_HOME=str(r/'cache'),TMPDIR=str(r/'tmp'),VERUS_ROOT=str(r/'bundle'),VERUS_Z3_PATH=str(r/'bundle/z3'),VERUS_INFER_CACHE=str(r/'candidate-cache'),RUST_MIN_STACK='20971520',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(r/'test-logs'))
def setup():
 r=pathlib.Path(tempfile.mkdtemp(prefix='rev-stage-a-'));r.chmod(0o700)
 for n in ['home','src','target','tmp','cache','cargo-home/registry','cargo-home/git','bundle','epochs']:(r/n).mkdir(parents=True,exist_ok=True)
 print(r,flush=True)
 for n in ['cache-inputs.json','index-git-authentication.json']:shutil.copy2(CACHE/n,r/n)
 archives=json.loads((r/'cache-inputs.json').read_text())['archives'];index=json.loads((r/'index-git-authentication.json').read_text())
 lock={p['name']+'-'+p['version']+'.crate':p['checksum'] for p in tomllib.loads((ROOT/'source/Cargo.lock').read_text())['package'] if 'checksum' in p}
 for rel,h in archives.items():
  p=CACHE/rel;need(sha(p)==h,'archive '+rel)
  if p.name in lock:need(lock[p.name]==h,'lock '+p.name)
  q=r/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);need(sha(q)==h,'copied archive')
 for rel,h in index.items():
  p=CACHE/'cargo-home'/rel;need(sha(p)==h,'index/git '+rel)
  q=r/'cargo-home'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);need(sha(q)==h,'copied index/git')
 (r/'cargo-home/git/db/getopts-a5b2265512d62342/refs').mkdir(exist_ok=True)
 z=pathlib.Path('/tmp/local-nonzero-ord-09t1tshb/bundle/z3');need(sha(z)=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','solver pin');shutil.copy2(z,r/'bundle/z3')
 freeze=json.loads((HERE.parent/'FREEZE.json').read_text())
 for p,h in freeze['toolchain'].items():need(sha(p)==h,'toolchain '+p)
 write(r/'bootstrap.json',dict(archives=archives,index_git=index,toolchain=freeze['toolchain'],solver_sha256=sha(r/'bundle/z3'),solver_mode=oct((r/'bundle/z3').stat().st_mode & 0o777),controller=sha(__file__)))
 subprocess.run(['git','init','-q',str(r/'src')],check=True)
 sync(r)
 (HERE/'PRIVATE_ROOT').write_text(str(r)+'\n')
 return r

def sync(r):
 manifest=shipping()
 for rel,h in manifest.items():
  p=ROOT/rel;q=r/'src'/rel;q.parent.mkdir(parents=True,exist_ok=True)
  if not q.exists() or sha(q)!=h:shutil.copy2(p,q)
 need(all(sha(r/'src'/p)==h for p,h in manifest.items()),'snapshot')
 subprocess.run(['git','add','source','dependencies','tools','rust-toolchain.toml'],cwd=r/'src',check=True)
 subprocess.run(['git','-c','user.name=Codex','-c','user.email=codex@openai.com','commit','-q','--allow-empty','-m','Owned Stage A source snapshot'],cwd=r/'src',check=True)
 write(r/'shipping.json',manifest)
def run(r,label,args,outputs=()):
 d=r/'epochs'/label;d.mkdir();manifest=shipping();need(manifest==json.loads((r/'shipping.json').read_text()),'sync source first')
 f=dict(argv=list(map(str,args)),cwd=str(r/'src/source'),environment=environment(r),source=manifest,tools={n:sha(r/'bundle'/n) for n in TOOLS if (r/'bundle'/n).exists() and n not in outputs},controller=sha(__file__))
 write(d/'freeze.json',f);start=time.monotonic()
 with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:p=subprocess.run(f['argv'],cwd=f['cwd'],env=f['environment'],stdout=out,stderr=err)
 guards=shipping()==manifest and all(sha(r/'src'/p)==h for p,h in manifest.items()) and all(sha(r/'bundle'/n)==h for n,h in f['tools'].items()) and sha(__file__)==f['controller']
 j=dict(exit=p.returncode,guards=guards,seconds=time.monotonic()-start,freeze_sha256=sha(d/'freeze.json'),stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(r/'bundle'/n) for n in outputs if (r/'bundle'/n).exists()})
 write(d/'receipt.json',j);print(label,json.dumps(j),flush=True);need(guards,'input drift');return j
if __name__=='__main__':
 need(sys.flags.dont_write_bytecode,'use -B')
 if sys.argv[1]=='setup':setup()
 else:
  r=pathlib.Path(os.environ.get('REV_STAGE_A_ROOT') or (HERE/'PRIVATE_ROOT').read_text().strip());action=sys.argv[1]
  if action=='sync':sync(r)
  elif action=='build':
   j=run(r,sys.argv[2],[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
   need(j['exit']==0,'build failed')
   for n in TOOLS:
    if n not in ['z3','libvstd.rlib','vstd.vir']:shutil.copy2(r/'target/debug'/n,r/'bundle'/n)
   (r/'bundle/verus-root').touch()
   if (r/'bundle/vstd').exists():shutil.rmtree(r/'bundle/vstd')
   shutil.copytree(r/'src/source/vstd',r/'bundle/vstd')
  elif action=='vstd':need(run(r,sys.argv[2],[r/'target/debug/vstd_build',r/'bundle'],outputs=['libvstd.rlib','vstd.vir'])['exit']==0,'vstd failure')
  elif action=='test':need(run(r,sys.argv[2],[TC/'bin/cargo','test','--offline','--locked','-j','2','-p','vir',sys.argv[3],'--','--test-threads=2'])['exit']==0,'VIR tests failed')
