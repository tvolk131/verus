"""Own private offline build; exact user-authorized solver only."""
import hashlib, json, os, shutil, subprocess, sys, tempfile, time, tomllib
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[3]
OLD=Path('/tmp/certified-foreign-bool-46i7h2jy')
TC=Path('/home/dev/.rustup/toolchains/1.99.1-x86_64-unknown-linux-gnu')
TOOLS=['verus','rust_verify','z3','libverus_builtin.rlib','libverus_builtin_macros.so','libverus_state_machines_macros.so','libvstd.rlib','vstd.vir']
def need(ok,message):
    if not ok: raise RuntimeError(message)
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest()
def write(p,v): Path(p).write_text(json.dumps(v,indent=2,sort_keys=True)+'\n')
def root(): return Path((HERE/'PRIVATE_ROOT').read_text().strip())
def manifest():
    paths=subprocess.check_output(['git','ls-files','-z','source','dependencies','tools','rust-toolchain.toml'],cwd=ROOT).decode().split('\0')
    return {p:sha(ROOT/p) for p in paths if p and not p.startswith('source/tools/research/miri_tagged_reverse_search/')}
def controllers(): return {p.name:sha(p) for p in HERE.glob('*.py')}
def env():
    r=root()
    return dict(HOME=str(r/'home'),PATH='/home/dev/.cargo/bin:/usr/bin:/bin',RUSTUP_TOOLCHAIN='1.99.1',VARGO_TOOLCHAIN='1.99.1-x86_64-unknown-linux-gnu',RUSTUP_HOME='/home/dev/.rustup',RUSTC_BOOTSTRAP='1',LD_LIBRARY_PATH=str(TC/'lib'),CARGO_HOME=str(r/'cargo-home'),CARGO_TARGET_DIR=str(r/'target'),CARGO_BUILD_BUILD_DIR=str(r/'build'),CARGO_BUILD_JOBS='2',CARGO_NET_OFFLINE='true',XDG_CACHE_HOME=str(r/'cache'),TMPDIR=str(r/'tmp'),VERUS_ROOT=str(r/'bundle'),VERUS_Z3_PATH=str(r/'bundle/z3'),VERUS_INFER_CACHE=str(r/'candidate-cache'),RUST_MIN_STACK='20971520',RUSTFLAGS='--cfg proc_macro_span --cfg verus_keep_ghost --cfg span_locations --cfg procmacro2_semver_exempt',VERUS_IN_VARGO='1',VERUS_TEST_LOG_DIR=str(r/'test-logs'))
def run(label,argv,outputs=()):
    r=root();d=r/label;d.mkdir();b=r/'bundle'
    f=dict(argv=list(map(str,argv)),cwd=str(ROOT/'source'),environment=env(),source=manifest(),controllers=controllers(),tools={n:sha(b/n) for n in TOOLS if (b/n).is_file() and n not in outputs},fixtures={str(p):sha(p) for p in argv if str(p).endswith('.rs') and Path(p).is_file()})
    archive=r/'controllers';archive.mkdir(exist_ok=True)
    for n,h in f['controllers'].items():
        q=archive/(h+'.py')
        if not q.exists():shutil.copyfile(HERE/n,q)
    write(d/'freeze.json',f);fsha=sha(d/'freeze.json');start=time.monotonic()
    with (d/'stdout').open('wb') as out,(d/'stderr').open('wb') as err:
        p=subprocess.run(f['argv'],cwd=f['cwd'],env=f['environment'],stdout=out,stderr=err)
    guards=(f['source']==manifest() and f['controllers']==controllers() and all(sha(b/n)==h for n,h in f['tools'].items()) and all(sha(n)==h for n,h in f['fixtures'].items()) and sha(d/'freeze.json')==fsha)
    j=dict(exit=p.returncode,seconds=time.monotonic()-start,guards=guards,freeze_sha256=fsha,stdout_sha256=sha(d/'stdout'),stderr_sha256=sha(d/'stderr'),outputs={n:sha(b/n) for n in outputs if (b/n).is_file()})
    try: j['verification']=json.loads((d/'stdout').read_text())['verification-results']
    except (ValueError,KeyError): pass
    write(d/'receipt.json',j);print(label,json.dumps(j),flush=True);need(guards,'input drift');return j

def safe_rel(rel,prefix):
    p=Path(rel);need(not p.is_absolute() and '..' not in p.parts,'unsafe input path');need(rel.startswith(prefix),'unexpected cache path');return p

def setup():
    need(not (HERE/'PRIVATE_ROOT').exists(),'setup already exists')
    r=Path(tempfile.mkdtemp(prefix='miri-tagged-reverse-search-'));r.chmod(0o700);(HERE/'PRIVATE_ROOT').write_text(str(r)+'\n')
    for d in ['home','tmp','bundle','cargo-home/registry','cargo-home/git','cache']:(r/d).mkdir(parents=True,exist_ok=True)
    archives_file=OLD/'cache-inputs.json';index_file=OLD/'index-git-authentication.json'
    archives=json.loads(archives_file.read_text())['archives'];index=json.loads(index_file.read_text())
    lock=ROOT/'source/Cargo.lock';checks={p['name']+'-'+p['version']+'.crate':p['checksum'] for p in tomllib.loads(lock.read_text())['package'] if 'checksum' in p}
    for rel,h in archives.items():
        rel=safe_rel(rel,'cargo-home/registry/cache/');p=OLD/rel
        need(sha(p)==h,'archive hash '+str(rel))
        if p.name in checks:need(h==checks[p.name],'lock checksum '+p.name)
        q=r/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
    for rel,h in index.items():
        rel=safe_rel(rel,'registry/' if rel.startswith('registry/') else 'git/');p=OLD/'cargo-home'/rel
        need(sha(p)==h,'index/git hash '+str(rel));q=r/'cargo-home'/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q)
    (r/'cargo-home/git/db/getopts-a5b2265512d62342/refs').mkdir(exist_ok=True)
    write(r/'bootstrap.json',dict(manifests={str(p):sha(p) for p in [archives_file,index_file]},lock_sha256=sha(lock),archives=archives,index_git=index))
    files=[TC/'bin/rustc',TC/'bin/cargo',*sorted((TC/'lib').glob('librustc_driver*')),*sorted((TC/'lib/rustlib/x86_64-unknown-linux-gnu/lib').glob('libcore-*.rlib'))]
    write(r/'toolchain.json',dict(rustc=subprocess.check_output([TC/'bin/rustc','-vV']).decode(),files={str(p):sha(p) for p in files}))
    # Exact completed solver is explicitly authorized, independently authenticated.
    z3=Path('/tmp/local-nonzero-ord-09t1tshb/bundle/z3')
    need(sha(z3)=='e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765','authorized solver hash')
    shutil.copy2(z3,r/'bundle/z3');(r/'bundle/z3').chmod(0o700)
    write(r/'setup.json',dict(source=manifest(),controllers=controllers(),solver_present=(r/'bundle/z3').is_file(),publication=False,goal_complete=False))
    print(r,flush=True)
def build():
    r=root();j=run('build-final',[TC/'bin/cargo','build','--offline','--locked','-j','2',*sum((['-p',p] for p in ['rust_verify','verus','verus_builtin','verus_builtin_macros','verus_state_machines_macros','vstd_build']),[])])
    need(j['exit']==0,'compiler build failed')
    for n in TOOLS:
        if n not in ['z3','libvstd.rlib','vstd.vir']:shutil.copy2(r/'target/debug'/n,r/'bundle'/n)
    (r/'bundle/verus-root').touch();shutil.copytree(ROOT/'source/vstd',r/'bundle/vstd')
def vstd():
    need((root()/'bundle/z3').is_file(),'authorized solver missing; vstd/proof trials forbidden')
    need(run('vstd',[root()/'target/debug/vstd_build',root()/'bundle'],['libvstd.rlib','vstd.vir'])['exit']==0,'matching vstd failed')
    need(all((root()/'bundle'/n).is_file() for n in TOOLS),'eight tools required')
    write(root()/'eight-tools.json',{n:sha(root()/'bundle'/n) for n in TOOLS})
if __name__=='__main__':
    need(sys.flags.dont_write_bytecode,'python -B required')
    {'setup':setup,'build':build,'vstd':vstd}[sys.argv[1]]()
